mod runner;

pub use kosame_repr::query::*;
pub use runner::*;

use crate::{Error, driver::Connection, params::Params};
use pollster::FutureExt;

pub trait Query {
    type Params: std::fmt::Debug;
    type Row;

    const REPR: Node<'static>;

    fn repr(&self) -> &'static Node<'static> {
        &Self::REPR
    }

    fn params(&self) -> &Self::Params;

    fn query_vec<'c, C>(
        &self,
        connection: &mut C,
    ) -> impl Future<Output = crate::Result<Vec<Self::Row>>>
    where
        C: Connection,
        Self::Params: Params<C::Params<'c>>,
        for<'b> Self::Row: From<&'b C::Row>,
    {
        async { RecordArrayRunner {}.run(connection, self).await }
    }

    fn query_one<'c, C>(&self, connection: &mut C) -> impl Future<Output = crate::Result<Self::Row>>
    where
        C: Connection,
        Self::Params: Params<C::Params<'c>>,
        for<'b> Self::Row: From<&'b C::Row>,
    {
        async {
            self.query_opt(connection)
                .await
                .and_then(|res| res.ok_or(Error::RowCount))
        }
    }

    fn query_opt<'c, C>(
        &self,
        connection: &mut C,
    ) -> impl Future<Output = crate::Result<Option<Self::Row>>>
    where
        C: Connection,
        Self::Params: Params<C::Params<'c>>,
        for<'b> Self::Row: From<&'b C::Row>,
    {
        async {
            self.query_vec(connection).await.and_then(|res| {
                let mut iter = res.into_iter();
                let row = iter.next();
                if row.is_some() && iter.next().is_some() {
                    return Err(Error::RowCount);
                }
                Ok(row)
            })
        }
    }

    fn query_vec_sync<'c, C>(&self, connection: &mut C) -> crate::Result<Vec<Self::Row>>
    where
        C: Connection,
        Self::Params: Params<C::Params<'c>>,
        for<'b> Self::Row: From<&'b C::Row>,
    {
        self.query_vec(connection).block_on()
    }

    fn query_one_sync<'c, C>(&self, connection: &mut C) -> crate::Result<Self::Row>
    where
        C: Connection,
        Self::Params: Params<C::Params<'c>>,
        for<'b> Self::Row: From<&'b C::Row>,
    {
        self.query_one(connection).block_on()
    }

    fn query_opt_sync<'c, C>(&self, connection: &mut C) -> crate::Result<Option<Self::Row>>
    where
        C: Connection,
        Self::Params: Params<C::Params<'c>>,
        for<'b> Self::Row: From<&'b C::Row>,
    {
        self.query_opt(connection).block_on()
    }
}
