use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use crate::{
    expr::Expr,
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
    visit::Visit,
};

pub struct Where {
    pub where_token: Token![where],
    pub expr: Expr,
}

impl ParseOption for Where {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![where])
    }
}

pub fn visit_where<'a>(visit: &mut (impl Visit<'a> + ?Sized), r#where: &'a Where) {
    visit.visit_expr(&r#where.expr);
}

impl Parse for Where {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            where_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Where {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::Where::new(#expr) }.to_tokens(tokens);
    }
}

impl PrettyPrint for Where {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_break(false);
        " ".pretty_print(printer);
        self.where_token.pretty_print(printer);
        " ".pretty_print(printer);
        self.expr.pretty_print(printer);
    }
}
