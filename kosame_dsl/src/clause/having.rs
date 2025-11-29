use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{expr::Expr, keyword, parse_option::ParseOption, visit::Visit};

pub struct Having {
    pub having: keyword::having,
    pub expr: Expr,
}

impl ParseOption for Having {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::having)
    }
}

impl Having {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visit<'a>) {
        self.expr.accept(visitor);
    }
}

impl Parse for Having {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            having: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Having {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Having::new(#expr) }.to_tokens(tokens);
    }
}
