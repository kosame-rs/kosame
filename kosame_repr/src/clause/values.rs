use std::fmt::Write;

use crate::expr::Expr;

pub struct Values<'a> {
    rows: &'a [ValuesRow<'a>],
}

impl<'a> Values<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(rows: &'a [ValuesRow<'a>]) -> Self {
        Self { rows }
    }

    #[inline]
    #[must_use] 
    pub const fn rows(&self) -> &'a [ValuesRow<'a>] {
        self.rows
    }
}

impl kosame_sql::FmtSql for Values<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        formatter.write_str(" values ")?;
        kosame_sql::Punctuated::new(self.rows, ",").fmt_sql(formatter)?;
        Ok(())
    }
}

pub struct ValuesRow<'a> {
    items: &'a [ValuesItem<'a>],
}

impl<'a> ValuesRow<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(items: &'a [ValuesItem<'a>]) -> Self {
        Self { items }
    }

    #[inline]
    #[must_use] 
    pub const fn items(&self) -> &'a [ValuesItem<'a>] {
        self.items
    }
}

impl kosame_sql::FmtSql for ValuesRow<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        formatter.write_str("(")?;
        kosame_sql::Punctuated::new(self.items, ",").fmt_sql(formatter)?;
        formatter.write_str(")")?;
        Ok(())
    }
}

pub enum ValuesItem<'a> {
    Default,
    Expr(Expr<'a>),
}

impl kosame_sql::FmtSql for ValuesItem<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        match self {
            Self::Default => formatter.write_str("default")?,
            Self::Expr(expr) => expr.fmt_sql(formatter)?,
        }
        Ok(())
    }
}
