//! A parser for the [radiotap](http://www.radiotap.org/) capture format.
//!
//! # Usage
//!
//! The `Radiotap::from_bytes(&capture)` constructor will parse all present
//! fields into a [Radiotap](struct.Radiotap.html) struct:
//!
//! ```
//! # use radiotap::Radiotap;
//! let capture = [
//!     0, 0, 56, 0, 107, 8, 52, 0, 185, 31, 155, 154, 0, 0, 0, 0, 20, 0, 124, 21, 64, 1, 213,
//!     166, 1, 0, 0, 0, 64, 1, 1, 0, 124, 21, 100, 34, 249, 1, 0, 0, 0, 0, 0, 0, 255, 1, 80,
//!     4, 115, 0, 0, 0, 1, 63, 0, 0,
//! ];
//!
//! let radiotap = Radiotap::from_bytes(&capture).unwrap();
//! println!("{:?}", radiotap.vht);
//! ```
//!
//! If you just want to parse a few specific fields from the radiotap capture
//! you can create an iterator using `RadiotapIterator::from_bytes(&capture)`:
//!
//! ```
//! # use radiotap::field::{self, FromBytes};
//! # use radiotap::RadiotapIterator;
//! let capture = [
//!     0, 0, 56, 0, 107, 8, 52, 0, 185, 31, 155, 154, 0, 0, 0, 0, 20, 0, 124, 21, 64, 1, 213,
//!     166, 1, 0, 0, 0, 64, 1, 1, 0, 124, 21, 100, 34, 249, 1, 0, 0, 0, 0, 0, 0, 255, 1, 80,
//!     4, 115, 0, 0, 0, 1, 63, 0, 0,
//! ];
//!
//! for element in RadiotapIterator::from_bytes(&capture).unwrap() {
//!     match element {
//!         Ok((field::Kind::Vht, data)) => {
//!             let vht = field::Vht::from_bytes(data).unwrap();
//!             println!("{:?}", vht);
//!         }
//!         _ => {}
//!     }
//! }
//! ```

pub mod ext;
pub mod field;

use std::{
    io::{self, Cursor},
    result,
};

use thiserror::Error;

use crate::field::*;

/// All errors returned and used by the radiotap module.
#[derive(Debug, Error)]
pub enum Error {
    /// The internal cursor on the data returned an IO error.
    #[error("parse error: {err}")]
    ParseError {
        #[from]
        err: io::Error,
    },
    /// The given data is not a complete radiotap capture.
    #[error("the given data is not a complete radiotap capture")]
    IncompleteError,
    /// The given data is shorter than the amount specified in the radiotap
    /// header.
    #[error("the given data is shorter than the amount specified in the radiotap header")]
    InvalidLength,
    /// The given data is not a valid radiotap capture.
    #[error("the given data is not a valid radiotap capture")]
    InvalidFormat,
    /// Unsupported radiotap header version.
    #[error("unsupported radiotap header version")]
    UnsupportedVersion,
    /// Unsupported radiotap field.
    #[error("unsupported radiotap field")]
    UnsupportedField,
}

type Result<T> = result::Result<T, Error>;

/// A trait to align an offset to particular word size, usually 1, 2, 4, or 8.
trait Align {
    /// Aligns the offset to `align` size.
    fn align(&mut self, align: u64);
}

impl<T> Align for Cursor<T> {
    /// Aligns the Cursor position to `align` size.
    fn align(&mut self, align: u64) {
        let p = self.position();
        self.set_position((p + align - 1) & !(align - 1));
    }
}

/// Represents an unparsed radiotap capture format, only the header field is
/// parsed.
#[derive(Debug, Clone)]
pub struct RadiotapIterator<'a> {
    header: Header,
    data: &'a [u8],
}

impl<'a> RadiotapIterator<'a> {
    pub fn from_bytes(input: &'a [u8]) -> Result<RadiotapIterator<'a>> {
        Ok(RadiotapIterator::parse(input)?.0)
    }

    pub fn parse(input: &'a [u8]) -> Result<(RadiotapIterator<'a>, &'a [u8])> {
        let header = Header::from_bytes(input)?;
        let (data, rest) = input.split_at(header.length);
        Ok((RadiotapIterator { header, data }, rest))
    }
}

/// An iterator over radiotap fields.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RadiotapIteratorIntoIter<'a> {
    present: Vec<Kind>,
    cursor: Cursor<&'a [u8]>,
}

