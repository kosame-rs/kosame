use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{clause::Fields, keyword, parse_option::ParseOption, visit::Visit};

pub struct Returning {
    pub returning: keyword::returning,
    pub fields: Fields,
}

impl ParseOption for Returning {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::returning)
    }
}

pub fn visit_returning<'a>(visit: &mut (impl Visit<'a> + ?Sized), returning: &'a Returning) {
    visit.visit_fields(&returning.fields);
}

impl Parse for Returning {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            returning: input.parse()?,
            fields: input.parse()?,
        })
    }
}

impl ToTokens for Returning {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = &self.fields;
        quote! {
            ::kosame::repr::clause::Returning::new(#fields)
        }
        .to_tokens(tokens);
    }
}
