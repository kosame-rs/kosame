use ::postgres_types::ToSql;
use postgres::{Client, Error, RowIter, Statement, Transaction};

use crate::driver::postgres_types;

/// Implementation of [`postgres_types::StatementCacheClient`] for synchronous PostgreSQL client.
///
/// This implementation allows [`postgres::Client`] to be used with the generic statement cache.
/// Note that despite the `async fn` signature in the trait, this implementation is actually
/// synchronous as the `postgres` crate itself is synchronous.
impl postgres_types::StatementCacheClient for Client {
    type Statement = Statement;
    type Error = Error;

    async fn prepare_typed(
        &mut self,
        query: &str,
        types: &[postgres_types::Type],
    ) -> Result<Self::Statement, Self::Error> {
        Client::prepare_typed(self, query, types)
    }
}

/// Type alias for a statement cache using the synchronous PostgreSQL client.
///
/// This is a convenience type that specializes [`postgres_types::StatementCache`] for use with
/// [`postgres::Client`]. It provides all the same functionality as the generic cache.
pub type StatementCache = postgres_types::StatementCache<postgres::Client>;

pub struct CachedClient {
    inner: Client,
    statement_cache: StatementCache,
}

impl CachedClient {
    #[must_use]
    pub fn new(inner: Client) -> Self {
        Self {
            inner,
            statement_cache: StatementCache::new(),
        }
    }

    pub async fn query_cached<T, P, I>(
        &mut self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<RowIter, Error> {
        let statement = self
            .statement_cache
            .prepare(&mut self.inner, statement.into())
            .await?;
        self.inner.query_raw(&statement, params.iter().map(|v| *v))
    }

    pub async fn transaction(&mut self) -> Result<CachedTransaction<'_>, Error> {
        Ok(CachedTransaction {
            inner: self.inner.transaction()?,
            statement_cache: &mut self.statement_cache,
        })
    }

    pub fn inner(&self) -> &Client {
        &self.inner
    }

    pub fn statement_cache(&self) -> &StatementCache {
        &self.statement_cache
    }

    pub fn statement_cache_mut(&mut self) -> &mut StatementCache {
        &mut self.statement_cache
    }
}

pub struct CachedTransaction<'a> {
    inner: Transaction<'a>,
    statement_cache: &'a mut StatementCache,
}

impl CachedTransaction<'_> {
    pub async fn query_cached<T, P, I>(
        &mut self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<RowIter, Error> {
        let statement = self
            .statement_cache
            .prepare(&mut self.inner.client(), statement.into())
            .await?;
        self.inner.query_raw(&statement, params.iter().map(|v| *v))
    }

    pub fn inner(&self) -> &Transaction {
        &self.inner
    }

    pub fn statement_cache(&self) -> &StatementCache {
        &self.statement_cache
    }

    pub fn statement_cache_mut(&mut self) -> &mut StatementCache {
        &mut self.statement_cache
    }
}
