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

pub struct Having {
    pub having: keyword::having,
    pub expr: Expr,
}

impl ParseOption for Having {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::having)
    }
}

pub fn visit_having<'a>(visit: &mut (impl Visit<'a> + ?Sized), having: &'a Having) {
    visit.visit_expr(&having.expr);
}

impl Parse for Having {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            having: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Having {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Having::new(#expr) }.to_tokens(tokens);
    }
}

impl PrettyPrint for Having {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_break();
        printer.scan_trivia(true, true);
        " ".pretty_print(printer);
        self.having.pretty_print(printer);
        printer.scan_indent(1);
        printer.scan_break();
        " ".pretty_print(printer);
        printer.scan_begin(BreakMode::Inconsistent);
        self.expr.pretty_print(printer);
        printer.scan_end();
        printer.scan_indent(-1);
    }
}
