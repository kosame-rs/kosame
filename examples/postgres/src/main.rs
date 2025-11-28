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
    let mut client = postgres::Client::connect(
        "postgres://postgres:postgres@localhost:5432/postgres",
        postgres::NoTls,
    )
    .unwrap();

    // Let's start by clearing the tables using `DELETE FROM` statements.
    kosame::pg_statement! { delete from schema::posts }.exec_sync(&mut client)?;
    kosame::pg_statement! { delete from schema::comments }.exec_sync(&mut client)?;

    // Insert some demo data using `INSERT INTO`.
    kosame::pg_statement! {
        insert into schema::posts
        values
            (0, "my post", "hi, this is a post"),
            (1, "another post", "very interesting content"),
            (2, "post without content", null),
    }
    .exec_sync(&mut client)?;
    kosame::pg_statement! {
        insert into schema::comments
        values
            (0, 2, "wow very insightful"),
            (1, 1, "nice"),
            (2, 1, "didn't read lol"),
    }
    .exec_sync(&mut client)?;

    // Upvote a comment using `UPDATE`.
    let comment_id = 2;
    let new_upvotes = kosame::pg_statement! {
        update
            schema::comments
        set
            upvotes = upvotes + 1
        where
            // The `comment_id` variable above is used as a bind parameter in this expression.
            id = :comment_id
        returning
            // We can return the updated value. Kosame infers the result type of this statement
            // to be `struct Row { new_upvotes: i32 }` without a database connection.
            comments.upvotes as new_upvotes
    }
    // With the `RETURNING` clause we can now use `query` instead of `exec` and retrieve data.
    .query_one_sync(&mut client)?
    .new_upvotes;

    println!("{new_upvotes}");
    // 1

    // Now let's perform a relational query. Relational queries fetch arbitrarily nested 1:N
    // (or 1:1) relationships in the requested shape, so that you do not need to manually
    // convert flat SQL tables into a struct hierarchy.
    // We want to read the post with ID = 1, together with its top five comments.
    let post_id = 1;
    let rows = kosame::pg_query! {
        // Attributes appearing here will be applied to all generated result structs.
        #[derive(Clone)]
        schema::posts {
            *, // Select all fields from the posts table.

            // Query related comments according to the relation defined in the schema.
            comments {
                // To save bandwidth, we select only the fields we need here.
                id,

                // Attributes on fields will also be applied to the result type fields.
                #[serde(rename = "serdeContent")]
                content,

                /// This triple-slash documentation comment will appear in your IDE on the
                /// generated type's `upvotes` field.
                upvotes,

                // Relational queries also use the familiar SQL-like syntax for
                // `where`, `order by`, `limit` and `offset`.
                order by upvotes desc
                limit 5
            },

            // We can also query arbitrary SQL-like expressions, but we need to specify a name and
            // Rust type at the end as they cannot be inferred by Kosame.
            content is not null as has_content: bool,

            where id = :post_id
        }
    }
    .query_opt_sync(&mut client)?;

    println!("{rows:#?}");
    // Some(
    //     Row {
    //         id: 1,
    //         renamed_title: "another post",
    //         content: Some(
    //             "very interesting content",
    //         ),
    //         comments: Many(
    //             [
    //                 RowComments {
    //                     id: 2,
    //                     content: "didn't read lol",
    //                     upvotes: 1,
    //                 },
    //                 RowComments {
    //                     id: 1,
    //                     content: "nice",
    //                     upvotes: 0,
    //                 },
    //             ],
    //         ),
    //         has_content: true,
    //     },
    // )

    // Relational queries are not well suited to every use case. To squeeze maximum performance and
    // flexibility out of your database, you may want to write SQL `SELECT` statements directly.
    // Kosame supports an SQL-like syntax with basic type inference for this scenario.
    let rows = kosame::pg_statement! {
        // A common table expression of all posts with non-null content.
        with posts_with_content as (
            select
                posts.id
            from
                schema::posts
            where
                content is not null
        )
        select
            // The type of this field is inferred as `i32`.
            posts_with_content.id,
            // This field would also be `i32`. However, because of the `left join`, Kosame knows it
            // may be null and thus infers the field type to be `Option<i32>`.
            top_comment.id as top_comment_id,
            // Kosame cannot currently infer the name and type of this expressions, so we must
            // declare them manually.
            coalesce(sum(comments.upvotes), 0) as total_upvotes: i64,
            // The $"..." syntax allows you inline raw SQL text into expressions, which can be
            // helpful for syntax that Kosame does not yet support.
            $"'[1, 2, 3]'::jsonb @> '[1, 3]'::jsonb" as raw_sql: bool,
        from
            posts_with_content
            left join schema::comments on posts_with_content.id = comments.post_id
            // Kosame supports subqueries, including `lateral` ones.
            left join lateral (
                select
                    comments.id
                from
                    schema::comments
                where
                    // We can access `posts_with_content` from the higher up scope here.
                    post_id = posts_with_content.id
                order by
                        upvotes desc
                limit 1
            ) as top_comment on true
        group by
            posts_with_content.id, top_comment.id
    }
    .query_vec_sync(&mut client)?;
    // The query above is of course inefficient and solely meant for demonstration purposes.

    println!("{rows:#?}");
    // [
    //     Row {
    //         id: 0,
    //         top_comment_id: None,
    //         total_upvotes: 0,
    //         raw_sql: true,
    //     },
    //     Row {
    //         id: 1,
    //         top_comment_id: Some(
    //             2,
    //         ),
    //         total_upvotes: 1,
    //         raw_sql: true,
    //     },
    // ]

    // With the "serde" feature enabled, the result of a statement or query can be serialized.
    println!("{}", serde_json::to_string(&rows).unwrap());
    // [{"id":0,"top_comment_id":null,"total_upvotes":0,"raw_sql":true},{"id":1,"top_comment_id":2,"total_upvotes":1,"raw_sql":true}]

    let rows = kosame::pg_statement! {
        select 5 as pip: i32, 6 as lel: i32
        union all
        select 7, 8
        order by 1 desc
        limit 2
    }
    .query_vec_sync(&mut client)?;
    println!("{rows:#?}");

    Ok(())
}
