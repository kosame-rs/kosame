use std::ops::{Deref, DerefMut};

use postgres::{Error, Statement, Transaction, TransactionBuilder};
use postgres_types::Type;

use super::StatementCache;

pub struct RawTransactionBuilder<'a> {
    pub(super) inner: TransactionBuilder<'a>,
    pub(super) statement_cache: &'a mut StatementCache,
}

impl<'a> RawTransactionBuilder<'a> {
    pub fn start(self) -> Result<RawTransaction<'a>, postgres::Error> {
        Ok(RawTransaction {
            inner: self.inner.start()?,
            statement_cache: self.statement_cache,
        })
    }
}

impl<'a> Deref for RawTransactionBuilder<'a> {
    type Target = TransactionBuilder<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RawTransactionBuilder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct RawTransaction<'a> {
    pub(super) inner: Transaction<'a>,
    pub(super) statement_cache: &'a mut StatementCache,
}

impl RawTransaction<'_> {
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

    pub fn commit(self) -> Result<(), postgres::Error> {
        self.inner.commit()
    }

    pub fn rollback(self) -> Result<(), postgres::Error> {
        self.inner.rollback()
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

impl<'a> Deref for RawTransaction<'a> {
    type Target = Transaction<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RawTransaction<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
