use std::fmt::Write;

use crate::{clause::{Values, Returning}, part::TargetTable};

pub struct Insert<'a> {
    target_table: TargetTable<'a>,
    values: Values<'a>,
    returning: Option<Returning<'a>>,
}

impl<'a> Insert<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(
        target_table: TargetTable<'a>,
        values: Values<'a>,
        returning: Option<Returning<'a>>,
    ) -> Self {
        Self {
            target_table,
            values,
            returning,
        }
    }

    #[inline]
    #[must_use] 
    pub const fn target_table(&self) -> &TargetTable<'a> {
        &self.target_table
    }

    #[inline]
    #[must_use] 
    pub const fn values(&self) -> &Values<'a> {
        &self.values
    }

    #[inline]
    #[must_use] 
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
        self.target_table.fmt_sql(formatter)?;

        self.values.fmt_sql(formatter)?;
        self.returning.fmt_sql(formatter)?;

        Ok(())
    }
}
