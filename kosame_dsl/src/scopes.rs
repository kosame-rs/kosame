use std::{cell::Cell, collections::HashSet};

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::{
    clause::FromItem,
    command::Command,
    correlations::CorrelationId,
    inferred_type::InferredType,
    part::TargetTable,
    query::{self, Query},
};

thread_local! {
    static SCOPE_ID_AUTO_INCREMENT: Cell<u32> = const { Cell::new(0) };
    static SCOPE_ID_CONTEXT: Cell<Option<ScopeId>> = const { Cell::new(None) };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ScopeId(u32);

impl ScopeId {
    #[must_use]
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

    #[must_use]
    pub fn of_scope() -> ScopeId {
        SCOPE_ID_CONTEXT
            .get()
            .expect("`ScopeId::of_scope` was called outside of a ScopeId scope")
    }

    pub fn reset() {
        SCOPE_ID_AUTO_INCREMENT.set(0);
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

impl Scopes<'_> {
    #[must_use]
    pub fn infer_type<'a>(
        &self,
        scope_id: ScopeId,
        table: Option<&Ident>,
        column: &'a Ident,
    ) -> Option<InferredType<'a>> {
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

        let correlations = self.items.iter().filter_map(|item| {
            item.name().map(|name| {
                let correlation_id = item.correlation_id();
                quote! {
                    pub use super::super::super::correlations::#correlation_id as #name;
                }
            })
        });

        let columns = self
            .items
            .iter()
            .filter(|item| !item.is_inherited())
            .filter_map(|item| item.name());

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
    TargetTable {
        target_table: &'a TargetTable,
    },
    FromItem {
        from_item: &'a FromItem,
        inherited_from: Option<ScopeId>,
        nullable: bool,
    },
    QueryNode {
        node: &'a query::Node,
        name: &'a Ident,
    },
}

impl ScopeItem<'_> {
    #[must_use] 
    pub fn correlation_id(&self) -> CorrelationId {
        match self {
            Self::TargetTable { target_table, .. } => target_table.table.correlation_id,
            Self::FromItem { from_item, .. } => from_item.correlation_id(),
            Self::QueryNode { node, .. } => node.correlation_id,
        }
    }

    #[must_use] 
    pub fn name(&self) -> Option<&Ident> {
        match self {
            Self::TargetTable { target_table, .. } => Some(target_table.name()),
            Self::FromItem { from_item, .. } => from_item.name(),
            Self::QueryNode { name, .. } => Some(name),
        }
    }

    #[must_use] 
    pub fn nullable(&self) -> bool {
        match self {
            Self::TargetTable { .. } => false,
            Self::FromItem { nullable, .. } => *nullable,
            Self::QueryNode { .. } => false,
        }
    }

    #[must_use] 
    pub fn is_inherited(&self) -> bool {
        match self {
            Self::TargetTable { .. } => false,
            Self::FromItem { inherited_from, .. } => inherited_from.is_some(),
            Self::QueryNode { .. } => false,
        }
    }
}

impl<'a> From<&'a Command> for Scopes<'a> {
    fn from(value: &'a Command) -> Self {
        fn inner<'a>(
            scopes: &mut Vec<Scope<'a>>,
            command: &'a Command,
            inherited_from_items: &mut Vec<(ScopeId, &'a FromItem)>,
        ) {
            let scope_id = command.scope_id;
            let from_items_truncate = inherited_from_items.len();

            let mut items = Vec::new();
            let mut shadow = HashSet::new();

            if let Some(with) = &command.with {
                for with_item in &with.items {
                    inner(scopes, &with_item.command, inherited_from_items);
                }
            }

            if let Some(target_table) = command.target_table() {
                shadow.insert(target_table.name());
                items.push(ScopeItem::TargetTable { target_table });
            }

            if let Some(from_chain) = command.from_chain() {
                let nullables = from_chain.nullables();

                for (from_item, nullable) in from_chain.into_iter().zip(nullables.into_iter()) {
                    inherited_from_items.push((scope_id, from_item));

                    if let Some(name) = from_item.name() {
                        shadow.insert(name);
                    }

                    if let FromItem::Subquery { command, .. } = from_item {
                        inner(scopes, command, inherited_from_items);
                    }

                    items.push(ScopeItem::FromItem {
                        from_item,
                        inherited_from: None,
                        nullable,
                    });
                }
            }

            inherited_from_items.truncate(from_items_truncate);

            for (inherited_from, from_item) in inherited_from_items.iter() {
                if let Some(name) = from_item.name()
                    && !shadow.contains(name)
                {
                    items.push(ScopeItem::FromItem {
                        from_item,
                        inherited_from: Some(*inherited_from),
                        nullable: false,
                    });
                }
            }

            scopes.push(Scope::new(scope_id, items));
        }

        let mut scopes = Vec::new();
        inner(&mut scopes, value, &mut Vec::new());
        scopes.sort_by_key(|v| v.id);
        Scopes { scopes }
    }
}

impl<'a> From<&'a Query> for Scopes<'a> {
    fn from(value: &'a Query) -> Self {
        fn inner<'a>(scopes: &mut Vec<Scope<'a>>, node: &'a query::Node, name: &'a Ident) {
            let scope_id = node.scope_id;
            let items = vec![ScopeItem::QueryNode { node, name }];

            for field in &node.fields {
                if let query::Field::Relation { node, name, .. } = field {
                    inner(scopes, node, name);
                }
            }

            scopes.push(Scope::new(scope_id, items));
        }

        let mut scopes = Vec::new();
        inner(
            &mut scopes,
            &value.body,
            &value
                .table
                .as_path()
                .segments
                .last()
                .expect("path cannot be empty")
                .ident,
        );
        scopes.sort_by_key(|v| v.id);
        Scopes { scopes }
    }
}
