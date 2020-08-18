//! Defines the Timestamp field.

use super::*;

use std::result::Result;
use std::time::Duration;
use std::time::SystemTime;

use thiserror::Error;

use crate::bytes::FromBytes;
use crate::field::Kind;
use crate::util::BoolExt;

impl_enum! {
    /// The time unit.
    pub enum Unit: u8 {
        Millis = 0,
        Micros = 1,
        Nanos = 2,
    }
}

/// An error returned when parsing a [`Unit`](enum.Unit.html) from the raw bits
/// in [`Timestamp.unit()`](struct.Timestamp.html#method.unit).
#[derive(Debug, Error)]
#[error("failed to parse time unit from value `{0}`")]
pub struct ParseUnitError(u8);

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

/// An error returned when parsing a
/// [`SamplingPosition`](enum.SamplingPosition.html) from the raw bits
/// in [`Timestamp.sampling_position()`](struct.Timestamp.html#method.
/// sampling_position).
#[derive(Debug, Error)]
#[error("failed to parse sampling position from value `{0}`")]
pub struct ParseSamplingPositionError(u8);

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
#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp {
    ts: u64,
    accuracy: u16,
    unit_position: u8,
    flags: Flags,
}

impl FromBytes for Timestamp {
    fn from_bytes(bytes: Bytes) -> crate::Result<Self> {
        ensure_length!(bytes.len() == Kind::Timestamp.size());
        let ts = bytes[0..8].try_read()?;
        let accuracy = bytes[8..10].try_read()?;
        let unit_position = bytes[10..11].try_read()?;
        let flags = bytes[11..12].try_read()?;
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
    /// Returns the flags describing the timestamp.
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Returns the raw timestamp value, in
    /// [`.unit()`](struct.Timestamp.html#method.unit) units.
    pub fn ts(&self) -> u64 {
        self.ts
    }

    /// Returns the time unit of the timestamp.
    pub fn unit(&self) -> Result<Unit, ParseUnitError> {
        let bits = self.unit_position & 0x0f;
        Unit::from_bits(bits).ok_or_else(|| ParseUnitError(bits))
    }

    /// Returns the timestamp as a duration since the UNIX Epoch.
    pub fn duration(&self) -> Result<Duration, ParseUnitError> {
        self.unit().map(|unit| unit.duration(self.ts))
    }

    /// Returns the timestamp as a system time.
    pub fn system_time(&self) -> Result<SystemTime, ParseUnitError> {
        self.duration().map(|d| SystemTime::UNIX_EPOCH + d)
    }

    /// Returns the accuracy of the timestamp as a duration.
    pub fn accuracy(&self) -> Option<Result<Duration, ParseUnitError>> {
        self.flags
            .contains(Flags::ACCURACY)
            .some(|| self.unit().map(|unit| unit.duration(self.accuracy.into())))
    }

    /// Returns the sampling position of the timstamp.
    pub fn sampling_position(&self) -> Result<SamplingPosition, ParseSamplingPositionError> {
        let bits = self.unit_position >> 4;
        SamplingPosition::from_bits(bits).ok_or_else(|| ParseSamplingPositionError(bits))
    }
}
