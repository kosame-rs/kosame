use std::{cell::Cell, collections::HashSet};

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::{
    clause::{FromItem, WithItem},
    command::Command,
    correlations::CorrelationId,
    inferred_type::InferredType,
};

thread_local! {
    static SCOPE_ID_AUTO_INCREMENT: Cell<u32> = const { Cell::new(0) };
    static SCOPE_ID_CONTEXT: Cell<Option<ScopeId>> = const { Cell::new(None) };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ScopeId(u32);

impl ScopeId {
    pub fn new() -> Self {
        let id = SCOPE_ID_AUTO_INCREMENT.get();
        SCOPE_ID_AUTO_INCREMENT.set(id + 1);
        Self(id)
    }

    pub fn scope(&self, f: impl FnOnce()) {
        let previous = SCOPE_ID_CONTEXT.with(|cell| cell.replace(Some(*self)));
        f();
        SCOPE_ID_CONTEXT.with(|cell| cell.replace(previous));
    }

    pub fn of_scope() -> ScopeId {
        SCOPE_ID_CONTEXT
            .get()
            .expect("`ScopeId::of_scope` was called outside of a ScopeId scope")
    }

    pub fn reset() {
        SCOPE_ID_AUTO_INCREMENT.set(0)
    }
}

impl Default for ScopeId {
    fn default() -> Self {
        Self::new()
    }
}

impl ToTokens for ScopeId {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        format_ident!("scope_{}", self.0).to_tokens(tokens);
    }
}

pub struct Scopes<'a> {
    scopes: Vec<Scope<'a>>,
}

impl<'a> Scopes<'a> {
    pub fn infer_type<'b>(
        &self,
        scope_id: ScopeId,
        table: Option<&Ident>,
        column: &'b Ident,
    ) -> Option<InferredType<'b>> {
        let scope = self
            .scopes
            .iter()
            .find(|scope| scope.id == scope_id)
            .expect("scope ID must be valid");
        scope.infer_type(table, column)
    }
}

impl ToTokens for Scopes<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let scopes = &self.scopes;
        quote! {
            mod scopes {
                #(#scopes)*
            }
        }
        .to_tokens(tokens);
    }
}

struct Scope<'a> {
    id: ScopeId,
    items: Vec<ScopeItem<'a>>,
}

impl<'a> Scope<'a> {
    fn new(id: ScopeId, items: Vec<ScopeItem<'a>>) -> Self {
        Self { id, items }
    }

    pub fn infer_type<'b>(
        &self,
        table: Option<&Ident>,
        column: &'b Ident,
    ) -> Option<InferredType<'b>> {
        let table = table?;
        let item = self.items.iter().find(|item| item.name() == Some(table))?;
        Some(InferredType::Correlation {
            correlation_id: item.correlation_id(),
            column,
            nullable: item.nullable(),
        })
    }
}

impl ToTokens for Scope<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.id;

        let correlations = self.items.iter().filter_map(|item| match item {
            ScopeItem::FromItem { from_item, .. } => {
                let correlation_id = from_item.correlation_id();
                from_item.name().map(|name| {
                    quote! {
                        pub use super::super::super::correlations::#correlation_id as #name;
                    }
                })
            }
        });

        let columns = self.items.iter().filter_map(|item| match item {
            ScopeItem::FromItem {
                from_item,
                inherited_from: None,
                ..
            } => from_item.name(),
            _ => None,
        });

        quote! {
            pub mod #name {
                pub mod tables {
                    #(#correlations)*
                }
                pub mod columns {
                    #(pub use super::tables::#columns::columns::*;)*
                }
            }
        }
        .to_tokens(tokens);
    }
}

pub enum ScopeItem<'a> {
    FromItem {
        from_item: &'a FromItem,
        with_item: Option<&'a WithItem>,
        inherited_from: Option<ScopeId>,
        nullable: bool,
    },
}

impl<'a> ScopeItem<'a> {
    pub fn correlation_id(&self) -> CorrelationId {
        match self {
            Self::FromItem { from_item, .. } => from_item.correlation_id(),
        }
    }

    pub fn name(&self) -> Option<&Ident> {
        match self {
            Self::FromItem { from_item, .. } => from_item.name(),
        }
    }

    pub fn nullable(&self) -> bool {
        match self {
            Self::FromItem { nullable, .. } => *nullable,
        }
    }
}

impl<'a> From<&'a Command> for Scopes<'a> {
    fn from(value: &'a Command) -> Self {
        fn inner<'a>(
            scopes: &mut Vec<Scope<'a>>,
            command: &'a Command,
            inherited_with_items: &mut Vec<&'a WithItem>,
            inherited_from_items: &mut Vec<(ScopeId, &'a FromItem)>,
        ) {
            let scope_id = command.scope_id;
            let with_items_truncate = inherited_with_items.len();
            let from_items_truncate = inherited_from_items.len();

            let mut items = Vec::new();
            let mut shadow = HashSet::new();

            if let Some(target_table) = command.target_table() {
                shadow.insert(target_table.name());
            }

            if let Some(with) = &command.with {
                for with_item in &with.items {
                    inner(
                        scopes,
                        &with_item.command,
                        inherited_with_items,
                        inherited_from_items,
                    );
                    inherited_with_items.push(with_item);
                }
            }

            if let Some(from_chain) = command.from_chain() {
                for from_item in from_chain {
                    inherited_from_items.push((scope_id, from_item));

                    if let Some(name) = from_item.name() {
                        shadow.insert(name);
                    }

                    if let FromItem::Subquery { command, .. } = from_item {
                        inner(scopes, command, inherited_with_items, inherited_from_items);
                    }

                    let with_item = match from_item {
                        FromItem::Table { table_path, .. } => match table_path.get_ident() {
                            Some(table) => inherited_with_items
                                .iter()
                                .rev()
                                .find(|with_item| with_item.alias.name == *table),
                            None => None,
                        },
                        _ => None,
                    };

                    items.push(ScopeItem::FromItem {
                        from_item,
                        inherited_from: None,
                        with_item: with_item.copied(),
                        nullable: false,
                    });
                }
            }

            inherited_with_items.truncate(with_items_truncate);
            inherited_from_items.truncate(from_items_truncate);

            for (inherited_from, from_item) in inherited_from_items.iter() {
                if let Some(name) = from_item.name()
                    && !shadow.contains(name)
                {
                    items.push(ScopeItem::FromItem {
                        from_item,
                        inherited_from: Some(*inherited_from),
                        with_item: None,
                        nullable: false,
                    });
                }
            }

            scopes.push(Scope::new(scope_id, items));
        }

        let mut scopes = Vec::new();
        inner(&mut scopes, value, &mut Vec::new(), &mut Vec::new());
        scopes.sort_by_key(|v| v.id);
        Scopes { scopes }
    }
}
