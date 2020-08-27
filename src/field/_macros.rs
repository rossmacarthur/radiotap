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
            /// Construct a new type from a bit. `None` if unknown.
            pub fn from_bit(bit: u32) -> Option<Self> {
                match bit {
                    $( $bit => Some(Self::$variant), )+
                    _ => None
                }
            }

            /// Returns the present bit for the type.
            pub fn bit(&self) -> u32 {
                match self { $( Self::$variant => $bit, )+ }
            }
        }

        impl crate::Kind for $name {
            /// Returns the alignment of the field.
            fn align(&self) -> usize {
                match self { $( Self::$variant => $align, )+ }
            }

            /// Returns the size of the field.
            fn size(&self) -> usize {
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

        #[allow(dead_code)]
        impl $name {
            pub(crate) fn from_bits(bits: $ty) -> Option<Self> {
                match bits {
                    $(
                        $value => Some(Self::$variant),
                    )+
                    _ => None
                }
            }

            /// Consumes this field and returns the underlying value.
            pub(crate) fn into_inner(self) -> $ty {
                match self {
                    $(
                        Self::$variant => $value,
                    )+
                }
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
        pub struct $name(pub(crate) $ty);

        impl FromBytes for $name {
            type Error = crate::bytes::Error;

            fn from_bytes(bytes: &mut crate::bytes::Bytes) -> crate::bytes::Result<Self> {
                bytes.read().map(Self)
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

        impl crate::bytes::FromBytes for $name {
            type Error = crate::bytes::Error;

            fn from_bytes(bytes: &mut crate::bytes::Bytes) -> crate::bytes::Result<Self> {
                bytes.read().map(Self::from_bits_truncate)
            }
        }

        impl $name {
            /// Consumes this field and returns the underlying value.
            #[inline]
            pub const fn into_inner(self) -> $ty {
                self.bits
            }
        }
    };
}
