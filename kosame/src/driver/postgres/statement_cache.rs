//! Statement cache for synchronous PostgreSQL connections.
//!
//! This module provides a statement cache implementation for the synchronous `postgres` crate.
//! It implements the [`postgres_types::StatementCacheClient`] trait for [`postgres::Client`],
//! allowing prepared statements to be cached and reused across multiple query executions.
//!
//! # Example
//!
//! ```rust,ignore
//! use kosame::driver::postgres::StatementCache;
//! use postgres::Client;
//!
//! let mut client = Client::connect("postgresql://user:pass@localhost/db", NoTls)?;
//! let mut cache = StatementCache::new();
//!
//! // First call prepares the statement
//! let stmt = cache.prepare(&mut client, "SELECT * FROM users WHERE id = $1").await?;
//!
//! // Second call reuses the cached statement
//! let stmt2 = cache.prepare(&mut client, "SELECT * FROM users WHERE id = $1").await?;
//! ```

use postgres::{Client, Error, Statement};

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
