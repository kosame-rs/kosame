use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use crate::{expr::Expr, visitor::Visitor};

pub struct Where {
    pub _where: Token![where],
    pub expr: Expr,
}

impl Where {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![where])
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.expr.accept(visitor);
    }
}

impl Parse for Where {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _where: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Where {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Where::new(#expr) }.to_tokens(tokens);
    }
}
