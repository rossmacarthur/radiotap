//! Defines the Channel field.

use crate::prelude::*;

impl_bitflags! {
    /// Flags describing the channel.
    pub struct Flags: u16 {
        /// Turbo channel.
        const TURBO = 0x0010;
        /// Complementary Code Keying (CCK) channel.
        const CCK = 0x0020;
        /// Orthogonal Frequency-Division Multiplexing (OFDM) channel.
        const OFDM = 0x0040;
        /// 2 GHz spectrum channel.
        const GHZ2 = 0x0080;
        /// 5 GHz spectrum channel.
        const GHZ5 = 0x0100;
        /// Only passive scan allowed.
        const PASSIVE = 0x0200;
        /// Dynamic CCK-OFDM channel.
        const DYNAMIC = 0x0400;
        /// Gaussian Frequency Shift Keying (GFSK) channel.
        const GFSK = 0x0800;
        /// GSM (900MHz) channel.
        const GSM = 0x1000;
        /// Static Turbo channel.
        const STURBO = 0x2000;
        /// Half rate channel.
        const HALF = 0x4000;
        /// Quarter rate channel.
        const QUARTER = 0x8000;
    }
}

/// Channel information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel {
    freq: u16,
    flags: Flags,
}

impl FromBytes for Channel {
    type Error = Error;

    fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
        let freq = bytes.read()?;
        let flags = bytes.read()?;
        Ok(Self { freq, flags })
    }
}

impl Channel {
    /// Returns the channel frequency in MHz.
    pub const fn freq(&self) -> u16 {
        self.freq
    }

    /// Returns flags describing the channel.
    pub const fn flags(&self) -> Flags {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(
            Channel::from_hex("9e098004").unwrap(),
            Channel {
                freq: 2462,
                flags: Flags::GHZ2 | Flags::DYNAMIC
            }
        );
    }
}
