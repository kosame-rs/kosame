use std::error::Error;

use kosame::prelude::*;

// Declare your database schema.
mod schema {
    use kosame::pg_table;

    pg_table! {
        // Kosame uses the familiar SQL syntax to declare tables.
        create table posts (
            id int primary key,

            // Kosame converts database identifiers to snake_case automatically and
            // has a default for most well known database types. You can rename them
            // or specify a different type if you prefer.
            #[kosame(rename = title, ty = ::std::string::String)]
            title text not null,

            content text,
        );

        // Define a relation to another table. This enables relational queries.
        comments: (id) <= comments (post_id),
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = connect().await;

    use kosame::pg_statement;

    // Let's start by clearing the tables using `DELETE FROM` statements.
    pg_statement! { delete from schema::posts }
        .exec(&mut client)
        .await?;
    pg_statement! { delete from schema::comments }
        .exec(&mut client)
        .await?;

    // Insert some demo data using `INSERT INTO`.
    pg_statement! {
        insert into schema::posts
        values
            (0, "my post", "hi, this is a post"),
            (1, "another post", "very interesting content"),
            (2, "post without content", null),
    }
    .exec(&mut client)
    .await?;
    pg_statement! {
        insert into schema::comments
        values
            (0, 2, "wow very insightful"),
            (1, 1, "nice"),
            (2, 1, "didn't read lol"),
    }
    .exec(&mut client)
    .await?;

    // Upvote a comment using `UPDATE`.
    let comment_id = 2;
    // pg_statement! {
    //     update
    //         schema::comments
    //     set
    //         upvotes = upvotes + 1
    //     where
    //         id = :comment_id
    // }
    // .exec(&mut client)
    // .await?;

    use kosame::pg_query;

    let rows = pg_query! {
        schema::posts {
            *,
            content is not null as has_content: bool,

            comments {
                id,
                content,
                upvotes,

                order by comments.upvotes desc
                limit 5
            }
        }
    }
    .query_vec(&mut client)
    .await?;
    println!("{:#?}", rows);

    Ok(())
}

// This function connects to a database using tokio-postgres.
async fn connect() -> tokio_postgres::Client {
    let (client, connection) = tokio_postgres::connect(
        "postgres://postgres:postgres@localhost:5432/postgres",
        tokio_postgres::NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
}
