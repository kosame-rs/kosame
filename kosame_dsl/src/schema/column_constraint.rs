use std::{fmt::Display, ops::Deref};

use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use crate::{expr::Expr, keyword};

pub struct ColumnConstraints(pub Vec<ColumnConstraint>);

impl ColumnConstraints {
    pub fn not_null(&self) -> Option<&NotNull> {
        self.0.iter().find_map(|c| match c {
            ColumnConstraint::NotNull(inner) => Some(inner),
            _ => None,
        })
    }

    pub fn primary_key(&self) -> Option<&PrimaryKey> {
        self.0.iter().find_map(|c| match c {
            ColumnConstraint::PrimaryKey(inner) => Some(inner),
            _ => None,
        })
    }

    pub fn default(&self) -> Option<&Default> {
        self.0.iter().find_map(|c| match c {
            ColumnConstraint::Default(inner) => Some(inner),
            _ => None,
        })
    }
}

impl Parse for ColumnConstraints {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut constraints = vec![];
        while !input.is_empty() && !input.peek(Token![,]) {
            constraints.push(input.parse()?);
        }
        Ok(Self(constraints))
    }
}

impl Deref for ColumnConstraints {
    type Target = Vec<ColumnConstraint>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused)]
pub enum ColumnConstraint {
    NotNull(NotNull),
    PrimaryKey(PrimaryKey),
    Default(Default),
}

impl Parse for ColumnConstraint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::not) {
            Ok(Self::NotNull(input.parse()?))
        } else if lookahead.peek(keyword::primary) {
            Ok(Self::PrimaryKey(input.parse()?))
        } else if lookahead.peek(keyword::default) {
            Ok(Self::Default(input.parse()?))
        } else {
            keyword::group_column_constraint::error(input);
        }
    }
}

impl Display for ColumnConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotNull(_) => f.write_str("not null")?,
            Self::PrimaryKey(_) => f.write_str("primary key")?,
            Self::Default(_) => f.write_str("default ...")?,
        };
        Ok(())
    }
}

pub struct NotNull {
    pub _not: keyword::not,
    pub _null: keyword::null,
}

impl Parse for NotNull {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _not: input.call(keyword::not::parse_autocomplete)?,
            _null: input.call(keyword::null::parse_autocomplete)?,
        })
    }
}

pub struct PrimaryKey {
    pub _primary: keyword::primary,
    pub _key: keyword::key,
}

impl Parse for PrimaryKey {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _primary: input.call(keyword::primary::parse_autocomplete)?,
            _key: input.call(keyword::key::parse_autocomplete)?,
        })
    }
}

pub struct Default {
    pub _default: keyword::default,
    pub expr: Expr,
}

impl Default {
    pub fn expr(&self) -> &Expr {
        &self.expr
    }
}

impl Parse for Default {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _default: input.call(keyword::default::parse_autocomplete)?,
            expr: input.parse()?,
        })
    }
}
