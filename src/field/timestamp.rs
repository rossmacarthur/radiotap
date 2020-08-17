//! Defines the Timestamp field.

use super::*;

bitflags! {
    pub struct Flags: u8 {
        /// 32-bit counter.
        const BIT_32 = 0x01;
        /// Accuracy field is known.
        const ACCURACY = 0x02;
    }
}

/// The time the frame was transmitted or received.
#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp {
    /// The actual timestamp.
    timestamp: u64,
    /// The accuracy of the timestamp.
    accuracy: u16,
    /// The unit/position of the timestamp value.
    unit_position: u8,
    /// Contains various encoded information.
    flags: Flags,
}

impl_from_bytes_bitflags!(Flags);

impl FromBytes for Timestamp {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        ensure_length!(bytes.len() == Kind::Timestamp.size());
        let timestamp = bytes[0..8].try_read()?;
        let accuracy = bytes[8..10].try_read()?;
        let unit_position = bytes[10..11].try_read()?;
        let flags = bytes[11..12].try_read()?;
        Ok(Self {
            timestamp,
            accuracy,
            unit_position,
            flags,
        })
    }
}
