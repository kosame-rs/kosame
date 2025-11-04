use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, Path};

use crate::{
    clause::FromItem, command::Command, part::TableAlias, path_ext::PathExt, visitor::Visitor,
};

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

pub struct Scope<'a> {
    index: usize,
    modules: Vec<ScopeModule<'a>>,
}

impl<'a> Scope<'a> {
    fn new(index: usize, modules: Vec<ScopeModule<'a>>) -> Self {
        Self { index, modules }
    }
}

impl ToTokens for Scope<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("scope_{}", self.index);
        let modules = &self.modules;
        quote! {
            pub mod #name {
                #(#modules)*
            }
        }
        .to_tokens(tokens);
    }
}

pub enum ScopeModule<'a> {
    Table {
        path: &'a Path,
        alias: Option<&'a Ident>,
    },
    Custom {
        name: &'a Ident,
        columns: Vec<&'a Ident>,
    },
}

impl ToTokens for ScopeModule<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Table {
                path,
                alias: Some(alias),
            } => {
                let path = path.to_call_site(4);
                let table_name = alias.to_string();
                quote! {
                    pub mod #alias {
                        pub const TABLE_NAME: &str = #table_name;
                        pub use #path::columns;
                    }
                }
            }
            Self::Table { path, .. } => {
                let path = path.to_call_site(3);
                quote! {
                    pub use #path;
                }
            }
            Self::Custom { name, columns } => {
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
        }
        .to_tokens(tokens);
    }
}

pub struct ScopesBuilder<'a> {
    scopes: Vec<Scope<'a>>,
}

impl<'a> ScopesBuilder<'a> {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn build(self) -> Scopes<'a> {
        Scopes {
            scopes: self.scopes,
        }
    }
}

impl<'a> Visitor<'a> for ScopesBuilder<'a> {
    fn visit_command(&mut self, command: &'a Command) {
        let mut modules = Vec::new();
        if let Some(target_table) = command.target_table() {
            modules.push(ScopeModule::Table {
                path: &target_table.table,
                alias: target_table.alias.as_ref().map(|alias| &alias.ident),
            });
        }
        if let Some(from_item) = command.from_item() {
            for from_item in from_item {
                match from_item {
                    FromItem::Table {
                        alias:
                            Some(TableAlias {
                                name,
                                columns: Some(columns),
                                ..
                            }),
                        ..
                    } => {
                        modules.push(ScopeModule::Custom {
                            name,
                            columns: columns.columns.iter().collect(),
                        });
                    }
                    FromItem::Table { table, alias, .. } => {
                        modules.push(ScopeModule::Table {
                            path: table,
                            alias: alias.as_ref().map(|alias| &alias.name),
                        });
                    }
                    _ => {}
                }
            }
        }
        self.scopes.push(Scope::new(self.scopes.len(), modules));
    }
}
