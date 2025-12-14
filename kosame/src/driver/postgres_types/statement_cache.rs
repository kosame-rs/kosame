use std::{borrow::Cow, collections::HashMap};

use postgres_types::Type;

#[derive(Default)]
pub struct StatementCache<S> {
    map: HashMap<Key<'static>, S>,
}

impl<S> StatementCache<S> {
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
    pub fn get(&self, query: &str, types: &[Type]) -> Option<S> {
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
    pub fn insert(&mut self, query: &str, types: &[Type], statement: S) {
        self.map
            .insert(Key::new(query, types).into_owned(), statement);
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
