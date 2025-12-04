use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::pretty::{BreakMode, Delim, PrettyPrint, Printer};

pub struct ColumnList {
    pub paren_token: syn::token::Paren,
    pub columns: Punctuated<Ident, Token![,]>,
}

impl Parse for ColumnList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            columns: content.parse_terminated(Ident::parse, Token![,])?,
        })
    }
}

impl ToTokens for ColumnList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let columns = self.columns.iter().map(std::string::ToString::to_string);
        quote! {
            ::kosame::repr::part::ColumnList::new(&[#(#columns),*])
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for ColumnList {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        " ".pretty_print(printer);
        self.paren_token
            .pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                self.columns.pretty_print(printer);
            });
    }
}
