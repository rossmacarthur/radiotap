//! Defines the MCS field.

use super::*;

use crate::util::BoolExt;

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
        const MCS = 0x02;
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
        /// Bit 1 of extension spatial streams value.
        const NESS_BIT_1 = 0x80;
    }
}

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
    flags: u8,
    /// The MCS index.
    index: u8,
}

impl FromBytes for Mcs {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        ensure_length!(bytes.len() == Kind::Mcs.size());
        let known = bytes[0..1].try_read()?;
        let flags = bytes[1..2].try_read()?;
        let index = bytes[2..3].try_read()?;
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
            .some(|| Bandwidth::from_bits(self.flags & 0b11).unwrap())
    }

    /// Returns the guard interval.
    pub fn guard_interval(&self) -> Option<GuardInterval> {
        self.known
            .contains(Known::GI)
            .some(|| GuardInterval::from_bits((self.flags & 0x04) >> 2).unwrap())
    }

    /// Returns the HT format.
    pub fn format(&self) -> Option<Format> {
        self.known
            .contains(Known::FMT)
            .some(|| Format::from_bits((self.flags & 0x08) >> 3).unwrap())
    }

    /// Returns the FEC type.
    pub fn fec(&self) -> Option<Fec> {
        self.known
            .contains(Known::FEC)
            .some(|| Fec::from_bits((self.flags & 0x10) >> 4).unwrap())
    }

    /// Returns the number of STBCs.
    pub fn stbc(&self) -> Option<u8> {
        self.known
            .contains(Known::STBC)
            .some(|| (self.flags & 0x60) >> 5)
    }

    /// Return the number of extension spatial streams.
    pub fn ness(&self) -> Option<u8> {
        self.known
            .contains(Known::NESS)
            .some(|| (self.known.bits() & 0x80) >> 6 | (self.flags & 0x80) >> 7)
    }
}
