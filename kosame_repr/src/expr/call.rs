use std::fmt::Write;

use super::Expr;

pub struct Call<'a> {
    function: &'a str,
    params: &'a [&'a Expr<'a>],
    keyword: bool,
}

impl<'a> Call<'a> {
    #[inline]
    #[must_use] 
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
        // Some functions like `coalesce` must not be quoted like an identifier, whereas others,
        // like `sum`, can be. User defined functions should be treated as identifiers.
        if self.keyword {
            formatter.write_str(self.function)?;
        } else {
            formatter.write_ident(self.function)?;
        }

        formatter.write_str("(")?;
        kosame_sql::Punctuated::new(self.params, ",").fmt_sql(formatter)?;
        formatter.write_str(")")?;
        Ok(())
    }
}
