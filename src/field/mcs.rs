//! Defines the MCS field.

use std::result;

use thiserror::Error;

use crate::field::{Fec, Field, FromArray, GuardInterval};

#[rustfmt::skip]
const RATE: [[f32; 4]; 32] = [
    //      20 MHz          40 MHz
    //   LGI     SGI     LGI     SGI
    [    6.5,    7.2,   13.5,   15.0  ], // MCS 0
    [   13.0,   14.4,   27.0,   30.0  ],
    [   19.5,   21.7,   40.5,   45.0  ],
    [   26.0,   28.9,   54.0,   60.0  ],
    [   39.0,   43.3,   81.0,   90.0  ],
    [   52.0,   57.8,  108.0,  120.0  ],
    [   58.5,   65.0,  121.5,  135.0  ],
    [   65.0,   72.2,  135.0,  150.0  ],

    [   13.0,   14.4,   27.0,   30.0  ], // MCS 8
    [   26.0,   28.9,   54.0,   60.0  ],
    [   39.0,   43.3,   81.0,   90.0  ],
    [   52.0,   57.8,  108.0,  120.0  ],
    [   78.0,   86.7,  162.0,  180.0  ],
    [  104.0,  115.6,  216.0,  240.0  ],
    [  117.0,  130.0,  243.0,  270.0  ],
    [  130.0,  144.4,  270.0,  300.0  ],

    [   19.5,   21.7,   40.5,   45.0  ], // MCS 16
    [   39.0,   43.3,   81.0,   90.0  ],
    [   58.5,   65.0,  121.5,  135.0  ],
    [   78.0,   86.7,  162.0,  180.0  ],
    [  117.0,  130.0,  243.0,  270.0  ],
    [  156.0,  173.3,  324.0,  360.0  ],
    [  175.5,  195.0,  364.5,  405.0  ],
    [  195.0,  216.7,  405.0,  450.0  ],

    [   26.0,   28.8,   54.0,   60.0  ], // MCS 24
    [   52.0,   57.6,  108.0,  120.0  ],
    [   78.0,   86.8,  162.0,  180.0  ],
    [  104.0,  115.6,  216.0,  240.0  ],
    [  156.0,  173.2,  324.0,  360.0  ],
    [  208.0,  231.2,  432.0,  480.0  ],
    [  234.0,  260.0,  486.0,  540.0  ],
    [  260.0,  288.8,  540.0,  600.0  ],
];

/// An error returned when parsing the datarate.
#[derive(Debug, Error)]
#[error("invalid MCS index `{0}`")]
pub struct InvalidDatarate(u8);

impl_enum! {
    /// The bandwidth.
    #[allow(
        clippy::unknown_clippy_lints,
        clippy::upper_case_acronyms,
    )]
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

bitflags! {
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

bitflags! {
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
/// Other rate fields: [`Rate`][super::Rate], [`Vht`][super::Vht]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 3)]
pub struct Mcs {
    /// Indicates which information is known.
    #[field(size = 1)]
    known: Known,
    /// Contains various encoded information.
    #[field(size = 1)]
    flags: Flags,
    /// The MCS index.
    index: u8,
}

impl Bandwidth {
    /// Returns the bandwidth in MHz.
    pub fn to_mhz(&self) -> u8 {
        match self {
            Self::BW20 => 20,
            Self::BW40 => 40,
            Self::BW20L => 40,
            Self::BW20U => 40,
        }
    }
}

impl From<bool> for Format {
    fn from(b: bool) -> Self {
        match b {
            false => Self::Mixed,
            true => Self::Greenfield,
        }
    }
}

impl Mcs {
    /// Returns the bandwidth.
    pub fn bandwidth(&self) -> Option<Bandwidth> {
        self.known
            .contains(Known::BW)
            .then(|| Bandwidth::from_bits((self.flags & Flags::BW_MASK).bits).unwrap())
    }

    /// Returns the MCS index.
    pub fn index(&self) -> Option<u8> {
        self.known.contains(Known::INDEX).then(|| self.index)
    }

    /// Returns the guard interval.
    pub fn guard_interval(&self) -> Option<GuardInterval> {
        self.known
            .contains(Known::GI)
            .then(|| GuardInterval::from_bool(self.flags.contains(Flags::GI)))
    }

    /// Returns the HT format.
    pub fn format(&self) -> Option<Format> {
        self.known
            .contains(Known::FMT)
            .then(|| self.flags.contains(Flags::FMT).into())
    }

    /// Returns the FEC type.
    pub fn fec(&self) -> Option<Fec> {
        self.known
            .contains(Known::FEC)
            .then(|| Fec::from_bool(self.flags.contains(Flags::FEC)))
    }

    /// Returns the number of STBCs.
    pub fn stbc(&self) -> Option<u8> {
        self.known
            .contains(Known::STBC)
            .then(|| (self.flags & Flags::STBC_MASK).bits >> STBC_SHIFT)
    }

    /// Return the number of extension spatial streams.
    pub fn ness(&self) -> Option<u8> {
        self.known.contains(Known::NESS).then(|| {
            (self.known & Known::NESS_BIT_1).bits >> NESS_BIT_1_SHIFT
                | (self.flags & Flags::NESS_BIT_0).bits >> NESS_BIT_0_SHIFT
        })
    }

    /// Returns the number of spatial streams (1 - 4) calculated using the MCS
    /// index.
    pub fn nss(&self) -> Option<u8> {
        self.index().map(|index| (index / 8) + 1)
    }

    /// Returns the data rate in megabits per second.
    pub fn to_mbps(&self) -> Option<result::Result<f32, InvalidDatarate>> {
        let index = self.index()?;
        if index > 31 {
            return Some(Err(InvalidDatarate(index)));
        }
        let row: usize = index.into();
        let b = match self.bandwidth()?.to_mhz() {
            20 => 0,
            40 => 2,
            _ => unreachable!(),
        };
        let col: usize = (b + self.guard_interval()?.into_inner()).into();
        Some(Ok(RATE[row][col]))
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

    use crate::assert_eq_f32;
    use crate::hex::FromHex;

    #[test]
    fn basic() {
        let mcs = Mcs::from_hex("1f0407");
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
        assert_eq_f32!(mcs.to_mbps().unwrap().unwrap(), 72.2);
    }

    #[test]
    fn datarate() {
        let mcs = Mcs::from_hex("1f140f");
        assert_eq_f32!(mcs.to_mbps().unwrap().unwrap(), 144.4);
    }
}
