use crate::execute::Execute;

use super::raw::{RawTransaction, RawTransactionBuilder};

pub struct TransactionBuilder<'a> {
    pub(super) inner: RawTransactionBuilder<'a>,
}

impl<'a> TransactionBuilder<'a> {
    #[must_use]
    pub fn raw(&self) -> &RawTransactionBuilder<'a> {
        &self.inner
    }

    #[must_use]
    pub fn raw_mut(&mut self) -> &mut RawTransactionBuilder<'a> {
        &mut self.inner
    }
}

pub struct Transaction<'a> {
    pub(super) inner: RawTransaction<'a>,
}

impl<'a> Transaction<'a> {
    pub fn execute<E: Execute<Self>>(&mut self, execute: E) -> E::Result {
        execute.execute(self)
    }

    #[must_use]
    pub fn raw(&self) -> &RawTransaction<'a> {
        &self.inner
    }

    #[must_use]
    pub fn raw_mut(&mut self) -> &mut RawTransaction<'a> {
        &mut self.inner
    }
}
