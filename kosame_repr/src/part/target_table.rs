use std::fmt::Write;

use crate::Ident;

pub struct TargetTable<'a> {
    table: Ident<'a>,
    alias: Option<Ident<'a>>,
}

impl<'a> TargetTable<'a> {
    #[inline]
    #[must_use]
    pub const fn new(table: &'a str, alias: Option<&'a str>) -> Self {
        Self {
            table: Ident::new(table),
            alias: Ident::from_option(alias),
        }
    }

    #[inline]
    #[must_use]
    pub const fn table(&self) -> Ident<'a> {
        self.table
    }

    #[inline]
    #[must_use]
    pub const fn alias(&self) -> Option<Ident<'a>> {
        self.alias
    }
}

impl kosame_sql::FmtSql for TargetTable<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        self.table.fmt_sql(formatter)?;
        if let Some(alias) = &self.alias {
            formatter.write_str(" as ")?;
            alias.fmt_sql(formatter)?;
        }
        Ok(())
    }
}
