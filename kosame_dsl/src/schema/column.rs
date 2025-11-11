use crate::{
    attribute::{CustomMeta, MetaLocation},
    data_type::DataType,
    path_ext::PathExt,
    quote_option::QuoteOption,
};

use super::column_constraint::ColumnConstraints;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Ident,
    parse::{Parse, ParseStream},
};

pub struct Column {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub data_type: DataType,
    pub constraints: ColumnConstraints,
}

impl Column {
    pub fn rust_name(&self) -> Ident {
        let meta = CustomMeta::parse_attrs(&self.attrs, MetaLocation::Column)
            .expect("custom meta should be checked earlier");
        match meta.rename {
            Some(rename) => rename.value,
            None => Ident::new(
                &self.name.to_string().to_case(Case::Snake),
                self.name.span(),
            ),
        }
    }
}

impl Parse for Column {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        CustomMeta::parse_attrs(&attrs, MetaLocation::Column)?;
        let name = input.parse()?;
        let data_type = input.parse()?;

        Ok(Self {
            attrs,
            name,
            data_type,
            constraints: input.parse()?,
        })
    }
}

impl ToTokens for Column {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let meta = CustomMeta::parse_attrs(&self.attrs, MetaLocation::Column)
            .expect("custom meta should be checked earlier");

        let name = self.name.to_string();
        let rust_name = self.rust_name();

        let data_type = &self.data_type;
        let data_type_string = data_type.name.to_string();
        let rust_type_not_null = match meta.type_override {
            Some(type_override) => type_override.value.to_call_site(3).to_token_stream(),
            None => quote! { #data_type },
        };
        let rust_type_nullable = quote! { Option<#data_type> };
        let rust_type_auto =
            if self.constraints.not_null().is_none() && self.constraints.primary_key().is_none() {
                rust_type_nullable.clone()
            } else {
                rust_type_not_null.clone()
            };

        let not_null = self.constraints.not_null().is_some();
        let primary_key = self.constraints.primary_key().is_some();
        let default = QuoteOption(self.constraints.default().map(|default| {
            let expr = default.expr();
            quote! { &#expr }
        }));

        quote! {
            pub mod #rust_name {
                pub const COLUMN_NAME: &str = #name;
                pub const COLUMN: ::kosame::repr::schema::Column<'_> = ::kosame::repr::schema::Column {
                    name: #name,
                    data_type: #data_type_string,
                    not_null: #not_null,
                    primary_key: #primary_key,
                    default: #default,
                };
                pub type TypeNotNull = #rust_type_not_null;
                pub type TypeNullable = #rust_type_nullable;
                pub type Type = #rust_type_auto;
            }
        }
        .to_tokens(tokens);
    }
}
