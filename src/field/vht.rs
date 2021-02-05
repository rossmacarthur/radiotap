//! Defines the VHT field.

use std::f32::NAN;
use std::result;

use thiserror::Error;

use crate::field::mcs;
use crate::field::{Fec, GuardInterval};
use crate::prelude::*;

#[rustfmt::skip]
const RATE: [[f32; 8]; 80] = [
    //      20 MHz          40 MHz           80 MHz            160 MHz
    //   LGI     SGI     LGI     SGI      LGI      SGI      LGI      SGI
    [    6.5,    7.2,   13.5,   15.0,    29.3,    32.5,    58.5,    65.0  ],
    [   13.0,   14.4,   27.0,   30.0,    58.5,    65.0,   117.0,   130.0  ],
    [   19.5,   21.7,   40.5,   45.0,    87.8,    97.5,   175.5,   195.0  ],
    [   26.0,   28.9,   54.0,   60.0,   117.0,   130.0,   234.0,   260.0  ],
    [   39.0,   43.3,   81.0,   90.0,   175.5,   195.0,   351.0,   390.0  ],
    [   52.0,   57.8,  108.0,  120.0,   234.0,   260.0,   468.0,   520.0  ],
    [   58.5,   65.0,  121.5,  135.0,   263.3,   292.5,   526.5,   585.0  ],
    [   65.0,   72.2,  135.0,  150.0,   292.5,   325.0,   585.0,   650.0  ],
    [   78.0,   86.7,  162.0,  180.0,   351.0,   390.0,   702.0,   780.0  ],
    [    NAN,    NAN,  180.0,  200.0,   390.0,   433.3,   780.0,   866.7  ],

    [   13.0,   14.4,   27.0,   30.0,    58.5,    65.0,   117.0,   130.0  ],
    [   26.0,   28.9,   54.0,   60.0,   117.0,   130.0,   234.0,   260.0  ],
    [   39.0,   43.3,   81.0,   90.0,   175.5,   195.0,   351.0,   390.0  ],
    [   52.0,   57.8,  108.0,  120.0,   234.0,   260.0,   468.0,   520.0  ],
    [   78.0,   86.7,  162.0,  180.0,   351.0,   390.0,   702.0,   780.0  ],
    [  104.0,  115.6,  216.0,  240.0,   468.0,   520.0,   936.0,  1040.0  ],
    [  117.0,  130.3,  243.0,  270.0,   526.5,   585.0,  1053.0,  1170.0  ],
    [  130.0,  144.4,  270.0,  300.0,   585.0,   650.0,  1170.0,  1300.0  ],
    [  156.0,  173.3,  324.0,  360.0,   702.0,   780.0,  1404.0,  1560.0  ],
    [   NAN,     NAN,  360.0,  400.0,   780.0,   866.7,  1560.0,  1733.3  ],

    [   19.5,   21.7,   40.5,   45.0,    87.8,    97.5,   175.5,   195.0  ],
    [   39.0,   43.3,   81.0,   90.0,   175.5,   195.0,   351.0,   390.0  ],
    [   58.5,   65.0,  121.5,  135.0,   263.3,   292.5,   526.5,   585.0  ],
    [   78.0,   86.7,  162.0,  180.0,   351.0,   390.0,   702.0,   780.0  ],
    [  117.0,  130.0,  243.0,  270.0,   526.5,   585.0,  1053.0,  1170.0  ],
    [  156.0,  173.3,  324.0,  360.0,   702.0,   780.0,  1404.0,  1560.0  ],
    [  175.5,  195.0,  364.5,  405.0,     NAN,     NAN,  1579.5,  1755.0  ],
    [  195.0,  216.7,  405.0,  450.0,   877.5,   975.0,  1755.0,  1950.0  ],
    [  234.0,  260.0,  486.0,  540.0,  1053.0,  1170.0,  2106.0,  2340.0  ],
    [  260.0,  288.9,  540.0,  600.0,  1170.0,  1300.0,     NAN,     NAN  ],

    [   26.0,   28.9,   54.0,   60.0,   117.0,   130.0,   234.0,   260.0  ],
    [   52.0,   57.8,  108.0,  120.0,   234.0,   260.0,   468.0,   520.0  ],
    [   78.0,   86.7,  162.0,  180.0,   351.0,   390.0,   702.0,   780.0  ],
    [  104.0,  115.6,  216.0,  240.0,   468.0,   520.0,   936.0,  1040.0  ],
    [  156.0,  173.3,  324.0,  360.0,   702.0,   780.0,  1404.0,  1560.0  ],
    [  208.0,  231.1,  432.0,  480.0,   936.0,  1040.0,  1872.0,  2080.0  ],
    [  234.0,  260.0,  486.0,  540.0,  1053.0,  1170.0,  2106.0,  2340.0  ],
    [  260.0,  288.9,  540.0,  600.0,  1170.0,  1300.0,  2340.0,  2600.0  ],
    [  312.0,  346.7,  648.0,  720.0,  1404.0,  1560.0,  2808.0,  3120.0  ],
    [    NAN,    NAN,  720.0,  800.0,  1560.0,  1733.3,  3120.0,  3466.7  ],

    [    NAN,    NAN,    NAN,    NAN,   146.3,   162.5,   292.5,   325.0  ],
    [    NAN,    NAN,    NAN,    NAN,   292.5,   325.0,   585.0,   650.0  ],
    [    NAN,    NAN,    NAN,    NAN,   438.8,   487.5,   877.5,   975.0  ],
    [    NAN,    NAN,    NAN,    NAN,   585.0,   650.0,  1170.0,  1300.0  ],
    [    NAN,    NAN,    NAN,    NAN,   877.5,   975.0,  1755.0,  1950.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1170.0,  1300.0,  2340.0,  2600.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1316.3,  1462.5,  2632.5,  2925.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1462.5,  1625.0,  2925.0,  3250.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1755.0,  1950.0,  3510.0,  3900.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1950.0,  2166.7,  3900.0,  4333.3  ],

    [    NAN,    NAN,    NAN,    NAN,   175.5,   195.0,   351.0,   390.0  ],
    [    NAN,    NAN,    NAN,    NAN,   351.0,   390.0,   702.0,   780.0  ],
    [    NAN,    NAN,    NAN,    NAN,   526.5,   585.0,  1053.0,  1170.0  ],
    [    NAN,    NAN,    NAN,    NAN,   702.0,   780.0,  1404.0,  1560.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1053.0,  1170.0,  2106.0,  2340.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1404.0,  1560.0,  2808.0,  3120.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1579.5,  1755.0,  3159.0,  3510.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1755.0,  1950.0,  3510.0,  3900.0  ],
    [    NAN,    NAN,    NAN,    NAN,  2106.0,  2340.0,  4212.0,  4680.0  ],
    [    NAN,    NAN,    NAN,    NAN,     NAN,     NAN,  4680.0,  5200.0  ],

    [    NAN,    NAN,    NAN,    NAN,   204.8,   227.5,   409.5,   455.0  ],
    [    NAN,    NAN,    NAN,    NAN,   409.5,   455.0,   819.0,   910.0  ],
    [    NAN,    NAN,    NAN,    NAN,   614.3,   682.5,  1228.5,  1365.0  ],
    [    NAN,    NAN,    NAN,    NAN,   819.0,   910.0,  1638.0,  1820.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1228.5,  1365.0,  2457.0,  2730.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1638.0,  1820.0,  3276.0,  3640.0  ],
    [    NAN,    NAN,    NAN,    NAN,     NAN,     NAN,  3685.5,  4095.0  ],
    [    NAN,    NAN,    NAN,    NAN,  2047.5,  2275.0,  4095.0,  4550.0  ],
    [    NAN,    NAN,    NAN,    NAN,  2457.0,  2730.0,  4914.0,  5460.0  ],
    [    NAN,    NAN,    NAN,    NAN,  2730.0,  3033.3,  5460.0,  6066.7  ],

    [    NAN,    NAN,    NAN,    NAN,   234.0,   260.0,   468.0,   520.0  ],
    [    NAN,    NAN,    NAN,    NAN,   468.0,   520.0,   936.0,  1040.0  ],
    [    NAN,    NAN,    NAN,    NAN,   702.0,   780.0,  1404.0,  1560.0  ],
    [    NAN,    NAN,    NAN,    NAN,   936.0,  1040.0,  1872.0,  2080.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1404.0,  1560.0,  2808.0,  3120.0  ],
    [    NAN,    NAN,    NAN,    NAN,  1872.0,  2080.0,  3744.0,  4160.0  ],
    [    NAN,    NAN,    NAN,    NAN,  2106.0,  2340.0,  4212.0,  4680.0  ],
    [    NAN,    NAN,    NAN,    NAN,  2340.0,  2600.0,  4680.0,  5200.0  ],
    [    NAN,    NAN,    NAN,    NAN,  2808.0,  3120.0,  5616.0,  6240.0  ],
    [    NAN,    NAN,    NAN,    NAN,  3120.0,  3466.7,  6240.0,  6933.3  ],
];

