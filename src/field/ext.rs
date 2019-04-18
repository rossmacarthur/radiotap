//! Extended Radiotap field definitions and parsers.

use crate::{Error, Result};

const HT_RATE: [[f32; 4]; 32] = [
    // 20 MHz LGI,20 MHz SGI,40 MHZ LGI,40 MHz SGI
    [6.50, 7.20, 13.50, 15.00],
    [13.00, 14.40, 27.00, 30.00],
    [19.50, 21.70, 40.50, 45.00],
    [26.00, 28.90, 54.00, 60.00],
    [39.00, 43.30, 81.00, 90.00],
    [52.00, 57.80, 108.00, 120.00],
    [58.50, 65.00, 121.50, 135.00],
    [65.00, 72.20, 135.00, 150.00],
    [13.00, 14.40, 27.00, 30.00],
    [26.00, 28.90, 54.00, 60.00],
    [39.00, 43.30, 81.00, 90.00],
    [52.00, 57.80, 108.00, 120.00],
    [78.00, 86.70, 162.00, 180.00],
    [104.00, 115.60, 216.00, 240.00],
    [117.00, 130.00, 243.00, 270.00],
    [130.00, 144.40, 270.00, 300.00],
    [19.50, 21.70, 40.50, 45.00],
    [39.00, 43.30, 81.00, 90.00],
    [58.50, 65.00, 121.50, 135.00],
    [78.00, 86.70, 162.00, 180.00],
    [117.00, 130.00, 243.00, 270.00],
    [156.00, 173.30, 324.00, 360.00],
    [175.50, 195.00, 364.50, 405.00],
    [195.00, 216.70, 405.00, 450.00],
    [26.00, 28.80, 54.00, 60.00],
    [52.00, 57.60, 108.00, 120.00],
    [78.00, 86.80, 162.00, 180.00],
    [104.00, 115.60, 216.00, 240.00],
    [156.00, 173.20, 324.00, 360.00],
    [208.00, 231.20, 432.00, 480.00],
    [234.00, 260.00, 486.00, 540.00],
    [260.00, 288.80, 540.00, 600.00],
];

