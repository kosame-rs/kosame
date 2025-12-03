use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    clause::peek_clause,
    expr::Expr,
    keyword,
    parse_option::ParseOption,
    pretty::{BreakMode, PrettyPrint, Printer},
    visit::Visit,
};

pub struct OrderBy {
    pub order: keyword::order,
    pub by: keyword::by,
    pub items: Punctuated<OrderByItem, Token![,]>,
}

impl ParseOption for OrderBy {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::order) && input.peek2(keyword::by)
    }
}

pub fn visit_order_by<'a>(visit: &mut (impl Visit<'a> + ?Sized), order_by: &'a OrderBy) {
    for item in &order_by.items {
        visit.visit_expr(&item.expr);
    }
}

impl Parse for OrderBy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            order: input.call(keyword::order::parse_autocomplete)?,
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
        quote! { ::kosame::repr::clause::OrderBy::new(&[#(#items),*]) }.to_tokens(tokens);
    }
}

impl PrettyPrint for OrderBy {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        " ".pretty_print(printer);
        self.order.pretty_print(printer);
        " ".pretty_print(printer);
        self.by.pretty_print(printer);
        printer.scan_break(false);
        printer.scan_indent(1);
        " ".pretty_print(printer);
        self.items.pretty_print(printer);
        printer.scan_indent(-1);
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
            dir: input.call(OrderByDir::parse_option)?,
            nulls: input.call(OrderByNulls::parse_option)?,
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

impl PrettyPrint for OrderByItem {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_begin(BreakMode::Inconsistent);
        self.expr.pretty_print(printer);
        self.dir.pretty_print(printer);
        self.nulls.pretty_print(printer);
        printer.scan_end();
    }
}

#[allow(unused)]
pub enum OrderByDir {
    Asc(keyword::asc),
    Desc(keyword::desc),
}

impl ParseOption for OrderByDir {
    fn peek(input: ParseStream) -> bool {
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

impl PrettyPrint for OrderByDir {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::Asc(asc) => {
                printer.scan_break(false);
                " ".pretty_print(printer);
                asc.pretty_print(printer);
            }
            Self::Desc(desc) => {
                printer.scan_break(false);
                " ".pretty_print(printer);
                desc.pretty_print(printer);
            }
        }
    }
}

#[allow(unused)]
pub enum OrderByNulls {
    First(keyword::nulls, keyword::first),
    Last(keyword::nulls, keyword::last),
}

impl ParseOption for OrderByNulls {
    fn peek(input: ParseStream) -> bool {
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

impl PrettyPrint for OrderByNulls {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::First(nulls, first) => {
                printer.scan_break(false);
                " ".pretty_print(printer);
                nulls.pretty_print(printer);
                " ".pretty_print(printer);
                first.pretty_print(printer);
            }
            Self::Last(nulls, last) => {
                printer.scan_break(false);
                " ".pretty_print(printer);
                nulls.pretty_print(printer);
                " ".pretty_print(printer);
                last.pretty_print(printer);
            }
        }
    }
}
