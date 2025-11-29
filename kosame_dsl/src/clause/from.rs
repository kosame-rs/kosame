use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::{
    clause::WithItem,
    command::Command,
    correlations::CorrelationId,
    expr::Expr,
    keyword,
    parse_option::ParseOption,
    part::{TableAlias, TablePath},
    quote_option::QuoteOption,
    scopes::ScopeId,
    visit::Visit,
};

pub struct From {
    pub from: keyword::from,
    pub chain: FromChain,
}

impl ParseOption for From {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::from)
    }
}

pub fn visit_from<'a>(visit: &mut (impl Visit<'a> + ?Sized), from: &'a From) {
    visit.visit_from_chain(&from.chain);
}

impl Parse for From {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            from: input.parse()?,
            chain: input.parse()?,
        })
    }
}

impl ToTokens for From {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item = &self.chain;
        quote! { ::kosame::repr::clause::From::new(#item) }.to_tokens(tokens);
    }
}

pub struct FromChain {
    pub start: FromItem,
    pub combinators: Vec<FromCombinator>,
}

impl FromChain {
    #[must_use]
    pub fn len(&self) -> usize {
        self.combinators.len() + 1
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn nullables(&self) -> Vec<bool> {
        let mut nullables = vec![false; self.len()];
        {
            let mut nullable_join_found = false;
            for (index, combinator) in self.combinators.iter().enumerate() {
                if let FromCombinator::Join { join_type, .. } = combinator
                    && let JoinType::Left(..) | JoinType::Full(..) = join_type
                {
                    nullable_join_found = true;
                }
                nullables[index + 1] = nullables[index + 1] || nullable_join_found;
            }
        }
        {
            let mut nullable_join_found = false;
            for (index, combinator) in self.combinators.iter().enumerate().rev() {
                if let FromCombinator::Join { join_type, .. } = combinator
                    && let JoinType::Right(..) | JoinType::Full(..) = join_type
                {
                    nullable_join_found = true;
                }
                nullables[index] = nullables[index] || nullable_join_found;
            }
        }
        nullables
    }

    #[must_use]
    pub fn iter(&self) -> FromIter<'_> {
        self.into_iter()
    }
}

pub fn visit_from_chain<'a>(visit: &mut (impl Visit<'a> + ?Sized), from_chain: &'a FromChain) {
    visit.visit_from_item(&from_chain.start);
    for combinator in &from_chain.combinators {
        visit.visit_from_combinator(combinator);
    }
}

impl Parse for FromChain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            start: input.parse()?,
            combinators: input.call(FromCombinator::parse_many)?,
        })
    }
}

impl ToTokens for FromChain {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let start = &self.start;
        let combinators = &self.combinators;
        quote! {
            ::kosame::repr::clause::FromChain::new(#start, &[#(#combinators),*])
        }
        .to_tokens(tokens);
    }
}

pub enum FromItem {
    Table {
        table_path: TablePath,
        alias: Option<TableAlias>,
        correlation_id: CorrelationId,
    },
    Subquery {
        lateral_keyword: Option<keyword::lateral>,
        paren_token: syn::token::Paren,
        command: Box<Command>,
        alias: Option<TableAlias>,
        correlation_id: CorrelationId,
    },
}

impl FromItem {
    #[must_use]
    pub fn name(&self) -> Option<&Ident> {
        match self {
            Self::Table {
                table_path, alias, ..
            } => Some(alias.as_ref().map_or_else(
                || {
                    &table_path
                        .as_path()
                        .segments
                        .last()
                        .expect("path cannot be empty")
                        .ident
                },
                |alias| &alias.name,
            )),
            Self::Subquery { alias, .. } => alias.as_ref().map(|alias| &alias.name),
        }
    }

    #[must_use]
    pub fn correlation_id(&self) -> CorrelationId {
        match self {
            Self::Table { correlation_id, .. } => *correlation_id,
            Self::Subquery { correlation_id, .. } => *correlation_id,
        }
    }

