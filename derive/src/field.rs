use std::env;

use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

use crate::args::Args;

struct Params {
    align: LitInt,
    size: LitInt,
}

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => derive_impl(input),
        _ => bail!(input, "`#[derive(Field)]` is only supported on structs"),
    }
}

fn derive_impl(input: &DeriveInput) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let Params { align, size } = parse_attr(input)?;

    // This is that we can use the derive macros from within the crate.
    let radiotap = if env::var("CARGO_CRATE_NAME").ok() == Some("radiotap".to_owned()) {
        quote! { crate }
    } else {
        quote! { ::radiotap }
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #radiotap::field::Field<#align, #size> for #ty #ty_generics
            #where_clause
        {
            #[inline]
            fn from_bytes(bytes: [u8; #size]) -> Self {
                bytes.into()
            }
        }
    })
}

fn parse_attr(input: &DeriveInput) -> Result<Params> {
    for attr in &input.attrs {
        if attr.path.is_ident("field") {
            let mut args: Args = attr.parse_args()?;
            let mut pop = |k| {
                args.remove_lit_int(k)?
                    .ok_or_else(|| error!(&attr, "missing `{}`", k))
            };
            let align = pop("align")?;
            let size = pop("size")?;
            args.ensure_empty()?;
            return Ok(Params { align, size });
        }
    }
    bail!(
        input,
        "`#[derive(Field)]` requires a `#[field(..)]` attribute"
    );
}