impl<'a> IntoIterator for &'a RadiotapIterator<'a> {
    type IntoIter = RadiotapIteratorIntoIter<'a>;
    type Item = Result<(Kind, &'a [u8])>;

    fn into_iter(self) -> Self::IntoIter {
        let present = self.header.present.iter().rev().cloned().collect();
        let mut cursor = Cursor::new(self.data);
        cursor.set_position(self.header.size as u64);
        RadiotapIteratorIntoIter { present, cursor }
    }
}

impl<'a> IntoIterator for RadiotapIterator<'a> {
    type IntoIter = RadiotapIteratorIntoIter<'a>;
    type Item = Result<(Kind, &'a [u8])>;

    fn into_iter(self) -> Self::IntoIter {
        let present = self.header.present.iter().rev().cloned().collect();
        let mut cursor = Cursor::new(self.data);
        cursor.set_position(self.header.size as u64);
        RadiotapIteratorIntoIter { present, cursor }
    }
}

impl<'a> Iterator for RadiotapIteratorIntoIter<'a> {
    type Item = Result<(Kind, &'a [u8])>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.present.pop() {
            Some(mut kind) => {
                // Align the cursor to the current field's needed alignment.
                self.cursor.align(kind.align());

                let mut start = self.cursor.position() as usize;
                let mut end = start + kind.size();

                // The header lied about how long the body was
                if end > self.cursor.get_ref().len() {
                    Some(Err(Error::IncompleteError))
                } else {
                    // Switching to a vendor namespace, and we don't know how to handle
                    // so we just return the entire vendor namespace section
                    if kind == Kind::VendorNamespace(None) {
                        match VendorNamespace::from_bytes(&self.cursor.get_ref()[start..end]) {
                            Ok(vns) => {
                                start += kind.size();
                                end += vns.skip_length as usize;
                                kind = Kind::VendorNamespace(Some(vns));
                            }
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    let data = &self.cursor.get_ref()[start..end];
                    self.cursor.set_position(end as u64);
                    Some(Ok((kind, data)))
                }
            }
            None => None,
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            version: 0,
            length: 8,
            present: Vec::new(),
            size: 8,
        }
    }
}

/// Represents a parsed radiotap capture, including the parsed header and all
/// fields as Option members.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Radiotap {
    pub header: Header,
    pub tsft: Option<Tsft>,
    pub flags: Option<Flags>,
    pub rate: Option<Rate>,
    pub channel: Option<Channel>,
    pub fhss: Option<Fhss>,
    pub antenna_signal: Option<AntennaSignal>,
    pub antenna_noise: Option<AntennaNoise>,
    pub lock_quality: Option<LockQuality>,
    pub tx_attenuation: Option<TxAttenuation>,
    pub tx_attenuation_db: Option<TxAttenuationDb>,
    pub tx_power: Option<TxPower>,
    pub antenna: Option<Antenna>,
    pub antenna_signal_db: Option<AntennaSignalDb>,
    pub antenna_noise_db: Option<AntennaNoiseDb>,
    pub rx_flags: Option<RxFlags>,
    pub tx_flags: Option<TxFlags>,
    pub rts_retries: Option<RtsRetries>,
    pub data_retries: Option<DataRetries>,
    pub xchannel: Option<XChannel>,
    pub mcs: Option<Mcs>,
    pub ampdu_status: Option<AmpduStatus>,
    pub vht: Option<Vht>,
    pub timestamp: Option<Timestamp>,
}

impl Radiotap {
    /// Returns the parsed [Radiotap](struct.Radiotap.html) from an input byte
    /// array.
    pub fn from_bytes(input: &[u8]) -> Result<Self> {
        Ok(Self::parse(input)?.0)
    }

