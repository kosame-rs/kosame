use std::{borrow::Cow, collections::HashMap};

use postgres_types::Type;

pub(crate) trait StatementCacheClient {
    type Statement: Clone;
    type Error;

    async fn prepare_typed(
        &mut self,
        query: &str,
        types: &[Type],
    ) -> Result<Self::Statement, Self::Error>;
}

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
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn get(&self, query: &str, types: &[Type]) -> Option<C::Statement> {
        self.map.get(&Key::new(query, types)).map(ToOwned::to_owned)
    }

    pub fn insert(&mut self, query: &str, types: &[Type], statement: C::Statement) {
        self.map
            .insert(Key::new(query, types).into_owned(), statement);
    }

    pub async fn prepare(&mut self, client: &mut C, query: &str) -> Result<C::Statement, C::Error> {
        self.prepare_typed(client, query, &[]).await
    }

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

#[derive(Debug, Eq, PartialEq, Hash)]
struct Key<'a> {
    query: Cow<'a, str>,
    types: Cow<'a, [Type]>,
}

impl<'a> Key<'a> {
    #[must_use]
    pub fn new(query: &'a str, types: &'a [Type]) -> Self {
        Self {
            query: query.into(),
            types: types.into(),
        }
    }

    pub fn into_owned(self) -> Key<'static> {
        Key::<'static> {
            query: self.query.into_owned().into(),
            types: self.types.into_owned().into(),
        }
    }
}
