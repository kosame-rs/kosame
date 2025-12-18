use std::ops::{Deref, DerefMut};

use postgres::{Client, Error, Statement};
use postgres_types::Type;

use super::{RawTransaction, RawTransactionBuilder, StatementCache};

pub struct RawClient {
    inner: Client,
    statement_cache: StatementCache,
}

impl RawClient {
    #[must_use]
    pub fn new(inner: Client) -> Self {
        Self {
            inner,
            statement_cache: StatementCache::new(),
        }
    }

    pub fn prepare_cached(&mut self, query: &str) -> Result<Statement, Error> {
        self.statement_cache.prepare(&mut self.inner, query)
    }

    pub fn prepare_typed_cached(
        &mut self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        self.statement_cache
            .prepare_typed(&mut self.inner, query, types)
    }

    pub fn transaction(&mut self) -> Result<RawTransaction<'_>, Error> {
        Ok(RawTransaction {
            inner: self.inner.transaction()?,
            statement_cache: &mut self.statement_cache,
        })
    }

    pub fn build_transaction(&mut self) -> RawTransactionBuilder<'_> {
        RawTransactionBuilder {
            inner: self.inner.build_transaction(),
            statement_cache: &mut self.statement_cache,
        }
    }

    pub fn statement_cache(&self) -> &StatementCache {
        &self.statement_cache
    }

    pub fn statement_cache_mut(&mut self) -> &mut StatementCache {
        &mut self.statement_cache
    }
}

impl Deref for RawClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RawClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
