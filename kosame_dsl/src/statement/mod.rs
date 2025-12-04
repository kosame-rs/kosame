use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Ident, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::{
    attribute::{CustomMeta, MetaLocation},
    bind_params::{BindParams, BindParamsClosure},
    command::Command,
    correlations::{CorrelationId, Correlations},
    parse_option::ParseOption,
    part::Alias,
    pretty::{BreakMode, Delim, PrettyPrint, Printer},
    row::Row,
    scopes::{ScopeId, Scopes},
    visit::Visit,
};

pub struct Statement {
    pub inner_attrs: Vec<Attribute>,
    pub paren_token: Option<syn::token::Paren>,
    pub command: Command,
    pub alias: Option<Alias>,
}

impl Statement {
    #[must_use]
    pub fn _custom_meta(&self) -> CustomMeta {
        CustomMeta::parse_attrs(&self.inner_attrs, MetaLocation::StatementInner)
            .expect("custom meta should be checked during parsing")
    }
}

pub fn visit_statement<'a>(visit: &mut (impl Visit<'a> + ?Sized), statement: &'a Statement) {
    visit.visit_command(&statement.command);
}

impl Parse for Statement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        CorrelationId::reset();
        ScopeId::reset();

        let inner_attrs = {
            let attrs = input.call(Attribute::parse_inner)?;
            CustomMeta::parse_attrs(&attrs, MetaLocation::StatementInner)?;
            attrs
        };
        if input.peek(syn::token::Paren) {
            let content;
            Ok(Self {
                inner_attrs,
                paren_token: Some(parenthesized!(content in input)),
                command: content.parse()?,
                alias: input.call(Alias::parse_option)?,
            })
        } else {
            Ok(Self {
                inner_attrs,
                paren_token: None,
                command: input.parse()?,
                alias: input.call(Alias::parse_option)?,
            })
        }
    }
}

impl ToTokens for Statement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Prepass to get table schemas
        // let custom_meta = self.custom_meta();

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

        let bind_params = BindParams::from(self);
        let correlations = Correlations::from(&self.command);
        let scopes = Scopes::from(&self.command);

        let command = &self.command;
        let fields = command.fields();
        let row = if let Some(fields) = fields {
            let row = Row::new(
                command.attrs.clone(),
                Ident::new("Row", Span::call_site()),
                fields
                    .iter()
                    .filter_map(|field| {
                        field.to_row_field(&correlations, &scopes, command.correlation_id)
                    })
                    .collect(),
            );
            quote! { #row }
        } else {
            quote! { pub enum Row {} }
        };

        let lifetime = (!bind_params.is_empty()).then_some(quote! { <'a> });

        let module_tokens = quote! {
            pub mod #module_name {
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
                #correlations
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

impl PrettyPrint for Statement {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.inner_attrs.pretty_print(printer);
        match self.paren_token {
            Some(paren_token) => {
                paren_token.pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                    self.command.pretty_print(printer);
                });
            }
            None => self.command.pretty_print(printer),
        }
        self.alias.pretty_print(printer);
    }
}