const VHT_RATE: [[f32; 8]; 80] = [
    // 20 MHz LGI,20 MHz SGI,40 MHz LGI,40 MHz SGI,80 MHZ LGI,80 MHz SGI,160 MHZ LGI,160 MHz SGI
    [6.5, 7.2, 13.5, 15.0, 29.3, 32.5, 58.5, 65.0],
    [13.0, 14.4, 27.0, 30.0, 58.5, 65.0, 117.0, 130.0],
    [19.5, 21.7, 40.5, 45.0, 87.8, 97.5, 175.5, 195.0],
    [26.0, 28.9, 54.0, 60.0, 117.0, 130.0, 234.0, 260.0],
    [39.0, 43.3, 81.0, 90.0, 175.5, 195.0, 351.0, 390.0],
    [52.0, 57.8, 108.0, 120.0, 234.0, 260.0, 468.0, 520.0],
    [58.5, 65.0, 121.5, 135.0, 263.3, 292.5, 526.5, 585.0],
    [65.0, 72.2, 135.0, 150.0, 292.5, 325.0, 585.0, 650.0],
    [78.0, 86.7, 162.0, 180.0, 351.0, 390.0, 702.0, 780.0],
    [-1.0, -1.0, 180.0, 200.0, 390.0, 433.3, 780.0, 866.7],
    [13.0, 14.4, 27.0, 30.0, 58.5, 65.0, 117.0, 130.0],
    [26.0, 28.9, 54.0, 60.0, 117.0, 130.0, 234.0, 260.0],
    [39.0, 43.3, 81.0, 90.0, 175.5, 195.0, 351.0, 390.0],
    [52.0, 57.8, 108.0, 120.0, 234.0, 260.0, 468.0, 520.0],
    [78.0, 86.7, 162.0, 180.0, 351.0, 390.0, 702.0, 780.0],
    [104.0, 115.6, 216.0, 240.0, 468.0, 520.0, 936.0, 1040.0],
    [117.0, 130.3, 243.0, 270.0, 526.5, 585.0, 1053.0, 1170.0],
    [130.0, 144.4, 270.0, 300.0, 585.0, 650.0, 1170.0, 1300.0],
    [156.0, 173.3, 324.0, 360.0, 702.0, 780.0, 1404.0, 1560.0],
    [-1.0, -1.0, 360.0, 400.0, 780.0, 866.7, 1560.0, 1733.3],
    [19.5, 21.7, 40.5, 45.0, 87.8, 97.5, 175.5, 195.0],
    [39.0, 43.3, 81.0, 90.0, 175.5, 195.0, 351.0, 390.0],
    [58.5, 65.0, 121.5, 135.0, 263.3, 292.5, 526.5, 585.0],
    [78.0, 86.7, 162.0, 180.0, 351.0, 390.0, 702.0, 780.0],
    [117.0, 130.0, 243.0, 270.0, 526.5, 585.0, 1053.0, 1170.0],
    [156.0, 173.3, 324.0, 360.0, 702.0, 780.0, 1404.0, 1560.0],
    [175.5, 195.0, 364.5, 405.0, -1.0, -1.0, 1579.5, 1755.0],
    [195.0, 216.7, 405.0, 450.0, 877.5, 975.0, 1755.0, 1950.0],
    [234.0, 260.0, 486.0, 540.0, 1053.0, 1170.0, 2106.0, 2340.0],
    [260.0, 288.9, 540.0, 600.0, 1170.0, 1300.0, -1.0, -1.0],
    [26.0, 28.9, 54.0, 60.0, 117.0, 130.0, 234.0, 260.0],
    [52.0, 57.8, 108.0, 120.0, 234.0, 260.0, 468.0, 520.0],
    [78.0, 86.7, 162.0, 180.0, 351.0, 390.0, 702.0, 780.0],
    [104.0, 115.6, 216.0, 240.0, 468.0, 520.0, 936.0, 1040.0],
    [156.0, 173.3, 324.0, 360.0, 702.0, 780.0, 1404.0, 1560.0],
    [208.0, 231.1, 432.0, 480.0, 936.0, 1040.0, 1872.0, 2080.0],
    [234.0, 260.0, 486.0, 540.0, 1053.0, 1170.0, 2106.0, 2340.0],
    [260.0, 288.9, 540.0, 600.0, 1170.0, 1300.0, 2340.0, 2600.0],
    [312.0, 346.7, 648.0, 720.0, 1404.0, 1560.0, 2808.0, 3120.0],
    [-1.0, -1.0, 720.0, 800.0, 1560.0, 1733.3, 3120.0, 3466.7],
    [-1.0, -1.0, -1.0, -1.0, 146.3, 162.5, 292.5, 325.0],
    [-1.0, -1.0, -1.0, -1.0, 292.5, 325.0, 585.0, 650.0],
    [-1.0, -1.0, -1.0, -1.0, 438.8, 487.5, 877.5, 975.0],
    [-1.0, -1.0, -1.0, -1.0, 585.0, 650.0, 1170.0, 1300.0],
    [-1.0, -1.0, -1.0, -1.0, 877.5, 975.0, 1755.0, 1950.0],
    [-1.0, -1.0, -1.0, -1.0, 1170.0, 1300.0, 2340.0, 2600.0],
    [-1.0, -1.0, -1.0, -1.0, 1316.3, 1462.5, 2632.5, 2925.0],
    [-1.0, -1.0, -1.0, -1.0, 1462.5, 1625.0, 2925.0, 3250.0],
    [-1.0, -1.0, -1.0, -1.0, 1755.0, 1950.0, 3510.0, 3900.0],
    [-1.0, -1.0, -1.0, -1.0, 1950.0, 2166.7, 3900.0, 4333.3],
    [-1.0, -1.0, -1.0, -1.0, 175.5, 195.0, 351.0, 390.0],
    [-1.0, -1.0, -1.0, -1.0, 351.0, 390.0, 702.0, 780.0],
    [-1.0, -1.0, -1.0, -1.0, 526.5, 585.0, 1053.0, 1170.0],
    [-1.0, -1.0, -1.0, -1.0, 702.0, 780.0, 1404.0, 1560.0],
    [-1.0, -1.0, -1.0, -1.0, 1053.0, 1170.0, 2106.0, 2340.0],
    [-1.0, -1.0, -1.0, -1.0, 1404.0, 1560.0, 2808.0, 3120.0],
    [-1.0, -1.0, -1.0, -1.0, 1579.5, 1755.0, 3159.0, 3510.0],
    [-1.0, -1.0, -1.0, -1.0, 1755.0, 1950.0, 3510.0, 3900.0],
    [-1.0, -1.0, -1.0, -1.0, 2106.0, 2340.0, 4212.0, 4680.0],
    [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 4680.0, 5200.0],
    [-1.0, -1.0, -1.0, -1.0, 204.8, 227.5, 409.5, 455.0],
    [-1.0, -1.0, -1.0, -1.0, 409.5, 455.0, 819.0, 910.0],
    [-1.0, -1.0, -1.0, -1.0, 614.3, 682.5, 1228.5, 1365.0],
    [-1.0, -1.0, -1.0, -1.0, 819.0, 910.0, 1638.0, 1820.0],
    [-1.0, -1.0, -1.0, -1.0, 1228.5, 1365.0, 2457.0, 2730.0],
    [-1.0, -1.0, -1.0, -1.0, 1638.0, 1820.0, 3276.0, 3640.0],
    [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 3685.5, 4095.0],
    [-1.0, -1.0, -1.0, -1.0, 2047.5, 2275.0, 4095.0, 4550.0],
    [-1.0, -1.0, -1.0, -1.0, 2457.0, 2730.0, 4914.0, 5460.0],
    [-1.0, -1.0, -1.0, -1.0, 2730.0, 3033.3, 5460.0, 6066.7],
    [-1.0, -1.0, -1.0, -1.0, 234.0, 260.0, 468.0, 520.0],
    [-1.0, -1.0, -1.0, -1.0, 468.0, 520.0, 936.0, 1040.0],
    [-1.0, -1.0, -1.0, -1.0, 702.0, 780.0, 1404.0, 1560.0],
    [-1.0, -1.0, -1.0, -1.0, 936.0, 1040.0, 1872.0, 2080.0],
    [-1.0, -1.0, -1.0, -1.0, 1404.0, 1560.0, 2808.0, 3120.0],
    [-1.0, -1.0, -1.0, -1.0, 1872.0, 2080.0, 3744.0, 4160.0],
    [-1.0, -1.0, -1.0, -1.0, 2106.0, 2340.0, 4212.0, 4680.0],
    [-1.0, -1.0, -1.0, -1.0, 2340.0, 2600.0, 4680.0, 5200.0],
    [-1.0, -1.0, -1.0, -1.0, 2808.0, 3120.0, 5616.0, 6240.0],
    [-1.0, -1.0, -1.0, -1.0, 3120.0, 3466.7, 6240.0, 6933.3],
];

