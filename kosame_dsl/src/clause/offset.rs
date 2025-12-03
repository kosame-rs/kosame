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

pub struct Offset {
    pub offset: keyword::offset,
    pub expr: Expr,
}

impl ParseOption for Offset {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::offset)
    }
}

pub fn visit_offset<'a>(visit: &mut (impl Visit<'a> + ?Sized), offset: &'a Offset) {
    visit.visit_expr(&offset.expr);
}

impl Parse for Offset {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            offset: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Offset {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Offset::new(#expr) }.to_tokens(tokens);
    }
}

impl PrettyPrint for Offset {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_break(false);
        " ".pretty_print(printer);
        self.offset.pretty_print(printer);
        printer.scan_indent(1);
        printer.scan_break(false);
        " ".pretty_print(printer);
        printer.scan_begin(BreakMode::Inconsistent);
        self.expr.pretty_print(printer);
        printer.scan_end();
        printer.scan_indent(-1);
    }
}
