use crate::{
    inferred_type::InferredType,
    pretty::{PrettyPrint, Printer},
    scopes::ScopeId,
};

use super::Visit;
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
    pub fn accept<'a>(&'a self, _visitor: &mut impl Visit<'a>) {}

    #[must_use]
    pub fn infer_name(&self) -> Option<&Ident> {
        Some(&self.name)
    }

    #[must_use]
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

    #[must_use]
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
                period_token: input.parse()?,
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

impl PrettyPrint for ColumnRef {
    fn pretty_print(&self, printer: &mut Printer) {
        self.correlation.pretty_print(printer);
        self.name.pretty_print(printer);
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
                .to_tokens(tokens);
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
    pub period_token: Token![.],
}

impl PrettyPrint for Correlation {
    fn pretty_print(&self, printer: &mut Printer) {
        self.name.pretty_print(printer);
        self.period_token.pretty_print(printer);
    }
}
