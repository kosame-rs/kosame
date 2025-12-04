use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    expr::Expr,
    keyword,
    pretty::{BreakMode, Delim, PrettyPrint, Printer},
    visit::Visit,
};

pub struct Values {
    pub values_keyword: keyword::values,
    pub rows: Punctuated<ValuesRow, Token![,]>,
}

impl Values {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::values)
    }
}

pub fn visit_values<'a>(visit: &mut (impl Visit<'a> + ?Sized), values: &'a Values) {
    for row in &values.rows {
        visit.visit_values_row(row);
    }
}

impl Parse for Values {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            values_keyword: input.parse()?,
            rows: {
                let mut punctuated = Punctuated::new();
                while input.peek(syn::token::Paren) {
                    punctuated.push(input.parse()?);
                    if !input.peek(Token![,]) {
                        break;
                    }
                    punctuated.push_punct(input.parse()?);
                }
                if punctuated.is_empty() {
                    return Err(syn::Error::new(input.span(), "values list cannot be empty"));
                }
                punctuated
            },
        })
    }
}

impl ToTokens for Values {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let rows = self.rows.iter();
        quote! { ::kosame::repr::clause::Values::new(&[#(#rows),*]) }.to_tokens(tokens);
    }
}

impl PrettyPrint for Values {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_break();
        printer.scan_trivia(true, true);
        " ".pretty_print(printer);
        self.values_keyword.pretty_print(printer);
        printer.scan_indent(1);
        printer.scan_break();
        " ".pretty_print(printer);
        self.rows.pretty_print(printer);
        printer.scan_indent(-1);
    }
}

pub struct ValuesRow {
    paren_token: syn::token::Paren,
    items: Punctuated<ValuesItem, Token![,]>,
}

pub fn visit_values_row<'a>(visit: &mut (impl Visit<'a> + ?Sized), values_row: &'a ValuesRow) {
    for item in &values_row.items {
        visit.visit_values_item(item);
    }
}

impl Parse for ValuesRow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            items: content.parse_terminated(ValuesItem::parse, Token![,])?,
        })
    }
}

impl ToTokens for ValuesRow {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items.iter();
        quote! { ::kosame::repr::clause::ValuesRow::new(&[#(#items),*]) }.to_tokens(tokens);
    }
}

impl PrettyPrint for ValuesRow {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.paren_token
            .pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                self.items.pretty_print(printer);
            });
    }
}

#[allow(unused)]
pub enum ValuesItem {
    Default(keyword::default),
    Expr(Expr),
}

pub fn visit_values_item<'a>(visit: &mut (impl Visit<'a> + ?Sized), values_item: &'a ValuesItem) {
    match values_item {
        ValuesItem::Default(..) => {}
        ValuesItem::Expr(expr) => visit.visit_expr(expr),
    }
}

impl Parse for ValuesItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::default) {
            Ok(Self::Default(input.parse()?))
        } else {
            Ok(Self::Expr(input.parse()?))
        }
    }
}

impl ToTokens for ValuesItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Default(..) => quote! {
                ::kosame::repr::clause::ValuesItem::Default
            },
            Self::Expr(expr) => quote! {
                ::kosame::repr::clause::ValuesItem::Expr(#expr)
            },
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for ValuesItem {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::Default(inner) => inner.pretty_print(printer),
            Self::Expr(inner) => inner.pretty_print(printer),
        }
    }
}
