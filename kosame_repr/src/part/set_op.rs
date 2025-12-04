use std::fmt::Write;

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