/// Returns the 802.11n data rate based on the MCS index, bandwidth, and guard
/// interval.
pub fn ht_rate(index: u8, bw: Bandwidth, gi: GuardInterval) -> Result<f32> {
    if index > 31 {
        return Err(Error::InvalidFormat);
    }

    let b = match bw.bandwidth {
        20 => 0,
        40 => 2,
        _ => return Err(Error::InvalidFormat),
    };

    let col = b + (if gi == GuardInterval::Short { 1 } else { 0 });

    Ok(HT_RATE[index as usize][col])
}

/// Returns the 802.11ac data rate based on the MCS index, bandwidth, guard
/// interval, and number of spatial streams.
pub fn vht_rate(index: u8, bw: Bandwidth, gi: GuardInterval, nss: u8) -> Result<f32> {
    if index > 9 || nss > 8 {
        return Err(Error::InvalidFormat);
    }

    let b = match bw.bandwidth {
        20 => 0,
        40 => 2,
        80 => 4,
        160 => 6,
        _ => return Err(Error::InvalidFormat),
    };

    let col = b + (if gi == GuardInterval::Short { 1 } else { 0 });
    let row = index + (nss - 1) * 10;

    let rate = VHT_RATE[row as usize][col];
    if rate < 0.0 {
        return Err(Error::InvalidFormat);
    }

    Ok(rate)
}

/// Flags describing the channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChannelFlags {
    /// Turbo channel.
    pub turbo: bool,
    /// Complementary Code Keying (CCK) channel.
    pub cck: bool,
    /// Orthogonal Frequency-Division Multiplexing (OFDM) channel.
    pub ofdm: bool,
    /// 2 GHz spectrum channel.
    pub ghz2: bool,
    /// 5 GHz spectrum channel.
    pub ghz5: bool,
    /// Only passive scan allowed.
    pub passive: bool,
    /// Dynamic CCK-OFDM channel.
    pub dynamic: bool,
    /// Gaussian Frequency Shift Keying (GFSK) channel.
    pub gfsk: bool,
}

