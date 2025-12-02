use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

use crate::{
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
};

pub struct Alias {
    pub as_token: Token![as],
    pub ident: Ident,
}

impl ParseOption for Alias {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![as])
    }
}

impl Parse for Alias {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            as_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl ToTokens for Alias {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_string().to_tokens(tokens);
    }
}

impl PrettyPrint for Alias {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        " ".pretty_print(printer);
        self.as_token.pretty_print(printer);
        " ".pretty_print(printer);
        self.ident.pretty_print(printer);
    }
}
