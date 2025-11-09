mod attribute;
mod bind_params;
mod clause;
mod command;
mod correlations;
mod data_type;
mod driver;
mod expr;
mod inferred_type;
mod keyword;
mod part;
mod path_ext;
mod query;
mod quote_option;
mod row;
mod schema;
mod scopes;
mod statement;
mod unique_macro;
mod visitor;

use proc_macro_error::proc_macro_error;
use quote::{ToTokens, quote};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_error]
#[proc_macro]
pub fn table(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as schema::Table);
    quote! { #input }.into()
}

#[proc_macro_error]
#[proc_macro]
pub fn statement(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as statement::Statement);
    quote! { #input }.into()
}

#[proc_macro_error]
#[proc_macro]
pub fn query(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as query::Query);
    quote! { #input }.into()
}

#[proc_macro_error]
#[proc_macro_derive(Row, attributes(star))]
pub fn derive_row(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let name = &input.ident;
    let syn::Data::Struct(data) = input.data else {
        proc_macro_error::abort_call_site!("#[derive(Row)] can only be used on structs.");
    };

    let mut tokens = proc_macro2::TokenStream::new();

    #[cfg(any(feature = "postgres", feature = "tokio-postgres"))]
    {
        let fields = data.fields.iter().enumerate().map(|(index, field)| {
            let name = &field.ident;
            quote! {
                #name: row.get(#index)
            }
        });

        quote! {
            impl From<&::kosame::driver::postgres_types::Row> for #name {
                fn from(row: &::kosame::driver::postgres_types::Row) -> Self {
                    Self {
                        #(#fields),*
                    }
                }
            }
        }
        .to_tokens(&mut tokens);
    }

    #[cfg(any(feature = "postgres", feature = "tokio-postgres"))]
    {
        let field_count = data.fields.len() as i32;
        let fields = data.fields.iter().map(|field| {
            let name = &field.ident;
            quote! {
                #name: ::kosame::driver::postgres_types::record_field_from_sql(&raw, &mut offset)?
            }
        });

        quote! {
            impl<'a> ::kosame::driver::postgres_types::FromSql<'a> for #name {
                fn accepts(ty: &::kosame::driver::postgres_types::Type) -> bool {
                    ty.name() == "record"
                }

                fn from_sql(
                    ty: &::kosame::driver::postgres_types::Type,
                    raw: &[u8],
                ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                    let column_count = ::kosame::driver::postgres_types::int4_from_sql(&raw[..4])?;
                    assert_eq!(column_count, #field_count);

                    let mut offset = 4;

                    Ok(Self {
                        #(#fields),*
                    })
                }
            }
        }
        .to_tokens(&mut tokens);
    }

    tokens.into()
}
