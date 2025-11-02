use std::fmt::Write;

use crate::{clause::*, schema::Table};

pub struct Insert<'a> {
    table: &'a Table<'a>,
    values: Values<'a>,
    returning: Option<Returning<'a>>,
}

impl<'a> Insert<'a> {
    #[inline]
    pub const fn new(
        table: &'a Table<'a>,
        values: Values<'a>,
        returning: Option<Returning<'a>>,
    ) -> Self {
        Self {
            table,
            values,
            returning,
        }
    }

    #[inline]
    pub const fn table(&self) -> &'a Table<'a> {
        self.table
    }

    #[inline]
    pub const fn values(&self) -> &Values<'a> {
        &self.values
    }

    #[inline]
    pub const fn returning(&self) -> Option<&Returning<'a>> {
        self.returning.as_ref()
    }
}

impl kosame_sql::FmtSql for Insert<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_str("insert into ")?;
        formatter.write_ident(self.table.name())?;

        self.values.fmt_sql(formatter)?;

        if let Some(returning) = &self.returning {
            returning.fmt_sql(formatter)?;
        }

        Ok(())
    }
}
