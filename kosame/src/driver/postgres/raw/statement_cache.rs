use std::ops::{Deref, DerefMut};

use postgres::{Error, GenericClient, Statement};
use postgres_types::Type;

use crate::driver::postgres_types::StatementCache as GenericStatementCache;

#[derive(Debug)]
pub struct StatementCache {
    inner: GenericStatementCache<Statement>,
}

impl StatementCache {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: GenericStatementCache::new(capacity),
        }
    }

    pub fn prepare(
        &mut self,
        client: &mut impl GenericClient,
        query: &str,
    ) -> Result<Statement, Error> {
        self.prepare_typed(client, query, &[])
    }

    pub fn prepare_typed(
        &mut self,
        client: &mut impl GenericClient,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        if let Some(statement) = self.get(query, types) {
            Ok(statement)
        } else {
            let stmt = client.prepare_typed(query, types)?;
            self.insert(query, types, stmt.clone());
            Ok(stmt)
        }
    }
}

impl Deref for StatementCache {
    type Target = GenericStatementCache<Statement>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for StatementCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
