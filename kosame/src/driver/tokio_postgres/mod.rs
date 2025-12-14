mod cached_client;
mod statement_cache;

pub use cached_client::*;
pub use statement_cache::*;

use crate::driver::Driver;

impl Driver for tokio_postgres::Client {
    type Dialect = kosame_sql::postgres::Dialect;
    type Params<'a> = Vec<&'a (dyn postgres_types::ToSql + std::marker::Sync + 'a)>;
    type Row = tokio_postgres::Row;
    type Error = tokio_postgres::Error;

    async fn exec(&mut self, sql: &str, params: &Self::Params<'_>) -> Result<u64, Self::Error> {
        tokio_postgres::Client::execute(self, sql, params).await
    }

    async fn query(
        &mut self,
        sql: &str,
        params: &Self::Params<'_>,
    ) -> Result<Vec<Self::Row>, Self::Error> {
        tokio_postgres::Client::query(self, sql, params).await
    }
}

impl Driver for tokio_postgres::Transaction<'_> {
    type Dialect = kosame_sql::postgres::Dialect;
    type Params<'a> = Vec<&'a (dyn postgres_types::ToSql + std::marker::Sync + 'a)>;
    type Row = tokio_postgres::Row;
    type Error = tokio_postgres::Error;

    async fn exec(&mut self, sql: &str, params: &Self::Params<'_>) -> Result<u64, Self::Error> {
        tokio_postgres::Transaction::execute(self, sql, params).await
    }

    async fn query(
        &mut self,
        sql: &str,
        params: &Self::Params<'_>,
    ) -> Result<Vec<Self::Row>, Self::Error> {
        tokio_postgres::Transaction::<'_>::query(self, sql, params).await
    }
}