/// Extended flags describing the channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct XChannelFlags {
    /// Turbo channel.
    pub turbo: bool,
    /// Complementary Code Keying (CCK) channel.
    pub cck: bool,
    /// Orthogonal Frequency-Division Multiplexing (OFDM) channel.
    pub ofdm: bool,
    /// 2 GHz spectrum channel.
    pub ghz2: bool,
    /// 5 GHz spectrum channel.
    pub ghz5: bool,
    /// Only passive scan allowed.
    pub passive: bool,
    /// Dynamic CCK-OFDM channel.
    pub dynamic: bool,
    /// Gaussian Frequency Shift Keying (GFSK) channel.
    pub gfsk: bool,
    /// GSM channel.
    pub gsm: bool,
    /// Static Turbo channel.
    pub sturbo: bool,
    /// Half rate channel.
    pub half: bool,
    /// Quarter rate channel.
    pub quarter: bool,
    /// HT Channel (20MHz Channel Width).
    pub ht20: bool,
    /// HT Channel (40MHz Channel Width with Extension channel above).
    pub ht40u: bool,
    /// HT Channel (40MHz Channel Width with Extension channel below).
    pub ht40d: bool,
}

/// Struct containing the bandwidth, sideband, and sideband index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Bandwidth {
    /// The bandwidth in MHz.
    pub bandwidth: u8,
    /// The sideband bandwidth in MHz.
    pub sideband: Option<u8>,
    /// The sideband index.
    pub sideband_index: Option<u8>,
}

impl Bandwidth {
    pub fn new(value: u8) -> Result<Bandwidth> {
        let (bandwidth, sideband, sideband_index) = match value {
            0 => (20, None, None),
            1 => (40, None, None),
            2 => (40, Some(20), Some(0)),
            3 => (40, Some(20), Some(1)),
            4 => (80, None, None),
            5 => (80, Some(40), Some(0)),
            6 => (80, Some(40), Some(1)),
            7 => (80, Some(20), Some(0)),
            8 => (80, Some(20), Some(1)),
            9 => (80, Some(20), Some(2)),
            10 => (80, Some(20), Some(3)),
            11 => (160, None, None),
            12 => (160, Some(80), Some(0)),
            13 => (160, Some(80), Some(1)),
            14 => (160, Some(40), Some(0)),
            15 => (160, Some(40), Some(1)),
            16 => (160, Some(40), Some(2)),
            17 => (160, Some(40), Some(3)),
            18 => (160, Some(20), Some(0)),
            19 => (160, Some(20), Some(1)),
            20 => (160, Some(20), Some(2)),
            21 => (160, Some(20), Some(3)),
            22 => (160, Some(20), Some(4)),
            23 => (160, Some(20), Some(5)),
            24 => (160, Some(20), Some(6)),
            25 => (160, Some(20), Some(7)),
            _ => {
                return Err(Error::InvalidFormat);
            }
        };
        Ok(Bandwidth {
            bandwidth,
            sideband,
            sideband_index,
        })
    }
}

/// Represents a [VHT](../struct.VHT.html) user, the [VHT](../struct.VHT.html)
/// encodes the MCS and NSS for up to four users.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VHTUser {
    /// The 802.11ac MCS index.
    pub index: u8,
    /// The FEC type.
    pub fec: FEC,
    /// Number of spatial streams (range 1 - 8).
    pub nss: u8,
    /// Number of space-time streams (range 1 - 16).
    pub nsts: u8,
    /// The datarate in Mbps
    pub datarate: Option<f32>,
}

/// The guard interval.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GuardInterval {
    /// 800 ns.
    Long,
    /// 400 ns.
    Short,
}

/// Forward error correction type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FEC {
    /// Binary convolutional coding.
    BCC,
    /// Low-density parity-check.
    LDPC,
}

/// The HT format.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HTFormat {
    Mixed,
    Greenfield,
}

/// The time unit of the [Timestamp](../struct.Timestamp.html).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TimeUnit {
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

impl TimeUnit {
    pub fn new(value: u8) -> Result<TimeUnit> {
        Ok(match value {
            0 => TimeUnit::Milliseconds,
            1 => TimeUnit::Microseconds,
            2 => TimeUnit::Nanoseconds,
            _ => {
                return Err(Error::InvalidFormat);
            }
        })
    }
}

/// The sampling position of the [Timestamp](../struct.Timestamp.html).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SamplingPosition {
    StartMPDU,
    StartPLCP,
    EndPPDU,
    EndMPDU,
    Unknown,
}

impl SamplingPosition {
    pub fn from(value: u8) -> Result<SamplingPosition> {
        Ok(match value {
            0 => SamplingPosition::StartMPDU,
            1 => SamplingPosition::StartPLCP,
            2 => SamplingPosition::EndPPDU,
            3 => SamplingPosition::EndMPDU,
            15 => SamplingPosition::Unknown,
            _ => return Err(Error::InvalidFormat),
        })
    }
}
