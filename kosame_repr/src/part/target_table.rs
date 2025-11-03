use std::fmt::Write;

pub struct TargetTable<'a> {
    table: &'a str,
    alias: Option<&'a str>,
}

impl<'a> TargetTable<'a> {
    #[inline]
    pub const fn new(table: &'a str, alias: Option<&'a str>) -> Self {
        Self { table, alias }
    }

    #[inline]
    pub const fn table(&self) -> &'a str {
        self.table
    }

    #[inline]
    pub const fn alias(&self) -> Option<&'a str> {
        self.alias
    }
}

impl kosame_sql::FmtSql for TargetTable<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_ident(self.table)?;
        if let Some(alias) = &self.alias {
            formatter.write_str(" as ")?;
            formatter.write_ident(alias)?;
        }
        Ok(())
    }
}
