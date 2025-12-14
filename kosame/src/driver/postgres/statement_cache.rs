use ::postgres_types::ToSql;
use postgres::{Client, Error, RowIter, Statement, Transaction};

use crate::driver::postgres_types;

/// Type alias for a statement cache using the synchronous PostgreSQL client.
///
/// This is a convenience type that specializes [`postgres_types::StatementCache`] for use with
/// [`postgres::Client`]. It provides all the same functionality as the generic cache.
pub type StatementCache = postgres_types::StatementCache<Statement>;

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
        let statement = match self.statement_cache.get(query, &[]) {
            Some(statement) => statement,
            None => {
                let statement = self.inner.prepare(query)?;
                self.statement_cache.insert(query, &[], statement.clone());
                statement
            }
        };
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
        let statement = match self.statement_cache.get(query, &[]) {
            Some(statement) => statement,
            None => {
                let statement = self.inner.prepare(query)?;
                self.statement_cache.insert(query, &[], statement.clone());
                statement
            }
        };
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
