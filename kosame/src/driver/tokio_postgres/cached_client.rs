use postgres_types::ToSql;
use tokio_postgres::{Client, Error, RowStream, Transaction};

use crate::driver::tokio_postgres::StatementCache;

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
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<RowStream, Error> {
        let statement = self.statement_cache.prepare(&self.inner, query).await?;
        self.inner
            .query_raw(&statement, params.iter().copied())
            .await
    }

    pub async fn transaction(&mut self) -> Result<CachedTransaction<'_>, Error> {
        Ok(CachedTransaction {
            inner: self.inner.transaction().await?,
            statement_cache: &mut self.statement_cache,
        })
    }

    #[must_use]
    pub fn statement_cache(&self) -> &StatementCache {
        &self.statement_cache
    }

    #[must_use]
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
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<RowStream, Error> {
        let statement = self
            .statement_cache
            .prepare(self.inner.client(), query)
            .await?;
        self.inner
            .query_raw(&statement, params.iter().copied())
            .await
    }

    #[must_use]
    pub fn statement_cache(&self) -> &StatementCache {
        self.statement_cache
    }

    #[must_use]
    pub fn statement_cache_mut(&mut self) -> &mut StatementCache {
        self.statement_cache
    }
}
