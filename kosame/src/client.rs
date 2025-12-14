use crate::exec::Exec;

enum PoolInner {}

pub struct Pool {
    inner: PoolInner,
}

impl Pool {
    async fn get(&mut self) -> Connection {
        unimplemented!();
    }
}

enum ConnectionInner {
    #[cfg(feature = "postgres")]
    Postgres(crate::driver::postgres::CachedClient),
    #[cfg(feature = "tokio-postgres")]
    TokioPostgres(crate::driver::tokio_postgres::CachedClient),
}

pub struct Connection {
    inner: ConnectionInner,
}

impl Connection {
    fn exec<T>(&mut self, target: &T) -> T::Response
    where
        T: Exec,
    {
        target.exec()
    }
}

trait Client {}
