use proc_macro_error::emit_error;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Ident, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    clause::peek_clause,
    correlations::{CorrelationId, Correlations},
    expr::ExprRoot,
    inferred_type::{InferredType, resolve_type},
    parse_option::ParseOption,
    part::{Alias, TypeOverride},
    pretty::{PrettyPrint, Printer},
    quote_option::QuoteOption,
    row::RowField,
    scopes::{ScopeId, Scopes},
    visit::Visit,
};

pub struct Field {
    pub attrs: Vec<Attribute>,
    pub expr: ExprRoot,
    pub alias: Option<Alias>,
    pub type_override: Option<TypeOverride>,
}

impl Field {
    #[must_use]
    pub fn to_row_field(
        &self,
        correlations: &Correlations<'_>,
        scopes: &Scopes<'_>,
        correlation_id: CorrelationId,
    ) -> Option<RowField> {
        let Some(name) = self.infer_name() else {
            emit_error!(
                self.expr.span(),
                "field name cannot be inferred";
                help = "consider adding an alias using `as my_alias`"
            );
            return None;
        };
        let Some(resolved_type) = resolve_type(correlations, scopes, correlation_id, name) else {
            emit_error!(
                self.expr.span(),
                "field type cannot be inferred";
                help = "consider adding a type override using `: RustType`";
                note = "Kosame can only infer the type of a column when it is qualified with its table, e.g. `posts.id` instead of `id`"
            );
            return None;
        };
        Some(RowField::new(
            self.attrs.clone(),
            name.clone(),
            resolved_type.to_token_stream(),
        ))
    }

    #[must_use]
    pub fn infer_name(&self) -> Option<&Ident> {
        self.alias
            .as_ref()
            .map(|alias| &alias.ident)
            .or_else(|| self.expr.infer_name())
    }

    #[must_use]
    pub fn infer_type(&self, scope_id: ScopeId) -> Option<InferredType<'_>> {
        self.type_override
            .as_ref()
            .map(|type_override| InferredType::RustType(&type_override.type_path))
            .or_else(|| self.expr.infer_type(scope_id))
    }
}

pub fn visit_field<'a>(visit: &mut (impl Visit<'a> + ?Sized), field: &'a Field) {
    visit.visit_expr_root(&field.expr);
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            expr: input.parse()?,
            alias: input.call(Alias::parse_option)?,
            type_override: input.call(TypeOverride::parse_option)?,
        })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        let alias = QuoteOption::from(&self.alias);
        quote! {
            ::kosame::repr::clause::Field::new(#expr, #alias)
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for Field {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.attrs.pretty_print(printer);
        self.expr.pretty_print(printer);
        self.alias.pretty_print(printer);
        self.type_override.pretty_print(printer);
    }
}

pub struct Fields(pub Punctuated<Field, Token![,]>);

impl Fields {
    pub fn iter(&self) -> impl Iterator<Item = &Field> {
        self.0.iter()
    }

    #[must_use]
    pub fn columns(&self) -> Vec<&Ident> {
        self.iter().filter_map(|field| field.infer_name()).collect()
    }
}

pub fn visit_fields<'a>(visit: &mut (impl Visit<'a> + ?Sized), fields: &'a Fields) {
    for field in fields.iter() {
        visit.visit_field(field);
    }
}

impl Parse for Fields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut fields = Punctuated::<Field, _>::new();

        while !input.is_empty() {
            if peek_clause(input) {
                break;
            }

            fields.push(input.parse()?);

            if !input.peek(Token![,]) {
                break;
            }
            fields.push_punct(input.parse()?);
        }

        Ok(Self(fields))
    }
}

impl ToTokens for Fields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = self.0.iter();
        quote! {
            ::kosame::repr::clause::Fields::new(&[
                #(#fields),*
            ])
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for Fields {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.0.pretty_print(printer);
    }
}
