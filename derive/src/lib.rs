#[macro_use]
mod _macros;
mod args;
mod field;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Field, attributes(field))]
pub fn derive_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    field::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
