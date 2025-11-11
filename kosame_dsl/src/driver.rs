use std::str::FromStr;

pub enum Driver {
    Postgres,
    TokioPostgres,
    Mysql,
    Rusqlite,
}

impl FromStr for Driver {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        match value {
            "postgres" => Ok(Self::Postgres),
            "tokio-postgres" => Ok(Self::TokioPostgres),
            "mysql" => Ok(Self::Mysql),
            "rusqlite" => Ok(Self::Rusqlite),
            _ => Err(()),
        }
    }
}
