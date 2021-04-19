#[macro_use]
mod macros;
mod args;
mod field;
mod from_array;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Field, attributes(field))]
pub fn derive_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    field::derive(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(FromArray)]
pub fn derive_from_array(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    from_array::derive(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
