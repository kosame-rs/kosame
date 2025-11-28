#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ident<'a>(&'a str);

impl<'a> Ident<'a> {
    #[inline]
    #[must_use]
    pub const fn new(ident: &'a str) -> Self {
        Self(ident)
    }

    #[inline]
    #[must_use]
    pub const fn from_option(ident: Option<&'a str>) -> Option<Self> {
        match ident {
            Some(ident) => Some(Self::new(ident)),
            None => None,
        }
    }
}

impl kosame_sql::FmtSql for Ident<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        D::fmt_ident(formatter, self.0)
    }
}

impl<'a> From<&'a str> for Ident<'a> {
    fn from(value: &'a str) -> Self {
        Ident::new(value)
    }
}
