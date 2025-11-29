use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{expr::Expr, keyword, parse_option::ParseOption, visit::Visit};

pub struct Limit {
    pub limit: keyword::limit,
    pub expr: Expr,
}

impl ParseOption for Limit {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::limit)
    }
}

impl Limit {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visit<'a>) {
        self.expr.accept(visitor);
    }
}

impl Parse for Limit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            limit: input.parse()?,
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
