#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unexpected number of rows in result set")]
    RowCount,
    #[error("SQL formatting failed: {0}")]
    FmtSql(
        #[from]
        #[source]
        kosame_sql::Error,
    ),
    #[error("driver error: {0}")]
    Driver(
        #[from]
        #[source]
        Box<dyn std::error::Error>,
    ),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
