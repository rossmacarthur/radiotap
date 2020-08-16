//! Radiotap field definitions and parsers.

pub mod ext;

use bitops::BitOps;
use byteorder::{ReadBytesExt, LE};
use std::io::{Cursor, Read};

use crate::{field::ext::*, Error, Result};

type Oui = [u8; 3];

/// The type of radiotap field.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Tsft,
    Flags,
    Rate,
    Channel,
    Fhss,
    AntennaSignal,
    AntennaNoise,
    LockQuality,
    TxAttenuation,
    TxAttenuationDb,
    TxPower,
    Antenna,
    AntennaSignalDb,
    AntennaNoiseDb,
    RxFlags,
    TxFlags,
    RtsRetries,
    DataRetries,
    XChannel,
    Mcs,
    AmpduStatus,
    Vht,
    Timestamp,
    VendorNamespace(Option<VendorNamespace>),
}

impl Kind {
    pub fn new(value: u8) -> Result<Self> {
        Ok(match value {
            0 => Self::Tsft,
            1 => Self::Flags,
            2 => Self::Rate,
            3 => Self::Channel,
            4 => Self::Fhss,
            5 => Self::AntennaSignal,
            6 => Self::AntennaNoise,
            7 => Self::LockQuality,
            8 => Self::TxAttenuation,
            9 => Self::TxAttenuationDb,
            10 => Self::TxPower,
            11 => Self::Antenna,
            12 => Self::AntennaSignalDb,
            13 => Self::AntennaNoiseDb,
            14 => Self::RxFlags,
            15 => Self::TxFlags,
            16 => Self::RtsRetries,
            17 => Self::DataRetries,
            18 => Self::XChannel,
            19 => Self::Mcs,
            20 => Self::AmpduStatus,
            21 => Self::Vht,
            22 => Self::Timestamp,
            _ => {
                return Err(Error::UnsupportedField);
            }
        })
    }

    /// Returns the align value for the field.
    pub fn align(&self) -> u64 {
        match self {
            Self::Tsft | Self::Timestamp => 8,
            Self::XChannel | Self::AmpduStatus => 4,
            Self::Channel
            | Self::Fhss
            | Self::LockQuality
            | Self::TxAttenuation
            | Self::TxAttenuationDb
            | Self::RxFlags
            | Self::TxFlags
            | Self::Vht
            | Self::VendorNamespace(_) => 2,
            _ => 1,
        }
    }

    /// Returns the size of the field.
    pub fn size(&self) -> usize {
        match self {
            Self::Vht | Self::Timestamp => 12,
            Self::Tsft | Self::AmpduStatus | Self::XChannel => 8,
            Self::VendorNamespace(_) => 6,
            Self::Channel => 4,
            Self::Mcs => 3,
            Self::Fhss
            | Self::LockQuality
            | Self::TxAttenuation
            | Self::TxAttenuationDb
            | Self::RxFlags
            | Self::TxFlags => 2,
            _ => 1,
        }
    }
}

pub trait FromBytes {
    fn from_bytes(input: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Parse any `Field` and return a `Result<Some<T>>`.
pub fn from_bytes_some<T>(input: &[u8]) -> Result<Option<T>>
where
    T: FromBytes,
{
    Ok(Some(FromBytes::from_bytes(input)?))
}

/// The radiotap header, contained in all radiotap captures.
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    /// The radiotap version, only version 0 is supported.
    pub version: u8,
    /// The length of the entire radiotap capture.
    pub length: usize,
    /// The size of the radiotap header.
    pub size: usize,
    /// The fields present in the radiotap capture.
    pub present: Vec<Kind>,
}

impl FromBytes for Header {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);

        let version = cursor.read_u8()?;
        if version != 0 {
            // We only support version 0
            return Err(Error::UnsupportedVersion);
        }

        cursor.read_u8()?; // Account for 1 byte padding field

