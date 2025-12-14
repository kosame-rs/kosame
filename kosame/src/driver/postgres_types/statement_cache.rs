//! Generic statement caching implementation for PostgreSQL-compatible clients.
//!
//! This module provides a generic statement cache that can work with any database client
//! implementing the [`StatementCacheClient`] trait. The cache stores prepared statements
//! by their query string and parameter types, avoiding redundant preparation calls.
//!
//! Statement caching improves performance by reusing prepared statements across multiple
//! executions, reducing the overhead of query parsing and planning on the database server.

use std::{borrow::Cow, collections::HashMap};

use postgres_types::Type;

/// Trait for database clients that can prepare typed statements.
///
/// This trait abstracts over different PostgreSQL client implementations (synchronous and
/// asynchronous) to enable generic statement caching. Clients implementing this trait can
/// be used with [`StatementCache`] to automatically cache prepared statements.
///
/// # Associated Types
///
/// * `Statement` - The prepared statement type (must be cloneable for caching)
/// * `Error` - The error type returned by statement preparation
///
/// # Example
///
/// ```rust,ignore
/// impl StatementCacheClient for tokio_postgres::Client {
///     type Statement = tokio_postgres::Statement;
///     type Error = tokio_postgres::Error;
///
///     async fn prepare_typed(
///         &mut self,
///         query: &str,
///         types: &[Type],
///     ) -> Result<Self::Statement, Self::Error> {
///         Client::prepare_typed(self, query, types).await
///     }
/// }
/// ```
pub(in crate::driver) trait StatementCacheClient {
    /// The prepared statement type returned by this client.
    type Statement: Clone;

    /// The error type returned when statement preparation fails.
    type Error;

    /// Prepares a SQL statement with explicit parameter types.
    ///
    /// # Arguments
    ///
    /// * `query` - The SQL query string to prepare
    /// * `types` - The expected PostgreSQL types for query parameters
    ///
    /// # Errors
    ///
    /// Returns an error if statement preparation fails (e.g., syntax error, invalid types).
    async fn prepare_typed(
        &mut self,
        query: &str,
        types: &[Type],
    ) -> Result<Self::Statement, Self::Error>;
}

/// A cache for prepared SQL statements.
///
/// `StatementCache` stores prepared statements keyed by their query string and parameter types.
/// When a statement is requested, the cache first checks if it has already been prepared. If so,
/// it returns the cached statement; otherwise, it prepares the statement using the provided client
/// and caches it for future use.
///
/// # Type Parameters
///
/// * `C` - The database client type implementing [`StatementCacheClient`]
///
/// # Cache Key
///
/// Statements are cached using both the query string and parameter types as the key. This means
/// that the same query with different parameter types will be cached separately.
///
/// # Example
///
/// ```rust,ignore
/// use kosame::driver::postgres_types::StatementCache;
///
/// let mut cache = StatementCache::new();
/// let mut client = /* ... */;
///
/// // First call prepares and caches the statement
/// let stmt = cache.prepare(&mut client, "SELECT * FROM users WHERE id = $1").await?;
///
/// // Second call returns the cached statement (no database round-trip)
/// let stmt2 = cache.prepare(&mut client, "SELECT * FROM users WHERE id = $1").await?;
/// ```
#[derive(Default)]
pub struct StatementCache<C>
where
    C: StatementCacheClient,
{
    map: HashMap<Key<'static>, C::Statement>,
}

