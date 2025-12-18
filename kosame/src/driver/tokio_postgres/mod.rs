mod connection;
mod raw;
mod statement_cache;

pub use connection::*;
pub use raw::*;
pub use statement_cache::*;

struct Driver {}

impl crate::driver::Driver for Driver {
    type Pool = Pool;
}

struct Pool {}

impl crate::driver::Pool for Pool {
    type PoolConnection = PoolConnection;

    #[allow(refining_impl_trait)]
    async fn get(&self) -> crate::Result<Self::PoolConnection> {
        unimplemented!();
    }
}

struct PoolConnection {}

impl crate::driver::PoolConnection for PoolConnection {}