/// An error returned when parsing a [`Bandwidth`](enum.Bandwidth.html) from the
/// raw bits in [`Vht.bandwidth()`](struct.Vht.html#method.bandwidth).
#[derive(Debug, Error)]
#[error("failed to parse bandwidth from value `{0:x}`")]
pub struct InvalidBandwidth(u8);

#[derive(Debug, Error)]
enum InvalidDatarateKind {
    /// The MCS index is invalid.
    #[error("invalid MCS index `{0}`")]
    Index(u8),
    /// The NSS is invalid.
    #[error("invalid NSS `{0}`")]
    Nss(u8),
    /// Failed to parse the bandwidth.
    #[error(transparent)]
    Bandwidth(#[from] InvalidBandwidth),
    /// The MCS index, guard interval, NSS and bandwidth combination is invalid.
    #[error("invalid MCS index, guard interval, NSS, and bandwidth combination")]
    Mismatch,
}

/// An error returned when parsing the datarate in
/// [`.to_mbps()`](struct.User.html#method.to_mbps).
#[derive(Debug, Error)]
#[error(transparent)]
pub struct InvalidDatarate {
    #[from]
    kind: InvalidDatarateKind,
}

impl_enum! {
    /// The bandwidth.
    pub enum Bandwidth: u8 {
        BW20 = 0,
        BW40 = 1,
        BW20L = 2,
        BW20U = 3,
        BW80 = 4,
        BW40L = 5,
        BW40U = 6,
        BW20LL = 7,
        BW20LU = 8,
        BW20UL = 9,
        BW20UU = 10,
        BW160 = 11,
        BW80L = 12,
        BW80U = 13,
        BW40LL = 14,
        BW40LU = 15,
        BW40UL = 16,
        BW40UU = 17,
        BW20LLL = 18,
        BW20LLU = 19,
        BW20LUL = 20,
        BW20LUU = 21,
        BW20ULL = 22,
        BW20ULU = 23,
        BW20UUL = 24,
        BW20UUU = 25,
    }
}

impl_bitflags! {
    /// Indicates what VHT information is known.
    pub struct Known: u16 {
        /// The space-time block coding (STBC) information is known.
        const STBC = 0x0001;
        /// The `TXOP_PS_NA` information is known.
        const TXOP_PS_NA = 0x0002;
        /// The guard interval is known.
        const GI = 0x0004;
        /// The short GI NSYM disambiguation is known.
        const SGI_NSYM_DA = 0x0008;
        /// The LDPC extra OFDM symbol is known.
        const LDPC_EXTRA_OFDM_SYM = 0x0010;
        /// The beamformed information is known.
        const BF = 0x0020;
        /// The bandwidth is known.
        const BW = 0x0040;
        /// The group ID is known.
        const G_ID = 0x0080;
        /// The partial AID is known/applicable.
        const P_AID = 0x0100;
    }
}

impl_bitflags! {
    /// Flags describing the VHT information.
    pub struct Flags: u8 {
        /// Endodes the space-time block coding (STBC).
        const STBC = 0x01;
        /// Encodes whether STAs may doze during TXOP.
        const TXOP_PS_NA = 0x02;
        /// Encodes the guard interval.
        const GI = 0x04;
        /// Encodes the short Guard Interval Nsym disambiguation.
        const SGI_NSYM_DA = 0x08;
        /// Encodes the LDPC extra OFDM symbol.
        const LDPC_EXTRA_OFDM_SYM = 0x10;
        /// Encodes whether this frame was beamformed.
        const BF = 0x20;
    }
}

/// A VHT user.
///
/// This is created by the [`.users()`](struct.Vht.html#method.users) method on
/// the [`Vht`](struct.Vht.html) field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct User<'a> {
    vht: &'a Vht,
    index: u8,
    nss: u8,
    nsts: u8,
    fec: Fec,
}

