//! Defines the MCS field.

use super::*;

use crate::util::BoolExt;

/// The bandwidth.
#[derive(Debug, Clone, PartialEq)]
pub enum Bandwidth {
    BW20,
    BW40,
    BW20L,
    BW20U,
}

/// The HT format.
#[derive(Debug, Clone, PartialEq)]
pub enum HtFormat {
    Mixed,
    Greenfield,
}

/// Forward error correction type.
#[derive(Debug, Clone, PartialEq)]
pub enum Fec {
    /// Binary convolutional coding.
    Bcc,
    /// Low-density parity-check.
    Ldpc,
}

impl_bitflags! {
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
/// The IEEE 802.11n data rate index. Usually only one of the
/// [Rate](../struct.Rate.html), [MCS](struct.Mcs.html), and
/// [VHT](../vht/struct.Vht.html) fields is present.
///
/// [Reference](http://www.radiotap.org/fields/MCS.html)
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
    /// The bandwidth.
    pub fn bandwidth(&self) -> Option<Bandwidth> {
        self.known
            .contains(Known::BW)
            .into_option(|| match self.flags & 0b11 {
                0 => Bandwidth::BW20,
                1 => Bandwidth::BW40,
                2 => Bandwidth::BW20L,
                3 => Bandwidth::BW20U,
                _ => unreachable!(),
            })
    }

    /// The guard interval.
    pub fn guard_interval(&self) -> Option<GuardInterval> {
        self.known.contains(Known::GI).into_option(|| {
            if self.flags & 0x04 > 0 {
                GuardInterval::Short
            } else {
                GuardInterval::Long
            }
        })
    }

    /// The HT format.
    pub fn ht_format(&self) -> Option<HtFormat> {
        self.known.contains(Known::FMT).into_option(|| {
            if self.flags & 0x08 > 0 {
                HtFormat::Greenfield
            } else {
                HtFormat::Mixed
            }
        })
    }

    /// The FEC type.
    pub fn fec(&self) -> Option<Fec> {
        self.known.contains(Known::FEC).into_option(|| {
            if self.flags & 0x10 > 0 {
                Fec::Ldpc
            } else {
                Fec::Bcc
            }
        })
    }

    /// Returns the number of STBCs.
    pub fn stbc(&self) -> Option<u8> {
        self.known
            .contains(Known::STBC)
            .into_option(|| self.flags & 0x60 >> 5)
    }

    /// Return the number of extension spatial streams.
    pub fn ness(&self) -> Option<u8> {
        self.known
            .contains(Known::NESS)
            .into_option(|| self.known.bits() & 0x80 >> 6 | self.flags & 0x80 >> 7)
    }
}