        let length = cursor.read_u16::<LE>()?;
        if input.len() < length as usize {
            return Err(Error::InvalidLength);
        }

        let mut present;
        let mut present_count = 0;
        let mut vendor_namespace = false;
        let mut kinds = Vec::new();

        loop {
            present = cursor.read_u32::<LE>()?;

            if !vendor_namespace {
                for bit in 0..29 {
                    if present.is_bit_set(bit) {
                        match Kind::new(present_count * 32 + bit) {
                            Ok(kind) => {
                                kinds.push(kind);
                            }
                            Err(Error::UnsupportedField) => {
                                // Does not matter, we will just parse the ones
                                // we can
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
            }

            // Need to move to radiotap namespace
            if present.is_bit_set(29) {
                present_count = 0;
                vendor_namespace = false;

            // Need to move to vendor namespace
            } else if present.is_bit_set(30) {
                present_count = 0;
                vendor_namespace = true;
                // We'll figure out what namespace it is later, just use none
                kinds.push(Kind::VendorNamespace(None))

            // Need to stay in the same namespace
            } else {
                present_count += 1;
            }

            // More present words do not exist
            if !present.is_bit_set(31) {
                break;
            }
        }

        Ok(Self {
            version,
            length: length as usize,
            size: cursor.position() as usize,
            present: kinds,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VendorNamespace {
    pub oui: Oui,
    pub sub_namespace: u8,
    pub skip_length: u16,
}

impl FromBytes for VendorNamespace {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        let mut oui = [0; 3];
        cursor.read_exact(&mut oui)?;
        let sub_namespace = cursor.read_u8()?;
        let skip_length = cursor.read_u16::<LE>()?;
        Ok(Self {
            oui,
            sub_namespace,
            skip_length,
        })
    }
}

/// Value in microseconds of the MAC’s 64-bit 802.11 Time Synchronization
/// Function timer when the first bit of the MPDU arrived at the MAC. For
/// received frames only.
#[derive(Debug, Clone, PartialEq)]
pub struct Tsft {
    pub value: u64,
}

impl FromBytes for Tsft {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u64::<LE>()?;
        Ok(Self { value })
    }
}

/// Properties of transmitted and received frames.
#[derive(Debug, Clone, PartialEq)]
pub struct Flags {
    /// The frame was sent/received during CFP.
    pub cfp: bool,
    /// The frame was sent/received with short preamble.
    pub preamble: bool,
    /// The frame was sent/received with WEP encryption.
    pub wep: bool,
    /// The frame was sent/received with fragmentation.
    pub fragmentation: bool,
    /// The frame includes FCS.
    pub fcs: bool,
    /// The frame has padding between 802.11 header and payload (to 32-bit
    /// boundary).
    pub data_pad: bool,
    /// The frame failed FCS check.
    pub bad_fcs: bool,
    /// The frame used short guard interval (HT).
    pub sgi: bool,
}

impl FromBytes for Flags {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let flags = Cursor::new(input).read_u8()?;
        Ok(Self {
            cfp: flags.is_flag_set(0x01),
            preamble: flags.is_flag_set(0x02),
            wep: flags.is_flag_set(0x04),
            fragmentation: flags.is_flag_set(0x08),
            fcs: flags.is_flag_set(0x10),
            data_pad: flags.is_flag_set(0x20),
            bad_fcs: flags.is_flag_set(0x40),
            sgi: flags.is_flag_set(0x80),
        })
    }
}

/// The legacy data rate in Mbps. Usually only one of the
/// [Rate](struct.Rate.html), [MCS](struct.MCS.html), and [VHT](struct.VHT.html)
/// fields is present.
#[derive(Debug, Clone, PartialEq)]
pub struct Rate {
    pub value: f32,
}

impl FromBytes for Rate {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = f32::from(Cursor::new(input).read_i8()?) / 2.0;
        Ok(Self { value })
    }
}

/// The transmitted or received frequency in MHz, including flags describing the
/// channel.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    /// The frequency in MHz.
    pub freq: u16,
    // The channel flags.
    pub flags: ChannelFlags,
}

impl FromBytes for Channel {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        let freq = cursor.read_u16::<LE>()?;
        let flags = cursor.read_u16::<LE>()?;
        let flags = ChannelFlags {
            turbo: flags.is_flag_set(0x0010),
            cck: flags.is_flag_set(0x0020),
            ofdm: flags.is_flag_set(0x0040),
            ghz2: flags.is_flag_set(0x0080),
            ghz5: flags.is_flag_set(0x0100),
            passive: flags.is_flag_set(0x0200),
            dynamic: flags.is_flag_set(0x0400),
            gfsk: flags.is_flag_set(0x0800),
        };
        Ok(Self { freq, flags })
    }
}

/// The hop set and pattern for frequency-hopping radios.
#[derive(Debug, Clone, PartialEq)]
pub struct Fhss {
    pub hopset: u8,
    pub pattern: u8,
}

impl FromBytes for Fhss {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        let hopset = cursor.read_u8()?;
        let pattern = cursor.read_u8()?;
        Ok(Self { hopset, pattern })
    }
}

