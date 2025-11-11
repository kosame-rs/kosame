use proc_macro_error::emit_error;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::path_ext::PathExt;

pub struct Relation {
    pub name: Ident,
    pub _colon: Token![:],
    pub source_paren: syn::token::Paren,
    pub source_columns: Punctuated<Ident, Token![,]>,
    pub arrow: Arrow,
    pub target_table: syn::Path,
    pub target_paren: syn::token::Paren,
    pub target_columns: Punctuated<Ident, Token![,]>,
}

impl Parse for Relation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source_content;
        let dest_content;
        let result = Self {
            name: input.parse()?,
            _colon: input.parse()?,
            source_paren: parenthesized!(source_content in input),
            source_columns: source_content.parse_terminated(Ident::parse, Token![,])?,
            arrow: input.parse()?,
            target_table: input.parse()?,
            target_paren: parenthesized!(dest_content in input),
            target_columns: dest_content.parse_terminated(Ident::parse, Token![,])?,
        };

        if result.source_columns.is_empty() {
            emit_error!(
                result.source_paren.span.span(),
                "at least one column must be specified for relation `{}`",
                result.name
            );
        }
        if result.source_columns.len() != result.target_columns.len() {
            emit_error!(
                result.target_paren.span.span(),
                "number of columns must match on both side of the relation `{}`",
                result.name
            );
        }

        Ok(result)
    }
}

impl ToTokens for Relation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let name_string = name.to_string();

        let target_table = &self.target_table.to_call_site(3);

        let source_columns = self.source_columns.iter().collect::<Vec<_>>();
        let target_columns = self.target_columns.iter().collect::<Vec<_>>();

        let arrow = &self.arrow;

        quote! {
            pub mod #name {
                pub use #target_table as target_table;

                pub mod source_columns {
                    #(pub use super::super::super::columns::#source_columns;)*
                }

                pub mod target_columns {
                    #(pub use super::target_table::columns::#target_columns;)*
                }

                pub const RELATION: ::kosame::repr::schema::Relation<'_> = ::kosame::repr::schema::Relation::new(
                    #name_string,
                    super::super::TABLE_NAME,
                    &[#(&source_columns::#source_columns::COLUMN),*],
                    target_table::TABLE_NAME,
                    &[#(&target_columns::#target_columns::COLUMN),*],
                );

                pub type Type<T> = #arrow;
            }
        }
        .to_tokens(tokens);
    }
}

#[allow(unused)]
pub enum Arrow {
    ManyToOne(Token![=>]),
    OneToMany(Token![<=]),
}

impl Parse for Arrow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=>]) {
            Ok(Self::ManyToOne(input.parse()?))
        } else if lookahead.peek(Token![<=]) {
            Ok(Self::OneToMany(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Arrow {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::ManyToOne(..) => quote! { ::kosame::relation::ZeroOrOne<T> },
            Self::OneToMany(..) => quote! { ::kosame::relation::Many<T> },
        }
        .to_tokens(tokens);
    }
}
