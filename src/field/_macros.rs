macro_rules! ensure_length {
    ($cond:expr) => {
        if !$cond {
            return Err(crate::Error::InvalidFieldLength);
        }
    };
}

macro_rules! impl_enum {
    (
        $(#[$outer:meta])*
        pub enum $name:ident: $ty:ty{
            $(
                $(#[$inner:ident $($args:tt)*])*
                $variant:ident = $value:expr,
            )+
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $(
                $(#[$inner $($args)*])*
                $variant = $value,
            )+
        }

        impl $name {
            pub(crate) fn from_bits(bits: $ty) -> Option<Self> {
                match bits {
                    $(
                        $value => Some(Self::$variant),
                    )+
                    _ => None
                }
            }
        }
    };
}

macro_rules! impl_from_bytes_newtype {
    ($ty:ty) => {
        impl FromBytes for $ty {
            fn from_bytes(bytes: Bytes) -> crate::Result<Self> {
                Ok(Self(bytes.try_read()?))
            }
        }
    };
}

macro_rules! impl_newtype {
    (
        $(#[$outer:meta])*
        pub struct $name:ident($ty:ty);
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name($ty);

        impl_from_bytes_newtype!($name);

        impl PartialEq<$ty> for $name {
            fn eq(&self, other: &$ty) -> bool {
                self.0.eq(other)
            }
        }

        impl $name {
            /// Consumes this field and returns the underlying value.
            #[inline]
            pub const fn into_inner(self) -> $ty {
                self.0
            }
        }
    };
}

macro_rules! impl_from_bytes_bitflags {
    ($ty:ty) => {
        impl FromBytes for $ty {
            fn from_bytes(bytes: Bytes) -> crate::Result<Self> {
                Ok(Self::from_bits_truncate(bytes.try_read()?))
            }
        }
    };
}

macro_rules! impl_bitflags {
    (
        $(#[$outer:meta])*
        pub struct $name:ident: $ty:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $flag:ident = $value:expr;
            )+
        }
    ) => {
        bitflags! {
            $(#[$outer])*
            pub struct $name: $ty {
                $(
                    $(#[$inner $($args)*])*
                    const $flag = $value;
                )+
            }
        }

        impl_from_bytes_bitflags!($name);

        impl $name {
            /// Consumes this field and returns the underlying value.
            #[inline]
            pub const fn into_inner(self) -> $ty {
                self.bits()
            }
        }
    };
}
