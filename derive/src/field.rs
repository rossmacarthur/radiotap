use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

use crate::args::Args;

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => derive_struct(input),
        _ => bail!(input, "`#[derive(Field)]` is only supported on structs"),
    }
}

pub fn derive_struct(input: &DeriveInput) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let Args { align, size } = Args::new(input)?;
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::radiotap::field::Field<#align, #size> for #ty #ty_generics
            #where_clause
        {
            #[inline]
            fn from_bytes(bytes: [u8; #size]) -> Self {
                bytes.into()
            }
        }
    })
}
