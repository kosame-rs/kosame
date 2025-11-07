use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Path,
    parse::{Parse, ParseStream},
    parse_quote,
};

use crate::{part::TablePath, path_ext::PathExt, scopes::ScopeId};

#[derive(Clone)]
pub struct DataType {
    pub name: Ident,
}

impl Parse for DataType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
        })
    }
}

impl ToTokens for DataType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.name.to_string().as_str() {
            // Built-in / Standard library types
            "bool" => quote! { bool },
            "char" => quote! { i8 },
            "smallint" | "smallserial" => quote! { i16 },
            "int" | "serial" => quote! { i32 },
            "oid" => quote! { u32 },
            "bigint" | "bigserial" => quote! { i64 },
            "real" => quote! { f32 },
            "double precision" => quote! { f64 },
            "varchar" | "text" | "citext" | "name" | "unknown" => quote! { ::std::string::String },
            "bytea" => quote! { ::std::vec::Vec<u8> },
            "hstore" => quote! { ::std::collections::HashMap<::std::string::String, ::std::option::Option<::std::string::String>> },
            "timestamp" | "timestamptz" | "timestamp with time zone" => quote! { ::std::time::SystemTime },
            "inet" => quote! { ::std::net::IpAddr },

            // Crates
            "uuid" => quote! { ::uuid::Uuid },
            "json" | "jsonb" => quote! { ::serde_json::Value },
            _ => {
                abort!(
                    self.name.span(),
                    "cannot determine rust type for unrecognized database type {}, requires type override",
                    self.name,
                );
            }
        }
        .to_tokens(tokens);
    }
}
