use syn::{
    Attribute, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::{
    attribute::{MetaRename, MetaTypeOverride},
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
};

pub struct ColumnAttributes {
    pub attrs: Vec<Attribute>,
    pub rename: Option<MetaRename>,
    pub type_override: Option<MetaTypeOverride>,
}

impl Parse for ColumnAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Self {
            attrs: Attribute::parse_outer(input)?,
            rename: None,
            type_override: None,
        };

        for attr in &result.attrs {
            if attr.path().is_ident("kosame") {
                let list = attr.meta.require_list()?;
                let items =
                    list.parse_args_with(Punctuated::<ColumnMeta, Token![,]>::parse_terminated)?;

                for item in items {
                    match item {
                        ColumnMeta::Rename(inner) => {
                            if result.rename.is_some() {
                                return Err(syn::Error::new(
                                    inner.path.span,
                                    "duplicate attribute meta",
                                ));
                            }
                            result.rename = Some(inner);
                        }
                        ColumnMeta::TypeOverride(inner) => {
                            if result.type_override.is_some() {
                                return Err(syn::Error::new(
                                    inner.path.span,
                                    "duplicate attribute meta",
                                ));
                            }
                            result.type_override = Some(inner);
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

impl PrettyPrint for ColumnAttributes {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.attrs.pretty_print(printer);
    }
}

pub enum ColumnMeta {
    Rename(MetaRename),
    TypeOverride(MetaTypeOverride),
}

impl Parse for ColumnMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if MetaRename::peek(input) {
            return Ok(Self::Rename(input.parse()?));
        }
        if MetaTypeOverride::peek(input) {
            return Ok(Self::TypeOverride(input.parse()?));
        }
        Err(syn::Error::new(input.span(), "unexpected attribute meta"))
    }
}
