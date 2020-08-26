//! A parser for the [radiotap](http://www.radiotap.org/) capture format.
//!
//! # Examples
//!
//! ### Parsing all fields
//!
//! The easiest way to use this crate is to parse a slice of bytes into a
//! [`Header`](struct.Header.html) struct.
//!
//! ```
//! // a capture off the wire or from a pcap file
//! let capture = &[0, 0, 0xd, 0x0, 0x5, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0x30, // ...
//! # ];
//! // parse the radiotap header from the capture into a `Header` struct
//! let header = radiotap::parse(capture).unwrap();
//!
//! // get the length of the entire header
//! let length = header.length();
//!
//! // unpack the desired parsed fields
//! let radiotap::Header { tsft, rate, .. } = header;
//! if let Some(tsft) = tsft {
//!     assert_eq!(tsft.into_inner(), 0x12345678);
//! }
//! # else { panic!("expected TSFT field") }
//! if let Some(rate) = rate {
//!     assert_eq!(rate.to_mbps(), 24.0);
//! }
//! # else { panic!("expected Rate field") }
//!
//! // using the length we can get the rest of the capture
//! // i.e. IEEE 802.11 header and body
//! let rest = &capture[length..];
//! ```
//!
//! ### Parsing no fields
//!
//! This crate can also be used to skip over the radiotap header without parsing
//! any of the fields.
//!
//! ```
//! // a capture off the wire or from a pcap file
//! let capture = &[0, 0, 0xd, 0x0, 0x5, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0x30, // ...
//! # ];
//!
//! // create an iterator which parses the first part of the
//! // radiotap header, enough to get a length
//! let iter = radiotap::Iter::new(capture).unwrap();
//!
//! // now we can get the rest of the capture
//! // i.e. IEEE 802.11 header and body
//! let rest = &capture[iter.length()..];
//! ```

pub mod bytes;
mod error;
pub mod field;
mod prelude;
mod util;

use std::error::Error as StdError;

use crate::error::ResultExt;
pub use crate::error::{Error, ErrorKind, Result};
use crate::field::{Kind, Type, VendorNamespace};
use crate::prelude::*;

/// The radiotap header version.
const VERSION: u8 = 0;

/// The presence bit representing the radiotap namespace.
const PRESENCE_DEFAULT_NAMESPACE: u32 = 29;

/// The presence bit representing a vendor namespace.
const PRESENCE_VENDOR_NAMESPACE: u32 = 30;

/// The presence bit representing another presence word follows.
const PRESENCE_EXT: u32 = 31;

/// A radiotap namespace.
#[derive(Debug, Clone)]
pub enum Namespace {
    /// The default radiotap namespace.
    Default,
    /// A custom vendor namespace.
    Vendor(VendorNamespace),
}

/// A generic field yielded by the radiotap iterator.
#[derive(Debug, Clone)]
pub struct Field {
    /// This field's namespace.
    namespace: Namespace,
    /// The presence bit for this field.
    bit: u32,
}

/// An iterator over a radiotap capture.
///
/// This type doesn't actually implement `Iterator` because it is fallible and
/// each yielded field requires that it is either explicitly read or skipped. If
/// you do not care about any vendor namespaces then you will want to use the
/// [`into_default`](struct.Iter.html#method.into_default) method to produce a
/// new 'filtered' iterator that skips over the vendor namespaces.
///
/// # Examples
///
/// ```
/// # fn main() -> radiotap::Result<()> {
/// let capture = &[0, 0, 0xd, 0x0, 0x5, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0x30, // ...
/// # ];
/// let mut iter = radiotap::Iter::new(capture)?;
///
/// while let Some(field) = iter.next()? {
///     match field.namespace() {
///         radiotap::Namespace::Default => {
///             let kind = match radiotap::field::Type::from_bit(field.bit()) {
///                 Some(kind) => kind,
///                 None => break,
///             };
///
///             match kind {
///                 radiotap::field::Type::Rate => {
///                     let rate: u8 = iter.read(kind)?;
///                     println!("Rate is {:.1} Mbps!", f32::from(rate) / 2.0);
///                 }
///                 kind => {
///                     iter.skip(kind)?;
///                 }
///             }
///         }
///         radiotap::Namespace::Vendor(vns) => iter.skip_vns(vns)?,
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    /// The raw bytes in this capture.
    bytes: Bytes<'a>,
    /// The expected length of the entire capture.
    length: usize,
    /// The presence words in this capture.
    presence: Vec<u32>,
    /// The current bit position in the presence words.
    position: u32,
    /// The current namespace.
    namespace: Namespace,
}

