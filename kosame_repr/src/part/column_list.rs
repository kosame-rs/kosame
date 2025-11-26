use std::fmt::Write;

pub struct ColumnList<'a> {
    columns: &'a [&'a str],
}

impl<'a> ColumnList<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(columns: &'a [&'a str]) -> Self {
        Self { columns }
    }

    #[inline]
    #[must_use] 
    pub const fn columns(&self) -> &'a [&'a str] {
        self.columns
    }
}

impl kosame_sql::FmtSql for ColumnList<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_str(" (")?;
        kosame_sql::Punctuated::new(self.columns, ",").fmt_sql(formatter)?;
        formatter.write_str(")")?;
        Ok(())
    }
}
