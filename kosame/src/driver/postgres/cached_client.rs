use ::postgres_types::ToSql;
use postgres::{Client, Error, RowIter, Transaction};

use crate::driver::postgres::StatementCache;

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

    pub fn query_cached<T, P, I>(
        &mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<RowIter, Error> {
        let statement = self.statement_cache.prepare(&mut self.inner, query)?;
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
    pub fn query_cached<T, P, I>(
        &mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<RowIter, Error> {
        let statement = self.statement_cache.prepare(&mut self.inner, query)?;
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
