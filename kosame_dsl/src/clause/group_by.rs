use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    clause::{Clause, peek_clause},
    expr::ExprRoot,
    keyword,
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
    visit::Visit,
};

pub struct GroupBy {
    pub group_keyword: keyword::group,
    pub by_keyword: keyword::by,
    pub items: Punctuated<GroupByItem, Token![,]>,
}

impl ParseOption for GroupBy {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::group) && input.peek2(keyword::by)
    }
}

pub fn visit_group_by<'a>(visit: &mut (impl Visit<'a> + ?Sized), group_by: &'a GroupBy) {
    for item in &group_by.items {
        visit.visit_expr_root(&item.expr);
    }
}

impl Parse for GroupBy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            group_keyword: input.call(keyword::group::parse_autocomplete)?,
            by_keyword: input.call(keyword::by::parse_autocomplete)?,
            items: {
                let mut punctuated = Punctuated::new();
                while !input.is_empty() {
                    if peek_clause(input) {
                        break;
                    }
                    punctuated.push(input.parse()?);
                    if !input.peek(Token![,]) {
                        break;
                    }
                    punctuated.push_punct(input.parse()?);
                }
                if punctuated.is_empty() {
                    return Err(syn::Error::new(
                        input.span(),
                        "group by clause cannot be empty",
                    ));
                }
                punctuated
            },
        })
    }
}

impl ToTokens for GroupBy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items.iter();
        quote! { ::kosame::repr::clause::GroupBy::new(&[#(#items),*]) }.to_tokens(tokens);
    }
}

impl PrettyPrint for GroupBy {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        Clause::new(&[&self.group_keyword, &self.by_keyword], &self.items).pretty_print(printer);
    }
}

pub struct GroupByItem {
    pub expr: ExprRoot,
}

impl Parse for GroupByItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            expr: input.parse()?,
        })
    }
}

impl ToTokens for GroupByItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        quote! { ::kosame::repr::clause::GroupByItem::new(#expr) }.to_tokens(tokens);
    }
}

impl PrettyPrint for GroupByItem {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.expr.pretty_print(printer);
    }
}
