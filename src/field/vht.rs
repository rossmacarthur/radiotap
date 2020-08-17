//! Defines the VHT field.

use super::*;

impl_bitflags! {
    pub struct Known: u16 {
        /// Indicates that the STBC is known.
        const STBC = 0x0001;
        /// Indicates that the TXOP_PS_NA field is known.
        const TXOP_PS_NA = 0x0002;
        /// Indicates that the guard interval is known.
        const GI = 0x0004;
        /// Indicates that the short GI NSYM disambiguation is known.
        const SGI_NSYM_DIS = 0x0008;
        /// Indicates that the LDPC extra OFDM symbol is known.
        const LDPC_EXTRA_OFDM_SYM = 0x0010;
        /// Indicates that the beamformed information is known.
        const BEAMFORMED = 0x0020;
        /// Indicates that the bandwidth is known.
        const BANDWIDTH = 0x0040;
        /// Indicates that the group ID is known.
        const GROUP_ID = 0x0080;
        /// Indicateds that the partial AID known/applicable.
        const PARTIAL_AID = 0x0100;
    }
}

impl_bitflags! {
    pub struct Flags: u8 {
        /// Space-time block coding.
        ///
        /// Set to 0 if no spatial streams of any user has STBC.
        /// Set to 1 if all spatial streams of all users have STBC.
        const STBC = 0x01;
        /// Valid only for AP transmitters.
        ///
        /// Set to 0 if STAs may doze during TXOP.
        /// Set to 1 if STAs may not doze during TXOP or transmitter is non-AP.
        const TXOP_PS_NA = 0x02;
        /// Set to 0 for long GI.
        /// Set to 1 for short GI.
        const GI = 0x04;
        /// Valid only if short GI is used.
        ///
        /// Set to 0 if NSYM mod 10 != 9 or short GI not used.
        /// Set to 1 if NSYM mod 10 = 9.
        const SGI_NSYM_M10_9 = 0x08;
        /// Set to 1 if one or more users are using LDPC and the encoding process resulted in extra OFDM symbol(s). Set to 0 otherwise.
        const LDPC_EXTRA_OFDM_SYM = 0x10;
        /// Valid for SU PPDUs only.
        const BEAMFORMED = 0x20;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct McsNss(u8);

/// The VHT information.
///
/// The IEEE 802.11ac data rate index. Usually only one of the
/// [Rate](../struct.Rate.html), [MCS](../mcs/struct.MCS.html), and
/// [VHT](struct.VHT.html) fields is present.
///
/// [Reference](http://www.radiotap.org/fields/VHT.html)
#[derive(Debug, Clone, PartialEq)]
pub struct Vht {
    /// Indicates which information is known.
    known: Known,
    /// Contains various encoded information.
    flags: Flags,
    /// Encodes the bandwidth.
    bandwidth: u8,
    /// Encodes the MCS and NSSS for up to four users.
    mcs_nss: [McsNss; 4],
    /// Encodes the FEC for up to four users.
    coding: u8,
    /// The Group ID of the frame.
    group_id: u8,
    /// A non-unique identifier of a STA to identify whether the transmissions
    /// are destined to a STA or not, used in conjunction with group ID.
    partial_aid: u16,
}

impl_from_bytes_newtype!(McsNss);

impl FromBytes for [McsNss; 4] {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        Ok([
            bytes[0..1].try_read()?,
            bytes[1..2].try_read()?,
            bytes[2..3].try_read()?,
            bytes[3..4].try_read()?,
        ])
    }
}

impl FromBytes for Vht {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
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
