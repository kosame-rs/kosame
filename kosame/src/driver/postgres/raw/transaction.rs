use std::ops::{Deref, DerefMut};

use postgres::{Error, RowIter, Transaction, TransactionBuilder};
use postgres_types::ToSql;

use super::StatementCache;

pub struct RawTransactionBuilder<'a> {
    pub(super) inner: TransactionBuilder<'a>,
    pub(super) statement_cache: &'a mut StatementCache,
}

impl<'a> Deref for RawTransactionBuilder<'a> {
    type Target = TransactionBuilder<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for RawTransactionBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct RawTransaction<'a> {
    pub(super) inner: Transaction<'a>,
    pub(super) statement_cache: &'a mut StatementCache,
}

impl RawTransaction<'_> {
    pub fn query_cached<'a>(
        &'a mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<RowIter<'a>, Error> {
        let statement = self.statement_cache.prepare(&mut self.inner, query)?;
        self.inner.query_raw(&statement, params.iter().copied())
    }

    #[must_use]
    pub fn inner(&self) -> &Transaction<'_> {
        &self.inner
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

impl<'a> DerefMut for RawTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
