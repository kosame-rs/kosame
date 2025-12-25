use syn::{
    Attribute, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::{
    attribute::{MetaDriver, MetaRename},
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
};

pub struct TableAttributes {
    pub inner_attrs: Vec<Attribute>,
    pub outer_attrs: Vec<Attribute>,
    pub driver: Option<MetaDriver>,
    pub rename: Option<MetaRename>,
}

impl Parse for TableAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Self {
            inner_attrs: Attribute::parse_inner(input)?,
            outer_attrs: Attribute::parse_outer(input)?,
            driver: None,
            rename: None,
        };

        for attr in &result.inner_attrs {
            if attr.path().is_ident("kosame") {
                let list = attr.meta.require_list()?;
                let items = list
                    .parse_args_with(Punctuated::<TableInnerMeta, Token![,]>::parse_terminated)?;

                for item in items {
                    match item {
                        TableInnerMeta::Driver(inner) => {
                            if result.driver.is_some() {
                                return Err(syn::Error::new(
                                    inner.path.span,
                                    "duplicate attribute meta",
                                ));
                            }
                            result.driver = Some(inner);
                        }
                    }
                }
            } else {
                return Err(syn::Error::new(
                    attr.span(),
                    "only `#![kosame(...)]` attributes allowed in this position",
                ));
            }
        }

        for attr in &result.outer_attrs {
            if attr.path().is_ident("kosame") {
                let list = attr.meta.require_list()?;
                let items = list
                    .parse_args_with(Punctuated::<TableOuterMeta, Token![,]>::parse_terminated)?;

                for item in items {
                    match item {
                        TableOuterMeta::Rename(inner) => {
                            if result.rename.is_some() {
                                return Err(syn::Error::new(
                                    inner.path.span,
                                    "duplicate attribute meta",
                                ));
                            }
                            result.rename = Some(inner);
                        }
                    }
                }
            } else {
                return Err(syn::Error::new(
                    attr.span(),
                    "only `#[kosame(...)]` attributes allowed in this position",
                ));
            }
        }

        Ok(result)
    }
}

impl PrettyPrint for TableAttributes {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.inner_attrs.pretty_print(printer);
        self.outer_attrs.pretty_print(printer);
    }
}

pub enum TableInnerMeta {
    Driver(MetaDriver),
}

impl Parse for TableInnerMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if MetaDriver::peek(input) {
            return Ok(Self::Driver(input.parse()?));
        }
        Err(syn::Error::new(input.span(), "unexpected attribute meta"))
    }
}

pub enum TableOuterMeta {
    Rename(MetaRename),
}

impl Parse for TableOuterMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if MetaRename::peek(input) {
            return Ok(Self::Rename(input.parse()?));
        }
        Err(syn::Error::new(input.span(), "unexpected attribute meta"))
    }
}
