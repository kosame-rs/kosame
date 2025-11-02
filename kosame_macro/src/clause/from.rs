use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Path, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::{
    command::Command, expr::Expr, keyword, part::TableAlias, path_ext::PathExt,
    quote_option::QuoteOption, visitor::Visitor,
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
        table: Path,
        alias: Option<TableAlias>,
    },
    Subquery {
        lateral_keyword: Option<keyword::lateral>,
        _paren_token: syn::token::Paren,
        command: Box<Command>,
        alias: Option<TableAlias>,
    },
    Join {
        left: Box<FromItem>,
        join_type: JoinType,
        right: Box<FromItem>,
        on: On,
    },
    NaturalJoin {
        _natural_keyword: keyword::natural,
        left: Box<FromItem>,
        join_type: JoinType,
        right: Box<FromItem>,
    },
    CrossJoin {
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
                lateral_keyword,
                _paren_token: parenthesized!(content in input),
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

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_from_item(self);
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
        visitor.end_from_item();
    }
}

impl Parse for FromItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut item = Self::parse_prefix(input)?;
        loop {
            if JoinType::peek(input) {
                item = FromItem::Join {
                    left: Box::new(item),
                    join_type: input.parse()?,
                    right: Box::new(Self::parse_prefix(input)?),
                    on: input.parse()?,
                };
                continue;
            }
            if input.peek(keyword::natural) {
                item = FromItem::NaturalJoin {
                    _natural_keyword: input.call(keyword::natural::parse_autocomplete)?,
                    left: Box::new(item),
                    join_type: input.parse()?,
                    right: Box::new(Self::parse_prefix(input)?),
                };
            }
            if input.peek(keyword::cross) {
                item = FromItem::CrossJoin {
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
            Self::Table { table, alias } => {
                let table = table.to_call_site(1);
                let alias = QuoteOption(alias.as_ref());
                quote! {
                    ::kosame::repr::clause::FromItem::Table {
                        table: &#table::TABLE,
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
                let alias = QuoteOption(alias.as_ref());
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
