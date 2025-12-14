use tokio_postgres::{Client, Error, Statement};

use crate::driver::postgres_types;

impl postgres_types::StatementCacheClient for Client {
    type Statement = Statement;
    type Error = Error;

    async fn prepare_typed(
        &mut self,
        query: &str,
        types: &[postgres_types::Type],
    ) -> Result<Self::Statement, Self::Error> {
        Client::prepare_typed(self, query, types).await
    }
}

pub type StatementCache = postgres_types::StatementCache<postgres::Client>;
