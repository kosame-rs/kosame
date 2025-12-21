use std::ops::{Deref, DerefMut};

use crate::driver::StandardPoolConfig;

use super::{Config, Connection};

pub type PoolConfig = StandardPoolConfig;

pub type PoolBuildError = deadpool::managed::BuildError;
pub type PoolError = deadpool::managed::PoolError<postgres::Error>;

#[derive(Clone)]
pub struct Pool {
    inner: deadpool::managed::Pool<Manager>,
}

impl Pool {
    pub fn new(connection_config: Config) -> Result<Self, PoolBuildError> {
        Ok(Self {
            inner: deadpool::managed::Pool::builder(Manager { connection_config })
                .config(StandardPoolConfig::default().into())
                .build()?,
        })
    }

    pub fn get(&self) -> Result<PoolConnection, PoolError> {
        Ok(PoolConnection {
            inner: pollster::block_on(self.inner.get())?,
        })
    }
}

pub struct PoolConnection {
    inner: deadpool::managed::Object<Manager>,
}

impl Deref for PoolConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PoolConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
struct Manager {
    connection_config: Config,
}

impl deadpool::managed::Manager for Manager {
    type Type = Connection;
    type Error = postgres::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        self.connection_config.connect()
    }

    async fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}