/// RF signal power at the antenna in dBm. Indicates the RF signal power at the
/// antenna, in decibels difference from 1mW.
#[derive(Debug, Clone, PartialEq)]
pub struct AntennaSignal {
    pub value: i8,
}

impl FromBytes for AntennaSignal {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_i8()?;
        Ok(Self { value })
    }
}

/// RF signal power at the antenna in dB. Indicates the RF signal power at the
/// antenna, in decibels difference from an arbitrary, fixed reference.
#[derive(Debug, Clone, PartialEq)]
pub struct AntennaSignalDb {
    pub value: u8,
}

impl FromBytes for AntennaSignalDb {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u8()?;
        Ok(Self { value })
    }
}

/// RF noise power at the antenna in dBm. Indicates the RF signal noise at the
/// antenna, in decibels  difference from 1mW.
#[derive(Debug, Clone, PartialEq)]
pub struct AntennaNoise {
    pub value: i8,
}

impl FromBytes for AntennaNoise {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_i8()?;
        Ok(Self { value })
    }
}

/// RF noise power at the antenna in dB. Indicates the RF signal noise at the
/// antenna, in decibels difference from an arbitrary, fixed reference.
#[derive(Debug, Clone, PartialEq)]
pub struct AntennaNoiseDb {
    pub value: u8,
}

impl FromBytes for AntennaNoiseDb {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u8()?;
        Ok(Self { value })
    }
}

/// Quality of Barker code lock, unitless. Monotonically nondecreasing with
/// "better" lock strength. Called "Signal Quality" in datasheets.
#[derive(Debug, Clone, PartialEq)]
pub struct LockQuality {
    pub value: u16,
}

impl FromBytes for LockQuality {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u16::<LE>()?;
        Ok(Self { value })
    }
}

/// Transmit power expressed as unitless distance from max power. 0 is max
/// power. Monotonically nondecreasing with lower power levels.
#[derive(Debug, Clone, PartialEq)]
pub struct TxAttenuation {
    pub value: u16,
}

impl FromBytes for TxAttenuation {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u16::<LE>()?;
        Ok(Self { value })
    }
}

/// Transmit power in dB. 0 is max power. Monotonically nondecreasing with lower
/// power levels.
#[derive(Debug, Clone, PartialEq)]
pub struct TxAttenuationDb {
    pub value: u16,
}

impl FromBytes for TxAttenuationDb {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u16::<LE>()?;
        Ok(Self { value })
    }
}

/// Transmit power in dBm. This is the absolute power level measured at the
/// antenna port.
#[derive(Debug, Clone, PartialEq)]
pub struct TxPower {
    pub value: i8,
}

impl FromBytes for TxPower {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_i8()?;
        Ok(Self { value })
    }
}

