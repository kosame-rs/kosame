use std::fmt::Write;

use crate::expr::Expr;

pub struct Having<'a> {
    expr: Expr<'a>,
}

impl<'a> Having<'a> {
    #[inline]
    #[must_use]
    pub const fn new(expr: Expr<'a>) -> Self {
        Self { expr }
    }

    #[inline]
    #[must_use]
    pub fn expr(&self) -> &Expr<'_> {
        &self.expr
    }
}

impl kosame_sql::FmtSql for Having<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        formatter.write_str(" having ")?;
        self.expr.fmt_sql(formatter)?;
        Ok(())
    }
}
