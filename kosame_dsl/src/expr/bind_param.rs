use crate::{
    inferred_type::InferredType,
    pretty::{PrettyPrint, Printer},
    scopes::ScopeId,
};

use super::Visitor;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

pub struct BindParam {
    pub colon_token: Token![:],
    pub name: Ident,
}

impl BindParam {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_bind_param(self);
    }

    #[must_use]
    pub fn infer_name(&self) -> Option<&Ident> {
        Some(&self.name)
    }

    #[must_use]
    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType<'_>> {
        None
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![:])
    }

    #[must_use]
    pub fn span(&self) -> Span {
        self.colon_token
            .span
            .join(self.name.span())
            .unwrap_or(self.name.span())
    }
}

impl Parse for BindParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            colon_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl ToTokens for BindParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        quote! { params::#name::BIND_PARAM }.to_tokens(tokens);
    }
}

impl PrettyPrint for BindParam {
    fn pretty_print(&self, printer: &mut Printer) {
        self.colon_token.pretty_print(printer);
        self.name.pretty_print(printer);
    }
}
