#[macro_export]
macro_rules! pg_table {
    ($($tokens:tt)*) => {
        ::kosame::table! {
            #![kosame(driver = "tokio-postgres")]
            $($tokens)*
        }
    };
}

#[macro_export]
macro_rules! pg_statement {
    ($($tokens:tt)*) => {
        ::kosame::statement! {
            #![kosame(driver = "tokio-postgres")]
            $($tokens)*
        }
    };
}

#[macro_export]
macro_rules! pg_query {
    ($($tokens:tt)*) => {
        ::kosame::query! {
            #![kosame(driver = "tokio-postgres")]
            $($tokens)*
        }
    };
}
