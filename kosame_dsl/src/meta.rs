use syn::{
    Ident, LitInt, LitStr, Path, Token, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::{driver::Driver, keyword, schema::Table};

pub enum MetaItem {
    Driver(MetaDriver),
    Rename(MetaRename),
    TypeOverride(MetaTypeOverride),
    Pass(MetaPass),
    Table(MetaTable),
}

impl Parse for MetaItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::__pass) {
            return Ok(Self::Pass(input.parse()?));
        }
        if input.peek(keyword::__table) {
            return Ok(Self::Table(input.parse()?));
        }

        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::driver) {
            Ok(Self::Driver(input.parse()?))
        } else if lookahead.peek(keyword::rename) {
            Ok(Self::Rename(input.parse()?))
        } else if lookahead.peek(keyword::ty) {
            Ok(Self::TypeOverride(input.parse()?))
        } else {
            keyword::group_attribute::error(input)
        }
    }
}

pub struct MetaDriver {
    pub path: keyword::driver,
    pub eq_token: Token![=],
    pub value: LitStr,
}

impl Parse for MetaDriver {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            eq_token: input.parse()?,
            value: {
                let value: LitStr = input.parse()?;
                if value.value().parse::<Driver>().is_err() {
                    return Err(syn::Error::new(value.span(), "unknown driver value"));
                }
                value
            },
        })
    }
}

pub struct MetaRename {
    pub path: keyword::rename,
    pub eq_token: Token![=],
    pub value: Ident,
}

impl Parse for MetaRename {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct MetaTypeOverride {
    pub path: keyword::ty,
    pub eq_token: Token![=],
    pub value: Path,
}

impl Parse for MetaTypeOverride {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct MetaPass {
    pub pass_keyword: keyword::__pass,
    pub eq_token: Token![=],
    pub value: LitInt,
}

impl Parse for MetaPass {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pass_keyword: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct MetaTable {
    pub table_keyword: keyword::__table,
    pub paren_token: syn::token::Paren,
    pub path: Path,
    pub eq_token: Token![=],
    pub value: Box<Table>,
}

impl Parse for MetaTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            table_keyword: input.parse()?,
            paren_token: parenthesized!(content in input),
            path: content.parse()?,
            eq_token: content.parse()?,
            value: Box::new(content.parse()?),
        })
    }
}
