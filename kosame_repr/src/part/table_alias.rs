use crate::{Ident, part::ColumnList};

pub struct TableAlias<'a> {
    alias: Ident<'a>,
    columns: Option<ColumnList<'a>>,
}

impl<'a> TableAlias<'a> {
    #[inline]
    #[must_use]
    pub const fn new(alias: &'a str, columns: Option<ColumnList<'a>>) -> Self {
        Self {
            alias: Ident::new(alias),
            columns,
        }
    }
}

impl kosame_sql::FmtSql for TableAlias<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        self.alias.fmt_sql(formatter)?;
        self.columns.fmt_sql(formatter)?;
        Ok(())
    }
}
