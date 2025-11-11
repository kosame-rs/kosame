use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{clause::Fields, keyword, visitor::Visitor};

pub struct Select {
    pub _select: keyword::select,
    pub fields: Fields,
}

impl Select {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::select)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.fields.accept(visitor);
    }
}

impl Parse for Select {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _select: input.call(keyword::select::parse_autocomplete)?,
            fields: input.parse()?,
        })
    }
}

impl ToTokens for Select {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = &self.fields;
        quote! {
            ::kosame::repr::clause::Select::new(#fields)
        }
        .to_tokens(tokens)
    }
}
