//! Defines the Timestamp field.

use std::result;
use std::time::Duration;
use std::time::SystemTime;

use thiserror::Error;

use crate::prelude::*;

/// An error returned when parsing a [`Unit`](enum.Unit.html) from the raw bits
/// in [`.unit()`](struct.Timestamp.html#method.unit).
#[derive(Debug, Error)]
#[error("failed to parse time unit from value `{0}`")]
pub struct InvalidUnit(u8);

/// An error returned when parsing a
/// [`SamplingPosition`](enum.SamplingPosition.html) from the raw bits
/// in [`.sampling_position()`](struct.Timestamp.html#method.sampling_position).
#[derive(Debug, Error)]
#[error("failed to parse sampling position from value `{0}`")]
pub struct InvalidSamplingPosition(u8);

impl_enum! {
    /// The time unit.
    pub enum Unit: u8 {
        Millis = 0,
        Micros = 1,
        Nanos = 2,
    }
}

impl_enum! {
    /// The sampling position.
    pub enum SamplingPosition: u8 {
        /// First MPDU bit/symbol.
        StartMpdu = 0,
        /// Signal acquisition at start of PLCP.
        PlcpSigAcq = 1,
        /// End of PPDU.
        EndPpdu = 2,
        /// End of MPDU.
        EndMpdu = 3,
        /// Unknown or vendor defined.
        Unknown = 15,
    }
}

impl_bitflags! {
    // Flags describing the timestamp.
    pub struct Flags: u8 {
        /// 32-bit counter.
        const BIT32 = 0x01;
        /// Accuracy field is known.
        const ACCURACY = 0x02;
    }
}

/// The time the frame was transmitted or received.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    ts: u64,
    accuracy: u16,
    unit_position: u8,
    flags: Flags,
}

impl FromBytes for Timestamp {
    type Error = Error;

    fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
        let ts = bytes.read()?;
        let accuracy = bytes.read()?;
        let unit_position = bytes.read()?;
        let flags = bytes.read()?;
        Ok(Self {
            ts,
            accuracy,
            unit_position,
            flags,
        })
    }
}

impl Unit {
    fn duration(&self, ts: u64) -> Duration {
        match self {
            Self::Millis => Duration::from_millis(ts),
            Self::Micros => Duration::from_micros(ts),
            Self::Nanos => Duration::from_nanos(ts),
        }
    }
}

impl Timestamp {
    /// Returns the time unit of the timestamp.
    pub fn unit(&self) -> result::Result<Unit, InvalidUnit> {
        let bits = self.unit_position & 0x0f;
        Unit::from_bits(bits).ok_or_else(|| InvalidUnit(bits))
    }

    /// Returns the timestamp as a duration since the UNIX Epoch.
    pub fn duration(&self) -> result::Result<Duration, InvalidUnit> {
        self.unit().map(|unit| unit.duration(self.ts))
    }

    /// Returns the timestamp as a system time.
    pub fn system_time(&self) -> result::Result<SystemTime, InvalidUnit> {
        self.duration().map(|d| SystemTime::UNIX_EPOCH + d)
    }

    /// Returns the accuracy of the timestamp as a duration.
    pub fn accuracy(&self) -> Option<result::Result<Duration, InvalidUnit>> {
        self.flags
            .contains(Flags::ACCURACY)
            .some(|| self.unit().map(|unit| unit.duration(self.accuracy.into())))
    }

    /// Returns the sampling position of the timstamp.
    pub fn sampling_position(&self) -> result::Result<SamplingPosition, InvalidSamplingPosition> {
        let bits = self.unit_position >> 4;
        SamplingPosition::from_bits(bits).ok_or_else(|| InvalidSamplingPosition(bits))
    }

    /// Returns the flags describing the timestamp.
    pub const fn flags(&self) -> Flags {
        self.flags
    }
}
