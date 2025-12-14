use kosame::prelude::*;

// Declare your database schema. You may split the schema into multiple Rust modules.
mod schema {
    use kosame::pg_table;

    pg_table! {
        // Kosame uses the familiar SQL syntax to declare tables.
        create table lel (id int primary key);
    }

    pg_table! {
        // Kosame uses the familiar SQL syntax to declare tables.
        create table posts (
            id int primary key,

            // Kosame converts database identifiers to snake_case automatically and
            // has a default Rust type for most well known database types. You can
            // rename them or specify a different type if you prefer.
            #[kosame(rename = renamed_title, ty = ::std::string::String)]
            title text not null,

            content text, // Trailing commas are allowed.
        );

        // Define a relation to another table. This enables relational queries.
        comments: (id) <= super::schema::comments (post_id),
    }

    pg_table! {
        create table comments (
            id int primary key,
            post_id int not null,
            content text not null,
            upvotes int not null default 0,
        );

        // You may also define the inverse relation if you need it.
        post: (post_id) => posts (id),
    }

    // The `kosame::pg_table!` macro is a shorthand for `kosame::table!` with the driver
    // attribute `#![kosame(driver = "tokio-postgres")]` prefilled. The same applies to
    // `kosame::pg_statement!` and `kosame::pg_query!`.
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = rusqlite::Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  BLOB
        )",
        (), // empty list of parameters.
    )?;

    Ok(())
}
