//! Defines the MCS field.

use crate::field::{Fec, GuardInterval};
use crate::prelude::*;

impl_enum! {
    /// The bandwidth.
    pub enum Bandwidth: u8 {
        BW20 = 0,
        BW40 = 1,
        BW20L = 2,
        BW20U = 3,
    }
}

impl_enum! {
    /// The HT format.
    pub enum Format: u8 {
        Mixed = 0,
        Greenfield = 1,
    }
}

impl_bitflags! {
    /// Indicates what MCS information is known.
    pub struct Known: u8 {
        /// The bandwidth is known.
        const BW = 0x01;
        /// The MCS index is known.
        const INDEX = 0x02;
        /// The guard interval is known.
        const GI = 0x04;
        /// The HT format is known.
        const FMT = 0x08;
        /// The FEC type is known.
        const FEC = 0x10;
        /// The number of STBC streams is known.
        const STBC = 0x20;
        /// The number of extension spatial streams is known.
        const NESS = 0x40;
        /// Number of extension spatial streams value (bit 1).
        const NESS_BIT_1 = 0x80;
    }
}

impl_bitflags! {
    /// Flags describing the MCS information.
    pub struct Flags: u8 {
        /// Encodes the bandwidth (bit 0).
        const BW_BIT_0 = 0x01;
        /// Encodes the bandwidth (bit 1).
        const BW_BIT_1 = 0x02;
        /// The bandwidth mask.
        const BW_MASK = Self::BW_BIT_0.bits | Self::BW_BIT_1.bits;
        /// Encodes the guard interval.
        const GI = 0x04;
        /// Encodes the HT format.
        const FMT = 0x08;
        /// Encodes the FEC type.
        const FEC = 0x10;
        /// Encodes the number of STBC streams (bit 0).
        const STBC_BIT_0 = 0x20;
        /// Encodes the number of STBC streams (bit 1).
        const STBC_BIT_1 = 0x40;
        /// The STBC mask.
        const STBC_MASK = Self::STBC_BIT_0.bits | Self::STBC_BIT_1.bits;
        /// Number of extension spatial streams value (bit 0).
        const NESS_BIT_0 = 0x80;
    }
}

const STBC_SHIFT: u8 = 5;
const NESS_BIT_1_SHIFT: u8 = 6;
const NESS_BIT_0_SHIFT: u8 = 5;

/// The MCS information.
///
/// The IEEE 802.11n data rate index.
///
/// Other rate fields: [Rate](../struct.Rate.html),
/// [VHT](../vht/struct.Vht.html)
#[derive(Debug, Clone, PartialEq)]
pub struct Mcs {
    /// Indicates which information is known.
    known: Known,
    /// Contains various encoded information.
    flags: Flags,
    /// The MCS index.
    index: u8,
}

impl From<bool> for Format {
    fn from(b: bool) -> Self {
        match b {
            false => Self::Mixed,
            true => Self::Greenfield,
        }
    }
}

impl FromBytes for Mcs {
    type Error = Error;

    fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
        let known = bytes.read()?;
        let flags = bytes.read()?;
        let index = bytes.read()?;
        Ok(Self {
            known,
            flags,
            index,
        })
    }
}

impl Mcs {
    /// Returns the bandwidth.
    pub fn bandwidth(&self) -> Option<Bandwidth> {
        self.known
            .contains(Known::BW)
            .some(|| Bandwidth::from_bits((self.flags & Flags::BW_MASK).bits).unwrap())
    }

    /// Returns the MCS index.
    pub fn index(&self) -> Option<u8> {
        self.known.contains(Known::INDEX).some(|| self.index)
    }

    /// Returns the guard interval.
    pub fn guard_interval(&self) -> Option<GuardInterval> {
        self.known
            .contains(Known::GI)
            .some(|| self.flags.contains(Flags::GI).into())
    }

    /// Returns the HT format.
    pub fn format(&self) -> Option<Format> {
        self.known
            .contains(Known::FMT)
            .some(|| self.flags.contains(Flags::FMT).into())
    }

    /// Returns the FEC type.
    pub fn fec(&self) -> Option<Fec> {
        self.known
            .contains(Known::FEC)
            .some(|| self.flags.contains(Flags::FEC).into())
    }

    /// Returns the number of STBCs.
    pub fn stbc(&self) -> Option<u8> {
        self.known
            .contains(Known::STBC)
            .some(|| (self.flags & Flags::STBC_MASK).bits >> STBC_SHIFT)
    }

    /// Return the number of extension spatial streams.
    pub fn ness(&self) -> Option<u8> {
        self.known.contains(Known::NESS).some(|| {
            (self.known & Known::NESS_BIT_1).bits >> NESS_BIT_1_SHIFT
                | (self.flags & Flags::NESS_BIT_0).bits >> NESS_BIT_0_SHIFT
        })
    }

    /// Returns the raw known information.
    pub const fn known(&self) -> Known {
        self.known
    }

    /// Returns the raw flags.
    pub const fn flags(&self) -> Flags {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mcs = Mcs::from_hex("1f0407").unwrap();
        assert_eq!(
            mcs,
            Mcs {
                known: Known::BW | Known::INDEX | Known::GI | Known::FMT | Known::FEC,
                flags: Flags::GI,
                index: 7
            }
        );
        assert_eq!(mcs.bandwidth(), Some(Bandwidth::BW20));
        assert_eq!(mcs.index(), Some(7));
        assert_eq!(mcs.guard_interval(), Some(GuardInterval::Short));
        assert_eq!(mcs.format(), Some(Format::Mixed));
        assert_eq!(mcs.fec(), Some(Fec::Bcc));
        assert_eq!(mcs.stbc(), None);
        assert_eq!(mcs.ness(), None);
    }
}
