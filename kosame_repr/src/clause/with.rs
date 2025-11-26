use std::fmt::Write;

use crate::{command::Command, part::TableAlias};

pub struct With<'a> {
    items: &'a [WithItem<'a>],
}

impl<'a> With<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(items: &'a [WithItem]) -> Self {
        Self { items }
    }
}

impl kosame_sql::FmtSql for With<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        formatter.write_str("with ")?;
        kosame_sql::Punctuated::new(self.items, ",").fmt_sql(formatter)?;
        formatter.write_str(" ")?;
        Ok(())
    }
}

pub struct WithItem<'a> {
    alias: TableAlias<'a>,
    command: Command<'a>,
}

impl<'a> WithItem<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(alias: TableAlias<'a>, command: Command<'a>) -> Self {
        Self { alias, command }
    }
}

impl kosame_sql::FmtSql for WithItem<'_> {
    #[inline]
    fn fmt_sql<D: kosame_sql::Dialect>(
        &self,
        formatter: &mut kosame_sql::Formatter<D>,
    ) -> kosame_sql::Result {
        self.alias.fmt_sql(formatter)?;
        formatter.write_str(" as (")?;
        self.command.fmt_sql(formatter)?;
        formatter.write_str(")")?;
        Ok(())
    }
}
