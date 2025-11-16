use std::{
    hash::{Hash, Hasher},
    sync::atomic::Ordering,
};

use crate::{
    attribute::{CustomMeta, MetaLocation},
    keyword,
    pretty_print::{BreakMode, PrettyPrint, Printer},
    row::{Row, RowField},
    unique_macro::unique_macro,
};

use super::{column::Column, relation::Relation};
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Ident, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct Table {
    _token_stream: TokenStream,

    pub _inner_attrs: Vec<Attribute>,
    pub _outer_attrs: Vec<Attribute>,

    pub _create: keyword::create,
    pub _table: keyword::table,
    pub name: Ident,

    pub _paren: syn::token::Paren,

    pub columns: Punctuated<Column, Token![,]>,

    pub _semi: Token![;],

    pub relations: Punctuated<Relation, Token![,]>,
}

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _token_stream: input.fork().parse()?,
            _inner_attrs: {
                let attrs = Attribute::parse_inner(input)?;
                CustomMeta::parse_attrs(&attrs, MetaLocation::TableInner)?;
                attrs
            },
            _outer_attrs: {
                let attrs = Attribute::parse_outer(input)?;
                CustomMeta::parse_attrs(&attrs, MetaLocation::TableOuter)?;
                attrs
            },
            _create: input.call(keyword::create::parse_autocomplete)?,
            _table: input.call(keyword::table::parse_autocomplete)?,
            name: input.parse()?,
            _paren: syn::parenthesized!(content in input),
            columns: content.parse_terminated(Column::parse, Token![,])?,
            _semi: input.parse()?,
            relations: input.parse_terminated(Relation::parse, Token![,])?,
        })
    }
}

impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.to_string();
        let rust_name = Ident::new(
            &self.name.to_string().to_case(Case::Snake),
            self.name.span(),
        );

        let columns = self.columns.iter();
        let relations = self.relations.iter();

        let column_names = self
            .columns
            .iter()
            .map(Column::rust_name)
            .collect::<Vec<_>>();
        let relation_names = self
            .relations
            .iter()
            .map(|relation| &relation.name)
            .collect::<Vec<_>>();

        let select_struct = Row::new(
            vec![],
            Ident::new("Select", Span::call_site()),
            self.columns
                .iter()
                .map(|column| {
                    let column = column.rust_name();
                    RowField::new(vec![], column.clone(), quote! { columns::#column::Type })
                })
                .collect(),
        );

        let star_macro = {
            let unique_macro_name = unique_macro!("__kosame_star_{}", self.name.span());
            let fields = self.columns.iter().map(|column| {
                let column_name = column.rust_name();
                RowField::new(
                    vec![],
                    column_name.clone(),
                    quote! { $($table_path)* ::columns::#column_name::Type },
                )
            });

            quote! {
                #[macro_export]
                macro_rules! #unique_macro_name {
                    (
                        ($($table_path:tt)*)
                        $(#[$meta:meta])* pub struct $name:ident { $($tokens:tt)* }
                    ) => {
                        $(#[$meta])*
                        pub struct $name {
                            #(#fields,)*
                            $($tokens)*
                        }
                    }
                }

                pub use #unique_macro_name as star;
            }
        };

        let inject_macro = {
            let unique_macro_name = unique_macro!("__kosame_inject_{}", self.name.span());
            let token_stream = &self._token_stream;

            quote! {
                #[macro_export]
                macro_rules! #unique_macro_name {
                    (
                        $(#![$acc:meta])*
                        ($($child:tt)*) {
                            $($content:tt)*
                        }
                        ($($table_path:tt)*)
                    ) => {
                        $($child)* {
                            #![kosame(__table($($table_path)* = #token_stream))]
                            $(#![$acc])*

                            $($content)*
                        }
                    }
                }

                pub use #unique_macro_name as inject;
            }
        };

        quote! {
            pub mod #rust_name {
                pub mod columns {
                    #(#columns)*
                }

                pub mod relations {
                    #(#relations)*
                }

                pub mod columns_and_relations {
                    pub use super::columns::*;
                    pub use super::relations::*;
                }

                pub const TABLE_NAME: &str = #name;
                pub const TABLE: ::kosame::repr::schema::Table<'_> = ::kosame::repr::schema::Table::new(
                    #name,
                    &[#(&columns::#column_names::COLUMN),*],
                    &[#(&relations::#relation_names::RELATION),*],
                );

                #select_struct

                #star_macro
                #inject_macro
            }
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for Table {
    fn pretty_print(&self, printer: &mut Printer) {
        printer.print_text("create table (");
        printer.print_begin(BreakMode::Consistent);
        printer.print_break("");
        for column in self.columns.iter() {
            column.pretty_print(printer);
            printer.print_text(",");
            printer.print_break(" ");
        }
        printer.print_end();
        printer.print_text(");");
    }
}
