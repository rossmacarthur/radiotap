//! Defines the A-MPDU Status field.

use super::*;

use crate::util::BoolExt;

impl_bitflags! {
    /// Flags describing the A-MPDU this frame was part of.
    pub struct Flags: u16 {
        /// Driver reports 0-length subframes.
        const REPORT_ZERO_LEN = 0x0001;
        /// Frame is 0-length subframe (valid only if REPORT_ZERO_LEN is set).
        const IS_ZERO_LEN = 0x0002;
        /// Last subframe is known (should be set for all subframes in an A-MPDU).
        const LAST_KNOWN = 0x0004;
        /// This frame is the last subframe.
        const IS_LAST = 0x0008;
        /// Delimiter CRC error.
        const DELIM_CRC_ERR = 0x0010;
        /// Delimiter CRC value known: the delimiter CRC value field is valid.
        const DELIM_CRC_KNOWN = 0x0020;
        /// EOF value.
        const EOF = 0x0040;
        /// EOF value known.
        const EOF_KNOWN = 0x0080;
    }
}

/// Indicates that the frame was received as part of an A-MPDU.
#[derive(Debug, Clone, PartialEq)]
pub struct AmpduStatus {
    /// The A-MPDU reference number.
    reference: u32,
    /// The A-AMPDU flags.
    flags: Flags,
    /// The A-MPDU subframe delimiter CRC.
    delim_crc: u8,
}

impl FromBytes for AmpduStatus {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        ensure_length!(bytes.len() == Kind::AmpduStatus.size());
        let reference = bytes[0..4].try_read()?;
        let flags = bytes[4..6].try_read()?;
        let delim_crc = bytes[6..7].try_read()?;
        Ok(Self {
            reference,
            flags,
            delim_crc,
        })
    }
}

impl AmpduStatus {
    /// Returns the raw A-MPDU flags.
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Returns the A-MPDU reference number.
    pub fn reference(&self) -> u32 {
        self.reference
    }

    /// Whether the frame is 0-length subframe of this A-MPDU.
    pub fn is_zero_len(&self) -> Option<bool> {
        self.flags
            .contains(Flags::REPORT_ZERO_LEN)
            .some(|| self.flags.contains(Flags::IS_ZERO_LEN))
    }

    /// Whether this frame is the last subframe of this A-MPDU.
    pub fn is_last(&self) -> Option<bool> {
        self.flags
            .contains(Flags::LAST_KNOWN)
            .some(|| self.flags.contains(Flags::IS_LAST))
    }

    /// Returns the A-MPDU subframe delimiter CRC value.
    pub fn delim_crc(&self) -> Option<u8> {
        self.flags
            .contains(Flags::DELIM_CRC_KNOWN)
            .some(|| self.delim_crc)
    }

    /// Whether there is an EOF on this A-MPDU subframe.
    pub fn has_eof(&self) -> Option<bool> {
        self.flags
            .contains(Flags::EOF_KNOWN)
            .some(|| self.flags.contains(Flags::EOF))
    }
}
