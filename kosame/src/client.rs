enum ConnectionInner {
    #[cfg(feature = "postgres")]
    Postgres(crate::driver::postgres::CachedClient),
    #[cfg(feature = "tokio-postgres")]
    TokioPostgres(crate::driver::tokio_postgres::CachedClient),
}
