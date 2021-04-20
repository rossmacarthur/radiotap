use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(data) => derive_impl(input, data),
        _ => bail!(input, "`#[derive(FromInto)]` is only supported on structs"),
    }
}

fn derive_impl(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let (inner_ty, body) = derive_body(&data.fields)?;
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::convert::From<#inner_ty> for #ty #ty_generics
            #where_clause
        {
            #[inline]
            fn from(t: #inner_ty) -> Self {
                #body
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#ty #ty_generics> for #inner_ty
            #where_clause
        {
            fn from(other: #ty #ty_generics) -> #inner_ty {
                other.0
            }
        }
    })
}

fn derive_body(fields: &Fields) -> Result<(&Type, TokenStream)> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) if named.len() == 1 => {
            let Field { ident, ty, .. } = &named[0];
            Ok((ty, quote! { Self { #ident: t } }))
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
            let Field { ty, .. } = &unnamed[0];
            Ok((ty, quote! { Self(t) }))
        }
        _ => bail!(fields, "only struct with single fields are supported"),
    }
}
