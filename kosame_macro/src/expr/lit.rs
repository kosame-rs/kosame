use crate::{data_type::InferredType, keyword, scopes::ScopeId};

use super::Visitor;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

#[allow(unused)]
pub enum Lit {
    Int(syn::LitInt),
    Float(syn::LitFloat),
    Str(syn::LitStr),
    Bool(syn::LitBool),
    Null(keyword::null),
}

impl Lit {
    pub fn accept<'a>(&'a self, _visitor: &mut impl Visitor<'a>) {}

    pub fn infer_name(&self) -> Option<&Ident> {
        None
    }

    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType> {
        None
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Int(inner) => inner.span(),
            Self::Float(inner) => inner.span(),
            Self::Str(inner) => inner.span(),
            Self::Bool(inner) => inner.span(),
            Self::Null(inner) => inner.span(),
        }
    }
}

impl Parse for Lit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::null) {
            return Ok(Self::Null(input.parse()?));
        }
        let lit = input.parse::<syn::Lit>()?;
        Ok(match lit {
            syn::Lit::Int(inner) => Self::Int(inner),
            syn::Lit::Float(inner) => Self::Float(inner),
            syn::Lit::Str(inner) => Self::Str(inner),
            syn::Lit::Bool(inner) => Self::Bool(inner),
            _ => {
                return Err(syn::Error::new(
                    lit.span(),
                    format!("unsupported literal type `{}`", lit.to_token_stream()),
                ));
            }
        })
    }
}

impl ToTokens for Lit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token_stream = match self {
            Self::Int(inner) => quote! { ::kosame::repr::expr::Lit::Int(#inner as i64) },
            Self::Float(inner) => quote! { ::kosame::repr::expr::Lit::Float(#inner as f64) },
            Self::Str(inner) => quote! { ::kosame::repr::expr::Lit::Str(#inner) },
            Self::Bool(inner) => quote! { ::kosame::repr::expr::Lit::Bool(#inner) },
            Self::Null(_) => quote! { ::kosame::repr::expr::Lit::Null },
        };
        token_stream.to_tokens(tokens);
    }
}
