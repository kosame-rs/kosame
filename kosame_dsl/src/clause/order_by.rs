use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{clause::peek_clause, expr::Expr, keyword, visitor::Visitor};

pub struct OrderBy {
    pub _order: keyword::order,
    pub _by: keyword::by,
    pub items: Punctuated<OrderByItem, Token![,]>,
}

impl OrderBy {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::order) && input.peek2(keyword::by)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        for item in &self.items {
            item.expr.accept(visitor);
        }
    }
}

impl Parse for OrderBy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _order: input.call(keyword::order::parse_autocomplete)?,
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
                        "order by clause cannot be empty",
                    ));
                }
                punctuated
            },
        })
    }
}

impl ToTokens for OrderBy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items.iter();
        quote! { ::kosame::repr::clause::OrderBy::new(&[#(#items),*]) }.to_tokens(tokens)
    }
}

pub struct OrderByItem {
    pub expr: Expr,
    pub dir: Option<OrderByDir>,
    pub nulls: Option<OrderByNulls>,
}

impl Parse for OrderByItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            expr: input.parse()?,
            dir: input.call(OrderByDir::parse_optional)?,
            nulls: input.call(OrderByNulls::parse_optional)?,
        })
    }
}

impl ToTokens for OrderByItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        let dir = match self.dir {
            Some(OrderByDir::Asc(_)) => quote! { Some(::kosame::repr::clause::OrderByDir::Asc) },
            Some(OrderByDir::Desc(_)) => quote! { Some(::kosame::repr::clause::OrderByDir::Desc) },
            None => quote! { None },
        };
        let nulls = match self.nulls {
            Some(OrderByNulls::First(..)) => {
                quote! { Some(::kosame::repr::clause::OrderByNulls::First) }
            }
            Some(OrderByNulls::Last(..)) => {
                quote! { Some(::kosame::repr::clause::OrderByNulls::Last) }
            }
            None => quote! { None },
        };

        quote! { ::kosame::repr::clause::OrderByItem::new(#expr, #dir, #nulls) }.to_tokens(tokens);
    }
}

#[allow(unused)]
pub enum OrderByDir {
    Asc(keyword::asc),
    Desc(keyword::desc),
}

impl OrderByDir {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::asc) || input.peek(keyword::desc)
    }
}

impl Parse for OrderByDir {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::asc) {
            Ok(Self::Asc(input.parse()?))
        } else if lookahead.peek(keyword::desc) {
            Ok(Self::Desc(input.parse()?))
        } else {
            keyword::group_order_by_dir::error(input);
        }
    }
}

#[allow(unused)]
pub enum OrderByNulls {
    First(keyword::nulls, keyword::first),
    Last(keyword::nulls, keyword::last),
}

impl OrderByNulls {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::nulls)
    }
}

impl Parse for OrderByNulls {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nulls = input.call(keyword::nulls::parse_autocomplete)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::first) {
            Ok(Self::First(nulls, input.parse()?))
        } else if lookahead.peek(keyword::last) {
            Ok(Self::Last(nulls, input.parse()?))
        } else {
            keyword::group_order_by_nulls::error(input);
        }
    }
}