    /// Returns the parsed [Radiotap](struct.Radiotap.html) and remaining data
    /// from an input byte array.
    pub fn parse(input: &[u8]) -> Result<(Self, &[u8])> {
        let (iterator, rest) = RadiotapIterator::parse(input)?;

        let mut radiotap = Self {
            header: iterator.header.clone(),
            ..Default::default()
        };

        for result in &iterator {
            let (field_kind, data) = result?;

            match field_kind {
                Kind::Tsft => radiotap.tsft = from_bytes_some(data)?,
                Kind::Flags => radiotap.flags = from_bytes_some(data)?,
                Kind::Rate => radiotap.rate = from_bytes_some(data)?,
                Kind::Channel => radiotap.channel = from_bytes_some(data)?,
                Kind::Fhss => radiotap.fhss = from_bytes_some(data)?,
                Kind::AntennaSignal => radiotap.antenna_signal = from_bytes_some(data)?,
                Kind::AntennaNoise => radiotap.antenna_noise = from_bytes_some(data)?,
                Kind::LockQuality => radiotap.lock_quality = from_bytes_some(data)?,
                Kind::TxAttenuation => radiotap.tx_attenuation = from_bytes_some(data)?,
                Kind::TxAttenuationDb => radiotap.tx_attenuation_db = from_bytes_some(data)?,
                Kind::TxPower => radiotap.tx_power = from_bytes_some(data)?,
                Kind::Antenna => radiotap.antenna = from_bytes_some(data)?,
                Kind::AntennaSignalDb => radiotap.antenna_signal_db = from_bytes_some(data)?,
                Kind::AntennaNoiseDb => radiotap.antenna_noise_db = from_bytes_some(data)?,
                Kind::RxFlags => radiotap.rx_flags = from_bytes_some(data)?,
                Kind::TxFlags => radiotap.tx_flags = from_bytes_some(data)?,
                Kind::RtsRetries => radiotap.rts_retries = from_bytes_some(data)?,
                Kind::DataRetries => radiotap.data_retries = from_bytes_some(data)?,
                Kind::XChannel => radiotap.xchannel = from_bytes_some(data)?,
                Kind::Mcs => radiotap.mcs = from_bytes_some(data)?,
                Kind::AmpduStatus => radiotap.ampdu_status = from_bytes_some(data)?,
                Kind::Vht => radiotap.vht = from_bytes_some(data)?,
                Kind::Timestamp => radiotap.timestamp = from_bytes_some(data)?,
                _ => {}
            }
        }

        Ok((radiotap, rest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_vendor() {
        let frame = [
            0, 0, 39, 0, 46, 72, 0, 192, 0, 0, 0, 128, 0, 0, 0, 160, 4, 0, 0, 0, 16, 2, 158, 9,
            160, 0, 227, 5, 0, 0, 255, 255, 255, 255, 2, 0, 222, 173, 4,
        ];

        assert_eq!(
            Radiotap::from_bytes(&frame).unwrap().rate.unwrap(),
            Rate { value: 2.0 }
        );
    }

    #[test]
    fn bad_version() {
        let frame = [
            1, 0, 39, 0, 46, 72, 0, 192, 0, 0, 0, 128, 0, 0, 0, 160, 4, 0, 0, 0, 16, 2, 158, 9,
            160, 0, 227, 5, 0, 0, 255, 255, 255, 255, 2, 0, 222, 173, 4,
        ];

        match Radiotap::from_bytes(&frame).unwrap_err() {
            Error::UnsupportedVersion => {}
            e => panic!("Error not UnsupportedVersion: {:?}", e),
        };
    }

    #[test]
    fn bad_header_length() {
        let frame = [
            0, 0, 40, 0, 46, 72, 0, 192, 0, 0, 0, 128, 0, 0, 0, 160, 4, 0, 0, 0, 16, 2, 158, 9,
            160, 0, 227, 5, 0, 0, 255, 255, 255, 255, 2, 0, 222, 173, 4,
        ];

        match Radiotap::from_bytes(&frame).unwrap_err() {
            Error::InvalidLength => {}
            e => panic!("Error not InvalidLength: {:?}", e),
        };
    }

    #[test]
    fn bad_actual_length() {
        let frame = [
            0, 0, 39, 0, 47, 72, 0, 192, 0, 0, 0, 128, 0, 0, 0, 160, 4, 0, 0, 0, 16, 2, 158, 9,
            160, 0, 227, 5, 0, 0, 255, 255, 255, 255, 2, 0, 222, 173, 4,
        ];

        match Radiotap::from_bytes(&frame).unwrap_err() {
            Error::IncompleteError => {}
            e => panic!("Error not IncompleteError: {:?}", e),
        };
    }

    #[test]
    fn bad_vendor() {
        let frame = [
            0, 0, 34, 0, 46, 72, 0, 192, 0, 0, 0, 128, 0, 0, 0, 160, 4, 0, 0, 0, 16, 2, 158, 9,
            160, 0, 227, 5, 0, 0, 255, 255, 255, 255,
        ];

        match Radiotap::from_bytes(&frame).unwrap_err() {
            Error::IncompleteError => {}
            e => panic!("Error not IncompleteError: {:?}", e),
        };
    }
}
