use std::fmt::Write;

use crate::Ident;

pub struct ColumnRef<'a> {
    correlation: Option<Ident<'a>>,
    column: Ident<'a>,
}

impl<'a> ColumnRef<'a> {
    #[inline]
    #[must_use]
    pub const fn new(correlation: Option<&'a str>, column: &'a str) -> Self {
        Self {
            correlation: Ident::from_option(correlation),
            column: Ident::new(column),
        }
    }
}

impl kosame_sql::FmtSql for ColumnRef<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        if let Some(correlation) = &self.correlation {
            correlation.fmt_sql(formatter)?;
            formatter.write_str(".")?;
        }
        self.column.fmt_sql(formatter)?;
        Ok(())
    }
}
