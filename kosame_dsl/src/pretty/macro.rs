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
                paren.pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                    inner.pretty_print(printer);
                });
            }
            Self::Braced { brace, inner } => {
                brace.pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                    printer.scan_text(" ".into(), super::TextMode::NoBreak);
                    inner.pretty_print(printer);
                    printer.scan_text(" ".into(), super::TextMode::NoBreak);
                });
            }
            Self::Bracketed { bracket, inner } => {
                bracket.pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                    inner.pretty_print(printer);
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parenthesized_short() {
        let source = "(foo)";
        let result = crate::pretty::pretty_print_macro_str::<Macro<syn::Ident>>(source, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "(foo)");
    }

    #[test]
    fn test_parenthesized_long() {
        let source = "(this_is_a_very_long_identifier_name_that_should_definitely_break_across_multiple_lines_when_pretty_printed)";
        let result = crate::pretty::pretty_print_macro_str::<Macro<syn::Ident>>(source, 0, 0);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            r"(
    this_is_a_very_long_identifier_name_that_should_definitely_break_across_multiple_lines_when_pretty_printed
)"
        );
    }

    #[test]
    fn test_braced_short() {
        let source = "{ foo }";
        let result = crate::pretty::pretty_print_macro_str::<Macro<syn::Ident>>(source, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{ foo }");
    }

    #[test]
    fn test_braced_long() {
        let source = "{ this_is_a_very_long_identifier_name_that_should_definitely_break_across_multiple_lines_when_pretty_printed }";
        let result = crate::pretty::pretty_print_macro_str::<Macro<syn::Ident>>(source, 0, 0);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            r"{
    this_is_a_very_long_identifier_name_that_should_definitely_break_across_multiple_lines_when_pretty_printed
}"
        );
    }

    #[test]
    fn test_bracketed_short() {
        let source = "[foo]";
        let result = crate::pretty::pretty_print_macro_str::<Macro<syn::Ident>>(source, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[foo]");
    }

    #[test]
    fn test_bracketed_long() {
        let source = "[this_is_a_very_long_identifier_name_that_should_definitely_break_across_multiple_lines_when_pretty_printed]";
        let result = crate::pretty::pretty_print_macro_str::<Macro<syn::Ident>>(source, 0, 0);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            r"[
    this_is_a_very_long_identifier_name_that_should_definitely_break_across_multiple_lines_when_pretty_printed
]"
        );
    }
}
