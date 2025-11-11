use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{expr::Expr, keyword, visitor::Visitor};

pub struct Values {
    pub _values_keyword: keyword::values,
    pub rows: Punctuated<ValuesRow, Token![,]>,
}

impl Values {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::values)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        for row in &self.rows {
            row.accept(visitor);
        }
    }
}

impl Parse for Values {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _values_keyword: input.parse()?,
            rows: {
                let mut punctuated = Punctuated::new();
                while input.peek(syn::token::Paren) {
                    punctuated.push(input.parse()?);
                    if !input.peek(Token![,]) {
                        break;
                    }
                    punctuated.push_punct(input.parse()?);
                }
                if punctuated.is_empty() {
                    return Err(syn::Error::new(input.span(), "values list cannot be empty"));
                }
                punctuated
            },
        })
    }
}

impl ToTokens for Values {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let rows = self.rows.iter();
        quote! { ::kosame::repr::clause::Values::new(&[#(#rows),*]) }.to_tokens(tokens);
    }
}

pub struct ValuesRow {
    _paren_token: syn::token::Paren,
    items: Punctuated<ValuesItem, Token![,]>,
}

impl ValuesRow {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        for item in &self.items {
            item.accept(visitor);
        }
    }
}

impl Parse for ValuesRow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _paren_token: parenthesized!(content in input),
            items: content.parse_terminated(ValuesItem::parse, Token![,])?,
        })
    }
}

impl ToTokens for ValuesRow {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items.iter();
        quote! { ::kosame::repr::clause::ValuesRow::new(&[#(#items),*]) }.to_tokens(tokens);
    }
}

#[allow(unused)]
pub enum ValuesItem {
    Default(keyword::default),
    Expr(Expr),
}

impl ValuesItem {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        match self {
            Self::Default(..) => {}
            Self::Expr(expr) => expr.accept(visitor),
        }
    }
}

impl Parse for ValuesItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::default) {
            Ok(Self::Default(input.parse()?))
        } else {
            Ok(Self::Expr(input.parse()?))
        }
    }
}

impl ToTokens for ValuesItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Default(..) => quote! {
                ::kosame::repr::clause::ValuesItem::Default
            },
            Self::Expr(expr) => quote! {
                ::kosame::repr::clause::ValuesItem::Expr(#expr)
            },
        }
        .to_tokens(tokens);
    }
}