impl<C> StatementCache<C>
where
    C: StatementCacheClient,
{
    /// Creates a new empty statement cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Clears all cached statements.
    ///
    /// After calling this method, the cache will be empty and subsequent prepare calls
    /// will need to re-prepare statements with the database.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Retrieves a cached statement if available.
    ///
    /// # Arguments
    ///
    /// * `query` - The SQL query string
    /// * `types` - The parameter types for the query
    ///
    /// # Returns
    ///
    /// Returns `Some(statement)` if a matching statement is cached, or `None` if no cached
    /// statement exists for this query and parameter type combination.
    pub fn get(&self, query: &str, types: &[Type]) -> Option<C::Statement> {
        self.map.get(&Key::new(query, types)).map(ToOwned::to_owned)
    }

    /// Manually inserts a prepared statement into the cache.
    ///
    /// # Arguments
    ///
    /// * `query` - The SQL query string
    /// * `types` - The parameter types for the query
    /// * `statement` - The prepared statement to cache
    ///
    /// This method is rarely needed as [`prepare`](Self::prepare) and
    /// [`prepare_typed`](Self::prepare_typed) handle caching automatically.
    pub fn insert(&mut self, query: &str, types: &[Type], statement: C::Statement) {
        self.map
            .insert(Key::new(query, types).into_owned(), statement);
    }

    /// Prepares a SQL statement, using a cached version if available.
    ///
    /// This is a convenience method that calls [`prepare_typed`](Self::prepare_typed) with
    /// an empty parameter type array.
    ///
    /// # Arguments
    ///
    /// * `client` - The database client to use for preparing the statement
    /// * `query` - The SQL query string to prepare
    ///
    /// # Returns
    ///
    /// Returns the prepared statement, either from the cache or freshly prepared.
    ///
    /// # Errors
    ///
    /// Returns an error if statement preparation fails.
    pub async fn prepare(&mut self, client: &mut C, query: &str) -> Result<C::Statement, C::Error> {
        self.prepare_typed(client, query, &[]).await
    }

    /// Prepares a SQL statement with explicit parameter types, using a cached version if available.
    ///
    /// If a matching statement is already cached, it is returned immediately without contacting
    /// the database. Otherwise, the statement is prepared using the provided client, cached, and
    /// then returned.
    ///
    /// # Arguments
    ///
    /// * `client` - The database client to use for preparing the statement
    /// * `query` - The SQL query string to prepare
    /// * `types` - The expected PostgreSQL types for query parameters
    ///
    /// # Returns
    ///
    /// Returns the prepared statement, either from the cache or freshly prepared.
    ///
    /// # Errors
    ///
    /// Returns an error if statement preparation fails (only when not cached).
    pub async fn prepare_typed(
        &mut self,
        client: &mut C,
        query: &str,
        types: &[Type],
    ) -> Result<C::Statement, C::Error> {
        match self.get(query, types) {
            Some(statement) => Ok(statement),
            None => {
                let statement = client.prepare_typed(query, types).await?;
                self.insert(query, types, statement.clone());
                Ok(statement)
            }
        }
    }
}

/// Cache key combining query string and parameter types.
///
/// This struct is used internally by [`StatementCache`] to uniquely identify prepared statements.
/// Two statements are considered the same only if both their query strings and parameter type
/// arrays match exactly.
///
/// The lifetime parameter allows the key to borrow data temporarily during lookups while storing
/// owned data in the cache HashMap.
#[derive(Debug, Eq, PartialEq, Hash)]
struct Key<'a> {
    /// The SQL query string
    query: Cow<'a, str>,
    /// The PostgreSQL parameter types for the query
    types: Cow<'a, [Type]>,
}

impl<'a> Key<'a> {
    /// Creates a new cache key from borrowed data.
    ///
    /// # Arguments
    ///
    /// * `query` - The SQL query string
    /// * `types` - The parameter types for the query
    #[must_use]
    pub fn new(query: &'a str, types: &'a [Type]) -> Self {
        Self {
            query: query.into(),
            types: types.into(),
        }
    }

    /// Converts this key into an owned version with a `'static` lifetime.
    ///
    /// This is used when inserting keys into the cache's HashMap, which requires owned data.
    /// If the `Cow` fields are already owned, this is a zero-cost operation; otherwise, it
    /// allocates and copies the data.
    pub fn into_owned(self) -> Key<'static> {
        Key::<'static> {
            query: self.query.into_owned().into(),
            types: self.types.into_owned().into(),
        }
    }
}
