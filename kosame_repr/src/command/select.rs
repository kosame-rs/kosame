use std::fmt::Write;

use crate::{
    clause::{Limit, Offset, OrderBy, SelectCore},
    command::Command,
};

pub struct Select<'a> {
    chain: SelectChain<'a>,
    order_by: Option<OrderBy<'a>>,
    limit: Option<Limit<'a>>,
    offset: Option<Offset<'a>>,
}

impl<'a> Select<'a> {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        chain: SelectChain<'a>,
        order_by: Option<OrderBy<'a>>,
        limit: Option<Limit<'a>>,
        offset: Option<Offset<'a>>,
    ) -> Self {
        Self {
            chain,
            order_by,
            limit,
            offset,
        }
    }

    #[inline]
    #[must_use]
    pub const fn chain(&self) -> &SelectChain<'a> {
        &self.chain
    }

    #[inline]
    #[must_use]
    pub const fn order_by(&self) -> Option<&OrderBy<'a>> {
        self.order_by.as_ref()
    }

    #[inline]
    #[must_use]
    pub const fn limit(&self) -> Option<&Limit<'a>> {
        self.limit.as_ref()
    }

    #[inline]
    #[must_use]
    pub const fn offset(&self) -> Option<&Offset<'a>> {
        self.offset.as_ref()
    }
}

impl kosame_sql::FmtSql for Select<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        self.chain.fmt_sql(formatter)?;
        self.order_by.fmt_sql(formatter)?;
        self.limit.fmt_sql(formatter)?;
        self.offset.fmt_sql(formatter)?;
        Ok(())
    }
}

pub struct SelectChain<'a> {
    start: SelectItem<'a>,
    combinators: &'a [SelectCombinator<'a>],
}

impl<'a> SelectChain<'a> {
    #[inline]
    #[must_use]
    pub const fn new(start: SelectItem<'a>, combinators: &'a [SelectCombinator<'a>]) -> Self {
        Self { start, combinators }
    }
}

impl kosame_sql::FmtSql for SelectChain<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        self.start.fmt_sql(formatter)?;
        for combinator in self.combinators {
            combinator.fmt_sql(formatter)?;
        }
        Ok(())
    }
}

pub enum SelectItem<'a> {
    Core(SelectCore<'a>),
    Paren(&'a Command<'a>),
}

impl kosame_sql::FmtSql for SelectItem<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        match self {
            Self::Core(core) => {
                core.fmt_sql(formatter)?;
            }
            Self::Paren(select) => {
                formatter.write_str("(")?;
                select.fmt_sql(formatter)?;
                formatter.write_str(")")?;
            }
        }
        Ok(())
    }
}

pub enum SetOp {
    Union,
    Intersect,
    Except,
}

impl kosame_sql::FmtSql for SetOp {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        match self {
            Self::Union => formatter.write_str("union"),
            Self::Intersect => formatter.write_str("intersect"),
            Self::Except => formatter.write_str("except"),
        }
    }
}

pub enum SetQuantifier {
    All,
    Distinct,
}

impl kosame_sql::FmtSql for SetQuantifier {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        match self {
            Self::All => formatter.write_str(" all"),
            Self::Distinct => Ok(()),
        }
    }
}

pub struct SelectCombinator<'a> {
    op: SetOp,
    quantifier: SetQuantifier,
    right: SelectItem<'a>,
}

impl<'a> SelectCombinator<'a> {
    #[inline]
    #[must_use]
    pub const fn new(op: SetOp, quantifier: SetQuantifier, right: SelectItem<'a>) -> Self {
        Self {
            op,
            quantifier,
            right,
        }
    }
}

impl kosame_sql::FmtSql for SelectCombinator<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_str(" ")?;
        self.op.fmt_sql(formatter)?;
        self.quantifier.fmt_sql(formatter)?;
        formatter.write_str(" ")?;
        self.right.fmt_sql(formatter)?;
        Ok(())
    }
}
