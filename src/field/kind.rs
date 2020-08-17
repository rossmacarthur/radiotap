use crate::field::VendorNamespace;
use crate::{Error, Result};

/// The type of radiotap field.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Kind {
    Tsft,
    Flags,
    Rate,
    Channel,
    Fhss,
    AntennaSignal,
    AntennaNoise,
    LockQuality,
    TxAttenuation,
    TxAttenuationDb,
    TxPower,
    Antenna,
    AntennaSignalDb,
    AntennaNoiseDb,
    RxFlags,
    TxFlags,
    RtsRetries,
    DataRetries,
    XChannel,
    Mcs,
    AmpduStatus,
    Vht,
    Timestamp,
    VendorNamespace(Option<VendorNamespace>),
}

impl Kind {
    pub fn new(value: u8) -> Result<Self> {
        Ok(match value {
            0 => Self::Tsft,
            1 => Self::Flags,
            2 => Self::Rate,
            3 => Self::Channel,
            4 => Self::Fhss,
            5 => Self::AntennaSignal,
            6 => Self::AntennaNoise,
            7 => Self::LockQuality,
            8 => Self::TxAttenuation,
            9 => Self::TxAttenuationDb,
            10 => Self::TxPower,
            11 => Self::Antenna,
            12 => Self::AntennaSignalDb,
            13 => Self::AntennaNoiseDb,
            14 => Self::RxFlags,
            15 => Self::TxFlags,
            16 => Self::RtsRetries,
            17 => Self::DataRetries,
            18 => Self::XChannel,
            19 => Self::Mcs,
            20 => Self::AmpduStatus,
            21 => Self::Vht,
            22 => Self::Timestamp,
            _ => {
                return Err(Error::UnsupportedField);
            }
        })
    }

    /// Returns the align value for the field.
    pub fn align(&self) -> u64 {
        match self {
            Self::Tsft | Self::Timestamp => 8,
            Self::XChannel | Self::AmpduStatus => 4,
            Self::Channel
            | Self::Fhss
            | Self::LockQuality
            | Self::TxAttenuation
            | Self::TxAttenuationDb
            | Self::RxFlags
            | Self::TxFlags
            | Self::Vht
            | Self::VendorNamespace(_) => 2,
            _ => 1,
        }
    }

    /// Returns the size of the field.
    pub fn size(&self) -> usize {
        match self {
            Self::Vht | Self::Timestamp => 12,
            Self::Tsft | Self::AmpduStatus | Self::XChannel => 8,
            Self::VendorNamespace(_) => 6,
            Self::Channel => 4,
            Self::Mcs => 3,
            Self::Fhss
            | Self::LockQuality
            | Self::TxAttenuation
            | Self::TxAttenuationDb
            | Self::RxFlags
            | Self::TxFlags => 2,
            _ => 1,
        }
    }
}
