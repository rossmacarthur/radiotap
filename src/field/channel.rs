//! Defines the Channel field.

use crate::field::splice;

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

impl From<[u8; 4]> for Channel {
    fn from(bytes: [u8; 4]) -> Self {
        let freq = u16::from_le_bytes(splice(bytes, 0));
        let flags = Flags::from(splice(bytes, 2));
        Self { freq, flags }
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

    use crate::hex::FromHex;

    #[test]
    fn basic() {
        assert_eq!(
            Channel::from_hex("9e098004"),
            Channel {
                freq: 2462,
                flags: Flags::GHZ2 | Flags::DYNAMIC
            }
        );
    }
}
