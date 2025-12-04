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

pub struct Offset {
    pub offset: keyword::offset,
    pub expr: ExprRoot,
}

impl ParseOption for Offset {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::offset)
    }
}

pub fn visit_offset<'a>(visit: &mut (impl Visit<'a> + ?Sized), offset: &'a Offset) {
    visit.visit_expr_root(&offset.expr);
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
        Clause::new(&[&self.offset], &self.expr).pretty_print(printer);
    }
}
