use std::fmt::Write;

use super::Expr;

pub struct Call<'a> {
    function: &'a str,
    params: &'a [&'a Expr<'a>],
    keyword: bool,
}

impl<'a> Call<'a> {
    #[inline]
    pub const fn new(function: &'a str, params: &'a [&'a Expr], keyword: bool) -> Self {
        Self {
            function,
            params,
            keyword,
        }
    }
}

impl kosame_sql::FmtSql for Call<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        if self.keyword {
            formatter.write_str(self.function)?;
        } else {
            formatter.write_ident(self.function)?;
        }
        formatter.write_str("(")?;
        for (index, param) in self.params.iter().enumerate() {
            param.fmt_sql(formatter)?;
            if index != self.params.len() - 1 {
                formatter.write_str(", ")?;
            }
        }
        formatter.write_str(")")?;
        Ok(())
    }
}