/// An iterator over a radiotap capture skipping any vendor namespaces.
///
/// This struct is created by the
/// [`into_default`](struct.Iter.html#method.into_default) method on
/// [`Iter`](struct.Iter.html).
///
/// This type doesn't actually implement Iterator because it is fallible and
/// each yielded field requires that it is either explicitly read or skipped.
///
/// # Examples
///
/// ```
/// # fn main() -> radiotap::Result<()> {
/// let capture = &[0, 0, 0xd, 0x0, 0x5, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0x30, // ...
/// # ];
/// let mut iter = radiotap::Iter::new(capture)?.into_default();
///
/// while let Some(kind) = iter.next()? {
///     match kind {
///         radiotap::field::Type::Rate => {
///             let rate: u8 = iter.read(kind)?;
///             println!("Rate is {:.1} Mbps!", f32::from(rate) / 2.0);
///         }
///         kind => iter.skip(kind)?,
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct IterDefault<'a> {
    inner: Iter<'a>,
}

/// A parsed radiotap capture.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Header {
    length: usize,
    pub tsft: Option<field::Tsft>,
    pub flags: Option<field::Flags>,
    pub rate: Option<field::Rate>,
    pub channel: Option<field::Channel>,
    pub fhss: Option<field::Fhss>,
    pub antenna_signal: Option<field::AntennaSignal>,
    pub antenna_noise: Option<field::AntennaNoise>,
    pub lock_quality: Option<field::LockQuality>,
    pub tx_attenuation: Option<field::TxAttenuation>,
    pub tx_attenuation_db: Option<field::TxAttenuationDb>,
    pub tx_power: Option<field::TxPower>,
    pub antenna: Option<field::Antenna>,
    pub antenna_signal_db: Option<field::AntennaSignalDb>,
    pub antenna_noise_db: Option<field::AntennaNoiseDb>,
    pub rx_flags: Option<field::RxFlags>,
    pub tx_flags: Option<field::TxFlags>,
    pub xchannel: Option<field::XChannel>,
    pub mcs: Option<field::Mcs>,
    pub ampdu_status: Option<field::AmpduStatus>,
    pub vht: Option<field::Vht>,
    pub timestamp: Option<field::Timestamp>,
}

impl Field {
    /// Returns this field's namespace.
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Returns this field's presence bit number.
    pub fn bit(&self) -> u32 {
        self.bit
    }
}

impl<'a> Iter<'a> {
    /// Returns a new radiotap iterator.
    ///
    /// # Errors
    ///
    /// This function will error if the radiotap version is unsupported or if
    /// there is not enough bytes in the capture for the length specified in the
    /// radiotap header.
    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        let mut bytes = Bytes::new(bytes);

        // the radiotap version, only 0 is supported
        let version = bytes.read().context(ErrorKind::Header)?;
        if version != VERSION {
            return Err(Error::new(ErrorKind::UnsupportedVersion(version)));
        }

        // padding byte
        bytes.advance(1).context(ErrorKind::Header)?;

        // the total length of the entire capture
        let length = bytes.read::<u16>().context(ErrorKind::Header)?.into();

        // the presence words
        let mut presence = Vec::new();
        loop {
            let word = bytes.read().context(ErrorKind::Header)?;
            presence.push(word);
            if word & (1 << PRESENCE_EXT) == 0 {
                break;
            }
        }

