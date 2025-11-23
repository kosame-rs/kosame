use std::borrow::Cow;

use proc_macro2::{Literal, Span};
use quote::ToTokens;
use syn::{Ident, Token, punctuated::Punctuated, spanned::Spanned};

use super::{PrettyPrint, Printer, TextMode};

pub trait Text {
    fn into_cow_str(self) -> Cow<'static, str>;
    fn span(&self) -> Option<Span>;
}

impl<T> PrettyPrint for Option<T>
where
    T: PrettyPrint,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        if let Some(inner) = self {
            inner.pretty_print(printer);
        }
    }
}

impl<T> PrettyPrint for T
where
    for<'b> &'b T: Text,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_text(self);
    }
}

impl<T> PrettyPrint for Punctuated<T, Token![,]>
where
    T: PrettyPrint,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        for (index, item) in self.pairs().enumerate() {
            item.value().pretty_print(printer);
            if index != self.len() - 1 {
                item.punct().unwrap().pretty_print(printer);
                printer.scan_break(" ");
            } else {
                printer.scan_text_with_mode(",", TextMode::Break);
            }
        }
    }
}

impl Text for &'static str {
    fn into_cow_str(self) -> Cow<'static, str> {
        self.into()
    }

    fn span(&self) -> Option<Span> {
        None
    }
}

impl Text for String {
    fn into_cow_str(self) -> Cow<'static, str> {
        self.into()
    }

    fn span(&self) -> Option<Span> {
        None
    }
}

impl Text for &Ident {
    fn into_cow_str(self) -> Cow<'static, str> {
        self.to_string().into()
    }

    fn span(&self) -> Option<Span> {
        Some(<Self as Spanned>::span(self))
    }
}

impl Text for &Literal {
    fn into_cow_str(self) -> Cow<'static, str> {
        self.to_string().into()
    }

    fn span(&self) -> Option<Span> {
        Some(<Self as Spanned>::span(self))
    }
}

macro_rules! impl_token {
    ($token:tt) => {
        impl Text for &syn::Token![$token] {
            fn into_cow_str(self) -> Cow<'static, str> {
                self.to_token_stream().to_string().into()
            }

            fn span(&self) -> Option<Span> {
                Some(<Self as Spanned>::span(self))
            }
        }
    };
}

impl_token!(=);
impl_token!(.);
impl_token!(,);
impl_token!(:);
impl_token!(;);
impl_token!(*);
impl_token!(/);
impl_token!(%);
impl_token!(+);
impl_token!(-);
impl_token!(>);
impl_token!(<);
impl_token!($);
impl_token!(as);