/// The VHT information.
///
/// The IEEE 802.11ac data rate index.
///
/// Other rate fields: [Rate](../struct.Rate.html),
/// [MCS](../mcs/struct.Mcs.html)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vht {
    known: Known,
    flags: Flags,
    bandwidth: u8,
    mcs_nss: [u8; 4],
    coding: u8,
    group_id: u8,
    partial_aid: u16,
}

impl From<mcs::Bandwidth> for Bandwidth {
    fn from(bw: mcs::Bandwidth) -> Self {
        match bw {
            mcs::Bandwidth::BW20 => Self::BW20,
            mcs::Bandwidth::BW40 => Self::BW40,
            mcs::Bandwidth::BW20L => Self::BW40L,
            mcs::Bandwidth::BW20U => Self::BW20U,
        }
    }
}

impl Bandwidth {
    /// Returns the bandwidth in MHz.
    pub fn to_mhz(&self) -> u8 {
        match self {
            Self::BW20 => 20,
            Self::BW40 => 40,
            Self::BW20L => 40,
            Self::BW20U => 40,
            Self::BW80 => 80,
            Self::BW40L => 80,
            Self::BW40U => 80,
            Self::BW20LL => 80,
            Self::BW20LU => 80,
            Self::BW20UL => 80,
            Self::BW20UU => 80,
            Self::BW160 => 160,
            Self::BW80L => 160,
            Self::BW80U => 160,
            Self::BW40LL => 160,
            Self::BW40LU => 160,
            Self::BW40UL => 160,
            Self::BW40UU => 160,
            Self::BW20LLL => 160,
            Self::BW20LLU => 160,
            Self::BW20LUL => 160,
            Self::BW20LUU => 160,
            Self::BW20ULL => 160,
            Self::BW20ULU => 160,
            Self::BW20UUL => 160,
            Self::BW20UUU => 160,
        }
    }
}

