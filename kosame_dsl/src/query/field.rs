use super::Node;
use crate::{
    expr::Expr,
    parse_option::ParseOption,
    part::{Alias, TypeOverride},
    path_ext::PathExt,
    pretty::{BreakMode, PrettyPrint, Printer},
    query::node_path::QueryNodePath,
    row::RowField,
};
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Ident, Path, Token,
    parse::{Parse, ParseStream},
    parse_quote,
};

pub enum Field {
    Column {
        attrs: Vec<Attribute>,
        name: Ident,
        alias: Option<Alias>,
        type_override: Option<TypeOverride>,
    },
    Relation {
        attrs: Vec<Attribute>,
        name: Ident,
        node: Box<Node>,
        alias: Option<Alias>,
    },
    Expr {
        attrs: Vec<Attribute>,
        expr: Expr,
        alias: Alias,
        type_override: TypeOverride,
    },
}

impl Field {
    #[must_use]
    pub fn name(&self) -> &Ident {
        match self {
            Self::Column { name, .. } => name,
            Self::Relation { name, .. } => name,
            Self::Expr { alias, .. } => &alias.ident,
        }
    }

    #[must_use]
    pub fn alias(&self) -> Option<&Alias> {
        match self {
            Self::Column { alias, .. } => alias.as_ref(),
            Self::Relation { alias, .. } => alias.as_ref(),
            Self::Expr { alias, .. } => Some(alias),
        }
    }

    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Column { name, .. } => name.span(),
            Self::Relation { name, .. } => name.span(),
            Self::Expr { alias, .. } => alias.ident.span(),
        }
    }

    /// Returns `true` if the query field is [`Column`].
    ///
    /// [`Column`]: Field::Column
    #[must_use]
    pub fn is_column(&self) -> bool {
        matches!(self, Self::Column { .. })
    }

    #[must_use]
    pub fn to_row_field(&self, table_path: &Path, node_path: &QueryNodePath) -> RowField {
        match self {
            Field::Column {
                attrs,
                name,
                alias,
                type_override,
                ..
            } => {
                let alias_or_name = alias.as_ref().map_or(name, |alias| &alias.ident).clone();

                let type_override_or_default = type_override.as_ref().map_or_else(
                    || parse_quote! { #table_path::columns::#name::Type },
                    |type_override| type_override.type_path.to_call_site(1),
                );

                RowField::new(
                    attrs.clone(),
                    alias_or_name,
                    type_override_or_default.to_token_stream(),
                )
            }
            Field::Relation {
                attrs, name, alias, ..
            } => {
                let alias_or_name = alias.as_ref().map_or(name, |alias| &alias.ident).clone();

                let mut node_path = node_path.clone();
                node_path.append(name.clone());
                let inner_type = node_path.to_struct_name("Row");

                RowField::new(
                    attrs.clone(),
                    alias_or_name,
                    quote! { #table_path::relations::#name::Type<#inner_type> },
                )
            }
            Field::Expr {
                attrs,
                alias,
                type_override,
                ..
            } => RowField::new(
                attrs.clone(),
                alias.ident.clone(),
                type_override.type_path.to_call_site(1).to_token_stream(),
            ),
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let fork = input.fork();
        let ident = fork.parse::<Ident>();

        if input.peek2(syn::token::Brace) {
            Ok(Self::Relation {
                attrs,
                name: input.parse()?,
                node: input.parse()?,
                alias: input.call(Alias::parse_option)?,
            })
        } else if ident.is_ok()
            && (fork.peek(Token![,])
                || Alias::peek(&fork)
                || TypeOverride::peek(&fork)
                || fork.is_empty())
        {
            Ok(Self::Column {
                attrs,
                name: input.parse()?,
                alias: input.call(Alias::parse_option)?,
                type_override: input.call(TypeOverride::parse_option)?,
            })
        } else {
            Ok(Self::Expr {
                attrs,
                expr: input.parse()?,
                alias: input.parse()?,
                type_override: input.parse()?,
            })
        }
    }
}

impl PrettyPrint for Field {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::Column {
                attrs,
                name,
                alias,
                type_override,
            } => {
                attrs.pretty_print(printer);
                printer.scan_indent(1);
                printer.scan_begin(BreakMode::Inconsistent);
                name.pretty_print(printer);
                alias.pretty_print(printer);
                type_override.pretty_print(printer);
                printer.scan_end();
                printer.scan_indent(-1);
            }
            Self::Relation {
                attrs,
                name,
                node,
                alias,
            } => {
                attrs.pretty_print(printer);
                name.pretty_print(printer);
                " ".pretty_print(printer);
                node.pretty_print(printer);
                alias.pretty_print(printer);
            }
            Self::Expr {
                attrs,
                expr,
                alias,
                type_override,
            } => {
                attrs.pretty_print(printer);
                printer.scan_indent(1);
                printer.scan_begin(BreakMode::Inconsistent);
                expr.pretty_print(printer);
                alias.pretty_print(printer);
                type_override.pretty_print(printer);
                printer.scan_end();
                printer.scan_indent(-1);
            }
        }
    }
}
