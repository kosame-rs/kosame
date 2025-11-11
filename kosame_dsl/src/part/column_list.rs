use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct ColumnList {
    pub _paren_token: syn::token::Paren,
    pub columns: Punctuated<Ident, Token![,]>,
}

impl Parse for ColumnList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _paren_token: parenthesized!(content in input),
            columns: content.parse_terminated(Ident::parse, Token![,])?,
        })
    }
}

impl ToTokens for ColumnList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let columns = self.columns.iter().map(|column| column.to_string());
        quote! {
            ::kosame::repr::part::ColumnList::new(&[#(#columns),*])
        }
        .to_tokens(tokens);
    }
}