        Ok(Self {
            bytes,
            length,
            presence,
            position: 0,
            namespace: Namespace::Default,
        })
    }

    /// Returns the version of the radiotap header.
    pub fn version(&self) -> u8 {
        VERSION
    }

    /// Returns the entire length of the radiotap header.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Produce a new iterator that filters out any vendor namespaces and yields
    /// a [`Type`](field/enum.Type.html) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// // a capture off the wire
    /// let capture = &[0, 0, 0xd, 0x0, 0x5, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0x30, // ...
    /// # ];
    ///
    /// let iter = radiotap::Iter::new(capture).unwrap().into_default();
    /// ```
    pub fn into_default(self) -> IterDefault<'a> {
        IterDefault { inner: self }
    }

    /// Returns the next field in the iterator.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<Field>> {
        loop {
            match self.presence.get((self.position / 32) as usize) {
                Some(presence) => {
                    let bit = self.position % 32;
                    self.position += 1;

                    // if the bit is not set, then continue to next bit
                    if presence & (1 << bit) == 0 {
                        continue;
                    }

                    match bit {
                        PRESENCE_DEFAULT_NAMESPACE => {
                            // switching to radiotap namespace
                            self.namespace = Namespace::Default;
                            continue;
                        }
                        PRESENCE_VENDOR_NAMESPACE => {
                            // switching to vendor namespace
                            let context =
                                || ErrorKind::Read(std::any::type_name::<VendorNamespace>());
                            self.bytes.align(2).with_context(context)?;
                            self.namespace =
                                Namespace::Vendor(self.bytes.read().with_context(context)?);
                            continue;
                        }
                        PRESENCE_EXT => {
                            // same namespace, next word
                            continue;
                        }
                        bit => {
                            break Ok(Some(Field {
                                namespace: self.namespace.clone(),
                                bit,
                            }))
                        }
                    }
                }
                None => break Ok(None),
            }
        }
    }

    /// Skip the given kind of field.
    pub fn skip<T: Kind>(&mut self, kind: T) -> Result<()> {
        self.bytes.align(kind.align()).context(ErrorKind::Skip)?;
        self.bytes.advance(kind.size()).context(ErrorKind::Skip)?;
        Ok(())
    }

    /// Skip the given vendor namespace.
    pub fn skip_vns(&mut self, vns: &VendorNamespace) -> Result<()> {
        self.bytes
            .advance(vns.skip_length())
            .context(ErrorKind::Skip)
    }

    /// Reads the given kind of field.
    pub fn read<T, U, E>(&mut self, kind: T) -> Result<U>
    where
        T: Kind,
        U: FromBytes<Error = E>,
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        let context = || ErrorKind::Read(std::any::type_name::<U>());
        self.bytes.align(kind.align()).with_context(context)?;
        let start_pos = self.bytes.pos();
        let field = U::from_bytes(&mut self.bytes).with_context(context)?;
        let end_pos = self.bytes.pos();
        if end_pos - start_pos != kind.size() {
            return Err(Error::new(context()));
        }
        Ok(field)
    }

    fn read_some<T, U, E>(&mut self, kind: T) -> Result<Option<U>>
    where
        T: Kind,
        U: FromBytes<Error = E>,
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        self.read(kind).map(Some)
    }
}

impl IterDefault<'_> {
    /// Returns the version of the radiotap header.
    pub fn version(&self) -> u8 {
        self.inner.version()
    }

    /// Returns the entire length of the radiotap header.
    pub fn length(&self) -> usize {
        self.inner.length()
    }

    /// Returns the next field type in the iterator.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<Type>> {
        match self.inner.next()? {
            Some(Field {
                namespace: Namespace::Default,
                bit,
            }) => Ok(Type::from_bit(bit)),

            Some(Field {
                namespace: Namespace::Vendor(vns),
                ..
            }) => {
                self.inner.skip_vns(&vns)?;
                self.next()
            }

            None => Ok(None),
        }
    }

    /// Skip the given kind of field.
    pub fn skip(&mut self, kind: Type) -> Result<()> {
        self.inner.skip(kind)
    }

    /// Reads the given kind of field.
    pub fn read<U, E>(&mut self, kind: Type) -> Result<U>
    where
        U: FromBytes<Error = E>,
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        self.inner.read(kind)
    }

    fn read_some<U, E>(&mut self, kind: Type) -> Result<Option<U>>
    where
        U: FromBytes<Error = E>,
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        self.inner.read_some(kind)
    }
}

