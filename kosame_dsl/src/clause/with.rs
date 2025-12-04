use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    command::{Command, CommandType},
    correlations::CorrelationId,
    keyword,
    parse_option::ParseOption,
    part::TableAlias,
    pretty::{BreakMode, Delim, PrettyPrint, Printer},
    visit::Visit,
};

pub struct With {
    pub with_keyword: keyword::with,
    pub items: Punctuated<WithItem, Token![,]>,
}

impl ParseOption for With {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::with)
    }
}

pub fn visit_with<'a>(visit: &mut (impl Visit<'a> + ?Sized), with: &'a With) {
    for item in &with.items {
        visit.visit_with_item(item);
    }
}

impl Parse for With {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            with_keyword: input.call(keyword::with::parse_autocomplete)?,
            items: {
                let mut punctuated = Punctuated::new();
                while !input.is_empty() {
                    if CommandType::peek(input) {
                        break;
                    }
                    punctuated.push(input.parse()?);
                    if CommandType::peek(input) {
                        break;
                    }
                    punctuated.push_punct(input.parse()?);
                }
                if punctuated.is_empty() {
                    return Err(syn::Error::new(input.span(), "with clause cannot be empty"));
                }
                punctuated
            },
        })
    }
}

impl ToTokens for With {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items.iter();
        quote! { ::kosame::repr::clause::With::new(&[#(#items),*]) }.to_tokens(tokens);
    }
}

impl PrettyPrint for With {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.with_keyword.pretty_print(printer);
        printer.scan_break();
        printer.scan_indent(1);
        " ".pretty_print(printer);
        printer.scan_begin(BreakMode::Consistent);
        self.items.pretty_print(printer);
        printer.scan_end();
        printer.scan_indent(-1);
    }
}

pub struct WithItem {
    pub alias: TableAlias,
    pub as_token: Token![as],
    pub paren_token: syn::token::Paren,
    pub command: Command,
    pub correlation_id: CorrelationId,
}

impl WithItem {
    #[must_use]
    pub fn columns(&self) -> Vec<&Ident> {
        match &self.alias.columns {
            Some(columns) => columns.columns.iter().collect(),
            None => self
                .command
                .fields()
                .into_iter()
                .flat_map(|fields| fields.columns())
                .collect(),
        }
    }
}

pub fn visit_with_item<'a>(visit: &mut (impl Visit<'a> + ?Sized), with_item: &'a WithItem) {
    visit.visit_command(&with_item.command);
}

impl Parse for WithItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            alias: input.parse()?,
            as_token: input.parse()?,
            paren_token: parenthesized!(content in input),
            command: content.parse()?,
            correlation_id: CorrelationId::new(),
        })
    }
}

impl ToTokens for WithItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let alias = &self.alias;
        let command = &self.command;
        quote! { ::kosame::repr::clause::WithItem::new(#alias, #command) }.to_tokens(tokens);
    }
}

impl PrettyPrint for WithItem {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.alias.pretty_print(printer);
        " ".pretty_print(printer);
        self.as_token.pretty_print(printer);
        self.paren_token
            .pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                self.command.pretty_print(printer);
            });
        " ".pretty_print(printer);
    }
}
