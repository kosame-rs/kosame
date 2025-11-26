use crate::pretty::{Span, Text, TextMode};

pub enum Delim<'a> {
    Paren(&'a syn::token::Paren),
    Brace(&'a syn::token::Brace),
    Bracket(&'a syn::token::Bracket),
}

impl Delim<'_> {
    #[must_use]
    pub fn span(&self) -> (Span, Span) {
        match self {
            Self::Paren(inner) => (inner.span.open().into(), inner.span.close().into()),
            Self::Brace(inner) => (inner.span.open().into(), inner.span.close().into()),
            Self::Bracket(inner) => (inner.span.open().into(), inner.span.close().into()),
        }
    }

    #[must_use]
    pub fn open_text(&self) -> Text {
        Text::new(
            match self {
                Self::Paren(..) => "(",
                Self::Brace(..) => "{",
                Self::Bracket(..) => "[",
            },
            Some(self.span().0),
            TextMode::Always,
        )
    }

    #[must_use]
    pub fn close_text(&self) -> Text {
        Text::new(
            match self {
                Self::Paren(..) => ")",
                Self::Brace(..) => "}",
                Self::Bracket(..) => "]",
            },
            Some(self.span().1),
            TextMode::Always,
        )
    }
}

impl<'a> From<&'a syn::token::Bracket> for Delim<'a> {
    fn from(v: &'a syn::token::Bracket) -> Self {
        Self::Bracket(v)
    }
}

impl<'a> From<&'a syn::token::Brace> for Delim<'a> {
    fn from(v: &'a syn::token::Brace) -> Self {
        Self::Brace(v)
    }
}

impl<'a> From<&'a syn::token::Paren> for Delim<'a> {
    fn from(v: &'a syn::token::Paren) -> Self {
        Self::Paren(v)
    }
}
