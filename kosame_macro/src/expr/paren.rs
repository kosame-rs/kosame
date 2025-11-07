use crate::inferred_type::InferredType;
use crate::scopes::ScopeId;

use super::Expr;
use super::Visitor;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::Ident;
use syn::spanned::Spanned;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
};

pub struct Paren {
    pub paren: syn::token::Paren,
    pub expr: Box<Expr>,
}

impl Paren {
    pub fn accept<'a>(&'a self, _visitor: &mut impl Visitor<'a>) {}

    pub fn span(&self) -> Span {
        self.paren.span.span()
    }

    pub fn infer_name(&self) -> Option<&Ident> {
        self.expr.infer_name()
    }

    pub fn infer_type(&self, scope_id: ScopeId) -> Option<InferredType> {
        self.expr.infer_type(scope_id)
    }
}

impl Parse for Paren {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren: parenthesized!(content in input),
            expr: content.parse()?,
        })
    }
}

impl ToTokens for Paren {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expr = &self.expr;
        quote! {
            ::kosame::repr::expr::Paren::new(&#expr)
        }
        .to_tokens(tokens);
    }
}
