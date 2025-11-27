use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
};

use crate::pretty::{BreakMode, Delim, PrettyPrint, Printer};

pub enum Macro<T> {
    Parenthesized {
        paren: syn::token::Paren,
        inner: T,
    },
    Braced {
        brace: syn::token::Brace,
        inner: T,
    },
    Bracketed {
        bracket: syn::token::Bracket,
        inner: T,
    },
}

impl<T> Parse for Macro<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let content;
        if lookahead.peek(syn::token::Paren) {
            Ok(Self::Parenthesized {
                paren: parenthesized!(content in input),
                inner: content.parse()?,
            })
        } else if lookahead.peek(syn::token::Brace) {
            Ok(Self::Braced {
                brace: braced!(content in input),
                inner: content.parse()?,
            })
        } else if lookahead.peek(syn::token::Bracket) {
            Ok(Self::Bracketed {
                bracket: bracketed!(content in input),
                inner: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl<T> PrettyPrint for Macro<T>
where
    T: PrettyPrint,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::Parenthesized { paren, inner } => {
                paren.pretty_print(printer, BreakMode::Consistent, |printer| {
                    inner.pretty_print(printer);
                });
            }
            Self::Braced { brace, inner } => {
                brace.pretty_print(printer, BreakMode::Consistent, |printer| {
                    printer.scan_text(" ".into(), super::TextMode::NoBreak);
                    inner.pretty_print(printer);
                    printer.scan_text(" ".into(), super::TextMode::NoBreak);
                });
            }
            Self::Bracketed { bracket, inner } => {
                bracket.pretty_print(printer, BreakMode::Consistent, |printer| {
                    inner.pretty_print(printer);
                });
            }
        }
    }
}
