use crate::{inferred_type::InferredType, scopes::ScopeId};

use super::Visitor;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

pub struct ColumnRef {
    pub correlation: Option<Correlation>,
    pub name: Ident,
}

impl ColumnRef {
    pub fn accept<'a>(&'a self, _visitor: &mut impl Visitor<'a>) {}

    pub fn infer_name(&self) -> Option<&Ident> {
        Some(&self.name)
    }

    pub fn infer_type(&self, scope_id: ScopeId) -> Option<InferredType<'_>> {
        Some(InferredType::Scope {
            scope_id,
            table: self
                .correlation
                .as_ref()
                .map(|correlation| &correlation.name),
            column: &self.name,
        })
    }

    pub fn span(&self) -> Span {
        if let Some(correlation) = &self.correlation {
            correlation
                .name
                .span()
                .join(self.name.span())
                .unwrap_or(self.name.span())
        } else {
            self.name.span()
        }
    }
}

impl Parse for ColumnRef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident1 = input.parse::<Ident>()?;
        if input.peek(Token![.]) {
            let correlation = Correlation {
                name: ident1,
                _period_token: input.parse()?,
            };
            Ok(Self {
                correlation: Some(correlation),
                name: input.parse()?,
            })
        } else {
            Ok(Self {
                correlation: None,
                name: ident1,
            })
        }
    }
}

impl ToTokens for ColumnRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let scope_id = ScopeId::of_scope();
        match &self.correlation {
            Some(correlation) => {
                let correlation = &correlation.name;
                quote! {
                    ::kosame::repr::expr::ColumnRef::new(
                        Some(scopes::#scope_id::tables::#correlation::TABLE_NAME),
                        scopes::#scope_id::tables::#correlation::columns::#name::COLUMN_NAME
                    )
                }
                .to_tokens(tokens)
            }
            None => quote! {
                ::kosame::repr::expr::ColumnRef::new(
                    ::core::option::Option::None,
                    scopes::#scope_id::columns::#name::COLUMN_NAME
                )
            }
            .to_tokens(tokens),
        }
    }
}

pub struct Correlation {
    pub name: Ident,
    pub _period_token: Token![.],
}
