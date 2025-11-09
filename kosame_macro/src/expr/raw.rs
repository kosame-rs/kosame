use crate::{inferred_type::InferredType, scopes::ScopeId};

use super::Visitor;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
};

pub struct Raw {
    pub dollar_token: Token![$],
    pub string: LitStr,
}

impl Raw {
    pub fn accept<'a>(&'a self, _visitor: &mut impl Visitor<'a>) {}

    pub fn infer_name(&self) -> Option<&Ident> {
        None
    }

    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType<'_>> {
        None
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![$])
    }

    pub fn span(&self) -> Span {
        self.dollar_token
            .span
            .join(self.string.span())
            .expect("same file")
    }
}

impl Parse for Raw {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            dollar_token: input.parse()?,
            string: input.parse()?,
        })
    }
}

impl ToTokens for Raw {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let string = &self.string;
        quote! { ::kosame::repr::expr::Raw::new(#string) }.to_tokens(tokens)
    }
}
