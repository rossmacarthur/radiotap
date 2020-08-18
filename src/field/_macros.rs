macro_rules! ensure_length {
    ($cond:expr) => {
        if !$cond {
            return Err(crate::Error::InvalidFieldLength);
        }
    };
}

macro_rules! impl_kind {
    (
        $(#[$outer:meta])*
        pub enum $name:ident {
            $(
                $(#[$inner:ident $($args:tt)*])*
                $variant:ident { bit: $bit:expr, align: $align:expr, size: $size:expr },
            )+
        }
    ) => {
        $(#[$outer])*
        pub enum $name {
            $(
                $(#[$inner $($args)*])*
                $variant,
            )+
        }

        impl $name {
            pub fn from_bit(bit: u32) -> Option<Self> {
                match bit {
                    $( $bit => Some(Self::$variant), )+
                    _ => None
                }
            }

            /// Returns the present bit value for the field.
            pub fn bit(&self) -> u32 {
                match self { $( Self::$variant => $bit, )+ }
            }

            /// Returns the alignment of the field.
            pub fn align(&self) -> u32 {
                match self { $( Self::$variant => $align, )+ }
            }

            /// Returns the size of the field.
            pub fn size(&self) -> usize {
                match self { $( Self::$variant => $size, )+ }
            }
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
            #[allow(dead_code)]
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
        impl crate::bytes::FromBytes for $ty {
            fn from_bytes(bytes: crate::bytes::Bytes) -> crate::Result<Self> {
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
        bitflags::bitflags! {
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
                self.bits
            }
        }
    };
}
