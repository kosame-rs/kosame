use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Path, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::{
    clause::WithItem,
    command::Command,
    expr::Expr,
    keyword,
    parent_map::{Id, ParentMap},
    part::TableAlias,
    path_ext::PathExt,
    quote_option::QuoteOption,
    visitor::Visitor,
};

pub struct From {
    pub _from: keyword::from,
    pub item: FromItem,
}

impl From {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::from)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.item.accept(visitor);
    }
}

impl Parse for From {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _from: input.parse()?,
            item: input.parse()?,
        })
    }
}

impl ToTokens for From {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item = &self.item;
        quote! { ::kosame::repr::clause::From::new(#item) }.to_tokens(tokens);
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
    pub _on_token: keyword::on,
    pub expr: Expr,
}

impl Parse for On {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _on_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

pub enum FromItem {
    Table {
        id: Id,
        table: Path,
        alias: Option<TableAlias>,
    },
    Subquery {
        id: Id,
        lateral_keyword: Option<keyword::lateral>,
        paren_token: syn::token::Paren,
        command: Box<Command>,
        alias: Option<TableAlias>,
    },
    Join {
        id: Id,
        left: Box<FromItem>,
        join_type: JoinType,
        right: Box<FromItem>,
        on: On,
    },
    NaturalJoin {
        id: Id,
        _natural_keyword: keyword::natural,
        left: Box<FromItem>,
        join_type: JoinType,
        right: Box<FromItem>,
    },
    CrossJoin {
        id: Id,
        left: Box<FromItem>,
        _cross_keyword: keyword::cross,
        _join_keyword: keyword::join,
        right: Box<FromItem>,
    },
}

impl FromItem {
    fn parse_prefix(input: ParseStream) -> syn::Result<Self> {
        let lateral_keyword = input
            .peek(keyword::lateral)
            .then(|| input.parse())
            .transpose()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Paren) {
            let content;
            Ok(Self::Subquery {
                id: Id::new(),
                lateral_keyword,
                paren_token: parenthesized!(content in input),
                command: content.parse()?,
                alias: input.call(TableAlias::parse_optional)?,
            })
        } else if lookahead.peek(Ident) {
            Ok(Self::Table {
                id: Id::new(),
                table: input.parse()?,
                alias: input.call(TableAlias::parse_optional)?,
            })
        } else {
            Err(lookahead.error())
        }
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_parent_node(self.into());
        match self {
            Self::Table { table, .. } => {
                visitor.visit_table_ref(table);
            }
            Self::Subquery { command, .. } => {
                command.accept(visitor);
            }
            Self::Join {
                left, right, on, ..
            } => {
                on.expr.accept(visitor);
                left.accept(visitor);
                right.accept(visitor);
            }
            Self::NaturalJoin { left, right, .. } => {
                left.accept(visitor);
                right.accept(visitor);
            }
            Self::CrossJoin { left, right, .. } => {
                left.accept(visitor);
                right.accept(visitor);
            }
        }
        visitor.end_parent_node();
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
            Self::Join { .. } => None,
            Self::NaturalJoin { .. } => None,
            Self::CrossJoin { .. } => None,
        }
    }

    pub fn with_item<'a>(&'a self, parent_map: &ParentMap<'a>) -> Option<&'a WithItem> {
        match self {
            Self::Table { table, .. } => {
                let table = table.get_ident()?;
                let mut command = parent_map.seek_parent::<_, Command>(self)?;
                loop {
                    if let Some(with) = &command.with {
                        for item in with.items.iter() {
                            if item.alias.name == *table {
                                return Some(item);
                            }
                        }
                    }
                    command = parent_map.seek_parent::<_, Command>(command)?;
                }
            }
            _ => None,
        }
    }

    pub fn id(&self) -> &Id {
        match self {
            Self::Table { id, .. } => id,
            Self::Subquery { id, .. } => id,
            Self::Join { id, .. } => id,
            Self::NaturalJoin { id, .. } => id,
            Self::CrossJoin { id, .. } => id,
        }
    }

    pub fn left(&self) -> Option<&FromItem> {
        match self {
            Self::Join { left, .. } => Some(left),
            Self::NaturalJoin { left, .. } => Some(left),
            Self::CrossJoin { left, .. } => Some(left),
            _ => None,
        }
    }

    pub fn right(&self) -> Option<&FromItem> {
        match self {
            Self::Join { right, .. } => Some(right),
            Self::NaturalJoin { right, .. } => Some(right),
            Self::CrossJoin { right, .. } => Some(right),
            _ => None,
        }
    }
}

impl Parse for FromItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut item = Self::parse_prefix(input)?;
        loop {
            if JoinType::peek(input) {
                item = FromItem::Join {
                    id: Id::new(),
                    left: Box::new(item),
                    join_type: input.parse()?,
                    right: Box::new(Self::parse_prefix(input)?),
                    on: input.parse()?,
                };
                continue;
            }
            if input.peek(keyword::natural) {
                item = FromItem::NaturalJoin {
                    id: Id::new(),
                    _natural_keyword: input.call(keyword::natural::parse_autocomplete)?,
                    left: Box::new(item),
                    join_type: input.parse()?,
                    right: Box::new(Self::parse_prefix(input)?),
                };
            }
            if input.peek(keyword::cross) {
                item = FromItem::CrossJoin {
                    id: Id::new(),
                    left: Box::new(item),
                    _cross_keyword: input.call(keyword::cross::parse_autocomplete)?,
                    _join_keyword: input.call(keyword::join::parse_autocomplete)?,
                    right: Box::new(Self::parse_prefix(input)?),
                };
            }
            break;
        }
        Ok(item)
    }
}

