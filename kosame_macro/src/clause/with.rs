use std::{cell::RefCell, collections::HashSet};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    command::{Command, CommandType},
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

    pub fn scoped(&self, f: impl FnOnce()) {
        fn scoped<'a>(f: impl FnOnce(), mut rest: impl Iterator<Item = &'a WithItem>) {
            if let Some(first) = rest.next() {
                first.scoped(|| scoped(f, rest));
            } else {
                f();
            }
        }
        scoped(f, self.items.iter());
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
        quote! { ::kosame::repr::clause::With::new(&[#(#items),*]) }.to_tokens(tokens)
    }
}

pub struct WithItem {
    pub alias: TableAlias,
    pub _as_token: Token![as],
    pub _paren_token: syn::token::Paren,
    pub command: Command,
}

impl WithItem {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.command.accept(visitor);
    }

    pub fn scoped<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        STACK.with_borrow_mut(|stack| stack.push(self));
        f();
        STACK.with_borrow_mut(|stack| stack.pop());
    }

    pub fn scoped_for_each<F>(mut f: F)
    where
        F: FnMut(&WithItem),
    {
        STACK.with_borrow(|stack| {
            let items = stack.iter().rev().map(|item| {
                // Safety: The references added to the stack in the `scoped` method outlive the
                // closure from which `iter_scope` is called, so the immutable pointer will also
                // outlive this block.
                unsafe { &**item }
            });

            let mut shadowed = HashSet::new();
            for item in items {
                if shadowed.contains(&item.alias.name) {
                    continue;
                }
                f(item);
                shadowed.insert(&item.alias.name);
            }
        });
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

thread_local! {
    static STACK: RefCell<Vec<*const WithItem>> = const { RefCell::new(Vec::new()) };
}
