use std::collections::HashMap;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Lit, LitInt, MetaNameValue, Result, Token};

pub struct Args {
    args: HashMap<String, MetaNameValue>,
}

impl Parse for Args {
    fn parse(tokens: ParseStream) -> Result<Self> {
        let parsed = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(tokens)?;

        let args: HashMap<_, _> = parsed
            .iter()
            .map(|kv| {
                let key = kv
                    .path
                    .get_ident()
                    .ok_or_else(|| error!(&kv.path, "expected identifier"))?
                    .to_string();
                Ok((key, kv.clone()))
            })
            .collect::<Result<_>>()?;

        Ok(Args { args })
    }
}

impl Args {
    pub fn remove_lit_int(&mut self, key: &str) -> Result<Option<LitInt>> {
        self.args
            .remove(key)
            .map(|kv| match &kv.lit {
                Lit::Int(v) => Ok(v.clone()),
                lit => bail!(lit, "expected integer"),
            })
            .transpose()
    }

    pub fn ensure_empty(self) -> Result<()> {
        if self.args.is_empty() {
            Ok(())
        } else {
            let error = self
                .args
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
