use std::collections::HashMap;

use syn::{
    Ident, LitInt, LitStr, Path, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{driver::Driver, keyword, proc_macro_error::emit_call_site_error, schema::Table};

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

        for attr in attrs {
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
