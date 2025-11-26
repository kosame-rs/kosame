use std::{fmt::Write, ops::Deref};

use crate::expr::Expr;

pub struct Field<'a> {
    expr: Expr<'a>,
    alias: Option<&'a str>,
}

impl<'a> Field<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(expr: Expr<'a>, alias: Option<&'a str>) -> Self {
        Self { expr, alias }
    }

    #[inline]
    #[must_use] 
    pub const fn expr(&self) -> &Expr<'a> {
        &self.expr
    }

    #[inline]
    #[must_use] 
    pub const fn alias(&self) -> Option<&'a str> {
        self.alias
    }
}

impl kosame_sql::FmtSql for Field<'_> {
    #[inline]
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        self.expr.fmt_sql(formatter)?;
        if let Some(alias) = &self.alias {
            formatter.write_str(" as ")?;
            formatter.write_ident(alias)?;
        }
        Ok(())
    }
}

pub struct Fields<'a>(&'a [Field<'a>]);

impl<'a> Fields<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(fields: &'a [Field<'a>]) -> Self {
        Self(fields)
    }
}

impl<'a> Deref for Fields<'a> {
    type Target = &'a [Field<'a>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl kosame_sql::FmtSql for Fields<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        kosame_sql::Punctuated::new(self.0, ",").fmt_sql(formatter)
    }
}
