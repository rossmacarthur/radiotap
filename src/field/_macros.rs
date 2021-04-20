macro_rules! impl_kind {
    (
        $(#[$outer:meta])*
        pub enum $Kind:ident {
            $(
                $(#[$inner:ident $($args:tt)*])*
                $Variant:ident { bit: $BIT:expr, align: $ALIGN:expr, size: $SIZE:expr },
            )+
        }
    ) => {
        $(#[$outer])*
        pub enum $Kind {
            $(
                $(#[$inner $($args)*])*
                $Variant,
            )+
        }

        impl $Kind {
            /// Construct a new type from a bit. `None` if unknown.
            pub fn from_bit(bit: u32) -> Option<Self> {
                match bit {
                    $( $BIT => Some(Self::$Variant), )+
                    _ => None
                }
            }

            /// Returns the present bit for the type.
            pub fn bit(&self) -> u32 {
                match self { $( Self::$Variant => $BIT, )+ }
            }
        }

        impl crate::Kind for $Kind {
            /// Returns the alignment of the field.
            fn align(&self) -> usize {
                match self { $( Self::$Variant => $ALIGN, )+ }
            }

            /// Returns the size of the field.
            fn size(&self) -> usize {
                match self { $( Self::$Variant => $SIZE, )+ }
            }
        }

    };
}

macro_rules! impl_enum {
    (
        $(#[$outer:meta])*
        pub enum $Field:ident: $ty:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                $Variant:ident = $VALUE:expr,
            )+
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub enum $Field {
            $(
                $(#[$inner $($args)*])*
                $Variant = $VALUE,
            )+
        }

        #[allow(dead_code)]
        impl $Field {
            pub(crate) fn from_bits(bits: $ty) -> Option<Self> {
                match bits {
                    $( $VALUE => Some(Self::$Variant), )+
                    _ => None
                }
            }

            pub(crate) fn into_inner(self) -> $ty {
                match self { $( Self::$Variant => $VALUE, )+ }
            }
        }
    };
}

macro_rules! impl_bitflags {
    (
        $(#[$outer:meta])*
        pub struct $Field:ident: $ty:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $FLAG:ident = $VALUE:expr;
            )+
        }
    ) => {
        bitflags::bitflags! {
            $(#[$outer])*
            pub struct $Field: $ty {
                $(
                    $(#[$inner $($args)*])*
                    const $FLAG = $VALUE;
                )+
            }
        }

        impl From<$ty> for $Field {
            fn from(t: $ty) -> Self {
                Self::from_bits_truncate(t)
            }
        }

        impl From<[u8; { std::mem::size_of::<$ty>() }]> for $Field {
            fn from(bytes: [u8; { std::mem::size_of::<$ty>() }]) -> Self {
                Self::from(<$ty>::from_le_bytes(bytes))
            }
        }

        impl $Field {
            /// Consumes this field and returns the underlying value.
            #[inline]
            pub const fn into_inner(self) -> $ty {
                self.bits
            }
        }
    };
}
