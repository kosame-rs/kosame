use std::fmt::Write;

use crate::expr::Expr;

pub struct OrderBy<'a> {
    items: &'a [OrderByItem<'a>],
}

impl<'a> OrderBy<'a> {
    #[inline]
    #[must_use]
    pub const fn new(items: &'a [OrderByItem]) -> Self {
        Self { items }
    }
}

impl kosame_sql::FmtSql for OrderBy<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        formatter.write_str(" order by ")?;
        kosame_sql::Punctuated::new(self.items, ",").fmt_sql(formatter)?;
        Ok(())
    }
}

pub struct OrderByItem<'a> {
    expr: Expr<'a>,
    dir: Option<OrderByDir>,
    nulls: Option<OrderByNulls>,
}

impl<'a> OrderByItem<'a> {
    #[inline]
    #[must_use]
    pub const fn new(expr: Expr<'a>, dir: Option<OrderByDir>, nulls: Option<OrderByNulls>) -> Self {
        Self { expr, dir, nulls }
    }
}

impl kosame_sql::FmtSql for OrderByItem<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        self.expr.fmt_sql(formatter)?;
        match self.dir {
            Some(OrderByDir::Asc) => formatter.write_str(" asc")?,
            Some(OrderByDir::Desc) => formatter.write_str(" desc")?,
            None => {}
        }
        match self.nulls {
            Some(OrderByNulls::First) => formatter.write_str(" nulls first")?,
            Some(OrderByNulls::Last) => formatter.write_str(" nulls last")?,
            None => {}
        }
        Ok(())
    }
}

pub enum OrderByDir {
    Asc,
    Desc,
}

pub enum OrderByNulls {
    First,
    Last,
}
