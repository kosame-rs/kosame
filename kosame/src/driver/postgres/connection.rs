use std::str::FromStr;

use super::{
    Transaction, TransactionBuilder,
    raw::{RawClient, RawConfig},
};

const DEFAULT_STATEMENT_CACHE_CAPACITY: usize = 64;

#[derive(Debug, Default)]
pub struct Config {
    inner: RawConfig,
    statement_cache_capacity: usize,
}

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RawConfig::new(),
            statement_cache_capacity: DEFAULT_STATEMENT_CACHE_CAPACITY,
        }
    }

    #[must_use]
    pub fn statement_cache_capacity(mut self, capacity: usize) -> Self {
        self.statement_cache_capacity = capacity;
        self
    }

    pub fn connect(&self) -> Result<Connection, postgres::Error> {
        Ok(Connection {
            // TODO: Support TLS
            inner: RawClient::new(
                self.inner.connect(postgres::NoTls)?,
                self.statement_cache_capacity,
            ),
        })
    }
}

impl FromStr for Config {
    type Err = <RawConfig as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Config {
            inner: s.parse()?,
            statement_cache_capacity: DEFAULT_STATEMENT_CACHE_CAPACITY,
        })
    }
}

pub struct Connection {
    inner: RawClient,
}

impl Connection {
    pub fn transaction(&mut self) -> Result<Transaction<'_>, postgres::Error> {
        Ok(Transaction {
            inner: self.inner.transaction()?,
        })
    }

    pub fn build_transaction(&mut self) -> TransactionBuilder<'_> {
        TransactionBuilder {
            inner: self.inner.build_transaction(),
        }
    }

    pub fn raw(&self) -> &RawClient {
        &self.inner
    }

    pub fn raw_mut(&mut self) -> &mut RawClient {
        &mut self.inner
    }
}