impl User<'_> {
    /// Returns the VHT index (1 - 9).
    pub const fn index(&self) -> u8 {
        self.index
    }

    /// Returns the number of spatial streams (1 - 8).
    pub const fn nss(&self) -> u8 {
        self.nss
    }

    /// Returns the number of space-time streams.
    pub const fn nsts(&self) -> u8 {
        self.nsts
    }

    /// Returns the FEC type.
    pub const fn fec(&self) -> Fec {
        self.fec
    }

    /// Returns the data rate in megabits per second.
    pub fn to_mbps(&self) -> Option<result::Result<f32, InvalidDatarate>> {
        if self.index > 9 {
            return Some(Err(InvalidDatarateKind::Index(self.index).into()));
        }
        if self.nss > 8 {
            return Some(Err(InvalidDatarateKind::Nss(self.nss).into()));
        }
        let bw = match self.vht.bandwidth()? {
            Ok(bw) => bw,
            Err(err) => {
                let inner: InvalidDatarateKind = err.into();
                return Some(Err(inner.into()));
            }
        };
        let b = match bw.to_mhz() {
            20 => 0,
            40 => 2,
            80 => 4,
            160 => 6,
            _ => unreachable!(),
        };
        let col: usize = (b + self.vht.guard_interval()?.into_inner()).into();
        let row: usize = (self.index + (self.nss - 1) * 10).into();
        let rate = RATE[row][col];
        if rate.is_nan() {
            return Some(Err(InvalidDatarateKind::Mismatch.into()));
        }
        Some(Ok(rate))
    }
}

impl FromBytes for Vht {
    type Error = Error;

    fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
        let known = bytes.read()?;
        let flags = bytes.read()?;
        let bandwidth = bytes.read()?;
        let mcs_nss = bytes.read()?;
        let coding = bytes.read()?;
        let group_id = bytes.read()?;
        let partial_aid = bytes.read()?;
        Ok(Self {
            known,
            flags,
            bandwidth,
            mcs_nss,
            coding,
            group_id,
            partial_aid,
        })
    }
}

