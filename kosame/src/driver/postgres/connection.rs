use crate::exec::Exec;

use super::StatementCache;

pub struct Connection {
    inner: postgres::Client,
    statement_cache: StatementCache,
}

impl Connection {
    fn exec<E>(&mut self, exec: &E) -> E::Response
    where
        E: Exec,
    {
        exec.exec(self)
    }
}
