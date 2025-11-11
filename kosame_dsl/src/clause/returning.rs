use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{clause::Fields, keyword, visitor::Visitor};

pub struct Returning {
    pub _returning: keyword::returning,
    pub fields: Fields,
}

impl Returning {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::returning)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.fields.accept(visitor);
    }
}

impl Parse for Returning {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _returning: input.parse()?,
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
        .to_tokens(tokens)
    }
}