impl ToTokens for FromItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Table { table, alias, .. } => {
                let alias = QuoteOption::from(alias);
                ParentMap::with(|parent_map| match self.with_item(parent_map) {
                    Some(with_item) => {
                        let name_string = with_item.alias.name.to_string();
                        quote! {
                            ::kosame::repr::clause::FromItem::Table {
                                table: #name_string,
                                alias: #alias,
                            }
                        }
                    }
                    None => {
                        let table = table.to_call_site(1);
                        quote! {
                            ::kosame::repr::clause::FromItem::Table {
                                table: #table::TABLE_NAME,
                                alias: #alias,
                            }
                        }
                    }
                })
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
            Self::Join {
                left,
                join_type,
                right,
                on,
                ..
            } => {
                let on = &on.expr;
                quote! {
                    ::kosame::repr::clause::FromItem::Join {
                        left: &#left,
                        join_type: #join_type,
                        right: &#right,
                        on: #on,
                    }
                }
            }
            Self::NaturalJoin {
                left,
                join_type,
                right,
                ..
            } => {
                quote! {
                    ::kosame::repr::clause::FromItem::NaturalJoin {
                        left: &#left,
                        join_type: #join_type,
                        right: &#right,
                    }
                }
            }
            Self::CrossJoin { left, right, .. } => {
                quote! {
                    ::kosame::repr::clause::FromItem::CrossJoin {
                        left: &#left,
                        right: &#right,
                    }
                }
            }
        }
        .to_tokens(tokens);
    }
}

pub struct FromItemIter<'a> {
    stack: Vec<&'a FromItem>,
}

impl<'a> std::convert::From<&'a FromItem> for FromItemIter<'a> {
    fn from(value: &'a FromItem) -> Self {
        let mut stack = Vec::<&'a FromItem>::new();
        stack.push(value);
        while let Some(left) = stack.last().unwrap().left() {
            stack.push(left);
        }
        Self { stack }
    }
}

impl<'a> Iterator for FromItemIter<'a> {
    type Item = &'a FromItem;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.stack.pop()?;
        if let Some(next) = result.right() {
            self.stack.push(next);
            while let Some(left) = self.stack.last().unwrap().left() {
                self.stack.push(left);
            }
        }
        Some(result)
    }
}

impl<'a> IntoIterator for &'a FromItem {
    type IntoIter = FromItemIter<'a>;
    type Item = &'a FromItem;

    fn into_iter(self) -> Self::IntoIter {
        FromItemIter::from(self)
    }
}
