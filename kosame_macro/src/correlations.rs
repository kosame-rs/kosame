use std::cell::Cell;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::{
    clause::{FromItem, WithItem},
    command::Command,
    inferred_type::InferredType,
    part::TablePath,
    path_ext::PathExt,
    query::{self, Query, QueryNodePath},
};

thread_local! {
    static CORRELATION_ID_AUTO_INCREMENT: Cell<u32> = const { Cell::new(0) };
    static CORRELATION_ID_CONTEXT: Cell<Option<CorrelationId>> = const { Cell::new(None) };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct CorrelationId(u32);

impl CorrelationId {
    pub fn new() -> Self {
        let id = CORRELATION_ID_AUTO_INCREMENT.get();
        CORRELATION_ID_AUTO_INCREMENT.set(id + 1);
        Self(id)
    }

    pub fn scope(&self, f: impl FnOnce()) {
        let previous = CORRELATION_ID_CONTEXT.with(|cell| cell.replace(Some(*self)));
        f();
        CORRELATION_ID_CONTEXT.with(|cell| cell.replace(previous));
    }

    pub fn of_scope() -> CorrelationId {
        CORRELATION_ID_CONTEXT
            .get()
            .expect("`ScopeId::of_scope` was called outside of a ScopeId scope")
    }

    pub fn reset() {
        CORRELATION_ID_AUTO_INCREMENT.set(0)
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl ToTokens for CorrelationId {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        format_ident!("correlation_{}", self.0).to_tokens(tokens);
    }
}

pub struct Correlations<'a> {
    correlations: Vec<Correlation<'a>>,
}

impl<'a> Correlations<'a> {
    pub fn infer_type(
        &'a self,
        correlation_id: CorrelationId,
        column: &'a Ident,
    ) -> Option<InferredType<'a>> {
        let correlation = self
            .correlations
            .iter()
            .find(|correlation| correlation.id() == correlation_id)
            .expect("scope ID must be valid");
        correlation.infer_type(column)
    }
}

impl ToTokens for Correlations<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let correlations = &self.correlations;
        quote! {
            mod correlations {
                #(#correlations)*
            }
        }
        .to_tokens(tokens);
    }
}

enum Correlation<'a> {
    Table(&'a TablePath, Option<&'a WithItem>),
    Command(&'a Command),
    WithItem(&'a WithItem),
    FromItem(&'a FromItem),
    QueryNodePath {
        node: &'a query::Node,
        table_path: &'a TablePath,
        node_path: QueryNodePath,
    },
}

impl<'a> Correlation<'a> {
    fn id(&self) -> CorrelationId {
        match self {
            Self::Table(inner, _) => inner.correlation_id,
            Self::Command(inner) => inner.correlation_id,
            Self::WithItem(inner) => inner.correlation_id,
            Self::FromItem(inner) => inner.correlation_id(),
            Self::QueryNodePath { node, .. } => node.correlation_id,
        }
    }

    fn _source_id(&self) -> Option<CorrelationId> {
        match self {
            Self::Table(_, with_item) => {
                with_item.as_ref().map(|with_item| with_item.correlation_id)
            }
            Self::Command(_) => None,
            Self::WithItem(inner) => Some(inner.command.correlation_id),
            Self::FromItem(inner) => match inner {
                FromItem::Table { table_path, .. } => Some(table_path.correlation_id),
                FromItem::Subquery { command, .. } => Some(command.correlation_id),
            },
            Self::QueryNodePath { .. } => None,
        }
    }

    pub fn infer_type(&'a self, column: &'a Ident) -> Option<InferredType<'a>> {
        match self {
            Self::Table(table_path, with_item) => match with_item {
                Some(with_item) => Some(InferredType::Correlation {
                    correlation_id: with_item.correlation_id,
                    column,
                    nullable: false,
                }),
                None => Some(InferredType::TableColumn { table_path, column }),
            },
            Self::Command(command) => {
                let field = command
                    .fields()?
                    .iter()
                    .find(|field| field.infer_name() == Some(column))?;
                field.infer_type(command.scope_id)
            }
            Self::WithItem(with_item) => Some(InferredType::Correlation {
                correlation_id: with_item.command.correlation_id,
                column,
                nullable: false,
            }),
            Self::FromItem(from_item) => match from_item {
                FromItem::Table { table_path, .. } => Some(InferredType::Correlation {
                    correlation_id: table_path.correlation_id,
                    column,
                    nullable: false,
                }),
                FromItem::Subquery { command, .. } => Some(InferredType::Correlation {
                    correlation_id: command.correlation_id,
                    column,
                    nullable: false,
                }),
            },
            Self::QueryNodePath { table_path, .. } => {
                Some(InferredType::TableColumn { table_path, column })
            }
        }
    }
}

impl ToTokens for Correlation<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id();

        match self {
            Self::Table(_, Some(with_item)) => {
                let source_id = with_item.correlation_id;
                quote! {
                    pub use #source_id as #id;
                }
            }
            Self::Table(table_path, None) => {
                let table_path = table_path.as_path().to_call_site(2);
                quote! {
                    pub use #table_path as #id;
                }
            }
            Self::Command(command) => match command.fields() {
                Some(fields) => {
                    let fields = fields.columns();
                    let field_strings = fields.iter().map(|field| field.to_string());
                    quote! {
                        pub mod #id {
                            pub mod columns {
                                #(
                                    pub mod #fields {
                                        pub const COLUMN_NAME: &str = #field_strings;
                                    }
                                )*
                            }
                        }
                    }
                }
                None => quote! { pub mod #id {} },
            },
            Self::WithItem(with_item) => match &with_item.alias.columns {
                Some(_) => {
                    unimplemented!();
                }
                None => {
                    let source_id = with_item.command.correlation_id;
                    let alias = with_item.alias.name.to_string();
                    quote! {
                        pub mod #id {
                            pub const TABLE_NAME: &str = #alias;
                            pub use super::#source_id::columns;
                        }
                    }
                }
            },
            Self::FromItem(from_item) => match from_item {
                FromItem::Table {
                    table_path, alias, ..
                } => {
                    let source_id = table_path.correlation_id;
                    match alias {
                        Some(alias) => {
                            let alias = alias.name.to_string();
                            quote! {
                                pub mod #id {
                                    pub const TABLE_NAME: &str = #alias;
                                    pub use super::#source_id::columns;
                                }
                            }
                        }
                        None => quote! { pub use #source_id as #id; },
                    }
                }
                FromItem::Subquery { command, alias, .. } => {
                    let source_id = command.correlation_id;
                    match alias {
                        Some(alias) => {
                            let alias = alias.name.to_string();
                            quote! {
                                pub mod #id {
                                    pub const TABLE_NAME: &str = #alias;
                                    pub use super::#source_id::columns;
                                }
                            }
                        }
                        None => quote! { pub use #source_id as #id; },
                    }
                }
            },
            Self::QueryNodePath {
                table_path,
                node_path,
                ..
            } => {
                let table_path = node_path.resolve(&table_path.as_path().to_call_site(2));
                quote! {
                    pub use #table_path as #id;
                }
            }
        }
        .to_tokens(tokens);
    }
}

