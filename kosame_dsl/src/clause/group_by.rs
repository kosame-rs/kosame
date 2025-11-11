use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{clause::peek_clause, expr::Expr, keyword, visitor::Visitor};

pub struct GroupBy {
    pub _group: keyword::group,
    pub _by: keyword::by,
    pub items: Punctuated<GroupByItem, Token![,]>,
}

impl GroupBy {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::group) && input.peek2(keyword::by)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        for item in &self.items {
            item.expr.accept(visitor);
        }
    }
}

impl Parse for GroupBy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _group: input.call(keyword::group::parse_autocomplete)?,
            _by: input.call(keyword::by::parse_autocomplete)?,
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
        quote! { ::kosame::repr::clause::GroupBy::new(&[#(#items),*]) }.to_tokens(tokens)
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
