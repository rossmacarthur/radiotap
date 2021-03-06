//! Field definitions.
//!
//! Each field helps to describe a sent or received IEEE 802.11 frame.

#[macro_use]
mod _macros;

pub use radiotap_derive::*;

/// An organizationally unique identifier.
pub type Oui = [u8; 3];

/////////////////////////////////////////////////////////////////////////
// The type of radiotap field.
/////////////////////////////////////////////////////////////////////////

/// Defines a radiotap field.
pub trait Field<const ALIGN: usize, const SIZE: usize>: Sized {
    fn from_bytes(bytes: [u8; SIZE]) -> Self;
}

/// A kind of radiotap field.
///
/// [`Type`](enum.Type.html) implements this to describe the alignment and size
/// for the default radiotap fields. Vendor namespaces need to implement this in
/// order to be able to use the iterator to parse their fields.
pub trait Kind {
    /// Returns the alignment of the field.
    fn align(&self) -> usize;

    /// Returns the size of the field.
    fn size(&self) -> usize;
}

impl_kind! {
    /// The type of radiotap field.
    ///
    /// Each variant corresponds to unique field in the radiotap capture.
    /// [`Kind`] is implemented to describe the alignment and size of each
    /// field, so that the iterator knows how to handle it.
    ///
    /// Not all of these types are parsed by this crate. The ones that have a
    /// corresponding field have the identical name in the
    /// [`field`][crate::field] module.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum Type {
        Tsft            { bit:  0, align: 8, size:  8 },
        Flags           { bit:  1, align: 1, size:  1 },
        Rate            { bit:  2, align: 1, size:  1 },
        Channel         { bit:  3, align: 2, size:  4 },
        Fhss            { bit:  4, align: 2, size:  2 },
        AntennaSignal   { bit:  5, align: 1, size:  1 },
        AntennaNoise    { bit:  6, align: 1, size:  1 },
        LockQuality     { bit:  7, align: 2, size:  2 },
        TxAttenuation   { bit:  8, align: 2, size:  2 },
        TxAttenuationDb { bit:  9, align: 2, size:  2 },
        TxPower         { bit: 10, align: 1, size:  1 },
        Antenna         { bit: 11, align: 1, size:  1 },
        AntennaSignalDb { bit: 12, align: 1, size:  1 },
        AntennaNoiseDb  { bit: 13, align: 1, size:  1 },
        RxFlags         { bit: 14, align: 2, size:  2 },
        TxFlags         { bit: 15, align: 2, size:  2 },
        RtsRetries      { bit: 16, align: 1, size:  1 },
        DataRetries     { bit: 17, align: 1, size:  1 },
        XChannel        { bit: 18, align: 4, size:  8 },
        Mcs             { bit: 19, align: 1, size:  3 },
        AmpduStatus     { bit: 20, align: 4, size:  8 },
        Vht             { bit: 21, align: 2, size: 12 },
        Timestamp       { bit: 22, align: 8, size: 12 },
        He              { bit: 23, align: 2, size: 12 },
        HeMu            { bit: 24, align: 2, size: 12 },
        HeMuUser        { bit: 25, align: 2, size:  6 },
        ZeroLenPsdu     { bit: 26, align: 1, size:  1 },
        LSig            { bit: 27, align: 2, size:  4 },
    }
}

/////////////////////////////////////////////////////////////////////////
// Vendor namespace field
/////////////////////////////////////////////////////////////////////////

/// A special field that describes a vendor namespace within a radiotap capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Field, FromArray)]
#[field(align = 2, size = 6)]
pub struct VendorNamespace {
    #[field(size = 3)]
    oui: Oui,
    sub_ns: u8,
    skip_length: u16,
}

impl VendorNamespace {
    /// An organizationally unique identifier for the vendor.
    ///
    /// Note: this not unique to the capture, there could be multiple vendor
    /// namespaces with this OUI.
    pub fn oui(&self) -> Oui {
        self.oui
    }

    /// The sub-namespace of this vendor namespace.
    pub fn sub_ns(&self) -> u8 {
        self.sub_ns
    }

    /// Specifies the number of bytes following this field that belong to this
    /// vendor namespace.
    pub fn skip_length(&self) -> usize {
        self.skip_length.into()
    }
}

/////////////////////////////////////////////////////////////////////////
// Common types
/////////////////////////////////////////////////////////////////////////

impl_enum! {
    /// The guard interval.
    #[non_exhaustive]
    pub enum GuardInterval: u8 {
        /// 800 ns.
        Long = 0,
        /// 400 ns.
        Short = 1,
    }
}

impl GuardInterval {
    fn from_bool(b: bool) -> Self {
        match b {
            false => Self::Long,
            true => Self::Short,
        }
    }
}

impl_enum! {
    /// Forward error correction type.
    pub enum Fec: u8 {
        /// Binary convolutional coding.
        Bcc = 0,
        /// Low-density parity-check.
        Ldpc = 1,
    }
}