impl<'a> From<&'a Command> for Correlations<'a> {
    fn from(value: &'a Command) -> Self {
        fn inner<'a>(
            correlations: &mut Vec<Correlation<'a>>,
            command: &'a Command,
            inherited_with_items: &mut Vec<&'a WithItem>,
        ) {
            let with_items_truncate = inherited_with_items.len();

            correlations.push(Correlation::Command(command));

            if let Some(with) = &command.with {
                for with_item in &with.items {
                    correlations.push(Correlation::WithItem(with_item));
                    inner(correlations, &with_item.command, inherited_with_items);
                    inherited_with_items.push(with_item);
                }
            }

            if let Some(target_table) = command.target_table() {
                correlations.push(Correlation::Table(&target_table.table, None));
            }

            if let Some(from_chain) = command.from_chain() {
                for from_item in from_chain {
                    correlations.push(Correlation::FromItem(from_item));

                    match from_item {
                        FromItem::Table { table_path, .. } => {
                            let with_item = match from_item {
                                FromItem::Table { table_path, .. } => {
                                    match table_path.get_ident() {
                                        Some(table) => inherited_with_items
                                            .iter()
                                            .rev()
                                            .find(|with_item| with_item.alias.name == *table),
                                        None => None,
                                    }
                                }
                                _ => None,
                            };
                            correlations.push(Correlation::Table(table_path, with_item.copied()));
                        }
                        FromItem::Subquery { command, .. } => {
                            inner(correlations, command, inherited_with_items);
                        }
                    }
                }
            }

            inherited_with_items.truncate(with_items_truncate);
        }

        let mut correlations = Vec::new();
        inner(&mut correlations, value, &mut Vec::new());
        Correlations { correlations }
    }
}

impl<'a> From<&'a Query> for Correlations<'a> {
    fn from(value: &'a Query) -> Self {
        fn inner<'a>(
            correlations: &mut Vec<Correlation<'a>>,
            query: &'a Query,
            node: &'a query::Node,
            node_path: QueryNodePath,
        ) {
            for field in node.fields.iter() {
                if let query::Field::Relation { node, name, .. } = field {
                    inner(
                        correlations,
                        query,
                        node,
                        node_path.clone().appended(name.clone()),
                    );
                }
            }

            correlations.push(Correlation::QueryNodePath {
                node,
                table_path: &query.table,
                node_path,
            });
        }

        let mut correlations = Vec::new();
        inner(&mut correlations, value, &value.body, QueryNodePath::new());
        Correlations { correlations }
    }
}
