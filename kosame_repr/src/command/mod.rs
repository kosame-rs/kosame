mod delete;
mod insert;
mod select;
mod update;

pub use delete::*;
pub use insert::*;
pub use select::*;
pub use update::*;

use crate::clause::With;

pub struct Command<'a> {
    with: Option<With<'a>>,
    command_type: CommandType<'a>,
}

impl<'a> Command<'a> {
    #[inline]
    #[must_use] 
    pub const fn new(with: Option<With<'a>>, command_type: CommandType<'a>) -> Self {
        Self { with, command_type }
    }
}

impl kosame_sql::FmtSql for Command<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        self.with.fmt_sql(formatter)?;
        self.command_type.fmt_sql(formatter)
    }
}

pub enum CommandType<'a> {
    Delete(Delete<'a>),
    Insert(Insert<'a>),
    Select(Select<'a>),
    Update(Update<'a>),
}

impl kosame_sql::FmtSql for CommandType<'_> {
    fn fmt_sql<D>(&self, formatter: &mut kosame_sql::Formatter<D>) -> kosame_sql::Result
    where
        D: kosame_sql::Dialect,
    {
        match self {
            Self::Delete(inner) => inner.fmt_sql(formatter),
            Self::Insert(inner) => inner.fmt_sql(formatter),
            Self::Select(inner) => inner.fmt_sql(formatter),
            Self::Update(inner) => inner.fmt_sql(formatter),
        }
    }
}
