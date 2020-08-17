//! Defines the XChannel field.

use super::*;

bitflags! {
    /// Extended flags describing the channel.
    pub struct Flags: u32 {
        /// Turbo channel.
        const TURBO = 0x0000_0010;
        /// Complementary Code Keying (CCK) channel.
        const CCK = 0x0000_0020;
        /// Orthogonal Frequency-Division Multiplexing (OFDM) channel.
        const OFDM = 0x0000_0040;
        /// 2 GHz spectrum channel.
        const GHZ_2 = 0x0000_0080;
        /// 5 GHz spectrum channel.
        const GHZ_5 = 0x0000_0100;
        /// Only passive scan allowed.
        const PASSIVE = 0x0000_0200;
        /// Dynamic CCK-OFDM channel.
        const DYNAMIC = 0x0000_0400;
        /// Gaussian Frequency Shift Keying (GFSK) channel.
        const GFSK = 0x0000_0800;
        /// GSM channel.
        const GSM = 0x0000_1000;
        /// Static Turbo channel.
        const TURBO_S = 0x0000_2000;
        /// Half rate channel.
        const HALF = 0x0000_4000;
        /// Quarter rate channel.
        const QUARTER = 0x0000_8000;
        /// HT Channel (20MHz Channel Width).
        const HT20 = 0x0001_0000;
        /// HT Channel (40MHz Channel Width with Extension channel above).
        const HT40U = 0x0002_0000;
        /// HT Channel (40MHz Channel Width with Extension channel below).
        const HT40D = 0x0004_0000;
    }
}

/// Extended channel information.
///
/// [Reference](https://www.radiotap.org/fields/XChannel.html)
#[derive(Debug, Clone, PartialEq)]
pub struct XChannel {
    /// The channel flags.
    flags: Flags,
    /// The frequency in MHz.
    freq: u16,
    /// The channel number.
    channel: u8,
    /// The max power.
    max_power: u8,
}

impl_from_bytes_bitflags!(Flags);

impl FromBytes for XChannel {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        ensure_length!(bytes.len() == Kind::XChannel.size());
        let flags = bytes[0..4].try_read()?;
        let freq = bytes[4..6].try_read()?;
        let channel = bytes[6..7].try_read()?;
        let max_power = bytes[7..8].try_read()?;
        Ok(Self {
            flags,
            freq,
            channel,
            max_power,
        })
    }
}
