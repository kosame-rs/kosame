mod record_array;

pub use record_array::*;

use crate::{driver::Connection, params::Params};

use super::{Query, Node, Field};

pub trait Runner {
    fn run<'a, C, Q>(
        &self,
        connection: &mut C,
        query: &Q,
    ) -> impl Future<Output = crate::Result<Vec<Q::Row>>>
    where
        C: Connection,
        Q: Query + ?Sized,
        Q::Params: Params<C::Params<'a>>,
        for<'b> Q::Row: From<&'b C::Row>;
}
