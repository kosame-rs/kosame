use crate::part::ColumnList;

pub struct TableAlias<'a> {
    alias: &'a str,
    columns: Option<ColumnList<'a>>,
}

impl<'a> TableAlias<'a> {
    #[inline]
    #[must_use]
    pub const fn new(alias: &'a str, columns: Option<ColumnList<'a>>) -> Self {
        Self { alias, columns }
    }
}

impl kosame_sql::FmtSql for TableAlias<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_ident(self.alias)?;
        self.columns.fmt_sql(formatter)?;
        Ok(())
    }
}
