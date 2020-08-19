//! Defines the VHT field.

use std::result::Result;

use thiserror::Error;

use crate::field::{Fec, GuardInterval, Kind};
use crate::prelude::*;

/// An error returned when parsing a [`Bandwidth`](enum.Bandwidth.html) from the
/// raw bits in [`Vht.bandwidth()`](struct.Vht.html#method.bandwidth).
#[derive(Debug, Error, PartialEq)]
#[error("failed to parse bandwidth from value `{0}`")]
pub struct ParseBandwidthError(u8);

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
#[derive(Debug, Clone, PartialEq)]
pub struct User {
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
#[derive(Debug, Clone, PartialEq)]
pub struct Vht {
    known: Known,
    flags: Flags,
    bandwidth: u8,
    mcs_nss: [u8; 4],
    coding: u8,
    group_id: u8,
    partial_aid: u16,
}

impl FromBytes for [u8; 4] {
    fn from_bytes(bytes: Bytes) -> crate::Result<Self> {
        Ok([
            bytes[0..1].try_read()?,
            bytes[1..2].try_read()?,
            bytes[2..3].try_read()?,
            bytes[3..4].try_read()?,
        ])
    }
}

impl User {
    /// Returns the VHT index (1 - 9).
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Returns the number of spatial streams (1 - 8).
    pub fn nss(&self) -> u8 {
        self.nss
    }

    /// Returns the number of space-time streams.
    pub fn nsts(&self) -> u8 {
        self.nsts
    }

    /// Returns the FEC type.
    pub fn fec(&self) -> Fec {
        self.fec
    }
}

impl FromBytes for Vht {
    fn from_bytes(bytes: Bytes) -> crate::Result<Self> {
        ensure_length!(bytes.len() == Kind::Vht.size());
        let known = bytes[0..2].try_read()?;
        let flags = bytes[2..3].try_read()?;
        let bandwidth = bytes[3..4].try_read()?;
        let mcs_nss = bytes[4..8].try_read()?;
        let coding = bytes[8..9].try_read()?;
        let group_id = bytes[9..10].try_read()?;
        let partial_aid = bytes[10..12].try_read()?;
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
            .some(|| self.flags.contains(Flags::GI).into())
    }

    /// Returns whether the frame was beamformed.
    pub fn is_beamformed(&self) -> Option<bool> {
        self.known
            .contains(Known::BF)
            .some(|| self.flags.contains(Flags::BF))
    }

    /// Returns the bandwidth.
    pub fn bandwidth(&self) -> Option<Result<Bandwidth, ParseBandwidthError>> {
        self.known.contains(Known::BW).some(|| {
            let bits = self.bandwidth & 0x1f;
            Bandwidth::from_bits(bits).ok_or_else(|| ParseBandwidthError(bits))
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
            let fec = ((self.coding & (1 << id)) > 0).into();
            user.replace(User {
                index,
                nss,
                nsts,
                fec,
            });
        }
        users
    }

    /// Returns the raw known information.
    pub fn known(&self) -> Known {
        self.known
    }

    /// Returns the raw flags.
    pub fn flags(&self) -> Flags {
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
        assert_eq!(vht.bandwidth(), Some(Ok(Bandwidth::BW80)));
        assert_eq!(vht.group_id(), None);
        assert_eq!(vht.partial_aid(), None);
        assert_eq!(
            vht.users(),
            [
                Some(User {
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
}
