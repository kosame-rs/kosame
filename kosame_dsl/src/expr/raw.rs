use crate::{
    inferred_type::InferredType,
    pretty::{PrettyPrint, Printer},
    scopes::ScopeId,
};

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

    #[must_use] 
    pub fn infer_name(&self) -> Option<&Ident> {
        None
    }

    #[must_use] 
    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType<'_>> {
        None
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![$])
    }

    #[must_use] 
    pub fn span(&self) -> Span {
        self.dollar_token
            .span
            .join(self.string.span())
            .unwrap_or(self.string.span())
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
        quote! { ::kosame::repr::expr::Raw::new(#string) }.to_tokens(tokens);
    }
}

impl PrettyPrint for Raw {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.dollar_token.pretty_print(printer);
        self.string.token().pretty_print(printer);
    }
}
