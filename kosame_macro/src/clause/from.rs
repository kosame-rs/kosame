use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Path, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::{
    command::Command, expr::Expr, keyword, part::TableAlias, quote_option::QuoteOption,
    scopes::ScopeId, visitor::Visitor,
};

pub struct From {
    pub _from: keyword::from,
    pub chain: FromChain,
}

impl From {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.chain.accept(visitor);
    }

    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::from)
    }
}

impl Parse for From {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _from: input.parse()?,
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
    start: FromItem,
    combinators: Vec<FromCombinator>,
}

impl FromChain {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.start.accept(visitor);
        for combinator in &self.combinators {
            combinator.accept(visitor);
        }
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
        table: Path,
        alias: Option<TableAlias>,
    },
    Subquery {
        lateral_keyword: Option<keyword::lateral>,
        paren_token: syn::token::Paren,
        command: Box<Command>,
        alias: Option<TableAlias>,
    },
}

impl FromItem {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        match self {
            Self::Table { table, .. } => {
                visitor.visit_table_ref(table);
            }
            Self::Subquery { command, .. } => {
                command.accept(visitor);
            }
        }
    }

    pub fn name(&self) -> Option<&Ident> {
        match self {
            Self::Table { table, alias, .. } => Some(
                alias
                    .as_ref()
                    .map(|alias| &alias.name)
                    .unwrap_or_else(|| &table.segments.last().expect("path cannot be empty").ident),
            ),
            Self::Subquery { alias, .. } => alias.as_ref().map(|alias| &alias.name),
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
                alias: input.call(TableAlias::parse_optional)?,
            })
        } else if lookahead.peek(Ident) {
            Ok(Self::Table {
                table: input.parse()?,
                alias: input.call(TableAlias::parse_optional)?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for FromItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Table { table, alias, .. } => {
                let table = alias
                    .as_ref()
                    .map(|alias| &alias.name)
                    .unwrap_or(&table.segments.last().expect("paths cannot be empty").ident);
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
                lateral_keyword: _lateral_keyword,
                command,
                alias,
                ..
            } => {
                let lateral = _lateral_keyword.is_some();
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
    pub _on_keyword: keyword::on,
    pub expr: Expr,
}

impl Parse for On {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _on_keyword: input.call(keyword::on::parse_autocomplete)?,
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
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        match self {
            Self::Join { right, on, .. } => {
                right.accept(visitor);
                on.expr.accept(visitor);
            }
            Self::NaturalJoin { right, .. } => {
                right.accept(visitor);
            }
            Self::CrossJoin { right, .. } => {
                right.accept(visitor);
            }
        }
    }

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

    pub fn right(&self) -> &FromItem {
        match self {
            Self::Join { right, .. } => &right,
            Self::NaturalJoin { right, .. } => &right,
            Self::CrossJoin { right, .. } => &right,
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
            _ => self.chain.combinators.get(self.index as usize)?.right(),
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
