use std::{borrow::Cow, ops::Deref};

use proc_macro2::Literal;
use quote::ToTokens;
use syn::{Ident, spanned::Spanned};

use super::{PrettyPrint, Printer, Span};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextMode {
    Always,
    NoBreak,
    Break,
}

#[derive(Debug, Clone)]
pub struct Text {
    string: Cow<'static, str>,
    span: Option<Span>,
    mode: TextMode,
}

impl Text {
    pub fn new(string: impl Into<Cow<'static, str>>, span: Option<Span>, mode: TextMode) -> Self {
        Self {
            string: string.into(),
            span,
            mode,
        }
    }

    #[must_use] 
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    #[must_use] 
    pub fn mode(&self) -> TextMode {
        self.mode
    }

    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use] 
    pub fn len(&self) -> usize {
        self.string.len()
    }
}

impl Deref for Text {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.string
    }
}

impl<T> PrettyPrint for T
where
    for<'b> &'b T: Into<Text>,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_text(self);
    }
}

impl From<&'static str> for Text {
    fn from(value: &'static str) -> Self {
        Self::new(value, None, TextMode::Always)
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value, None, TextMode::Always)
    }
}

impl From<&Ident> for Text {
    fn from(value: &Ident) -> Self {
        Self::new(
            value.to_string(),
            Some(value.span().into()),
            TextMode::Always,
        )
    }
}

impl From<&Literal> for Text {
    fn from(value: &Literal) -> Self {
        Self::new(
            value.to_string(),
            Some(value.span().into()),
            TextMode::Always,
        )
    }
}

macro_rules! impl_token {
    ($token:tt) => {
        impl From<&syn::Token![$token]> for Text {
            fn from(value: &syn::Token![$token]) -> Self {
                Self::new(
                    value.to_token_stream().to_string(),
                    Some(value.span().into()),
                    TextMode::Always,
                )
            }
        }
    };
}

impl_token!(#);
impl_token!(!);
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
impl_token!(=>);
impl_token!(<=);
