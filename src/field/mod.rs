//! Radiotap field definitions.
//!
//! Each field helps to describe a sent or received IEEE 802.11 frame.

#[macro_use]
mod _macros;
mod kind;

use std::io::{Cursor, Read};

use bitflags::bitflags;
use bitops::BitOps;
use byteorder::{ReadBytesExt, LE};

use crate::bytes::{Bytes, BytesExt, FromBytes};
pub use crate::field::kind::Kind;
use crate::{Error, Oui, Result};

/////////////////////////////////////////////////////////////////////////
// Special fields
/////////////////////////////////////////////////////////////////////////

/// The radiotap header, contained in all radiotap captures.
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    /// The radiotap version, only version 0 is supported.
    pub version: u8,
    /// The length of the entire radiotap capture.
    pub length: usize,
    /// The size of the radiotap header.
    pub size: usize,
    /// The fields present in the radiotap capture.
    pub present: Vec<Kind>,
}

impl FromBytes for Header {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let version = cursor.read_u8()?;
        if version != 0 {
            // We only support version 0
            return Err(Error::UnsupportedVersion);
        }

        cursor.read_u8()?; // Account for 1 byte padding field

        let length = cursor.read_u16::<LE>()?;
        if bytes.len() < length as usize {
            return Err(Error::InvalidLength);
        }

        let mut present;
        let mut present_count = 0;
        let mut vendor_namespace = false;
        let mut kinds = Vec::new();

        loop {
            present = cursor.read_u32::<LE>()?;

            if !vendor_namespace {
                for bit in 0..29 {
                    if present.is_bit_set(bit) {
                        match Kind::new(present_count * 32 + bit) {
                            Ok(kind) => {
                                kinds.push(kind);
                            }
                            Err(Error::UnsupportedField) => {
                                // Does not matter, we will just parse the ones
                                // we can
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
            }

            // Need to move to radiotap namespace
            if present.is_bit_set(29) {
                present_count = 0;
                vendor_namespace = false;

            // Need to move to vendor namespace
            } else if present.is_bit_set(30) {
                present_count = 0;
                vendor_namespace = true;
                // We'll figure out what namespace it is later, just use none
                kinds.push(Kind::VendorNamespace(None))

            // Need to stay in the same namespace
            } else {
                present_count += 1;
            }

            // More present words do not exist
            if !present.is_bit_set(31) {
                break;
            }
        }

        Ok(Self {
            version,
            length: length as usize,
            size: cursor.position() as usize,
            present: kinds,
        })
    }
}

/////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct VendorNamespace {
    pub oui: Oui,
    pub sub_namespace: u8,
    pub skip_length: u16,
}

impl FromBytes for VendorNamespace {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut oui = [0; 3];
        cursor.read_exact(&mut oui)?;
        let sub_namespace = cursor.read_u8()?;
        let skip_length = cursor.read_u16::<LE>()?;
        Ok(Self {
            oui,
            sub_namespace,
            skip_length,
        })
    }
}

/////////////////////////////////////////////////////////////////////////
// Common types
/////////////////////////////////////////////////////////////////////////

/// The guard interval.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum GuardInterval {
    /// 800 ns.
    Long,
    /// 400 ns.
    Short,
}

/////////////////////////////////////////////////////////////////////////
// Fields
/////////////////////////////////////////////////////////////////////////

pub mod ampdu_status;
pub mod channel;
pub mod mcs;
pub mod timestamp;
pub mod vht;
pub mod xchannel;

pub use crate::field::ampdu_status::AmpduStatus;
pub use crate::field::channel::Channel;
pub use crate::field::mcs::Mcs;
pub use crate::field::timestamp::Timestamp;
pub use crate::field::vht::Vht;
pub use crate::field::xchannel::XChannel;

impl_newtype! {
    /// The TSFT value.
    ///
    /// Value in microseconds of the MAC’s 64-bit 802.11 Time Synchronization
    /// Function Timer when the first bit of the MPDU arrived at the MAC. For
    /// received frames only.
    ///
    /// [Reference](http://www.radiotap.org/fields/TSFT.html)
    pub struct Tsft(u64);
}

impl_newtype! {
    /// Tx/Rx data rate.
    ///
    /// This is the legacy data rate. Usually only one of the
    /// [Rate](struct.Rate.html), [MCS](./mcs/struct.Mcs.html), and
    /// [VHT](./vht/struct.Vht.html) fields are present.
    pub struct Rate(u8);
}

impl_newtype! {
    /// RF signal power at the antenna in dBm.
    ///
    /// It indicates the RF signal power at the antenna, in decibels difference from
    /// 1mW.
    ///
    /// [Reference](http://www.radiotap.org/fields/Antenna%20signal.html)
    pub struct AntennaSignal(i8);
}

impl_newtype! {
    /// RF signal power at the antenna in dB.
    ///
    /// It indicates the RF signal power at the antenna, in decibels difference from
    /// an arbitrary, fixed reference.
    ///
    /// [Reference](http://www.radiotap.org/fields/dB%20antenna%20signal.html)
    pub struct AntennaSignalDb(u8);
}

