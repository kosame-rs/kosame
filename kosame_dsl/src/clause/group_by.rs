use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{clause::peek_clause, expr::Expr, keyword, parse_option::ParseOption, visit::Visit};

pub struct GroupBy {
    pub group: keyword::group,
    pub by: keyword::by,
    pub items: Punctuated<GroupByItem, Token![,]>,
}

impl ParseOption for GroupBy {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::group) && input.peek2(keyword::by)
    }
}

impl GroupBy {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visit<'a>) {
        for item in &self.items {
            item.expr.accept(visitor);
        }
    }
}

impl Parse for GroupBy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            group: input.call(keyword::group::parse_autocomplete)?,
            by: input.call(keyword::by::parse_autocomplete)?,
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

pub struct GroupByItem {
    pub expr: Expr,
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
