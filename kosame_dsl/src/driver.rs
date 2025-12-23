use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Driver {
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

impl FromStr for Driver {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        match value {
            "postgres" => Ok(Self::Postgres),
            "tokio-postgres" => Ok(Self::TokioPostgres),

            "mysql" => Ok(Self::Mysql),
            "mysql_async" => Ok(Self::MysqlAsync),

            "rusqlite" => Ok(Self::Rusqlite),
            "tokio_rusqlite" => Ok(Self::TokioRusqlite),
            _ => Err(()),
        }
    }
}
