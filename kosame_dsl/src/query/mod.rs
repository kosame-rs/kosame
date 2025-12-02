mod field;
mod node;
mod node_path;
mod star;

pub use field::*;
pub use node::visit_node;
pub use node::*;
pub use node_path::*;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Ident,
    parse::{Parse, ParseStream},
};

use crate::{
    attribute::{CustomMeta, MetaLocation},
    bind_params::{BindParams, BindParamsClosure},
    correlations::{CorrelationId, Correlations},
    parse_option::ParseOption,
    part::{Alias, TablePath},
    path_ext::PathExt,
    pretty::{PrettyPrint, Printer},
    scopes::{ScopeId, Scopes},
};

pub struct Query {
    pub inner_attrs: Vec<Attribute>,
    pub outer_attrs: Vec<Attribute>,
    pub table: TablePath,
    pub body: Node,
    pub alias: Option<Alias>,
}

impl Parse for Query {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        ScopeId::reset();
        CorrelationId::reset();
        Ok(Self {
            inner_attrs: {
                let attrs = Attribute::parse_inner(input)?;
                CustomMeta::parse_attrs(&attrs, MetaLocation::QueryInner)?;
                attrs
            },
            outer_attrs: {
                let attrs = Attribute::parse_outer(input)?;
                CustomMeta::parse_attrs(&attrs, MetaLocation::QueryOuter)?;
                attrs
            },
            table: input.parse()?,
            body: input.parse()?,
            alias: input.call(Alias::parse_option)?,
        })
    }
}

impl ToTokens for Query {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_name = match &self.alias {
            Some(alias) => &alias.ident,
            None => &Ident::new("internal", Span::call_site()),
        };

        let bind_params = BindParams::from(self);
        let correlations = Correlations::from(self);
        let scopes = Scopes::from(self);

        let node_tokens = {
            let mut tokens = proc_macro2::TokenStream::new();
            self.body
                .to_row_tokens(&mut tokens, self, &QueryNodePath::new());
            tokens
        };

        let query_node = {
            let mut tokens = TokenStream::new();
            self.body
                .to_query_node_tokens(&mut tokens, self, &QueryNodePath::new());
            tokens
        };

        let lifetime = (!bind_params.is_empty()).then_some(quote! { <'a> });

        let module_tokens = quote! {
            pub mod #module_name {
                #correlations

                #node_tokens

                pub struct Query #lifetime {
                    params: Params #lifetime,
                }

                impl #lifetime Query #lifetime {
                    pub fn new(params: Params #lifetime) -> Self { Self { params } }
                }

                impl #lifetime ::kosame::query::Query for Query #lifetime {
                    type Params = Params #lifetime;
                    type Row = Row;

                    const REPR: ::kosame::query::Node<'static> = #query_node;

                    fn params(&self) -> &Self::Params {
                        &self.params
                    }
                }

                #bind_params

                #scopes
            }
        };

        if self.alias.is_some() {
            module_tokens.to_tokens(tokens);
        } else {
            let bind_params_closure = BindParamsClosure::new(module_name, &bind_params);
            quote! {
                {
                    #bind_params_closure
                    #module_tokens
                    #module_name::Query::new(closure)
                }
            }
            .to_tokens(tokens);
        }
    }
}

impl PrettyPrint for Query {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.inner_attrs.pretty_print(printer);
        self.outer_attrs.pretty_print(printer);
        self.table.pretty_print(printer);
        self.body.pretty_print(printer);
        self.alias.pretty_print(printer);
    }
}
