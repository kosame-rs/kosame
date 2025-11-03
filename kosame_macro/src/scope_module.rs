use proc_macro_error::OptionExt;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Path};

use crate::{
    clause::{FromItem, FromItemIter},
    command::Command,
    parent_map::{Parent, ParentMap},
    part::{TableAlias, TargetTable},
    path_ext::PathExt,
};

pub struct ScopeIter<'a> {
    command: &'a Command,
    target_table: Option<&'a TargetTable>,
    from_items: Option<FromItemIter<'a>>,
    recursive: Option<&'a ParentMap<'a>>,
}

impl<'a> ScopeIter<'a> {
    fn new(command: &'a Command, recursive: Option<&'a ParentMap<'a>>) -> Self {
        Self {
            command,
            target_table: command.target_table(),
            from_items: command
                .command_type
                .from_item()
                .map(|from_item| from_item.into_iter()),
            recursive,
        }
    }
}

impl<'a> Iterator for ScopeIter<'a> {
    type Item = ScopeIterItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(target_table) = self.target_table.take() {
            return Some(ScopeIterItem::TargetTable(target_table));
        }
        if let Some(next) = self
            .from_items
            .as_mut()
            .and_then(|from_items| from_items.next())
        {
            return Some(ScopeIterItem::FromItem(next));
        }
        if let Some(parent_map) = &self.recursive {
            let from_item = parent_map.seek_parent::<_, FromItem>(self.command)?;
            self.command = parent_map.seek_parent::<_, Command>(self.command)?;
            match from_item {
                FromItem::Subquery {
                    lateral_keyword, ..
                } => {
                    if lateral_keyword.is_none() {
                        return None;
                    }
                    if let Some(Parent::FromItem(parent)) = parent_map.parent(from_item)
                        && let Some(right) = parent.right()
                        && std::ptr::eq(right, from_item)
                    {
                        self.from_items = parent.left().map(|left| left.into_iter());
                        return self.next();
                    }
                }
                _ => return None,
            }
        }
        None
    }
}

pub enum ScopeIterItem<'a> {
    TargetTable(&'a TargetTable),
    FromItem(&'a FromItem),
}

pub struct ScopeModule<'a> {
    command: &'a Command,
}

impl<'a> ScopeModule<'a> {
    pub fn new(command: &'a Command) -> Self {
        Self { command }
    }
}

impl ToTokens for ScopeModule<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = ParentMap::with(|parent_map| {
            ScopeIter::new(self.command, Some(parent_map))
                .flat_map(|item| match item {
                    ScopeIterItem::TargetTable(target_table) => {
                        ScopeModuleItem::try_from(&FromItem::Table {
                            table: target_table.table.clone(),
                            alias: None,
                        })
                        .ok()
                    }
                    ScopeIterItem::FromItem(from_item) => ScopeModuleItem::try_from(from_item).ok(),
                })
                .map(|item| item.to_token_stream())
                .collect::<Vec<_>>()
        });

        let columns = ScopeIter::new(self.command, None)
            .flat_map(|item| match item {
                ScopeIterItem::TargetTable(target_table) => Some(target_table.name()),
                ScopeIterItem::FromItem(from_item) => from_item.name(),
            })
            .map(|name| {
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

impl TryFrom<&FromItem> for ScopeModuleItem {
    type Error = ();
    fn try_from(value: &FromItem) -> Result<Self, Self::Error> {
        match value {
            FromItem::Table { table, alias } => match alias {
                Some(TableAlias {
                    name,
                    columns: Some(columns),
                    ..
                }) => Ok(ScopeModuleItem::Custom {
                    correlation: name.clone(),
                    columns: columns.columns.iter().cloned().collect(),
                }),
                Some(TableAlias {
                    name,
                    columns: None,
                    ..
                }) => Ok(ScopeModuleItem::Aliased {
                    table: table.clone(),
                    alias: name.clone(),
                }),
                None => Ok(ScopeModuleItem::Existing(table.clone())),
            },
            FromItem::Subquery { command, alias, .. } => match alias {
                Some(
                    alias @ TableAlias {
                        columns: Some(columns),
                        ..
                    },
                ) => Ok(ScopeModuleItem::Custom {
                    correlation: alias.name.clone(),
                    columns: columns.columns.iter().cloned().collect(),
                }),
                Some(alias) => Ok(ScopeModuleItem::Custom {
                    correlation: alias.name.clone(),
                    columns: command
                        .fields()
                        .expect_or_abort("subquery must have return fields")
                        .iter()
                        .filter_map(|field| field.infer_name().cloned())
                        .collect(),
                }),
                None => Err(()),
            },
            _ => Err(()),
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
