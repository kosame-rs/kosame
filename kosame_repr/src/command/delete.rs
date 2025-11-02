use std::fmt::Write;

use crate::{clause::*, schema::Table};

pub struct Delete<'a> {
    table: &'a Table<'a>,
    using: Option<FromItem<'a>>,
    r#where: Option<Where<'a>>,
    returning: Option<Returning<'a>>,
}

impl<'a> Delete<'a> {
    #[inline]
    pub const fn new(
        table: &'a Table<'a>,
        using: Option<FromItem<'a>>,
        r#where: Option<Where<'a>>,
        returning: Option<Returning<'a>>,
    ) -> Self {
        Self {
            table,
            using,
            r#where,
            returning,
        }
    }

    #[inline]
    pub const fn table(&self) -> &'a Table<'a> {
        self.table
    }
}

impl kosame_sql::FmtSql for Delete<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_str("delete from ")?;
        formatter.write_ident(self.table.name())?;

        if let Some(using) = &self.using {
            formatter.write_str(" using ")?;
            using.fmt_sql(formatter)?;
        }
        if let Some(r#where) = &self.r#where {
            r#where.fmt_sql(formatter)?;
        }
        if let Some(returning) = &self.returning {
            returning.fmt_sql(formatter)?;
        }

        Ok(())
    }
}
