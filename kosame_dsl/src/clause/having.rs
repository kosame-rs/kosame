use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{expr::Expr, keyword, visitor::Visitor};

pub struct Having {
    pub _having: keyword::having,
    pub expr: Expr,
}

impl Having {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::having)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.expr.accept(visitor);
    }
}

impl Parse for Having {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _having: input.parse()?,
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
