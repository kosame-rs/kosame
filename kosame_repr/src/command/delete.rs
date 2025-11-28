use std::fmt::Write;

use crate::{
    clause::{FromChain, Returning, Where},
    part::TargetTable,
};

pub struct Delete<'a> {
    target_table: TargetTable<'a>,
    using: Option<FromChain<'a>>,
    r#where: Option<Where<'a>>,
    returning: Option<Returning<'a>>,
}

impl<'a> Delete<'a> {
    #[inline]
    #[must_use]
    pub const fn new(
        target_table: TargetTable<'a>,
        using: Option<FromChain<'a>>,
        r#where: Option<Where<'a>>,
        returning: Option<Returning<'a>>,
    ) -> Self {
        Self {
            target_table,
            using,
            r#where,
            returning,
        }
    }

    #[inline]
    #[must_use]
    pub const fn target_table(&self) -> &TargetTable<'a> {
        &self.target_table
    }
}

impl kosame_sql::FmtSql for Delete<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_str("delete from ")?;
        self.target_table.fmt_sql(formatter)?;

        if let Some(using) = &self.using {
            formatter.write_str(" using ")?;
            using.fmt_sql(formatter)?;
        }

        self.r#where.fmt_sql(formatter)?;
        self.returning.fmt_sql(formatter)?;

        Ok(())
    }
}
