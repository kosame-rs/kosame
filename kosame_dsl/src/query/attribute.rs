use syn::{
    Attribute, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::{
    attribute::MetaDriver,
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
};

pub struct QueryAttributes {
    pub inner_attrs: Vec<Attribute>,
    pub outer_attrs: Vec<Attribute>,
    pub driver: Option<MetaDriver>,
}

impl Parse for QueryAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Self {
            inner_attrs: Attribute::parse_inner(input)?,
            outer_attrs: Attribute::parse_outer(input)?,
            driver: None,
        };

        for attr in &result.inner_attrs {
            if attr.path().is_ident("kosame") {
                let list = attr.meta.require_list()?;
                let items = list
                    .parse_args_with(Punctuated::<QueryInnerMeta, Token![,]>::parse_terminated)?;

                for item in items {
                    match item {
                        QueryInnerMeta::Driver(inner) => {
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

        Ok(result)
    }
}

impl PrettyPrint for QueryAttributes {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.inner_attrs.pretty_print(printer);
        self.outer_attrs.pretty_print(printer);
    }
}

pub enum QueryInnerMeta {
    Driver(MetaDriver),
}

impl Parse for QueryInnerMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if MetaDriver::peek(input) {
            return Ok(Self::Driver(input.parse()?));
        }
        Err(syn::Error::new(input.span(), "unexpected attribute meta"))
    }
}
