use tokio_postgres::{Client, Error, Statement};

use crate::driver::postgres_types;

/// Implementation of [`postgres_types::StatementCacheClient`] for asynchronous PostgreSQL client.
///
/// This implementation allows [`tokio_postgres::Client`] to be used with the generic statement cache.
/// Unlike the synchronous `postgres` implementation, this one is truly asynchronous and uses `.await`
/// when preparing statements.
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

/// Type alias for a statement cache using the asynchronous PostgreSQL client.
///
/// This is a convenience type that specializes [`postgres_types::StatementCache`] for use with
/// [`tokio_postgres::Client`]. It provides all the same functionality as the generic cache.
pub type StatementCache = postgres_types::StatementCache<tokio_postgres::Client>;