impl_newtype! {
    /// RF noise power at the antenna in dBm.
    ///
    /// It indicates the RF signal noise at the antenna, in decibels  difference
    /// from 1mW.
    ///
    /// [Reference](https://www.radiotap.org/fields/Antenna%20noise.html)
    pub struct AntennaNoise(i8);
}

impl_newtype! {
    /// RF noise power at the antenna in dB.
    ///
    /// It indicates the RF signal noise at the antenna, in decibels difference from
    /// an arbitrary, fixed reference.
    ///
    /// [Reference](https://www.radiotap.org/fields/dB%20antenna%20noise.html)
    pub struct AntennaNoiseDb(u8);
}

impl_newtype! {
    /// Quality of Barker code lock, unitless.
    ///
    /// Monotonically nondecreasing with "better" lock strength. Called "Signal
    /// Quality" in datasheets.
    ///
    /// [Reference](https://www.radiotap.org/fields/Lock%20quality.html)
    pub struct LockQuality(u16);
}

impl_newtype! {
    /// Transmit power expressed as unitless distance from max power.
    ///
    /// 0 is max power. Monotonically nondecreasing with lower power levels.
    ///
    /// [Reference](https://www.radiotap.org/fields/TX%20attenuation.html)
    pub struct TxAttenuation(u16);
}

impl_newtype! {
    /// Transmit power expressed as decibel distance from max power set at factory
    /// calibration.
    ///
    /// 0 is max power. Monotonically nondecreasing with lower power levels.
    ///
    /// [Reference](https://www.radiotap.org/fields/dB%20TX%20attenuation.html)
    pub struct TxAttenuationDb(u16);
}

impl_newtype! {
    /// Transmit power in dBm.
    ///
    /// This is the absolute power level measured at the antenna port.
    ///
    /// [Reference](https://www.radiotap.org/fields/dBm%20TX%20power.html)
    pub struct TxPower(i8);
}

impl_newtype! {
    /// The antenna index.
    ///
    /// Unitless indication of the Rx/Tx antenna for this packet. The first antenna
    /// is antenna 0.
    ///
    /// [Reference](https://www.radiotap.org/fields/Antenna.html)
    pub struct Antenna(u8);
}

impl_newtype! {
    /// Number of RTS retries a transmitted frame used.
    ///
    /// [Reference](https://www.radiotap.org/fields/RTS%20retries.html)
    pub struct RtsRetries(u8);
}

impl_newtype! {
    /// Number of data retries a transmitted frame used.
    ///
    /// [Reference](https://www.radiotap.org/fields/data%20retries.html)
    pub struct DataRetries(u8);
}

impl_bitflags! {
    /// Properties of transmitted and received frames.
    ///
    /// [Reference](http://www.radiotap.org/fields/Flags.html)
    pub struct Flags: u8 {
        /// The frame was sent/received during CFP.
        const CFP = 0x01;
        /// The frame was sent/received with short preamble.
        const PREAMBLE = 0x02;
        /// The frame was sent/received with WEP encryption.
        const WEP = 0x04;
        /// The frame was sent/received with fragmentation.
        const FRAG = 0x08;
        /// The frame includes FCS.
        const FCS = 0x10;
        /// The frame has padding between 802.11 header and payload (to 32-bit
        /// boundary).
        const DATA_PAD = 0x20;
        /// The frame failed FCS check.
        const BAD_FCS = 0x40;
        /// The frame used short guard interval (HT).
        const SGI = 0x80;
    }
}

impl_bitflags! {
    /// Properties of received frames.
    ///
    /// [Reference](https://www.radiotap.org/fields/RX%20flags.html)
    pub struct RxFlags: u16 {
        /// PLCP CRC check failed.
        const BAD_PLCP = 0x0002;
    }
}

impl_bitflags! {
    /// Properties of transmitted frames.
    ///
    /// [Reference](https://www.radiotap.org/fields/TX%20flags.html)
    pub struct TxFlags: u16 {
        /// Transmission failed due to excessive retries.
        const FAIL = 0x0001;
        /// Transmission used CTS-to-self protection.
        const CTX = 0x0002;
        /// Transmission used RTS/CTS handshake.
        const RTS = 0x0004;
        /// Transmission shall not expect an ACK frame and not retry when no ACK is
        /// received.
        const NO_ACK = 0x0008;
        /// Transmission includes a pre-configured sequence number that should not
        /// be changed by the driver's TX handlers.
        const NO_SEQ = 0x0010;
    }
}

/// The hop set and pattern for frequency-hopping radios.
///
/// [Reference](http://www.radiotap.org/fields/FHSS.html)
#[derive(Debug, Clone, PartialEq)]
pub struct Fhss {
    hop_set: u8,
    hop_pattern: u8,
}

impl FromBytes for Fhss {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        ensure_length!(bytes.len() == Kind::Fhss.size());
        let hop_set = bytes[0].into();
        let hop_pattern = bytes[1].into();
        Ok(Self {
            hop_set,
            hop_pattern,
        })
    }
}

impl Fhss {
    /// Returns the hop set.
    pub fn hop_set(&self) -> u8 {
        self.hop_set
    }

    /// Returns the hop pattern.
    pub fn hop_pattern(&self) -> u8 {
        self.hop_pattern
    }
}
