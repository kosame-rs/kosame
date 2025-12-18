use super::raw::{RawTransaction, RawTransactionBuilder};

pub struct TransactionBuilder<'a> {
    pub(super) inner: RawTransactionBuilder<'a>,
}

impl<'a> TransactionBuilder<'a> {
    pub fn raw(&self) -> &RawTransactionBuilder<'a> {
        &self.inner
    }

    pub fn raw_mut(&mut self) -> &mut RawTransactionBuilder<'a> {
        &mut self.inner
    }
}

pub struct Transaction<'a> {
    pub(super) inner: RawTransaction<'a>,
}

impl<'a> Transaction<'a> {
    pub fn raw(&self) -> &RawTransaction<'a> {
        &self.inner
    }

    pub fn raw_mut(&mut self) -> &mut RawTransaction<'a> {
        &mut self.inner
    }
}
