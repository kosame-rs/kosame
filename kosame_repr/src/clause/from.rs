use std::fmt::Write;

use crate::{command::Command, expr::Expr, part::TableAlias};

pub struct From<'a> {
    chain: FromChain<'a>,
}

impl<'a> From<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(chain: FromChain<'a>) -> Self {
        Self { chain }
    }

    #[inline]
    #[must_use] 
    pub const fn chain(&self) -> &FromChain<'a> {
        &self.chain
    }
}

impl kosame_sql::FmtSql for From<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        formatter.write_str(" from ")?;
        self.chain.fmt_sql(formatter)?;
        Ok(())
    }
}

pub struct FromChain<'a> {
    start: FromItem<'a>,
    combinators: &'a [FromCombinator<'a>],
}

impl<'a> FromChain<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(start: FromItem<'a>, combinators: &'a [FromCombinator<'a>]) -> Self {
        Self { start, combinators }
    }
}

impl kosame_sql::FmtSql for FromChain<'_> {
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

impl kosame_sql::FmtSql for JoinType {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        match self {
            Self::Inner => formatter.write_str(" inner join "),
            Self::Left => formatter.write_str(" left join "),
            Self::Right => formatter.write_str(" right join "),
            Self::Full => formatter.write_str(" full join "),
        }
    }
}

pub enum FromItem<'a> {
    Table {
        table: &'a str,
        alias: Option<TableAlias<'a>>,
    },
    Subquery {
        lateral: bool,
        command: &'a Command<'a>,
        alias: Option<TableAlias<'a>>,
    },
}

impl kosame_sql::FmtSql for FromItem<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        match self {
            Self::Table { table, alias } => {
                formatter.write_ident(table)?;
                if let Some(alias) = alias {
                    formatter.write_str(" as ")?;
                    alias.fmt_sql(formatter)?;
                }
            }
            Self::Subquery {
                lateral,
                command: select,
                alias,
            } => {
                if *lateral {
                    formatter.write_str("lateral ")?;
                }
                formatter.write_str("(")?;
                select.fmt_sql(formatter)?;
                formatter.write_str(")")?;
                if let Some(alias) = alias {
                    formatter.write_str(" as ")?;
                    alias.fmt_sql(formatter)?;
                }
            }
        }

        Ok(())
    }
}

pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

pub enum FromCombinator<'a> {
    Join {
        join_type: JoinType,
        right: FromItem<'a>,
        on: Expr<'a>,
    },
    NaturalJoin {
        join_type: JoinType,
        right: FromItem<'a>,
    },
    CrossJoin {
        right: FromItem<'a>,
    },
}

impl kosame_sql::FmtSql for FromCombinator<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        match self {
            Self::Join {
                join_type,
                right,
                on,
            } => {
                join_type.fmt_sql(formatter)?;
                right.fmt_sql(formatter)?;
                formatter.write_str(" on ")?;
                on.fmt_sql(formatter)?;
            }
            Self::NaturalJoin { join_type, right } => {
                formatter.write_str(" natural")?;
                join_type.fmt_sql(formatter)?;
                right.fmt_sql(formatter)?;
            }
            Self::CrossJoin { right } => {
                formatter.write_str(" cross join ")?;
                right.fmt_sql(formatter)?;
            }
        }
        Ok(())
    }
}
