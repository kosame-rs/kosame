use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    expr::Expr,
    keyword,
    parse_option::ParseOption,
    pretty::{BreakMode, PrettyPrint, Printer},
    visit::Visit,
};

pub struct Limit {
    pub limit: keyword::limit,
    pub expr: Expr,
}

impl ParseOption for Limit {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::limit)
    }
}

pub fn visit_limit<'a>(visit: &mut (impl Visit<'a> + ?Sized), limit: &'a Limit) {
    visit.visit_expr(&limit.expr);
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
        printer.scan_break(false);
        printer.scan_trivia();
        " ".pretty_print(printer);
        self.limit.pretty_print(printer);
        printer.scan_indent(1);
        printer.scan_break(false);
        " ".pretty_print(printer);
        printer.scan_begin(BreakMode::Inconsistent);
        self.expr.pretty_print(printer);
        printer.scan_end();
        printer.scan_indent(-1);
    }
}
