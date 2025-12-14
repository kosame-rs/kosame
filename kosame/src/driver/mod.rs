#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "tokio-postgres")]
pub mod tokio_postgres;

#[cfg(any(feature = "postgres", feature = "tokio-postgres"))]
#[doc(hidden)]
pub mod postgres_types;

pub trait Driver {
    type Dialect: kosame_sql::Dialect;
    type Params<'a>;
    type Row;
    type Error: std::error::Error + 'static;

    fn exec(
        &mut self,
        sql: &str,
        params: &Self::Params<'_>,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send;

    fn query(
        &mut self,
        sql: &str,
        params: &Self::Params<'_>,
    ) -> impl Future<Output = Result<Vec<Self::Row>, Self::Error>> + Send;
}

pub trait SyncDriver {
    type Dialect: kosame_sql::Dialect;
    type Params<'a>;
    type Row;
    type RowIter: Iterator<Item = Self::Row>;
    type Error: std::error::Error + 'static;

    fn execute(&mut self, sql: &str, params: &Self::Params<'_>) -> Result<u64, Self::Error>;
    fn query(&mut self, sql: &str, params: &Self::Params<'_>)
    -> Result<Self::RowIter, Self::Error>;
}
