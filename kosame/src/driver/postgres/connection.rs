use std::str::FromStr;

use super::{
    Transaction, TransactionBuilder,
    raw::{RawClient, RawConnectionConfig},
};

#[derive(Debug, Default)]
pub struct ConnectionConfig {
    inner: RawConnectionConfig,
}

impl ConnectionConfig {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RawConnectionConfig::new(),
        }
    }

    pub fn connect(&self) -> Result<Connection, postgres::Error> {
        Ok(Connection {
            // TODO: Support TLS
            inner: RawClient::new(self.inner.connect(postgres::NoTls)?),
        })
    }
}

impl FromStr for ConnectionConfig {
    type Err = <RawConnectionConfig as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ConnectionConfig { inner: s.parse()? })
    }
}

pub struct Connection {
    inner: RawClient,
}

impl Connection {
    pub fn transaction(&mut self) -> Result<Transaction, postgres::Error> {
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
