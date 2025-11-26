use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

use crate::{
    part::{Alias, TablePath},
    path_ext::PathExt,
    quote_option::QuoteOption,
    visitor::Visitor,
};

pub struct TargetTable {
    pub table: TablePath,
    pub alias: Option<Alias>,
}

impl TargetTable {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_table_path(&self.table);
    }

    #[must_use] 
    pub fn name(&self) -> &Ident {
        self.alias
            .as_ref()
            .map(|alias| &alias.ident)
            .unwrap_or_else(|| {
                &self
                    .table
                    .as_path()
                    .segments
                    .last()
                    .expect("path cannot be empty")
                    .ident
            })
    }
}

impl Parse for TargetTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            table: input.parse()?,
            alias: input.call(Alias::parse_optional)?,
        })
    }
}

impl ToTokens for TargetTable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table = self.table.as_path().to_call_site(1);
        let alias = QuoteOption::from(&self.alias);
        quote! {
            ::kosame::repr::part::TargetTable::new(#table::TABLE_NAME, #alias)
        }
        .to_tokens(tokens);
    }
}
