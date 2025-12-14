use std::ops::{Deref, DerefMut};

use postgres_types::Type;
use tokio_postgres::{Client, Error, Statement};

use crate::driver::postgres_types::StatementCache as GenericStatementCache;

pub struct StatementCache {
    inner: GenericStatementCache<Statement>,
}

impl StatementCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: GenericStatementCache::new(),
        }
    }

    pub async fn prepare(&mut self, client: &Client, query: &str) -> Result<Statement, Error> {
        self.prepare_typed(client, query, &[]).await
    }

    pub async fn prepare_typed(
        &mut self,
        client: &Client,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        match self.get(query, types) {
            Some(statement) => Ok(statement),
            None => {
                let stmt = client.prepare_typed(query, types).await?;
                self.insert(query, types, stmt.clone());
                Ok(stmt)
            }
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
