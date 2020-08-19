//! A parser for the [radiotap](http://www.radiotap.org/) capture format.

pub mod bytes;
pub mod field;
mod prelude;
mod util;

use std::result;

use thiserror::Error;

use crate::bytes::{Bytes, FromBytes};
use crate::field::{Kind, Type, VendorNamespace};

/// A result type to use throughout this crate.
pub type Result<T> = result::Result<T, Error>;

/// The radiotap header version.
const VERSION: u8 = 0;

/// The presence bit representing the radiotap namespace.
const PRESENCE_DEFAULT_NAMESPACE: u32 = 29;

/// The presence bit representing a vendor namespace.
const PRESENCE_VENDOR_NAMESPACE: u32 = 30;

/// The presence bit representing another presence word follows.
const PRESENCE_EXT: u32 = 31;

/// All errors that can occur in this crate..
#[derive(Debug, PartialEq, Error)]
#[non_exhaustive]
pub enum Error {
    /// Unsupported radiotap version.
    #[error("unsupported radiotap version `{version}`")]
    UnsupportedVersion { version: u8 },

    /// The given data is shorter than the amount required.
    #[error("the given data of length `{actual}` is shorter than the required `{required}`")]
    InvalidLength { required: usize, actual: usize },
}

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

/// A parsed radiotap capture.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Radiotap {
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
        let version = bytes.read()?;
        if version != VERSION {
            return Err(Error::UnsupportedVersion { version });
        }

        // padding byte
        bytes.advance(1)?;

        // the total length of the entire capture
        let length = bytes.read::<u16>()?.into();
        if bytes.len() < length {
            return Err(Error::InvalidLength {
                required: length,
                actual: bytes.len(),
            });
        }

        // the presence words
        let mut presence = Vec::new();
        loop {
            let word = bytes.read()?;
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

    /// The version of the radiotap header.
    #[inline]
    pub fn version(&self) -> u8 {
        VERSION
    }

    /// The length of the entire radiotap header.
    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }

    /// Returns the next field in the iterator.
    pub fn next_field(&mut self) -> Result<Option<Field>> {
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
                            self.bytes.align(2)?;
                            self.namespace = Namespace::Vendor(self.bytes.read()?);
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
        self.bytes.align(kind.align())?;
        self.bytes.advance(kind.size())?;
        Ok(())
    }

    /// Skip the given vendor namespace.
    pub fn skip_vns(&mut self, vns: &VendorNamespace) -> Result<()> {
        self.bytes.advance(vns.skip_length())
    }

    /// Reads the given kind of field.
    pub fn read<T: Kind, U: FromBytes>(&mut self, kind: T) -> Result<U> {
        self.bytes.align(kind.align())?;
        let field = U::from_bytes(&mut self.bytes.bytes(kind.size())?)?;
        self.bytes.advance(kind.size())?;
        Ok(field)
    }

    #[inline]
    fn read_some<T: Kind, U: FromBytes>(&mut self, kind: T) -> Result<Option<U>> {
        self.read(kind).map(Some)
    }
}

pub fn parse(bytes: &[u8]) -> Result<Radiotap> {
    let mut iter = Iter::new(bytes)?;
    let mut radiotap = Radiotap {
        length: iter.length,
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
    while let Some(field) = iter.next_field()? {
        match field.namespace() {
            Namespace::Default => {
                let kind = match Type::from_bit(field.bit()) {
                    // we cannot continue here because we don't
                    // know how to advance the iterator
                    None => break,
                    Some(kind) => kind,
                };
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
            Namespace::Vendor(vns) => {
                iter.skip_vns(&vns)?;
            }
        }
    }
    Ok(radiotap)
}

impl Radiotap {
    /// Returns the length of the entire radiotap header.
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
        assert_eq!(radiotap.len(), 56);
        assert_eq!(radiotap.tsft.unwrap().into_inner(), 77325725);
        assert_eq!(
            radiotap.flags.unwrap(),
            field::Flags::PREAMBLE | field::Flags::FCS
        );
        assert_eq!(radiotap.rate.unwrap().to_mbps(), 24.0);
    }
}
