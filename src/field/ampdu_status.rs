//! Defines the A-MPDU Status field.

use super::*;

impl_bitflags! {
    /// The A-MPDU flags.
    pub struct Flags: u16 {
        /// Driver reports 0-length subframes.
        const REPORT_ZERO_LEN = 0x0001;
        /// Frame is 0-length subframe (valid only if REPORT_ZERO_LEN is set).
        const IS_ZERO_LEN = 0x0002;
        /// Last subframe is known (should be set for all subframes in an A-MPDU)
        const LAST_KNOWN = 0x0004;
        /// This frame is the last subframe.
        const IS_LAST = 0x0008;
        /// Delimiter CRC error.
        const DELIM_CRC_ERR = 0x0010;
        /// Delimiter CRC value known: the delimiter CRC value field is valid.
        const DELIM_CRC_KNOWN = 0x0020;
        /// EOF value.
        const EOF = 0x0040;
        /// EOF value known
        const EOF_KNOWN = 0x0080;
    }
}

/// Indicates that the frame was received as part of an A-MPDU.
///
/// [Reference](http://www.radiotap.org/fields/A-MPDU%20status.html)
#[derive(Debug, Clone, PartialEq)]
pub struct AmpduStatus {
    /// The A-MPDU reference number.
    reference: u32,
    /// The flags.
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
