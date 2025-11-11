use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{expr::Expr, keyword, visitor::Visitor};

pub struct Limit {
    pub _limit: keyword::limit,
    pub expr: Expr,
}

impl Limit {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::limit)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.expr.accept(visitor);
    }
}

impl Parse for Limit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _limit: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Limit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Limit::new(#expr) }.to_tokens(tokens);
    }
}
