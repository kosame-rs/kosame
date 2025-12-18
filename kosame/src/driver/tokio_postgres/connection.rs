use super::{RawClient, RawTransaction};

pub struct Connection {
    inner: RawClient,
}

impl crate::driver::Connection for Connection {
    type Transaction<'a> = Transaction<'a>;
}

pub struct Transaction<'a> {
    inner: RawTransaction<'a>,
}

impl<'a> crate::driver::Transaction<'a> for Transaction<'a> {}
