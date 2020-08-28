pub trait BoolExt {
    fn some<T>(self, f: impl FnOnce() -> T) -> Option<T>;
}

impl BoolExt for bool {
    fn some<T>(self, f: impl FnOnce() -> T) -> Option<T> {
        if self {
            Some(f())
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod fromhex {
    use std::result;

    use crate::prelude::*;

    pub trait FromHex: FromBytes {
        /// Construct a type from a hex string of bytes.
        fn from_hex(s: &str) -> result::Result<Self, Self::Error> {
            let b = hex::decode(s).unwrap();
            Self::from_bytes(&mut Bytes::from_slice(&b))
        }
    }

    impl<T> FromHex for T where T: FromBytes {}
}
