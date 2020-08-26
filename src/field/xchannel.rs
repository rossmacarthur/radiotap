//! Defines the XChannel field.

use crate::prelude::*;

impl_bitflags! {
    /// Extended flags describing the channel.
    pub struct Flags: u32 {
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
        /// Half rate channel (10 MHz Channel Width).
        const HALF = 0x4000;
        /// Quarter rate channel (5 MHz Channel Width).
        const QUARTER = 0x8000;
        /// HT Channel (20 MHz Channel Width).
        const HT20 = 0x0001_0000;
        /// HT Channel (40 MHz Channel Width with Extension channel above).
        const HT40U = 0x0002_0000;
        /// HT Channel (40 MHz Channel Width with Extension channel below).
        const HT40D = 0x0004_0000;
    }
}

/// Extended channel information.
#[derive(Debug, Clone, PartialEq)]
pub struct XChannel {
    flags: Flags,
    freq: u16,
    channel: u8,
}

impl FromBytes for XChannel {
    type Error = Error;

    fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
        let flags = bytes.read()?;
        let freq = bytes.read()?;
        let channel = bytes.read()?;
        Ok(Self {
            flags,
            freq,
            channel,
        })
    }
}

impl XChannel {
    /// Returns flags describing the channel.
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Returns the channel frequency in MHz.
    pub fn freq(&self) -> u16 {
        self.freq
    }

    /// Returns the channel number.
    pub fn channel(&self) -> u8 {
        self.channel
    }
}
