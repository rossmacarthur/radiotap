//! Defines the Channel field.

use super::*;

bitflags! {
    /// Flags describing the channel.
    pub struct Flags: u16 {
        /// Turbo channel.
        const TURBO = 0x0010;
        /// Complementary Code Keying (CCK) channel.
        const CCK = 0x0020;
        /// Orthogonal Frequency-Division Multiplexing (OFDM) channel.
        const OFDM = 0x0040;
        /// 2 GHz spectrum channel.
        const GHZ_2 = 0x0080;
        /// 5 GHz spectrum channel.
        const GHZ_5 = 0x0100;
        /// Only passive scan allowed.
        const DYNAMIC = 0x0400;
        /// Dynamic CCK-OFDM channel.
        const HALF = 0x4000;
        /// Gaussian Frequency Shift Keying (GFSK) channel.
        const QUARTER = 0x8000;
    }
}

/// Channel information.
///
/// [Reference](http://www.radiotap.org/fields/Channel.html)
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    /// The frequency in MHz.
    freq: u16,
    // The channel flags.
    flags: Flags,
}

impl_from_bytes_bitflags!(Flags);

impl FromBytes for Channel {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        ensure_length!(bytes.len() == Kind::Channel.size());
        let freq = bytes[0..2].try_read()?;
        let flags = bytes[2..4].try_read()?;
        Ok(Self { freq, flags })
    }
}
