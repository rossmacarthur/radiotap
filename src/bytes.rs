use std::mem;

// ///
// trait BytesExt<const N: usize> {
//     pub fn <T: FromBytes>(self) -> T {
//         T::from_bytes(self)
//     }
// }


/// Infallible conversion of bytes to a new type.
pub trait FromBytes<const N: usize> {
    fn from_bytes(bytes: [u8; N]) -> Self;
}


macro_rules! impl_primitive {
    ($Type:ty) => {
        impl FromBytes<{ mem::size_of::<$Type>() }> for $Type {
            fn from_bytes(bytes: [u8; { mem::size_of::<$Type>() }]) -> Self {
                Self::from_le_bytes(bytes)
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

/////////////////////////////////////////////////////////////////////////
// Unit tests
/////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;


}
