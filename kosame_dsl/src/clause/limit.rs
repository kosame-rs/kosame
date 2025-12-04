use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    clause::Clause,
    expr::ExprRoot,
    keyword,
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
    visit::Visit,
};

pub struct Limit {
    pub limit: keyword::limit,
    pub expr: ExprRoot,
}

impl ParseOption for Limit {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::limit)
    }
}

pub fn visit_limit<'a>(visit: &mut (impl Visit<'a> + ?Sized), limit: &'a Limit) {
    visit.visit_expr_root(&limit.expr);
}

impl Parse for Limit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            limit: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Limit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Limit::new(#expr) }.to_tokens(tokens);
    }
}

impl PrettyPrint for Limit {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        Clause::new(&[&self.limit], &self.expr).pretty_print(printer);
    }
}
