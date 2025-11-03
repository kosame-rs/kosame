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
        let mut tables = TokenStream::new();
        ParentMap::with(|parent_map| {
            fn table_tokens(path: &Path, alias: Option<&Ident>) -> TokenStream {
                match alias {
                    Some(alias) => {
                        let path = path.to_call_site(4);
                        let table_name = alias.to_string();
                        quote! {
                            pub mod #alias {
                                pub const TABLE_NAME: &str = #table_name;
                                pub use #path::columns;
                            }
                        }
                    }
                    None => {
                        let path = path.to_call_site(3);
                        quote! { pub use #path; }
                    }
                }
            }

            fn custom_tokens(name: &Ident, columns: &[&Ident]) -> TokenStream {
                let name_string = name.to_string();
                let column_strings = columns.iter().map(|column| column.to_string());
                quote! {
                    pub mod #name {
                        pub const TABLE_NAME: &str = #name_string;
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

            for item in ScopeIter::new(self.command, Some(parent_map)) {
                match item {
                    ScopeIterItem::TargetTable(target_table) => table_tokens(
                        &target_table.table,
                        target_table.alias.as_ref().map(|alias| &alias.ident),
                    ),
                    ScopeIterItem::FromItem(from_item) => match from_item {
                        FromItem::Table { table, alias } => match alias {
                            Some(TableAlias {
                                name,
                                columns: Some(columns),
                                ..
                            }) => custom_tokens(name, &columns.columns.iter().collect::<Vec<_>>()),
                            _ => table_tokens(table, alias.as_ref().map(|alias| &alias.name)),
                        },
                        FromItem::Subquery { command, alias, .. } => match alias {
                            Some(
                                alias @ TableAlias {
                                    columns: Some(columns),
                                    ..
                                },
                            ) => custom_tokens(
                                &alias.name,
                                &columns.columns.iter().collect::<Vec<_>>(),
                            ),
                            Some(alias) => custom_tokens(
                                &alias.name,
                                &command
                                    .fields()
                                    .expect_or_abort("subquery must have return fields")
                                    .iter()
                                    .filter_map(|field| field.infer_name())
                                    .collect::<Vec<_>>(),
                            ),
                            None => quote! {},
                        },
                        _ => quote! {},
                    },
                }
                .to_tokens(&mut tables);
            }
        });

        let mut local_columns = TokenStream::new();
        for item in ScopeIter::new(self.command, None) {
            let name = match item {
                ScopeIterItem::TargetTable(target_table) => Some(target_table.name()),
                ScopeIterItem::FromItem(from_item) => from_item.name(),
            };
            if let Some(name) = name {
                quote! { pub use super::tables::#name::columns::*; }.to_tokens(&mut local_columns);
            }
        }

        quote! {
            mod scope {
                pub mod tables { #tables }
                pub mod columns { #local_columns }
            }
        }
        .to_tokens(tokens);
    }
}