/// Indication of the transmit/receive antenna for this frame. The first antenna
/// is antenna 0.
#[derive(Debug, Clone, PartialEq)]
pub struct Antenna {
    pub value: u8,
}

impl FromBytes for Antenna {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u8()?;
        Ok(Self { value })
    }
}

/// Properties of received frames.
#[derive(Debug, Clone, PartialEq)]
pub struct RxFlags {
    pub bad_plcp: bool,
}

impl FromBytes for RxFlags {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let flags = Cursor::new(input).read_u16::<LE>()?;
        Ok(Self {
            bad_plcp: flags.is_flag_set(0x0002),
        })
    }
}

/// Properties of transmitted frames.
#[derive(Debug, Clone, PartialEq)]
pub struct TxFlags {
    /// Transmission failed due to excessive retries.
    pub fail: bool,
    /// Transmission used CTS-to-self protection.
    pub cts: bool,
    /// Transmission used RTS/CTS handshake.
    pub rts: bool,
    /// Transmission shall not expect an ACK frame and not retry when no ACK is
    /// received.
    pub no_ack: bool,
    /// Transmission includes a pre-configured sequence number that should not
    /// be changed by the driver's TX handlers.
    pub no_seq: bool,
}

impl FromBytes for TxFlags {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let flags = Cursor::new(input).read_u8()?;
        Ok(Self {
            fail: flags.is_flag_set(0x0001),
            cts: flags.is_flag_set(0x0002),
            rts: flags.is_flag_set(0x0004),
            no_ack: flags.is_flag_set(0x0008),
            no_seq: flags.is_flag_set(0x0010),
        })
    }
}

/// Number of RTS retries a transmitted frame used.
#[derive(Debug, Clone, PartialEq)]
pub struct RtsRetries {
    pub value: u8,
}

impl FromBytes for RtsRetries {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u8()?;
        Ok(Self { value })
    }
}

/// Number of data retries a transmitted frame used.
#[derive(Debug, Clone, PartialEq)]
pub struct DataRetries {
    pub value: u8,
}

impl FromBytes for DataRetries {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let value = Cursor::new(input).read_u8()?;
        Ok(Self { value })
    }
}

/// Extended channel information.
#[derive(Debug, Clone, PartialEq)]
pub struct XChannel {
    /// The channel flags.
    pub flags: XChannelFlags,
    /// The frequency in MHz.
    pub freq: u16,
    /// The channel number.
    pub channel: u8,
    /// The max power.
    pub max_power: u8,
}

impl FromBytes for XChannel {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        let flags = cursor.read_u32::<LE>()?;
        let freq = cursor.read_u16::<LE>()?;
        let channel = cursor.read_u8()?;
        let max_power = cursor.read_u8()?;
        Ok(Self {
            flags: XChannelFlags {
                turbo: flags.is_flag_set(0x0000_0010),
                cck: flags.is_flag_set(0x0000_0020),
                ofdm: flags.is_flag_set(0x0000_0040),
                ghz2: flags.is_flag_set(0x0000_0080),
                ghz5: flags.is_flag_set(0x0000_0100),
                passive: flags.is_flag_set(0x0000_0200),
                dynamic: flags.is_flag_set(0x0000_0400),
                gfsk: flags.is_flag_set(0x0000_0800),
                gsm: flags.is_flag_set(0x0000_1000),
                sturbo: flags.is_flag_set(0x0000_2000),
                half: flags.is_flag_set(0x0000_4000),
                quarter: flags.is_flag_set(0x0000_8000),
                ht20: flags.is_flag_set(0x0001_0000),
                ht40u: flags.is_flag_set(0x0002_0000),
                ht40d: flags.is_flag_set(0x0004_0000),
            },
            freq,
            channel,
            max_power,
        })
    }
}

