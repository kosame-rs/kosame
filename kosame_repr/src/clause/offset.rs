use std::fmt::Write;

use crate::expr::Expr;

pub struct Offset<'a> {
    expr: Expr<'a>,
}

impl<'a> Offset<'a> {
    #[inline]
    #[must_use]
    pub const fn new(expr: Expr<'a>) -> Self {
        Self { expr }
    }
}

impl kosame_sql::FmtSql for Offset<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        formatter.write_str(" offset ")?;
        self.expr.fmt_sql(formatter)?;
        Ok(())
    }
}
