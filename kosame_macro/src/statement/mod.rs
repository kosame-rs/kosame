use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Ident, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::{
    attribute::{CustomMeta, MetaLocation},
    bind_params::{BindParamsBuilder, BindParamsClosure},
    command::Command,
    correlations::{CorrelationId, Correlations},
    part::Alias,
    row::Row,
    scopes::{ScopeId, Scopes},
    visitor::Visitor,
};

pub struct Statement {
    token_stream: TokenStream,

    pub inner_attrs: Vec<Attribute>,
    pub _paren_token: Option<syn::token::Paren>,
    pub command: Command,
    pub alias: Option<Alias>,
}

impl Statement {
    pub fn custom_meta(&self) -> CustomMeta {
        CustomMeta::parse_attrs(&self.inner_attrs, MetaLocation::StatementInner)
            .expect("custom meta should be checked during parsing")
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.command.accept(visitor);
    }
}

impl Parse for Statement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        CorrelationId::reset();
        ScopeId::reset();

        let token_stream = input.fork().parse()?;
        let inner_attrs = {
            let attrs = input.call(Attribute::parse_inner)?;
            CustomMeta::parse_attrs(&attrs, MetaLocation::StatementInner)?;
            attrs
        };
        if input.peek(syn::token::Paren) {
            let content;
            Ok(Self {
                token_stream,
                inner_attrs,
                _paren_token: Some(parenthesized!(content in input)),
                command: content.parse()?,
                alias: input.call(Alias::parse_optional)?,
            })
        } else {
            Ok(Self {
                token_stream,
                inner_attrs,
                _paren_token: None,
                command: input.parse()?,
                alias: input.call(Alias::parse_optional)?,
            })
        }
    }
}

impl ToTokens for Statement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Prepass to get table schemas
        let custom_meta = self.custom_meta();

        // fn collect_table_refs<'a>(statement: &'a Statement) -> HashSet<&'a Path> {
        //     let mut table_refs = HashSet::<&'a Path>::new();
        //     statement.accept(&mut |path: &'a Path| {
        //         table_refs.insert(path);
        //     });
        //     table_refs
        // }
        // let table_refs = collect_table_refs(self);

        // if custom_meta.pass == 0 {
        //     let mut table_refs = TableRefs::new();
        //     self.accept(&mut table_refs);
        //     let table_refs = table_refs.build();
        //     if !table_refs.is_empty() {
        //         let token_stream = self.token_stream.clone();
        //         let mut result = quote! {
        //             (::kosame::statement!) {
        //                 #![kosame(__pass = 1)]
        //                 #token_stream
        //             }
        //         };
        //
        //         for (index, table_ref) in table_refs.iter().enumerate() {
        //             if index == table_refs.len() - 1 {
        //                 result = quote! {
        //                     #table_ref::inject! {
        //                         #result
        //                         (#table_ref)
        //                     }
        //                 }
        //             } else {
        //                 result = quote! {
        //                     (#table_ref::inject!) {
        //                         #result
        //                         (#table_ref)
        //                     }
        //                 }
        //             }
        //         }
        //
        //         result.to_tokens(tokens);
        //         return;
        //     }
        // }

        let module_name = match &self.alias {
            Some(alias) => &alias.ident,
            None => &Ident::new("internal", Span::call_site()),
        };

        let bind_params = {
            let mut builder = BindParamsBuilder::new();
            self.command.accept(&mut builder);
            builder.build()
        };
        let correlations = Correlations::from(&self.command);
        let scopes = Scopes::from(&self.command);

        let command = &self.command;
        let fields = command.fields();
        let row = match fields {
            Some(fields) => {
                let row = Row::new(
                    command.attrs.to_owned(),
                    Ident::new("Row", Span::call_site()),
                    fields
                        .iter()
                        .filter_map(|field| {
                            field.to_row_field(&correlations, &scopes, command.correlation_id)
                        })
                        .collect(),
                );
                quote! { #row }
            }
            None => quote! { pub enum Row {} },
        };

        let lifetime = (!bind_params.is_empty()).then_some(quote! { <'a> });

        let module_tokens = quote! {
            pub mod #module_name {
                #correlations

                pub struct Statement #lifetime {
                    params: Params #lifetime,
                }

                impl #lifetime Statement #lifetime {
                    pub fn new(params: Params #lifetime) -> Self { Self { params } }
                }

                impl #lifetime ::kosame::statement::Statement for Statement #lifetime {
                    type Params = Params #lifetime;
                    type Row = Row;

                    const REPR: ::kosame::repr::command::Command<'static> = #command;

                    fn params(&self) -> &Self::Params {
                        &self.params
                    }
                }

                #row

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
                    #module_name::Statement::new(closure)
                }
            }
            .to_tokens(tokens);
        }
    }
}
