use kosame::driver::postgres::Pool;

// Declare your database schema. You may split the schema into multiple Rust modules.
mod schema {
    use kosame::driver::postgres::table;

    table! {
        // Kosame uses the familiar SQL syntax to declare tables.
        create table lel (id int primary key);
    }

    table! {
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

    table! {
        create table comments (
            id int primary key,
            post_id int not null,
            content text not null,
            upvotes int not null default 0,
        );

        // You may also define the inverse relation if you need it.
        post: (post_id) => posts (id),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Pool::new("postgres://kosame:kosame@localhost:5432/kosame".parse()?)?;

    use kosame::driver::postgres::statement;

    pool.execute(statement! { delete from schema::posts });

    Ok(())
}
