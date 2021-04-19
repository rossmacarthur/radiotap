#![cfg(test)]

use std::convert::TryInto;

pub trait FromHex<const N: usize>: From<[u8; N]> {
    /// Construct a type from a hex string of bytes.
    fn from_hex(s: &str) -> Self {
        let b = hex::decode(s).unwrap();
        Self::from(b.try_into().unwrap())
    }
}

impl<T, const N: usize> FromHex<N> for T where T: From<[u8; N]> {}
