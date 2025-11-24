use proc_macro2::extra::DelimSpan;

pub enum Delimiter<'a> {
    Paren(&'a syn::token::Paren),
    Brace(&'a syn::token::Brace),
    Bracket(&'a syn::token::Bracket),
}

impl Delimiter<'_> {
    pub fn span(&self) -> Option<DelimSpan> {
        match self {
            Self::Paren(inner) => Some(inner.span),
            Self::Brace(inner) => Some(inner.span),
            Self::Bracket(inner) => Some(inner.span),
        }
    }
}

impl<'a> From<&'a syn::token::Bracket> for Delimiter<'a> {
    fn from(v: &'a syn::token::Bracket) -> Self {
        Self::Bracket(v)
    }
}

impl<'a> From<&'a syn::token::Brace> for Delimiter<'a> {
    fn from(v: &'a syn::token::Brace) -> Self {
        Self::Brace(v)
    }
}

impl<'a> From<&'a syn::token::Paren> for Delimiter<'a> {
    fn from(v: &'a syn::token::Paren) -> Self {
        Self::Paren(v)
    }
}
