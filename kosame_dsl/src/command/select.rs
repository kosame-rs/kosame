use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    clause::{self, Fields, FromChain, Limit, Offset, OrderBy},
    command::Command,
    keyword,
    parse_option::ParseOption,
    quote_option::QuoteOption,
    scopes::Scoped,
    visit::Visit,
};

pub struct Select {
    pub chain: SelectChain,
    pub order_by: Option<OrderBy>,
    pub limit: Option<Limit>,
    pub offset: Option<Offset>,
}

impl Select {
    #[must_use]
    pub fn fields(&self) -> &Fields {
        match &self.chain.start {
            SelectItem::Core(core) => &core.select.fields,
            SelectItem::Paren(paren) => paren.fields().expect("nested select must have fields"),
        }
    }

    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub fn from_chain(&self) -> Option<&FromChain> {
        if !self.chain.combinators.is_empty() {
            return None;
        }
        match &self.chain.start {
            SelectItem::Core(core) => core.from.as_ref().map(|from| &from.chain),
            SelectItem::Paren(paren) => paren.from_chain(),
        }
    }

    pub fn peek(input: ParseStream) -> bool {
        SelectItem::peek(input)
    }
}

pub fn visit_select_command<'a>(visit: &mut (impl Visit<'a> + ?Sized), select: &'a Select) {
    visit.visit_select_chain(&select.chain);
    if let Some(inner) = select.order_by.as_ref() {
        visit.visit_order_by(inner);
    }
    if let Some(inner) = select.limit.as_ref() {
        visit.visit_limit(inner);
    }
    if let Some(inner) = select.offset.as_ref() {
        visit.visit_offset(inner);
    }
}

impl Parse for Select {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let chain = input.parse()?;
        let order_by = input.call(OrderBy::parse_option)?;
        let limit = input.call(Limit::parse_option)?;
        let offset = input.call(Offset::parse_option)?;

        Ok(Self {
            chain,
            order_by,
            limit,
            offset,
        })
    }
}

impl ToTokens for Select {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let chain = &self.chain;
        let order_by = QuoteOption::from(&self.order_by);
        let limit = QuoteOption::from(&self.limit);
        let offset = QuoteOption::from(&self.offset);

        quote! {
            ::kosame::repr::command::Select::new(
                #chain,
                #order_by,
                #limit,
                #offset,
            )
        }
        .to_tokens(tokens);
    }
}

pub struct SelectChain {
    pub start: SelectItem,
    pub combinators: Vec<SelectCombinator>,
}

impl SelectChain {
    #[must_use]
    pub fn iter(&self) -> SelectIter<'_> {
        self.into_iter()
    }
}

pub fn visit_select_chain<'a>(visit: &mut (impl Visit<'a> + ?Sized), select_chain: &'a SelectChain) {
    visit.visit_select_item(&select_chain.start);
    for combinator in &select_chain.combinators {
        visit.visit_select_combinator(combinator);
    }
}

impl Parse for SelectChain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start = input.parse()?;
        let mut combinators = Vec::new();

        // Parse set operations (UNION, INTERSECT, EXCEPT)
        while input.peek(keyword::union)
            || input.peek(keyword::intersect)
            || input.peek(keyword::except)
        {
            combinators.push(input.parse()?);
        }

        Ok(Self { start, combinators })
    }
}

impl ToTokens for SelectChain {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let start = &self.start;
        let combinators = &self.combinators;

        quote! {
            ::kosame::repr::command::SelectChain::new(
                #start,
                &[#(#combinators),*],
            )
        }
        .to_tokens(tokens);
    }
}

pub enum SelectItem {
    Core(Box<clause::SelectCore>),
    Paren(Box<Command>),
}

impl SelectItem {
    pub fn peek(input: ParseStream) -> bool {
        clause::SelectCore::peek(input) || input.peek(syn::token::Paren)
    }
}

pub fn visit_select_item<'a>(visit: &mut (impl Visit<'a> + ?Sized), select_item: &'a SelectItem) {
    match select_item {
        SelectItem::Core(core) => visit.visit_select_core(core),
        SelectItem::Paren(command) => visit.visit_command(command),
    }
}

impl Parse for SelectItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Ok(Self::Paren(content.parse()?))
        } else {
            let core = input.parse()?;
            Ok(Self::Core(core))
        }
    }
}

impl ToTokens for SelectItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Core(core) => {
                quote! {
                    ::kosame::repr::command::SelectItem::Core(#core)
                }
                .to_tokens(tokens);
            }
            Self::Paren(paren) => {
                quote! {
                    ::kosame::repr::command::SelectItem::Paren(&#paren)
                }
                .to_tokens(tokens);
            }
        }
    }
}

pub struct SelectCombinator {
    pub op: SetOp,
    pub quantifier: SetQuantifier,
    pub right: SelectItem,
}

pub fn visit_select_combinator<'a>(visit: &mut (impl Visit<'a> + ?Sized), select_combinator: &'a SelectCombinator) {
    visit.visit_select_item(&select_combinator.right);
}

impl Parse for SelectCombinator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let quantifier = if input.peek(keyword::all) {
            input.parse()?
        } else {
            SetQuantifier::Distinct
        };
        let right = input.parse()?;

        Ok(Self {
            op,
            quantifier,
            right,
        })
    }
}

impl ToTokens for SelectCombinator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let op = &self.op;
        let quantifier = &self.quantifier;
        let right = &self.right;

        quote! {
            ::kosame::repr::command::SelectCombinator::new(
                #op,
                #quantifier,
                #right,
            )
        }
        .to_tokens(tokens);
    }
}

pub enum SetOp {
    Union(keyword::union),
    Intersect(keyword::intersect),
    Except(keyword::except),
}

impl Parse for SetOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::union) {
            Ok(Self::Union(input.parse()?))
        } else if lookahead.peek(keyword::intersect) {
            Ok(Self::Intersect(input.parse()?))
        } else if lookahead.peek(keyword::except) {
            Ok(Self::Except(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for SetOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Union(_) => {
                quote! { ::kosame::repr::command::SetOp::Union }
            }
            Self::Intersect(_) => {
                quote! { ::kosame::repr::command::SetOp::Intersect }
            }
            Self::Except(_) => {
                quote! { ::kosame::repr::command::SetOp::Except }
            }
        }
        .to_tokens(tokens);
    }
}

pub enum SetQuantifier {
    All(keyword::all),
    Distinct,
}

impl Parse for SetQuantifier {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::all) {
            Ok(Self::All(input.parse()?))
        } else {
            Ok(Self::Distinct)
        }
    }
}

impl ToTokens for SetQuantifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::All(_) => {
                quote! { ::kosame::repr::command::SetQuantifier::All }
            }
            Self::Distinct => {
                quote! { ::kosame::repr::command::SetQuantifier::Distinct }
            }
        }
        .to_tokens(tokens);
    }
}

pub struct SelectIter<'a> {
    chain: &'a SelectChain,
    index: i32,
}

impl<'a> Iterator for SelectIter<'a> {
    type Item = &'a SelectItem;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.index {
            -1 => &self.chain.start,
            _ => {
                &self
                    .chain
                    .combinators
                    .get::<usize>(self.index.try_into().unwrap())?
                    .right
            }
        };
        self.index += 1;
        Some(item)
    }
}

impl<'a> IntoIterator for &'a SelectChain {
    type IntoIter = SelectIter<'a>;
    type Item = &'a SelectItem;

    fn into_iter(self) -> Self::IntoIter {
        SelectIter {
            index: -1,
            chain: self,
        }
    }
}