impl Fec {
    fn from_bool(b: bool) -> Self {
        match b {
            false => Self::Bcc,
            true => Self::Ldpc,
        }
    }
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

/// The TSFT value.
///
/// Value in microseconds of the MAC’s 64-bit 802.11 Time Synchronization
/// Function Timer when the first bit of the MPDU arrived at the MAC. For
/// received frames only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 8, size = 8)]
pub struct Tsft(pub u64);

/// Tx/Rx legacy data rate.
///
/// Other rate fields: [`Mcs`], [`Vht`]
///
/// The raw value's unit is 500 Kbps. Use the [`.to_mbps()`][Rate::to_mbps]
/// method to get the rate in megabits per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 1)]
pub struct Rate(pub u8);

impl Rate {
    /// Returns the data rate in megabits per second.
    pub fn to_mbps(&self) -> f32 {
        f32::from(self.0) / 2.0
    }
}

/// RF signal power at the antenna in dBm.
///
/// It indicates the RF signal power at the antenna, in decibels difference from
/// one milliwatt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 1)]
pub struct AntennaSignal(pub i8);

/// RF signal power at the antenna in dB.
///
/// It indicates the RF signal power at the antenna, in decibels difference from
/// an arbitrary, fixed reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 1)]
pub struct AntennaSignalDb(pub u8);

/// RF noise power at the antenna in dBm.
///
/// It indicates the RF signal noise at the antenna, in decibels difference from
/// one milliwatt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 1)]
pub struct AntennaNoise(pub i8);

/// RF noise power at the antenna in dB.
///
/// It indicates the RF signal noise at the antenna, in decibels difference from
/// an arbitrary, fixed reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 1)]
pub struct AntennaNoiseDb(pub u8);

/// Quality of Barker code lock, unitless.
///
/// Monotonically nondecreasing with "better" lock strength. Called "Signal
/// Quality" in datasheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 2, size = 2)]
pub struct LockQuality(pub u16);

/// Transmit power expressed as unitless distance from max power.
///
/// Zero is max power. Monotonically nondecreasing with lower power levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 2, size = 2)]
pub struct TxAttenuation(pub u16);

/// Transmit power expressed as decibel distance from max power set at factory
/// calibration.
///
/// Zero is max power. Monotonically nondecreasing with lower power levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 2, size = 2)]
pub struct TxAttenuationDb(pub u16);

/// Transmit power in dBm.
///
/// This is the absolute power level measured at the antenna port in decibels
/// difference from one milliwatt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 1)]
pub struct TxPower(pub i8);

/// The antenna index.
///
/// Unitless indication of the Rx/Tx antenna for this packet. The first antenna
/// is antenna zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Field, FromArray)]
#[field(align = 1, size = 1)]
pub struct Antenna(pub u8);

bitflags! {
    /// Flags describing transmitted and received frames.
    #[derive(Field)]
    #[field(align = 1, size = 1)]
    pub struct Flags: u8 {
        /// The frame was sent/received during CFP.
        const CFP = 0x01;
        /// The frame was sent/received with short preamble.
        const PREAMBLE = 0x02;
        /// The frame was sent/received with WEP encryption.
        const WEP = 0x04;
        /// The frame was sent/received with fragmentation.
        const FRAG = 0x08;
        /// The frame includes FCS at the end.
        const FCS = 0x10;
        /// The frame has padding between 802.11 header and payload.
        const DATA_PAD = 0x20;
        /// The frame failed FCS check.
        const BAD_FCS = 0x40;
        /// The frame was sent/received with HT short Guard Interval.
        const SHORT_GI = 0x80;
    }
}

bitflags! {
    /// Properties of received frames.
    #[derive(Field)]
    #[field(align = 2, size = 2)]
    pub struct RxFlags: u16 {
        /// PLCP CRC check failed.
        const BAD_PLCP = 0x0002;
    }
}

bitflags! {
    /// Properties of transmitted frames.
    #[derive(Field)]
    #[field(align = 2, size = 2)]
    pub struct TxFlags: u16 {
        /// Transmission failed due to excessive retries.
        const FAIL = 0x0001;
        /// Transmission used CTS-to-self protection.
        const CTS = 0x0002;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Field, FromArray)]
#[field(align = 2, size = 2)]
pub struct Fhss {
    hop_set: u8,
    hop_pattern: u8,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every() {
        fn check<U: From<[u8; N]>, const N: usize>() -> U {
            U::from([0; N])
        }

        macro_rules! check {
            ($Field:ident) => {
                let _field: $Field = check();
            };
        }

        check!(Tsft);
        check!(Flags);
        check!(Rate);
        check!(Channel);
        check!(Fhss);
        check!(AntennaSignal);
        check!(AntennaNoise);
        check!(LockQuality);
        check!(TxAttenuation);
        check!(TxAttenuationDb);
        check!(TxPower);
        check!(Antenna);
        check!(AntennaSignalDb);
        check!(AntennaNoiseDb);
        check!(RxFlags);
        check!(TxFlags);
        // check!(RtsRetries);
        // check!(DataRetries);
        check!(XChannel);
        check!(Mcs);
        check!(AmpduStatus);
        check!(Vht);
        check!(Timestamp);
        // check!(He);
        // check!(HeMu);
        // check!(HeMuUser);
        // check!(ZeroLenPsdu);
        // check!(LSig);
    }
}