/// The IEEE 802.11n data rate index. Usually only one of the
/// [Rate](struct.Rate.html), [MCS](struct.MCS.html), and [VHT] fields is
/// present.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Mcs {
    /// The bandwidth.
    pub bw: Option<Bandwidth>,
    /// The 802.11n MCS index.
    pub index: Option<u8>,
    /// The guard interval.
    pub gi: Option<GuardInterval>,
    /// The HT format.
    pub format: Option<HtFormat>,
    /// The FEC type.
    pub fec: Option<Fec>,
    /// Number of STBC streams.
    pub stbc: Option<u8>,
    /// Number of extension spatial streams.
    pub ness: Option<u8>,
    /// The datarate in Mbps
    pub datarate: Option<f32>,
}

impl FromBytes for Mcs {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        let mut mcs = Self::default();

        let known = cursor.read_u8()?;
        let flags = cursor.read_u8()?;
        let index = cursor.read_u8()?;

        if known.is_flag_set(0x01) {
            mcs.bw = Some(Bandwidth::new(flags & 0x03)?)
        }

        if known.is_flag_set(0x02) {
            mcs.index = Some(index);
        }

        if known.is_flag_set(0x04) {
            mcs.gi = Some(if flags.is_flag_set(0x04) {
                GuardInterval::Short
            } else {
                GuardInterval::Long
            })
        }

        if known.is_flag_set(0x08) {
            mcs.format = Some(if flags.is_flag_set(0x08) {
                HtFormat::Greenfield
            } else {
                HtFormat::Mixed
            });
        }

        if known.is_flag_set(0x10) {
            mcs.fec = Some(if flags.is_flag_set(0x10) {
                Fec::Ldpc
            } else {
                Fec::Bcc
            });
        }

        if known.is_flag_set(0x20) {
            mcs.stbc = Some(flags.bits_as_int(5, 2));
        }

        if known.is_flag_set(0x40) {
            // Yes this is stored weirdly
            mcs.ness = Some(known & 0x80 >> 6 | flags & 0x80 >> 7)
        }

        mcs.datarate = match (&mcs.bw, &mcs.gi) {
            (Some(bw), Some(gi)) => Some(ht_rate(index, bw, gi)?),
            _ => None,
        };

        Ok(mcs)
    }
}

/// The presence of this field indicates that the frame was received as part of
/// an a-MPDU.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AmpduStatus {
    /// The A-MPDU reference number.
    pub reference: u32,
    /// Whether this is a 0-length subframe.
    pub zero_length: Option<bool>,
    /// Whether this is the last subframe of this A-MPDU.
    pub last: Option<bool>,
    /// The A-MPDU subframe delimiter CRC.
    pub delimiter_crc: Option<u8>,
}

impl FromBytes for AmpduStatus {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        let mut ampdu = Self::default();

        ampdu.reference = cursor.read_u32::<LE>()?;
        let flags = cursor.read_u16::<LE>()?;
        let delim_crc = cursor.read_u8()?;

        if flags.is_flag_set(0x0001) {
            ampdu.zero_length = Some(flags.is_flag_set(0x0002));
        }

        if flags.is_flag_set(0x0004) {
            ampdu.last = Some(flags.is_flag_set(0x0008));
        }

        if !flags.is_flag_set(0x0010) && flags.is_flag_set(0x0020) {
            ampdu.delimiter_crc = Some(delim_crc);
        }

        Ok(ampdu)
    }
}

/// The IEEE 802.11ac data rate index. Usually only one of the
/// [Rate](struct.Rate.html), [MCS](struct.MCS.html), and [VHT](struct.VHT.html)
/// fields is present.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Vht {
    /// Whether all spatial streams of all users have STBC.
    pub stbc: Option<bool>,
    /// Whether STAs may not doze during TXOP or transmitter is non-AP.
    pub txop_ps: Option<bool>,
    /// The guard interval.
    pub gi: Option<GuardInterval>,
    /// False if NSYM mod 10 != 9 or short GI not used. True if NSYM mod 10 = 9.
    pub sgi_nsym_da: Option<bool>,
    /// Whether one or more users are using LDPC and the encoding process
    /// resulted in extra OFDM symbol(s).
    pub ldpc_extra: Option<bool>,
    /// The frame was transmitted/received using beamforming.
    pub beamformed: Option<bool>,
    /// The bandwidth.
    pub bw: Option<Bandwidth>,
    /// The Group ID of the frame.
    pub group_id: Option<u8>,
    /// A non-unique identifier of a STA to identify whether the transmissions
    /// are destined to a STA or not, used in conjunction with GroupID.
    pub partial_aid: Option<u16>,
    /// The users for the current group.
    pub users: [Option<VHTUser>; 4],
}

