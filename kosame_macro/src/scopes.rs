use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, Path};

use crate::{
    clause::{FromItem, WithItem},
    command::Command,
    part::TableAlias,
    path_ext::PathExt,
};

static SCOPE_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ScopeId(u32);

impl ScopeId {
    pub fn new() -> Self {
        let increment = SCOPE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(increment)
    }

    pub fn reset() {
        SCOPE_ID.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn to_ident(self) -> Ident {
        format_ident!("scope_{}", self.0)
    }
}

impl Default for ScopeId {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Scopes<'a> {
    scopes: Vec<Scope<'a>>,
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
    modules: Vec<ScopeModule<'a>>,
}

impl<'a> Scope<'a> {
    fn new(id: ScopeId, modules: Vec<ScopeModule<'a>>) -> Self {
        Self { id, modules }
    }
}

impl ToTokens for Scope<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.id.to_ident();
        let tables = &self.modules;
        let columns = self
            .modules
            .iter()
            .filter(|module| !module.is_inherited())
            .map(|module| module.name());
        quote! {
            pub mod #name {
                pub mod tables {
                    #(#tables)*
                }
                pub mod columns {
                    #(pub use super::tables::#columns::columns::*;)*
                }
            }
        }
        .to_tokens(tokens);
    }
}

enum ScopeModule<'a> {
    Table {
        path: &'a Path,
        alias: Option<&'a Ident>,
    },
    Custom {
        name: &'a Ident,
        columns: Vec<CustomColumn<'a>>,
    },
    Inherited {
        source_id: ScopeId,
        name: &'a Ident,
    },
}

impl<'a> ScopeModule<'a> {
    fn name(&self) -> &'a Ident {
        match self {
            Self::Table {
                alias: Some(alias), ..
            } => alias,
            Self::Table { path, .. } => &path.segments.last().expect("path cannot be empty").ident,
            Self::Custom { name, .. } => name,
            Self::Inherited { name, .. } => name,
        }
    }

    fn is_inherited(&self) -> bool {
        matches!(self, Self::Inherited { .. })
    }
}

impl ToTokens for ScopeModule<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Table {
                path,
                alias: Some(alias),
            } => {
                let path = path.to_call_site(5);
                let table_name = alias.to_string();
                quote! {
                    pub mod #alias {
                        pub const TABLE_NAME: &str = #table_name;
                        pub use #path::columns;
                    }
                }
            }
            Self::Table { path, .. } => {
                let path = path.to_call_site(4);
                quote! {
                    pub use #path;
                }
            }
            Self::Custom { name, columns } => {
                let name_string = name.to_string();
                quote! {
                    pub mod #name {
                        pub const TABLE_NAME: &str = #name_string;
                        pub mod columns {
                            #(#columns)*
                        }
                    }
                }
            }
            Self::Inherited { source_id, name } => {
                let source_name = source_id.to_ident();
                quote! {
                    pub use super::super::#source_name::tables::#name;
                }
            }
        }
        .to_tokens(tokens);
    }
}

struct CustomColumn<'a> {
    name: &'a Ident,
}

impl ToTokens for CustomColumn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let name_string = self.name.to_string();
        quote! {
            pub mod #name {
                pub const COLUMN_NAME: &str = #name_string;
            }
        }
        .to_tokens(tokens);
    }
}

