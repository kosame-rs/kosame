use crate::{clause, clause::*};

pub struct Select<'a> {
    select: clause::Select<'a>,
    from: Option<From<'a>>,
    r#where: Option<Where<'a>>,
    group_by: Option<GroupBy<'a>>,
    having: Option<Having<'a>>,
    order_by: Option<OrderBy<'a>>,
    limit: Option<Limit<'a>>,
    offset: Option<Offset<'a>>,
}

impl<'a> Select<'a> {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        select: clause::Select<'a>,
        from: Option<From<'a>>,
        r#where: Option<Where<'a>>,
        group_by: Option<GroupBy<'a>>,
        having: Option<Having<'a>>,
        order_by: Option<OrderBy<'a>>,
        limit: Option<Limit<'a>>,
        offset: Option<Offset<'a>>,
    ) -> Self {
        Self {
            select,
            from,
            r#where,
            group_by,
            having,
            order_by,
            limit,
            offset,
        }
    }

    #[inline]
    pub const fn select(&self) -> &clause::Select<'_> {
        &self.select
    }

    #[inline]
    pub const fn from(&self) -> Option<&From<'_>> {
        self.from.as_ref()
    }

    #[inline]
    pub const fn r#where(&self) -> Option<&Where<'_>> {
        self.r#where.as_ref()
    }

    #[inline]
    pub const fn group_by(&self) -> Option<&GroupBy<'_>> {
        self.group_by.as_ref()
    }

    #[inline]
    pub const fn having(&self) -> Option<&Having<'_>> {
        self.having.as_ref()
    }

    #[inline]
    pub const fn order_by(&self) -> Option<&OrderBy<'_>> {
        self.order_by.as_ref()
    }

    #[inline]
    pub const fn limit(&self) -> Option<&Limit<'_>> {
        self.limit.as_ref()
    }

    #[inline]
    pub const fn offset(&self) -> Option<&Offset<'_>> {
        self.offset.as_ref()
    }
}

impl kosame_sql::FmtSql for Select<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        self.select.fmt_sql(formatter)?;
        if let Some(inner) = self.from.as_ref() {
            inner.fmt_sql(formatter)?;
        }
        if let Some(inner) = self.r#where.as_ref() {
            inner.fmt_sql(formatter)?;
        }
        if let Some(inner) = self.group_by.as_ref() {
            inner.fmt_sql(formatter)?;
        }
        if let Some(inner) = self.having.as_ref() {
            inner.fmt_sql(formatter)?;
        }
        if let Some(inner) = self.order_by.as_ref() {
            inner.fmt_sql(formatter)?;
        }
        if let Some(inner) = self.limit.as_ref() {
            inner.fmt_sql(formatter)?;
        }
        if let Some(inner) = self.offset.as_ref() {
            inner.fmt_sql(formatter)?;
        }

        Ok(())
    }
}
