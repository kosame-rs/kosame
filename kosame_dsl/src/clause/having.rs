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

pub struct Having {
    pub having_keyword: keyword::having,
    pub expr: ExprRoot,
}

impl ParseOption for Having {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::having)
    }
}

pub fn visit_having<'a>(visit: &mut (impl Visit<'a> + ?Sized), having: &'a Having) {
    visit.visit_expr_root(&having.expr);
}

impl Parse for Having {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            having_keyword: input.parse()?,
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
        Clause::new(&[&self.having_keyword], &self.expr).pretty_print(printer);
    }
}