/// Parse a radiotap header from the given capture.
pub fn parse(capture: &[u8]) -> Result<Header> {
    let mut iter = Iter::new(capture)?.into_default();
    let mut radiotap = Header {
        length: iter.length(),
        tsft: None,
        flags: None,
        rate: None,
        channel: None,
        fhss: None,
        antenna_signal: None,
        antenna_noise: None,
        lock_quality: None,
        tx_attenuation: None,
        tx_attenuation_db: None,
        tx_power: None,
        antenna: None,
        antenna_signal_db: None,
        antenna_noise_db: None,
        rx_flags: None,
        tx_flags: None,
        xchannel: None,
        mcs: None,
        ampdu_status: None,
        vht: None,
        timestamp: None,
    };
    while let Some(kind) = iter.next()? {
        match kind {
            Type::Tsft => radiotap.tsft = iter.read_some(kind)?,
            Type::Flags => radiotap.flags = iter.read_some(kind)?,
            Type::Rate => radiotap.rate = iter.read_some(kind)?,
            Type::Channel => radiotap.channel = iter.read_some(kind)?,
            Type::Fhss => radiotap.fhss = iter.read_some(kind)?,
            Type::AntennaSignal => radiotap.antenna_signal = iter.read_some(kind)?,
            Type::AntennaNoise => radiotap.antenna_noise = iter.read_some(kind)?,
            Type::LockQuality => radiotap.lock_quality = iter.read_some(kind)?,
            Type::TxAttenuation => radiotap.tx_attenuation = iter.read_some(kind)?,
            Type::TxAttenuationDb => radiotap.tx_attenuation_db = iter.read_some(kind)?,
            Type::TxPower => radiotap.tx_power = iter.read_some(kind)?,
            Type::Antenna => radiotap.antenna = iter.read_some(kind)?,
            Type::AntennaSignalDb => radiotap.antenna_signal_db = iter.read_some(kind)?,
            Type::AntennaNoiseDb => radiotap.antenna_noise_db = iter.read_some(kind)?,
            Type::RxFlags => radiotap.rx_flags = iter.read_some(kind)?,
            Type::TxFlags => radiotap.tx_flags = iter.read_some(kind)?,
            Type::XChannel => radiotap.xchannel = iter.read_some(kind)?,
            Type::Mcs => radiotap.mcs = iter.read_some(kind)?,
            Type::AmpduStatus => radiotap.ampdu_status = iter.read_some(kind)?,
            Type::Vht => radiotap.vht = iter.read_some(kind)?,
            Type::Timestamp => radiotap.timestamp = iter.read_some(kind)?,
            kind => iter.skip(kind)?,
        }
    }
    Ok(radiotap)
}

impl Header {
    /// Returns the version of the radiotap header.
    #[inline]
    pub fn version(&self) -> u8 {
        VERSION
    }

    /// Returns the length of the entire radiotap header.
    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        // Radiotap Header v0, Length 56
        //     Header version: 0
        //     Header pad: 0
        //     Header length: 56
        //     Present flags
        //         Present flags word: 0xc000086f
        //         Present flags word: 0x40000001
        //     MAC timestamp: 77325725
        //     Flags: 0x12
        //     Data Rate: 24.0 Mb/s
        //     Channel frequency: 2437 [BG 6]
        //     Channel flags: 0x0480, 2 GHz spectrum, Dynamic CCK-OFDM
        //     Antenna signal: -76dBm
        //     Antenna noise: -89dBm
        //     Antenna: 0
        //     Vendor namespace: Broadcom-0
        //         Vendor OUI: 00:10:18 (Broadcom)
        //         Vendor sub namespace: 0
        //         Vendor data length: 3
        //         Vendor data
        //     Vendor namespace: Broadcom-3
        //         Vendor OUI: 00:10:18 (Broadcom)
        //         Vendor sub namespace: 3
        //         Vendor data length: 6
        //         Vendor data

        let capture = hex::decode(
            "000038006f0800c001000040040030309de59b040000000012308509\
             8004b4a7008700101800030002000000001018030600400002000000",
        )
        .unwrap();

        let radiotap = parse(&capture).unwrap();
        assert_eq!(radiotap.length(), 56);
        assert_eq!(radiotap.tsft.unwrap().into_inner(), 77325725);
        assert_eq!(
            radiotap.flags.unwrap(),
            field::Flags::PREAMBLE | field::Flags::FCS
        );
        assert_eq!(radiotap.rate.unwrap().to_mbps(), 24.0);
    }
}
