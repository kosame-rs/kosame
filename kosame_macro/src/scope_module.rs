use std::collections::HashMap;

use proc_macro_error::OptionExt;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Path};

use crate::{
    clause::FromItem,
    command::{Command, CommandType},
    part::TableAlias,
    path_ext::PathExt,
    statement::CommandTree,
};

#[derive(Default, Clone)]
pub struct ScopeModule {
    parent: Option<Box<ScopeModule>>,
    items: Vec<ScopeModuleItem>,
}

impl From<&Command> for ScopeModule {
    fn from(value: &Command) -> Self {
        let mut items = vec![];
        fn collect_from_items(result: &mut Vec<ScopeModuleItem>, from_item: &FromItem) {
            match from_item {
                FromItem::Table { table, alias } => match alias {
                    Some(TableAlias {
                        name,
                        columns: Some(columns),
                        ..
                    }) => {
                        result.push(ScopeModuleItem::Custom {
                            correlation: name.clone(),
                            columns: columns.columns.iter().cloned().collect(),
                        });
                    }
                    Some(TableAlias {
                        name,
                        columns: None,
                        ..
                    }) => {
                        result.push(ScopeModuleItem::Aliased {
                            table: table.clone(),
                            alias: name.clone(),
                        });
                    }
                    None => {
                        result.push(ScopeModuleItem::Existing(table.clone()));
                    }
                },
                FromItem::Subquery { command, alias, .. } => {
                    if let Some(alias) = alias {
                        if let Some(columns) = &alias.columns {
                            result.push(ScopeModuleItem::Custom {
                                correlation: alias.name.clone(),
                                columns: columns.columns.iter().cloned().collect(),
                            });
                        } else {
                            result.push(ScopeModuleItem::Custom {
                                correlation: alias.name.clone(),
                                columns: command
                                    .fields()
                                    .expect_or_abort("subquery must have return fields")
                                    .iter()
                                    .filter_map(|field| field.infer_name().cloned())
                                    .collect(),
                            });
                        }
                    }
                }
                FromItem::Join { left, right, .. } => {
                    collect_from_items(result, left);
                    collect_from_items(result, right);
                }
                FromItem::NaturalJoin { left, right, .. } => {
                    collect_from_items(result, left);
                    collect_from_items(result, right);
                }
                FromItem::CrossJoin { left, right, .. } => {
                    collect_from_items(result, left);
                    collect_from_items(result, right);
                }
            }
        }

        match &value.command_type {
            CommandType::Delete(delete) => {
                collect_from_items(
                    &mut items,
                    &FromItem::Table {
                        table: delete.table.clone(),
                        alias: None,
                    },
                );
                if let Some(using) = &delete.using {
                    collect_from_items(&mut items, &using.item);
                }
            }
            CommandType::Select(select) => {
                if let Some(from) = &select.from {
                    collect_from_items(&mut items, &from.item);
                }
            }
            CommandType::Insert(insert) => {
                collect_from_items(
                    &mut items,
                    &FromItem::Table {
                        table: insert.table.clone(),
                        alias: None,
                    },
                );
            }
            CommandType::Update(update) => {
                collect_from_items(
                    &mut items,
                    &FromItem::Table {
                        table: update.table.clone(),
                        alias: None,
                    },
                );
                if let Some(from) = &update.from {
                    collect_from_items(&mut items, &from.item);
                }
            }
        }

        Self {
            parent: CommandTree::with(|command_tree| {
                command_tree
                    .parent(value)
                    .map(ScopeModule::from)
                    .map(Box::new)
            }),
            items,
        }
    }
}

impl ToTokens for ScopeModule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut items = HashMap::new();
        fn collect_items_recursive<'a>(
            items: &mut HashMap<&'a Ident, &'a ScopeModuleItem>,
            current: &'a ScopeModule,
        ) {
            for item in &current.items {
                items.entry(item.name()).or_insert(item);
            }
            if let Some(parent) = &current.parent {
                collect_items_recursive(items, parent);
            }
        }
        collect_items_recursive(&mut items, self);
        let items = items.values();

        let columns = self.items.iter().map(|table| {
            let name = table.name();
            quote! {
                pub use super::tables::#name::columns::*;
            }
        });

        quote! {
            mod scope {
                pub mod tables {
                    #(#items)*
                }
                pub mod columns {
                    #(#columns)*
                }
            }
        }
        .to_tokens(tokens);
    }
}

#[derive(Clone)]
enum ScopeModuleItem {
    Existing(Path),
    Aliased {
        table: Path,
        alias: Ident,
    },
    Custom {
        correlation: Ident,
        columns: Vec<Ident>,
    },
}

impl ScopeModuleItem {
    fn name(&self) -> &Ident {
        match self {
            Self::Existing(table) => &table.segments.last().expect("paths cannot be empty").ident,
            Self::Aliased { alias, .. } => alias,
            Self::Custom { correlation, .. } => correlation,
        }
    }
}

impl ToTokens for ScopeModuleItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ScopeModuleItem::Existing(table) => {
                let table = table.to_call_site(3);
                quote! { pub use #table; }
            }
            ScopeModuleItem::Aliased { table, alias } => {
                let table = table.to_call_site(4);
                let table_name = alias.to_string();
                quote! {
                    pub mod #alias {
                        pub const TABLE_NAME: &str = #table_name;
                        pub use #table::columns;
                    }
                }
            }
            ScopeModuleItem::Custom {
                correlation,
                columns,
            } => {
                let table_name = correlation.to_string();
                let column_strings = columns.iter().map(|column| column.to_string());
                quote! {
                    pub mod #correlation {
                        pub const TABLE_NAME: &str = #table_name;
                        pub mod columns {
                            #(
                                pub mod #columns {
                                    pub const COLUMN_NAME: &str = #column_strings;
                                }
                            )*
                        }
                    }
                }
            }
        }
        .to_tokens(tokens);
    }
}
