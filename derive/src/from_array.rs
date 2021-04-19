use std::mem::size_of;

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::*;

use crate::args::Args;

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(data) => derive_impl(input, data),
        _ => bail!(input, "`#[derive(FromArray)]` is only supported on structs"),
    }
}

fn derive_impl(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let (size, body) = derive_body(&data.fields)?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::convert::From<[u8; #size]> for #ty #ty_generics
            #where_clause
        {
            #[inline]
            fn from(bytes: [u8; #size]) -> Self {
                #body
            }
        }
    })
}

fn derive_body(fields: &Fields) -> Result<(usize, TokenStream)> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            derive_assigns(named).map(|(s, a)| (s, quote! { Self { #(#a),* } }))
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            derive_assigns(unnamed).map(|(s, a)| (s, quote! { Self ( #(#a),* ) }))
        }
        Fields::Unit => Ok((0, quote! { Self })),
    }
}

fn derive_assigns(fields: &Punctuated<Field, Token![,]>) -> Result<(usize, Vec<TokenStream>)> {
    let mut assigns = Vec::new();
    let mut index = 0;
    for field in fields {
        let (size, from_array) = derive_ty_from_array(index, field)?;
        index += size;
        assigns.push(match &field.ident {
            Some(ident) => quote! { #ident: #from_array },
            None => quote! { #from_array },
        });
    }
    Ok((index, assigns))
}

fn derive_ty_from_array(index: usize, field: &Field) -> Result<(usize, TokenStream)> {
    let sizes = [
        ("u8", size_of::<u8>()),
        ("i8", size_of::<i8>()),
        ("u16", size_of::<u16>()),
        ("i16", size_of::<i16>()),
        ("i32", size_of::<i32>()),
        ("u32", size_of::<u32>()),
        ("u64", size_of::<u64>()),
        ("i64", size_of::<i64>()),
        ("u128", size_of::<u128>()),
        ("i128", size_of::<i128>()),
        ("f32", size_of::<f32>()),
        ("f64", size_of::<f64>()),
    ];

    let Field { ty, .. } = field;

    if let Some(size) = parse_attr(field)? {
        let array = splice_bytes(index, size);
        return Ok((size, quote! { #ty::from(#array) }));
    }

    match ty {
        // Fixed array like [u8; 4]
        Type::Array(TypeArray {
            elem,
            len: Expr::Lit(ExprLit {
                lit: Lit::Int(lit), ..
            }),
            ..
        }) if is_ident(elem, "u8") => {
            let size = lit.base10_parse()?;
            return Ok((size, splice_bytes(index, size)));
        }

        // Unit type
        Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty() => return Ok((0, quote! { () })),

        // Check types matching ident
        _ => {
            for (name, size) in &sizes {
                if is_ident(ty, name) {
                    let array = splice_bytes(index, *size);
                    let q = quote! { #ty::from_le_bytes(#array) };
                    return Ok((*size, q));
                }
            }
        }
    }

    bail!(
        ty,
        "unsupported type\n\n  = \
         help: annotate the size using #[field(size = ..)]\n"
    );
}

fn parse_attr(input: &Field) -> Result<Option<usize>> {
    for attr in &input.attrs {
        if attr.path.is_ident("field") {
            let mut args: Args = attr.parse_args()?;
            let size = args
                .remove_lit_int("size")?
                .map(|lit| lit.base10_parse())
                .transpose()?;
            args.ensure_empty()?;
            return Ok(size);
        }
    }
    Ok(None)
}

fn is_ident(ty: &Type, name: &str) -> bool {
    matches!(ty, Type::Path(ty) if ty.path.is_ident(name))
}

fn splice_bytes(index: usize, size: usize) -> TokenStream {
    let upper = index + size;
    quote! {{
        let ptr = bytes[#index..#upper].as_ptr() as *const [u8; #size];
        unsafe { *ptr }
    }}
}