impl Vht {
    /// Returns whether all spatial streams of all users have STBC.
    pub fn has_stbc(&self) -> Option<bool> {
        self.known
            .contains(Known::STBC)
            .some(|| self.flags.contains(Flags::STBC))
    }

    /// Returns the guard interval.
    pub fn guard_interval(&self) -> Option<GuardInterval> {
        self.known
            .contains(Known::GI)
            .some(|| GuardInterval::from_bool(self.flags.contains(Flags::GI)))
    }

    /// Returns whether the frame was beamformed.
    pub fn is_beamformed(&self) -> Option<bool> {
        self.known
            .contains(Known::BF)
            .some(|| self.flags.contains(Flags::BF))
    }

    /// Returns the bandwidth.
    pub fn bandwidth(&self) -> Option<result::Result<Bandwidth, InvalidBandwidth>> {
        self.known.contains(Known::BW).some(|| {
            let bits = self.bandwidth & 0x1f;
            Bandwidth::from_bits(bits).ok_or(InvalidBandwidth(bits))
        })
    }

    /// Returns the group ID of the frame.
    ///
    /// The group ID can be used to differentiate between SU PPDUs (group ID is
    /// 0 or 63) and MU PPDUs (group ID is 1 through 62).
    pub fn group_id(&self) -> Option<u8> {
        self.known.contains(Known::G_ID).some(|| self.group_id)
    }

    /// Returns the partial aid.
    ///
    /// This is a non-unique identifier of a STA to identify whether the
    /// transmissions are destined to a STA or not, used in conjunction with
    /// group ID.
    pub fn partial_aid(&self) -> Option<u16> {
        self.known.contains(Known::P_AID).some(|| self.partial_aid)
    }

    /// Returns the VHT users.
    pub fn users(&self) -> [Option<User>; 4] {
        let mut users: [Option<User>; 4] = Default::default();
        for (i, user) in users.iter_mut().enumerate() {
            let mcs_nss = self.mcs_nss[i];
            let nss = mcs_nss & 0x0f;
            if nss == 0 {
                continue;
            }
            let index = (mcs_nss & 0xf0) >> 4;
            let stbc: u8 = self.has_stbc().unwrap_or(false).into();
            let nsts = nss << stbc;
            let id = i as u8;
            let fec = Fec::from_bool((self.coding & (1 << id)) > 0);
            user.replace(User {
                vht: self,
                index,
                nss,
                nsts,
                fec,
            });
        }
        users
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
        let vht = Vht::from_hex("440004041200000000000000").unwrap();
        assert_eq!(
            vht,
            Vht {
                known: Known::BW | Known::GI,
                flags: Flags::GI,
                bandwidth: 4,
                mcs_nss: [18, 0, 0, 0],
                coding: 0,
                group_id: 0,
                partial_aid: 0,
            }
        );

        assert_eq!(vht.has_stbc(), None);
        assert_eq!(vht.guard_interval(), Some(GuardInterval::Short));
        assert_eq!(vht.is_beamformed(), None);
        assert_eq!(vht.bandwidth().unwrap().unwrap(), Bandwidth::BW80);
        assert_eq!(vht.group_id(), None);
        assert_eq!(vht.partial_aid(), None);
        assert_eq!(
            vht.users(),
            [
                Some(User {
                    vht: &vht,
                    index: 1,
                    nss: 2,
                    nsts: 2,
                    fec: Fec::Bcc
                }),
                None,
                None,
                None,
            ]
        );
    }

    fn check_datarate(hex: &str, datarate: f32) {
        let vht = Vht::from_hex(hex).unwrap();
        let users: Vec<_> = vht.users().into();
        let datarates: Vec<_> = users
            .into_iter()
            .map(|o| o.map(|u| u.to_mbps()).flatten().map(|r| r.unwrap()))
            .collect();
        assert_eq!(datarates, vec![Some(datarate), None, None, None]);
    }

    #[test]
    fn datarate() {
        check_datarate("440004040200000000000000", 65.0);
        check_datarate("440004041200000000000000", 130.0);
        check_datarate("440004049200000000000000", 866.7);
    }
}
