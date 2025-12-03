use proc_macro2::Literal;
use quote::ToTokens;
use syn::spanned::Spanned;

use super::{PrettyPrint, Printer, TextMode};

impl PrettyPrint for &'static str {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        if *self == " " {
            printer.scan_no_break_trivia();
        }
        printer.scan_text((*self).into(), TextMode::Always);
        printer.advance_cursor(self);
    }
}

impl PrettyPrint for String {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_text(self.clone().into(), TextMode::Always);
    }
}

impl PrettyPrint for Literal {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.move_cursor(self.span().start());
        printer.scan_text(self.to_string().into(), TextMode::Always);
        printer.move_cursor(self.span().end());
    }
}

impl PrettyPrint for syn::Ident {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.move_cursor(self.span().start());
        printer.scan_text(self.to_string().into(), TextMode::Always);
        printer.move_cursor(self.span().end());
    }
}

impl PrettyPrint for syn::Lit {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.move_cursor(self.span().start());
        printer.scan_text(self.to_token_stream().to_string().into(), TextMode::Always);
        printer.move_cursor(self.span().end());
    }
}

macro_rules! impl_token {
    ($token:tt, $literal:literal) => {
        impl PrettyPrint for syn::Token![$token] {
            fn pretty_print(&self, printer: &mut Printer<'_>) {
                printer.move_cursor(self.span().start());
                printer.scan_text($literal.into(), TextMode::Always);
                printer.move_cursor(self.span().end());
            }
        }
    };
}

impl_token!(#, "#");
impl_token!(!, "!");
impl_token!(=, "=");
impl_token!(., ".");
impl_token!(,, ",");
impl_token!(:, ":");
impl_token!(;, ";");
impl_token!(*, "*");
impl_token!(/, "/");
impl_token!(%, "%");
impl_token!(+, "+");
impl_token!(-, "-");
impl_token!(>, ">");
impl_token!(<, "<");
impl_token!($, "$");
impl_token!(as, "as");
impl_token!(=>, "=>");
impl_token!(<=, "<=");
impl_token!(where, "where");

macro_rules! impl_has_token {
    ($($for:tt)*) => {
        impl PrettyPrint for $($for)* {
            fn pretty_print(&self, printer: &mut Printer<'_>) {
                self.token().pretty_print(printer);
            }
        }
    };
}

impl_has_token!(syn::LitBool);
impl_has_token!(syn::LitInt);
impl_has_token!(syn::LitFloat);
impl_has_token!(syn::LitStr);
