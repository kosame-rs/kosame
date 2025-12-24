use std::{fmt::Display, str::FromStr};

use crate::{database::DatabaseKind, runtime::RuntimeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverKind {
    // Postgres
    Postgres,
    TokioPostgres,

    // MySQL
    Mysql,
    MysqlAsync,

    // SQLite
    Rusqlite,
    TokioRusqlite,
}

impl DriverKind {
    /// Returns the database management system this driver communicates with.
    #[must_use]
    pub fn database(self) -> DatabaseKind {
        match self {
            Self::Postgres => DatabaseKind::Postgres,
            Self::TokioPostgres => DatabaseKind::Postgres,
            Self::Mysql => DatabaseKind::Mysql,
            Self::MysqlAsync => DatabaseKind::Mysql,
            Self::Rusqlite => DatabaseKind::Sqlite,
            Self::TokioRusqlite => DatabaseKind::Sqlite,
        }
    }

    /// Returns the asynchronous runtime this driver requires, or `None` if it is a synchronous
    /// (blocking) driver. A driver counts as blocking even if it uses an asynchronous runtime internally.
    #[must_use]
    pub fn runtime(self) -> Option<RuntimeKind> {
        match self {
            Self::Postgres => None,
            Self::TokioPostgres => Some(RuntimeKind::Tokio),
            Self::Mysql => None,
            Self::MysqlAsync => Some(RuntimeKind::Tokio),
            Self::Rusqlite => None,
            Self::TokioRusqlite => Some(RuntimeKind::Tokio),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseDriverError;

impl FromStr for DriverKind {
    type Err = ParseDriverError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "postgres" => Ok(Self::Postgres),
            "tokio-postgres" => Ok(Self::TokioPostgres),

            "mysql" => Ok(Self::Mysql),
            "mysql_async" => Ok(Self::MysqlAsync),

            "rusqlite" => Ok(Self::Rusqlite),
            "tokio_rusqlite" => Ok(Self::TokioRusqlite),
            _ => Err(ParseDriverError),
        }
    }
}

impl Display for DriverKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgres => f.write_str("postgres"),
            Self::TokioPostgres => f.write_str("tokio-postgres"),
            Self::Mysql => f.write_str("mysql"),
            Self::MysqlAsync => f.write_str("mysql_async"),
            Self::Rusqlite => f.write_str("rusqlite"),
            Self::TokioRusqlite => f.write_str("tokio_rusqlite"),
        }
    }
}
