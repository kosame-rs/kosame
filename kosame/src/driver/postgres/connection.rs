use super::{Transaction, TransactionBuilder, raw::RawClient};

pub type ConnectionConfig = postgres::Config;

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
