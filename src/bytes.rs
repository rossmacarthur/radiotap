//! Defines a cursor over a slice of bytes.

use std::mem;

use crate::{Error, Result};

/// A cursor over a slice of bytes.
#[derive(Debug, Clone)]
pub struct Bytes<'a> {
    inner: &'a [u8],
    pos: usize,
}

/// Fallible conversion of bytes to a new type.
///
/// All provided radiotap fields already implement this trait, but if you want
/// to read a custom field from the radiotap iterator then you will need to
/// implement it.
///
/// # Examples
///
/// In the following example we parse a copy of the built-in
/// [`Rate`](field/struct.Rate.html) field.
///
/// ```rust
/// use radiotap::{Bytes, FromBytes, Result};
///
/// struct MyRate {
///     value: u8
/// }
///
/// impl FromBytes for MyRate {
///     fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
///         let value = bytes.read()?;
///         Ok(Self { value })
///     }
/// }
/// ```
pub trait FromBytes: Sized {
    /// Construct a type from bytes.
    ///
    /// This method is often used implicitly through
    /// [`Bytes`](struct.Bytes.html)'s [`read`](struct.Bytes.html#read) method.
    fn from_bytes(bytes: &mut Bytes) -> Result<Self>;

    /// Construct a type from a hex string of bytes.
    #[cfg(test)]
    fn from_hex(s: &str) -> Result<Self> {
        let b = hex::decode(s).unwrap();
        let mut bytes = Bytes::new(&b);
        Self::from_bytes(&mut bytes)
    }
}

impl<'a> Bytes<'a> {
    /// Returns a new cursor over a slice of bytes.
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            inner: bytes,
            pos: 0,
        }
    }

    /// Returns the total length of the original underlying buffer.
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the remaining length of the underlying buffer.
    fn remaining(&self) -> usize {
        self.len().saturating_sub(self.pos)
    }

    fn checked_pos(&self, new_pos: usize) -> Result<usize> {
        if new_pos > self.len() {
            Err(Error::InvalidLength {
                required: new_pos - self.pos,
                actual: self.remaining(),
            })
        } else {
            Ok(new_pos)
        }
    }

    fn set_pos(&mut self, new_pos: usize) -> Result<()> {
        self.pos = self.checked_pos(new_pos)?;
        Ok(())
    }

    /// Aligns the bytes to a particular word.
    ///
    /// # Panics
    ///
    /// If the align size is a not one of the following powers of two: 1, 2, 4,
    /// 8, or 16.
    pub(crate) fn align(&mut self, align: usize) -> Result<()> {
        assert!(matches!(align, 1 | 2 | 4 | 8 | 16));
        self.set_pos((self.pos + align - 1) & !(align - 1))
    }

    /// Advances the cursor over some bytes.
    pub(crate) fn advance(&mut self, size: usize) -> Result<()> {
        self.set_pos(self.pos + size)
    }

    /// Gets a slice of the given size and doesn't advance the underlying
    /// buffer.
    pub(crate) fn slice(&self, size: usize) -> Result<&'a [u8]> {
        let end = self.checked_pos(self.pos + size)?;
        Ok(&self.inner[self.pos..end])
    }

    /// Gets a new bytes of the given size and doesn't advance the underlying
    /// buffer of this bytes.
    pub(crate) fn bytes(&self, size: usize) -> Result<Self> {
        Ok(Self::new(self.slice(size)?))
    }

    pub(crate) fn read_slice(&mut self, size: usize) -> Result<&'a [u8]> {
        let start = self.pos;
        self.pos = self.checked_pos(start + size)?;
        Ok(&self.inner[start..self.pos])
    }

    /// Allows types implementing [`FromBytes`](trait.FromBytes.html) to be
    /// easily read from this bytes.
    ///
    /// # Errors
    ///
    /// If there is not enough bytes remaining to take the action required.
    pub fn read<T: FromBytes>(&mut self) -> Result<T> {
        T::from_bytes(self)
    }
}

macro_rules! impl_primitive {
    ($ty:ty) => {
        impl FromBytes for $ty {
            fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
                const COUNT: usize = mem::size_of::<$ty>();
                let mut buf = [0; COUNT];
                buf.copy_from_slice(bytes.read_slice(COUNT)?);
                Ok(Self::from_le_bytes(buf))
            }
        }
    };
}

impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u128);

impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(i128);

macro_rules! impl_array {
    ($size:expr) => {
        impl<T: FromBytes + Default> FromBytes for [T; $size] {
            fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
                let mut buf = Self::default();
                for i in 0..$size {
                    buf[i] = bytes.read()?;
                }
                Ok(buf)
            }
        }
    };
}

impl_array!(1);
impl_array!(2);
impl_array!(3);
impl_array!(4);

/////////////////////////////////////////////////////////////////////////
// Unit tests
/////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_align() {
        fn check_align(align: usize, expected_pos: usize) {
            let mut bytes = Bytes {
                inner: &[0; 25],
                pos: 13,
            };
            bytes.align(align).unwrap();
            assert_eq!(bytes.pos, expected_pos);
        }

        check_align(1, 13);
        check_align(2, 14);
        check_align(4, 16);
        check_align(8, 16);
        check_align(16, 16);
    }

    #[test]
    fn bytes_read_primitive_x8() {
        let mut bytes = Bytes::new(&[1, !2 + 1]);
        assert_eq!(bytes.read::<u8>().unwrap(), 1);
        assert_eq!(bytes.read::<i8>().unwrap(), -2);
    }

    #[test]
    fn bytes_read_primitive_x16() {
        let mut bytes = Bytes::new(&[0xfb, 0xfa, 0xff, 0xff]);
        assert_eq!(bytes.read::<u16>().unwrap(), 0xfafb);
        assert_eq!(bytes.read::<i16>().unwrap(), -0x0001);
    }

    #[test]
    fn bytes_read_array_primitives() {
        let mut bytes = Bytes::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        assert_eq!(
            bytes.read::<[u32; 3]>().unwrap(),
            [0x04030201, 0x08070605, 0x0C0B0A09]
        );
    }

    #[test]
    fn bytes_read_array_newtype() {
        #[derive(Debug, Default, PartialEq)]
        struct NewType(i16);

        impl FromBytes for NewType {
            fn from_bytes(bytes: &mut Bytes) -> Result<Self> {
                bytes.read().map(Self)
            }
        }

        let mut bytes = Bytes::new(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(
            bytes.read::<[NewType; 3]>().unwrap(),
            [NewType(0x0201), NewType(0x0403), NewType(0x0605)]
        );
    }
}
