use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    clause::peek_clause,
    expr::Expr,
    keyword,
    pretty::{BreakMode, PrettyPrint, Printer},
    visit::Visit,
};

pub struct Set {
    set_keyword: keyword::set,
    items: Punctuated<SetItem, Token![,]>,
}

impl Set {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::set)
    }
}

pub fn visit_set<'a>(visit: &mut (impl Visit<'a> + ?Sized), set: &'a Set) {
    for item in &set.items {
        visit.visit_set_item(item);
    }
}

impl Parse for Set {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            set_keyword: input.parse()?,
            items: {
                let mut items = Punctuated::<SetItem, _>::new();
                while !input.is_empty() {
                    if peek_clause(input) {
                        break;
                    }

                    items.push(input.parse()?);

                    if !input.peek(Token![,]) {
                        break;
                    }
                    items.push_punct(input.parse()?);
                }

                items
            },
        })
    }
}

impl ToTokens for Set {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items.iter();
        quote! {
            ::kosame::repr::clause::Set::new(&[#(#items),*])
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for Set {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_break();
        printer.scan_trivia(true, true);

        " ".pretty_print(printer);
        self.set_keyword.pretty_print(printer);

        printer.scan_indent(1);
        printer.scan_break();
        self.items.pretty_print(printer);
        printer.scan_indent(-1);
    }
}

pub enum SetItem {
    Default {
        column: Ident,
        eq_token: Token![=],
        default_keyword: keyword::default,
    },
    Expr {
        column: Ident,
        eq_token: Token![=],
        expr: Expr,
    },
}

pub fn visit_set_item<'a>(visit: &mut (impl Visit<'a> + ?Sized), set_item: &'a SetItem) {
    match set_item {
        SetItem::Default { .. } => {}
        SetItem::Expr { expr, .. } => {
            visit.visit_expr(expr);
        }
    }
}

impl Parse for SetItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let column: Ident = input.parse()?;
        let eq_token = input.parse()?;

        if input.peek(keyword::default) {
            Ok(Self::Default {
                column,
                eq_token,
                default_keyword: input.parse()?,
            })
        } else {
            Ok(Self::Expr {
                column,
                eq_token,
                expr: input.parse()?,
            })
        }
    }
}

impl ToTokens for SetItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Default { column, .. } => {
                let column = column.to_string();
                quote! {
                    ::kosame::repr::clause::SetItem::Default { column: #column }
                }
            }
            Self::Expr { column, expr, .. } => {
                let column = column.to_string();
                quote! {
                    ::kosame::repr::clause::SetItem::Expr { column: #column, expr: #expr }
                }
            }
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for SetItem {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::Default {
                column,
                eq_token,
                default_keyword,
            } => {
                column.pretty_print(printer);
                " ".pretty_print(printer);
                eq_token.pretty_print(printer);
                " ".pretty_print(printer);
                default_keyword.pretty_print(printer);
            }
            Self::Expr {
                column,
                eq_token,
                expr,
            } => {
                column.pretty_print(printer);
                " ".pretty_print(printer);
                eq_token.pretty_print(printer);
                " ".pretty_print(printer);

                printer.scan_indent(1);
                printer.scan_begin(BreakMode::Inconsistent);
                expr.pretty_print(printer);
                printer.scan_end();
                printer.scan_indent(-1);
            }
        }
    }
}
