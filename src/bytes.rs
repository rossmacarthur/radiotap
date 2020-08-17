use std::convert::TryInto;

use crate::Result;

/// Simple type alias for a slice of bytes.
pub type Bytes<'a> = &'a [u8];

/// Fallible conversion of bytes to a new type.
pub trait FromBytes: Sized {
    fn from_bytes(bytes: Bytes) -> Result<Self>;
}

/// Allows types implementing `FromBytes` to be easily read from a `Bytes`.
pub trait BytesExt {
    fn try_read<T: FromBytes>(&self) -> Result<T>;
}

impl BytesExt for [u8] {
    fn try_read<T: FromBytes>(&self) -> Result<T> {
        T::from_bytes(&self)
    }
}

macro_rules! impl_from_bytes_primitive {
    ($ty:ty) => {
        impl FromBytes for $ty {
            fn from_bytes(bytes: Bytes) -> Result<Self> {
                Ok(<$ty>::from_le_bytes(
                    bytes
                        .try_into()
                        .map_err(|_| crate::Error::InvalidFieldLength)?,
                ))
            }
        }
    };
}

impl_from_bytes_primitive!(u8);
impl_from_bytes_primitive!(u16);
impl_from_bytes_primitive!(u32);
impl_from_bytes_primitive!(u64);

impl_from_bytes_primitive!(i8);
impl_from_bytes_primitive!(i16);
impl_from_bytes_primitive!(i32);
impl_from_bytes_primitive!(i64);

/// Parse anything that implements `FromBytes` and return a `Result<Some<T>>`.
pub fn from_bytes_some<T>(bytes: Bytes) -> Result<Option<T>>
where
    T: FromBytes,
{
    Ok(Some(FromBytes::from_bytes(bytes)?))
}
