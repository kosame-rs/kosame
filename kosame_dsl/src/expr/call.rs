use crate::{
    inferred_type::InferredType,
    pretty::{BreakMode, Delim, PrettyPrint, Printer},
    scopes::ScopeId,
};

use super::{Expr, Visit};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

pub struct Call {
    pub function: Ident,
    pub paren: syn::token::Paren,
    pub params: Punctuated<Expr, Token![,]>,
}

impl Call {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Ident) && input.peek2(syn::token::Paren)
    }

    #[must_use]
    pub fn infer_name(&self) -> Option<&Ident> {
        Some(&self.function)
    }

    #[must_use]
    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType<'_>> {
        None
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visit<'a>) {
        for param in &self.params {
            param.accept(visitor);
        }
    }

    #[must_use]
    pub fn span(&self) -> Span {
        self.function
            .span()
            .join(self.paren.span.span())
            .unwrap_or(self.function.span())
    }
}

impl Parse for Call {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            function: input.parse()?,
            paren: parenthesized!(content in input),
            params: content.parse_terminated(Expr::parse, Token![,])?,
        })
    }
}

impl ToTokens for Call {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let function_name = &self.function.to_string();
        let params = self.params.iter();

        // Some functions like `coalesce` must not be quoted like an identifier, whereas others,
        // like `sum`, can be. User defined functions should be treated as identifiers.
        let keyword = matches!(
            function_name.as_ref(),
            "coalesce" | "greatest" | "least" | "nullif"
        );

        quote! {
            ::kosame::repr::expr::Call::new(
                #function_name,
                &[#(#params),*],
                #keyword,
            )
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for Call {
    fn pretty_print(&self, printer: &mut Printer) {
        self.function.pretty_print(printer);
        self.paren
            .pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                self.params.pretty_print(printer);
            });
    }
}
