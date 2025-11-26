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
    part::TableAlias,
    visitor::Visitor,
};

pub struct With {
    pub _with_keyword: keyword::with,
    pub items: Punctuated<WithItem, Token![,]>,
}

impl With {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::with)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        for item in &self.items {
            item.accept(visitor);
        }
    }
}

impl Parse for With {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _with_keyword: input.call(keyword::with::parse_autocomplete)?,
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

pub struct WithItem {
    pub alias: TableAlias,
    pub _as_token: Token![as],
    pub _paren_token: syn::token::Paren,
    pub command: Command,
    pub correlation_id: CorrelationId,
}

impl WithItem {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.command.accept(visitor);
    }

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

impl Parse for WithItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            alias: input.parse()?,
            _as_token: input.parse()?,
            _paren_token: parenthesized!(content in input),
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