impl<'a> From<&'a Command> for Scopes<'a> {
    fn from(value: &'a Command) -> Self {
        fn inner<'a>(
            scopes: &mut Vec<Scope<'a>>,
            command: &'a Command,
            inherited_with_items: &mut Vec<&'a WithItem>,
            inherited_from_items: &mut Vec<(ScopeId, &'a Ident)>,
        ) {
            let mut shadow = HashSet::new();

            let mut inherited_with_items_count = 0;
            if let Some(with) = &command.with {
                for item in with.items.iter() {
                    inner(
                        scopes,
                        &item.command,
                        inherited_with_items,
                        inherited_from_items,
                    );

                    inherited_with_items.push(item);
                    inherited_with_items_count += 1;
                }
            }

            let mut modules = Vec::new();
            if let Some(target_table) = command.target_table() {
                let module = ScopeModule::Table {
                    path: &target_table.table,
                    alias: target_table.alias.as_ref().map(|alias| &alias.ident),
                };
                shadow.insert(module.name());
                modules.push(module);
            }

            let mut inherited_from_items_count = 0;
            if let Some(from_item) = command.from_item() {
                for from_item in from_item {
                    let module = if let FromItem::Table { table, alias, .. } = from_item
                        && let Some(table) = table.get_ident()
                        && let Some(with_item) = inherited_with_items
                            .iter()
                            .rev()
                            .find(|with_item| with_item.alias.name == *table)
                    {
                        Some(ScopeModule::Custom {
                            name: table,
                            columns: alias
                                .as_ref()
                                .unwrap_or(&with_item.alias)
                                .columns
                                .as_ref()
                                .map(|columns| {
                                    columns
                                        .columns
                                        .iter()
                                        .map(|name| CustomColumn { name })
                                        .collect()
                                })
                                .unwrap_or_else(|| {
                                    with_item
                                        .command
                                        .fields()
                                        .into_iter()
                                        .flat_map(|fields| {
                                            fields.iter().flat_map(|field| field.infer_name())
                                        })
                                        .map(|name| CustomColumn { name })
                                        .collect()
                                }),
                        })
                    } else {
                        match from_item {
                            FromItem::Table {
                                alias:
                                    Some(TableAlias {
                                        name,
                                        columns: Some(columns),
                                        ..
                                    }),
                                ..
                            } => Some(ScopeModule::Custom {
                                name,
                                columns: columns
                                    .columns
                                    .iter()
                                    .map(|name| CustomColumn { name })
                                    .collect(),
                            }),
                            FromItem::Table { table, alias, .. } => Some(ScopeModule::Table {
                                path: table,
                                alias: alias.as_ref().map(|alias| &alias.name),
                            }),
                            FromItem::Subquery {
                                lateral_keyword,
                                command,
                                alias,
                                ..
                            } => {
                                let mut clean_from_items = Vec::new();
                                inner(
                                    scopes,
                                    command,
                                    inherited_with_items,
                                    match lateral_keyword {
                                        Some(..) => inherited_from_items,
                                        None => &mut clean_from_items,
                                    },
                                );
                                alias.as_ref().map(|alias| ScopeModule::Custom {
                                    name: &alias.name,
                                    columns: alias
                                        .columns
                                        .as_ref()
                                        .map(|columns| {
                                            columns
                                                .columns
                                                .iter()
                                                .map(|name| CustomColumn { name })
                                                .collect()
                                        })
                                        .unwrap_or_else(|| {
                                            command
                                                .fields()
                                                .into_iter()
                                                .flat_map(|fields| {
                                                    fields
                                                        .iter()
                                                        .flat_map(|field| field.infer_name())
                                                })
                                                .map(|name| CustomColumn { name })
                                                .collect()
                                        }),
                                })
                            }
                            _ => None,
                        }
                    };
                    if let Some(module) = module {
                        shadow.insert(module.name());
                        inherited_from_items.push((command.scope_id, module.name()));
                        inherited_from_items_count += 1;

                        modules.push(module);
                    }
                }
            }

            for (source_id, name) in inherited_from_items.iter() {
                if !shadow.contains(name) {
                    modules.push(ScopeModule::Inherited {
                        source_id: *source_id,
                        name,
                    });
                }
            }

            inherited_with_items.truncate(inherited_with_items.len() - inherited_with_items_count);
            inherited_from_items.truncate(inherited_from_items.len() - inherited_from_items_count);

            scopes.push(Scope::new(command.scope_id, modules));
        }

        let mut scopes = Vec::new();
        inner(&mut scopes, value, &mut Vec::new(), &mut Vec::new());
        scopes.sort_by_key(|v| v.id);
        Scopes { scopes }
    }
}
