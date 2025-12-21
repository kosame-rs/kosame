#[macro_export]
macro_rules! postgres_table {
    ($($tokens:tt)*) => {
        ::kosame::generic_table! {
            #![kosame(driver = "postgres")]
            $($tokens)*
        }
    };
}
pub use postgres_table as table;

#[macro_export]
macro_rules! postgres_statement {
    ($($tokens:tt)*) => {
        ::kosame::generic_statement! {
            #![kosame(driver = "postgres")]
            $($tokens)*
        }
    };
}
pub use postgres_statement as statement;

#[macro_export]
macro_rules! postgres_query {
    ($($tokens:tt)*) => {
        ::kosame::generic_query! {
            #![kosame(driver = "postgres")]
            $($tokens)*
        }
    };
}
pub use postgres_query as query;
