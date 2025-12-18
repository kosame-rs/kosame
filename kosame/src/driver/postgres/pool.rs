use crate::driver::StandardPoolConfig;

use super::Connection;

pub type PoolConfig = StandardPoolConfig;

pub struct Pool {
    inner: deadpool::managed::Pool<Manager>,
}

impl Pool {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: deadpool::managed::Pool::builder(Manager {})
                .config(StandardPoolConfig::default().into())
                .build(),
        }
    }
}

pub struct PoolConnection {
    inner: deadpool::managed::Object<Manager>,
}

struct Manager {}

impl deadpool::managed::Manager for Manager {
    type Type = Connection;
    type Error = postgres::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {}

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        todo!()
    }
}
