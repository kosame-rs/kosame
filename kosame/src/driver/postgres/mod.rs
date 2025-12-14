mod cached_client;
mod statement_cache;

pub use cached_client::*;
pub use statement_cache::*;

use crate::driver::Driver;

impl Driver for postgres::Client {
    type Dialect = kosame_sql::postgres::Dialect;
    type Params<'a> = Vec<&'a (dyn postgres_types::ToSql + std::marker::Sync + 'a)>;
    type Row = postgres::Row;
    type Error = postgres::Error;

    async fn exec(&mut self, sql: &str, params: &Self::Params<'_>) -> Result<u64, Self::Error> {
        postgres::Client::execute(self, sql, params)
    }

    async fn query(
        &mut self,
        sql: &str,
        params: &Self::Params<'_>,
    ) -> Result<Vec<Self::Row>, Self::Error> {
        postgres::Client::query(self, sql, params)
    }
}

impl Driver for postgres::Transaction<'_> {
    type Dialect = kosame_sql::postgres::Dialect;
    type Params<'a> = Vec<&'a (dyn postgres_types::ToSql + std::marker::Sync + 'a)>;
    type Row = postgres::Row;
    type Error = postgres::Error;

    async fn exec(&mut self, sql: &str, params: &Self::Params<'_>) -> Result<u64, Self::Error> {
        postgres::Transaction::execute(self, sql, params)
    }

    async fn query(
        &mut self,
        sql: &str,
        params: &Self::Params<'_>,
    ) -> Result<Vec<Self::Row>, Self::Error> {
        postgres::Transaction::<'_>::query(self, sql, params)
    }
}
