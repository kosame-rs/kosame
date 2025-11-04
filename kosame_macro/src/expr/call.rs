use crate::{data_type::InferredType, scopes::ScopeId};

use super::{Expr, Visitor};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

pub struct Call {
    pub function: Ident,
    pub paren: syn::token::Paren,
    pub params: Punctuated<Expr, Token![,]>,
}

impl Call {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Ident) && input.peek2(syn::token::Paren)
    }

    pub fn infer_name(&self) -> Option<&Ident> {
        Some(&self.function)
    }

    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType> {
        None
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        for param in self.params.iter() {
            param.accept(visitor);
        }
    }

    pub fn span(&self) -> Span {
        self.function
            .span()
            .join(self.paren.span.span())
            .expect("same file")
    }
}

impl Parse for Call {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            function: input.parse()?,
            paren: parenthesized!(content in input),
            params: content.parse_terminated(Expr::parse, Token![,])?,
        })
    }
}

impl ToTokens for Call {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let function_name = &self.function.to_string();
        let params = self.params.iter();
        quote! {
            ::kosame::repr::expr::Call::new(
                #function_name,
                &[#(&#params),*]
            )
        }
        .to_tokens(tokens)
    }
}