    #[must_use]
    pub fn columns<'a>(&'a self, with_item: Option<&'a WithItem>) -> Vec<&'a Ident> {
        match self {
            Self::Table { alias, .. } => match with_item {
                Some(with_item) => with_item.columns(),
                None => match alias {
                    Some(
                        TableAlias {
                            columns: Some(columns),
                            ..
                        },
                        ..,
                    ) => columns.columns.iter().collect(),
                    _ => Vec::new(),
                },
            },
            Self::Subquery { command, .. } => command
                .fields()
                .into_iter()
                .flat_map(|fields| fields.columns())
                .collect(),
        }
    }
}

pub fn visit_from_item<'a>(visit: &mut (impl Visit<'a> + ?Sized), from_item: &'a FromItem) {
    match from_item {
        FromItem::Table { table_path, .. } => {
            visit.visit_table_path(table_path);
        }
        FromItem::Subquery { command, .. } => {
            visit.visit_command(command);
        }
    }
}

impl Parse for FromItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lateral_keyword = input
            .peek(keyword::lateral)
            .then(|| input.parse())
            .transpose()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Paren) {
            let content;
            Ok(Self::Subquery {
                lateral_keyword,
                paren_token: parenthesized!(content in input),
                command: content.parse()?,
                alias: input.call(TableAlias::parse_option)?,
                correlation_id: CorrelationId::new(),
            })
        } else if lookahead.peek(Ident) {
            Ok(Self::Table {
                table_path: input.parse()?,
                alias: input.call(TableAlias::parse_option)?,
                correlation_id: CorrelationId::new(),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for FromItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Table {
                table_path, alias, ..
            } => {
                let table = alias.as_ref().map_or(
                    &table_path
                        .as_path()
                        .segments
                        .last()
                        .expect("paths cannot be empty")
                        .ident,
                    |alias| &alias.name,
                );
                let alias = QuoteOption::from(alias);
                let scope_id = ScopeId::of_scope();
                quote! {
                    ::kosame::repr::clause::FromItem::Table {
                        table: scopes::#scope_id::tables::#table::TABLE_NAME,
                        alias: #alias,
                    }
                }
            }
            Self::Subquery {
                lateral_keyword,
                command,
                alias,
                ..
            } => {
                let lateral = lateral_keyword.is_some();
                let alias = QuoteOption::from(alias);
                quote! {
                    ::kosame::repr::clause::FromItem::Subquery {
                        lateral: #lateral,
                        command: &#command,
                        alias: #alias,
                    }
                }
            }
        }
        .to_tokens(tokens);
    }
}

#[allow(unused)]
pub enum JoinType {
    Inner(keyword::inner, keyword::join),
    Left(keyword::left, keyword::join),
    Right(keyword::right, keyword::join),
    Full(keyword::full, keyword::join),
}

impl JoinType {
    fn peek(input: ParseStream) -> bool {
        macro_rules! check {
            ($kw:expr) => {
                if input.peek($kw) {
                    return true;
                }
            };
        }
        check!(keyword::inner);
        check!(keyword::left);
        check!(keyword::right);
        check!(keyword::full);
        false
    }
}

impl Parse for JoinType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::inner) {
            Ok(Self::Inner(
                input.call(keyword::inner::parse_autocomplete)?,
                input.call(keyword::join::parse_autocomplete)?,
            ))
        } else if lookahead.peek(keyword::left) {
            Ok(Self::Left(
                input.call(keyword::left::parse_autocomplete)?,
                input.call(keyword::join::parse_autocomplete)?,
            ))
        } else if lookahead.peek(keyword::right) {
            Ok(Self::Right(
                input.call(keyword::right::parse_autocomplete)?,
                input.call(keyword::join::parse_autocomplete)?,
            ))
        } else if lookahead.peek(keyword::full) {
            Ok(Self::Full(
                input.call(keyword::full::parse_autocomplete)?,
                input.call(keyword::join::parse_autocomplete)?,
            ))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for JoinType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Inner(..) => quote! { ::kosame::repr::clause::JoinType::Inner },
            Self::Left(..) => quote! { ::kosame::repr::clause::JoinType::Left },
            Self::Right(..) => quote! { ::kosame::repr::clause::JoinType::Right },
            Self::Full(..) => quote! { ::kosame::repr::clause::JoinType::Full },
        }
        .to_tokens(tokens);
    }
}

