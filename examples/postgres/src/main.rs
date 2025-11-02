use kosame::query::{Query, RecordArrayRunner};
use kosame::statement::Statement;

// Declare your database schema.
mod schema {
    kosame::pg_table! {
        // Kosame uses the familiar SQL syntax to define tables.
        create table kek (
            id int primary key,
        );
    }

    kosame::pg_table! {
        // Kosame uses the familiar SQL syntax to define tables.
        create table posts (
            id int primary key default uuidv7(),
            #[kosame(rename = title, ty = ::std::string::String)]
            title text not null,
            content text,
        );

        // Define a relation to another table. This enables relational queries.
        comments: (id) <= comments (post_id),
    }

    kosame::pg_table! {
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

fn main() {
    let mut client = postgres::Client::connect(
        "postgres://postgres:postgres@localhost:5432/postgres",
        postgres::NoTls,
    )
    .unwrap();

    // let rows = kosame::pg_query! {
    //     schema::posts {
    //         *,
    //         comments {
    //             *
    //         }
    //     }
    // }
    // .exec_sync(&mut client, &mut RecordArrayRunner {})
    // .unwrap();
    //

    let id = 7;

    let statement = kosame::pg_statement! {
        select
            pip.id: i32
        from schema::posts
        left join lateral (select id from schema::comments where comments.id > 5) as pip on true
    };

    use kosame::sql::FmtSql;
    let sql = statement
        .repr()
        .to_sql_string::<kosame::sql::postgres::Dialect>()
        .unwrap();
    println!("{}", sql);

    let rows = statement.exec_vec_sync(&mut client).unwrap();

    println!("{:#?}", rows);
}
