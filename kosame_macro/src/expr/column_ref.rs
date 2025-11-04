use crate::{
    data_type::InferredType,
    parent_map::{Id, ParentMap},
    scope::{ScopeIter, ScopeIterItem},
    scopes::ScopeId,
};

use super::Visitor;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
    parse_quote,
};

pub struct ColumnRef {
    pub id: Id,
    pub correlation: Option<Correlation>,
    pub name: Ident,
}

impl ColumnRef {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_parent_node(self.into());
        visitor.end_parent_node();
    }

    pub fn infer_name(&self) -> Option<&Ident> {
        Some(&self.name)
    }

    pub fn infer_type(&self, scope_id: ScopeId) -> Option<InferredType> {
        match &self.correlation {
            Some(correlation) => ParentMap::with(|parent_map| {
                for item in ScopeIter::new(parent_map, self, true) {
                    match item {
                        ScopeIterItem::TargetTable(target_table) => {
                            if *target_table.name() == correlation.name {
                                let table = &target_table.table;
                                let column = &self.name;
                                return Some(InferredType::Column(parse_quote! {
                                    #table::columns::#column
                                }));
                            }
                        }
                        ScopeIterItem::FromItem(from_Item) => {}
                    }
                }
                None
            }),
            None => None,
        }
    }

    pub fn span(&self) -> Span {
        if let Some(correlation) = &self.correlation {
            correlation
                .name
                .span()
                .join(self.name.span())
                .expect("same file")
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
                id: Id::new(),
                correlation: Some(correlation),
                name: input.parse()?,
            })
        } else {
            Ok(Self {
                id: Id::new(),
                correlation: None,
                name: ident1,
            })
        }
    }
}

impl ToTokens for ColumnRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        match &self.correlation {
            Some(correlation) => {
                let correlation = &correlation.name;
                quote! {
                    ::kosame::repr::expr::ColumnRef::new(
                        Some(scope::tables::#correlation::TABLE_NAME),
                        scope::tables::#correlation::columns::#name::COLUMN_NAME
                    )
                }
                .to_tokens(tokens)
            }
            None => quote! {
                ::kosame::repr::expr::ColumnRef::new(
                    ::core::option::Option::None,
                    scope::columns::#name::COLUMN_NAME
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