pub struct On {
    pub on_keyword: keyword::on,
    pub expr: Expr,
}

impl Parse for On {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            on_keyword: input.call(keyword::on::parse_autocomplete)?,
            expr: input.parse()?,
        })
    }
}

pub enum FromCombinator {
    Join {
        join_type: JoinType,
        right: Box<FromItem>,
        on: On,
    },
    NaturalJoin {
        _natural_keyword: keyword::natural,
        join_type: JoinType,
        right: Box<FromItem>,
    },
    CrossJoin {
        _cross_keyword: keyword::cross,
        _join_keyword: keyword::join,
        right: Box<FromItem>,
    },
}

impl FromCombinator {
    pub fn peek(input: ParseStream) -> bool {
        JoinType::peek(input) || input.peek(keyword::natural) || input.peek(keyword::cross)
    }

    fn parse_many(input: ParseStream) -> syn::Result<Vec<FromCombinator>> {
        let mut result = Vec::new();
        while Self::peek(input) {
            result.push(input.parse()?);
        }
        Ok(result)
    }

    #[must_use]
    pub fn right(&self) -> &FromItem {
        match self {
            Self::Join { right, .. } => right,
            Self::NaturalJoin { right, .. } => right,
            Self::CrossJoin { right, .. } => right,
        }
    }
}

pub fn visit_from_combinator<'a>(visit: &mut (impl Visit<'a> + ?Sized), from_combinator: &'a FromCombinator) {
    match from_combinator {
        FromCombinator::Join { right, on, .. } => {
            visit.visit_from_item(right);
            visit.visit_expr(&on.expr);
        }
        FromCombinator::NaturalJoin { right, .. } => {
            visit.visit_from_item(right);
        }
        FromCombinator::CrossJoin { right, .. } => {
            visit.visit_from_item(right);
        }
    }
}

impl Parse for FromCombinator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if JoinType::peek(input) {
            Ok(Self::Join {
                join_type: input.parse()?,
                right: Box::new(input.parse()?),
                on: input.parse()?,
            })
        } else if input.peek(keyword::natural) {
            Ok(Self::NaturalJoin {
                _natural_keyword: input.call(keyword::natural::parse_autocomplete)?,
                join_type: input.parse()?,
                right: Box::new(input.parse()?),
            })
        } else if input.peek(keyword::cross) {
            Ok(Self::CrossJoin {
                _cross_keyword: input.call(keyword::cross::parse_autocomplete)?,
                _join_keyword: input.call(keyword::join::parse_autocomplete)?,
                right: Box::new(input.parse()?),
            })
        } else {
            Err(syn::Error::new(input.span(), "expected join"))
        }
    }
}

impl ToTokens for FromCombinator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Join {
                join_type,
                right,
                on,
                ..
            } => {
                let on = &on.expr;
                quote! {
                    ::kosame::repr::clause::FromCombinator::Join {
                        join_type: #join_type,
                        right: #right,
                        on: #on,
                    }
                }
            }
            Self::NaturalJoin {
                join_type, right, ..
            } => {
                quote! {
                    ::kosame::repr::clause::FromCombinator::NaturalJoin {
                        join_type: #join_type,
                        right: #right,
                    }
                }
            }
            Self::CrossJoin { right, .. } => {
                quote! {
                    ::kosame::repr::clause::FromCombinator::CrossJoin {
                        right: #right,
                    }
                }
            }
        }
        .to_tokens(tokens);
    }
}

pub struct FromIter<'a> {
    chain: &'a FromChain,
    index: i32,
}

impl<'a> Iterator for FromIter<'a> {
    type Item = &'a FromItem;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.index {
            -1 => &self.chain.start,
            _ => self
                .chain
                .combinators
                .get::<usize>(self.index.try_into().unwrap())?
                .right(),
        };
        self.index += 1;
        Some(item)
    }
}

impl<'a> IntoIterator for &'a FromChain {
    type IntoIter = FromIter<'a>;
    type Item = &'a FromItem;

    fn into_iter(self) -> Self::IntoIter {
        FromIter {
            index: -1,
            chain: self,
        }
    }
}
