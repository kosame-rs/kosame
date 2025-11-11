use std::collections::HashMap;

use proc_macro_error::emit_call_site_error;
use syn::{
    Ident, LitInt, LitStr, Path, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{driver::Driver, keyword, schema::Table};

#[derive(Default)]
pub struct CustomMeta {
    pub driver: Option<MetaDriver>,
    pub rename: Option<MetaRename>,
    pub type_override: Option<MetaTypeOverride>,

    pub pass: u32,
    pub tables: HashMap<Path, Table>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MetaLocation {
    TableInner,
    TableOuter,
    Column,
    QueryInner,
    QueryOuter,
    StatementInner,
}

impl CustomMeta {
    pub fn parse_attrs(attrs: &[syn::Attribute], location: MetaLocation) -> syn::Result<Self> {
        let mut result = Self::default();

        for attr in attrs.iter() {
            if attr.path().is_ident("kosame") {
                let list = attr.meta.require_list()?;
                let items =
                    list.parse_args_with(Punctuated::<MetaItem, Token![,]>::parse_terminated)?;

                for item in items {
                    macro_rules! fill_or_error {
                        ($name:ident, $str:literal, $location_allowed:expr) => {{
                            if result.$name.is_some() {
                                return Err(syn::Error::new(
                                    $name.path.span,
                                    format!("duplicate use of meta argument `{}`", $str),
                                ));
                            }
                            if !($location_allowed) {
                                return Err(syn::Error::new(
                                    $name.path.span,
                                    format!(
                                        "meta argument `{}` not allowed in this location",
                                        $str
                                    ),
                                ));
                            }
                            result.$name = Some($name);
                        }};
                    }

                    match item {
                        MetaItem::Driver(driver) => {
                            fill_or_error!(
                                driver,
                                "driver",
                                location == MetaLocation::TableInner
                                    || location == MetaLocation::QueryInner
                                    || location == MetaLocation::StatementInner
                            );
                        }
                        MetaItem::Rename(rename) => {
                            fill_or_error!(rename, "rename", location == MetaLocation::Column);
                        }
                        MetaItem::TypeOverride(type_override) => {
                            fill_or_error!(type_override, "ty", location == MetaLocation::Column);
                        }
                        MetaItem::Pass(pass) => {
                            result.pass = pass.value.base10_parse()?;
                        }
                        MetaItem::Table(table) => {
                            result.tables.insert(table.path, *table.value);
                        }
                    }
                }
            }
        }

        match location {
            MetaLocation::TableInner | MetaLocation::QueryInner | MetaLocation::StatementInner
                if result.driver.is_none() =>
            {
                emit_call_site_error!(
                    "missing `driver` attribute, e.g. #[kosame(driver = \"tokio-postgres\")]"
                );
            }
            _ => {}
        }

        Ok(result)
    }
}

enum MetaItem {
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
            keyword::group_attribute::error(input);
        }
    }
}

pub struct MetaDriver {
    pub path: keyword::driver,
    pub _eq_token: Token![=],
    pub _value: LitStr,
}

impl Parse for MetaDriver {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            _eq_token: input.parse()?,
            _value: {
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
    pub _eq_token: Token![=],
    pub value: Ident,
}

impl Parse for MetaRename {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct MetaTypeOverride {
    pub path: keyword::ty,
    pub _eq_token: Token![=],
    pub value: Path,
}

impl Parse for MetaTypeOverride {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct MetaPass {
    pub _pass_keyword: keyword::__pass,
    pub _eq_token: Token![=],
    pub value: LitInt,
}

impl Parse for MetaPass {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _pass_keyword: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct MetaTable {
    pub _table_keyword: keyword::__table,
    pub _paren_token: syn::token::Paren,
    pub path: Path,
    pub _eq_token: Token![=],
    pub value: Box<Table>,
}

impl Parse for MetaTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _table_keyword: input.parse()?,
            _paren_token: parenthesized!(content in input),
            path: content.parse()?,
            _eq_token: content.parse()?,
            value: Box::new(content.parse()?),
        })
    }
}
