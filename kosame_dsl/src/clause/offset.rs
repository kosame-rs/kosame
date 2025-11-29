use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{expr::Expr, keyword, parse_option::ParseOption, visit::Visit};

pub struct Offset {
    pub offset: keyword::offset,
    pub expr: Expr,
}

impl ParseOption for Offset {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::offset)
    }
}

impl Offset {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visit<'a>) {
        self.expr.accept(visitor);
    }
}

impl Parse for Offset {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            offset: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Offset {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Offset::new(#expr) }.to_tokens(tokens);
    }
}