impl FromBytes for Vht {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);
        let mut vht = Self::default();

        let known = cursor.read_u16::<LE>()?;
        let flags = cursor.read_u8()?;
        let bandwidth = cursor.read_u8()?;
        let mut mcs_nss = [0; 4];
        cursor.read_exact(&mut mcs_nss)?;
        let coding = cursor.read_u8()?;
        let group_id = cursor.read_u8()?;
        let partial_aid = cursor.read_u16::<LE>()?;

        if known.is_flag_set(0x0001) {
            vht.stbc = Some(flags.is_flag_set(0x01));
        }

        if known.is_flag_set(0x0002) {
            vht.txop_ps = Some(flags.is_flag_set(0x02));
        }

        if known.is_flag_set(0x0004) {
            vht.gi = Some(if flags & 0x04 > 0 {
                GuardInterval::Short
            } else {
                GuardInterval::Long
            })
        }

        if known.is_flag_set(0x0008) {
            vht.sgi_nsym_da = Some(flags.is_flag_set(0x08));
        }

        if known.is_flag_set(0x0010) {
            vht.ldpc_extra = Some(flags.is_flag_set(0x10));
        }

        if known.is_flag_set(0x0020) {
            vht.beamformed = Some(flags.is_flag_set(0x20));
        }

        if known.is_flag_set(0x0040) {
            vht.bw = Some(Bandwidth::new(bandwidth & 0x1f)?)
        }

        if known.is_flag_set(0x0080) {
            vht.group_id = Some(group_id);
        }

        if known.is_flag_set(0x0100) {
            vht.partial_aid = Some(partial_aid);
        }

        for (i, user) in mcs_nss.iter().enumerate() {
            let nss = user & 0x0f;

            if nss == 0 {
                continue;
            }

            let index = (user & 0xf0) >> 4;
            let nsts = nss << (flags & 0x01);
            let id = i as u8;

            let datarate = match (&vht.bw, &vht.gi) {
                (Some(bw), Some(gi)) => Some(vht_rate(index, bw, gi, nss)?),
                _ => None,
            };

            vht.users[id as usize] = Some(VHTUser {
                index,
                fec: match (coding & 2 ^ id) >> id {
                    1 => Fec::Ldpc,
                    _ => Fec::Bcc,
                },
                nss,
                nsts,
                datarate,
            });
        }

        Ok(vht)
    }
}

/// The time the frame was transmitted or received.
#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp {
    /// The actual timestamp.
    pub timestamp: u64,
    /// The unit of the timestamp value.
    pub unit: TimeUnit,
    /// The sampling position of the timestamp.
    pub position: SamplingPosition,
    /// The accuracy of the timestamp.
    pub accuracy: Option<u16>,
}

impl FromBytes for Timestamp {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.read_u64::<LE>()?;
        let mut accuracy = Some(cursor.read_u16::<LE>()?);
        let unit_position = cursor.read_u8()?;
        let unit = TimeUnit::new(unit_position & 0x0f)?;
        let position = SamplingPosition::from(unit_position & 0xf0 >> 4)?;
        let flags = cursor.read_u8()?;

        if !flags.is_flag_set(0x02) {
            accuracy = None;
        }

        Ok(Self {
            timestamp,
            unit,
            position,
            accuracy,
        })
    }
}
