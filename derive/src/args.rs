use std::collections::HashMap;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Error, Lit, LitInt, MetaNameValue, Result, Token};

pub struct Args {
    pub align: LitInt,
    pub size: LitInt,
}

impl Args {
    pub fn new(input: &DeriveInput) -> Result<Self> {
        for attr in &input.attrs {
            if attr.path.is_ident("field") {
                return attr.parse_args();
            }
        }
        bail!(
            input,
            "`#[derive(Field)]` requires a `#[field(..)]` attribute"
        );
    }
}

impl Parse for Args {
    fn parse(tokens: ParseStream) -> Result<Self> {
        let parsed = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(tokens)?;

        let mut args: HashMap<_, _> = parsed
            .iter()
            .map(|kv| {
                let key = kv
                    .path
                    .get_ident()
                    .ok_or_else(|| error!(&kv.path, "expected identifier"))?
                    .to_string();
                Ok((key, kv))
            })
            .collect::<Result<_>>()?;

        let mut get = |key: &str| {
            args.remove(key)
                .ok_or_else(|| error!(&parsed, "missing `{}`", key))
                .and_then(|kv| match &kv.lit {
                    Lit::Int(v) => Ok(v.clone()),
                    lit => bail!(lit, "expected integer"),
                })
        };

        let align = get("align")?;
        let size = get("size")?;

        if args.is_empty() {
            Ok(Self { align, size })
        } else {
            let error = args
                .into_iter()
                .map(|(_, kv)| error!(kv, "unexpected attribute argument"))
                .reduce(|mut a, b| {
                    a.combine(b);
                    a
                })
                .unwrap();
            Err(error)
        }
    }
}
