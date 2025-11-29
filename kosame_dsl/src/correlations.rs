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
    scopes::Scoped,
    visit::Visit,
};

thread_local! {
    static CORRELATION_ID_AUTO_INCREMENT: Cell<u32> = const { Cell::new(0) };
    static CORRELATION_ID_CONTEXT: Cell<Option<CorrelationId>> = const { Cell::new(None) };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct CorrelationId(u32);

impl CorrelationId {
    #[must_use]
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

    #[must_use]
    pub fn of_scope() -> CorrelationId {
        CORRELATION_ID_CONTEXT
            .get()
            .expect("`ScopeId::of_scope` was called outside of a ScopeId scope")
    }

    pub fn reset() {
        CORRELATION_ID_AUTO_INCREMENT.set(0);
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
    #[must_use]
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
            Self::Command(command) => {
                if let Some(fields) = command.fields() {
                    let fields = fields.columns();
                    let field_strings = fields.iter().map(std::string::ToString::to_string);
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
                } else {
                    quote! { pub mod #id {} }
                }
            }
            Self::WithItem(with_item) => {
                if with_item.alias.columns.is_some() {
                    unimplemented!();
                } else {
                    let source_id = with_item.command.correlation_id;
                    let alias = with_item.alias.name.to_string();
                    quote! {
                        pub mod #id {
                            pub const TABLE_NAME: &str = #alias;
                            pub use super::#source_id::columns;
                        }
                    }
                }
            }
            Self::FromItem(from_item) => match from_item {
                FromItem::Table {
                    table_path, alias, ..
                } => {
                    let source_id = table_path.correlation_id;
                    if let Some(alias) = alias {
                        let alias = alias.name.to_string();
                        quote! {
                            pub mod #id {
                                pub const TABLE_NAME: &str = #alias;
                                pub use super::#source_id::columns;
                            }
                        }
                    } else {
                        quote! { pub use #source_id as #id; }
                    }
                }
                FromItem::Subquery { command, alias, .. } => {
                    let source_id = command.correlation_id;
                    if let Some(alias) = alias {
                        let alias = alias.name.to_string();
                        quote! {
                            pub mod #id {
                                pub const TABLE_NAME: &str = #alias;
                                pub use super::#source_id::columns;
                            }
                        }
                    } else {
                        quote! { pub use #source_id as #id; }
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
        #[derive(Default)]
        struct Visitor<'a> {
            correlations: Vec<Correlation<'a>>,
            inherited_with_items: Vec<&'a WithItem>,
        }

        impl<'a> Visit<'a> for Visitor<'a> {
            fn visit_command(&mut self, command: &'a Command) {
                self.correlations.push(Correlation::Command(command));
                let with_items_truncate = self.inherited_with_items.len();

                if let Some(with) = &command.with {
                    for with_item in &with.items {
                        self.correlations.push(Correlation::WithItem(with_item));
                        self.visit_command(&with_item.command);
                        self.inherited_with_items.push(with_item);
                    }
                }

                if let Some(target_table) = command.target_table() {
                    self.correlations
                        .push(Correlation::Table(&target_table.table, None));
                }

                if let Some(from_chain) = command.from_chain() {
                    for from_item in from_chain {
                        self.correlations.push(Correlation::FromItem(from_item));

                        match from_item {
                            FromItem::Table { table_path, .. } => {
                                let with_item = match from_item {
                                    FromItem::Table { table_path, .. } => {
                                        match table_path.get_ident() {
                                            Some(table) => {
                                                self.inherited_with_items.iter().rev().find(
                                                    |with_item| with_item.alias.name == *table,
                                                )
                                            }
                                            None => None,
                                        }
                                    }
                                    FromItem::Subquery { .. } => None,
                                };
                                self.correlations
                                    .push(Correlation::Table(table_path, with_item.copied()));
                            }
                            FromItem::Subquery { command, .. } => {
                                self.visit_command(command);
                            }
                        }
                    }
                }

                self.inherited_with_items.truncate(with_items_truncate);
            }
        }

        let mut visitor = Visitor::default();
        visitor.visit_command(value);
        Correlations {
            correlations: visitor.correlations,
        }
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
            for field in &node.fields {
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
