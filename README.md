<div align="center">
    <picture>
        <source srcset="https://raw.githubusercontent.com/kosame-orm/kosame/refs/heads/main/misc/readme/logo-white.svg" media="(prefers-color-scheme: dark)">
        <img width="256" src="https://raw.githubusercontent.com/kosame-orm/kosame/refs/heads/main/misc/readme/logo-black.svg" alt="Kosame Logo">
    </picture>
</div>

<div align="center">
    <h3>Macro-based Rust ORM focused on developer ergonomics</h3> 

[![Crates.io](https://img.shields.io/crates/v/kosame.svg?style=flat-square)](https://crates.io/crates/kosame)
[![Docs.rs](https://img.shields.io/badge/Docs-kosame-66c2a5?style=flat-square&labelColor=555&logo=rust&logoColor=white)](https://docs.rs/kosame)
[![GitHub](https://img.shields.io/badge/GitHub-kosame--orm%2Fkosame-blue?style=flat-square&logo=github)](https://github.com/kosame-orm/kosame)
[![License](https://img.shields.io/crates/l/kosame.svg?style=flat-square)](https://crates.io/crates/kosame)

</div>

<br />

Kosame (小雨, Japanese for "light rain" or "drizzle") is a Rust ORM inspired by [Prisma](https://github.com/prisma/prisma) and [Drizzle](https://github.com/drizzle-team/drizzle-orm).

Some TypeScript ORMs like Prisma can infer the result type of a database query based solely on the database schema and the query itself. Conversely, most Rust ORMs require developers to manually define a struct for the query's results, even though this type is tightly coupled to the query itself. Kosame was born out of a desire to have this level of developer ergonomics in Rust, using macro magic. Kosame also offers relational queries, allowing you to fetch multiple nested 1:N relationships in a single statement.

Kosame requires no active database connection during development and has no build step. Despite this, Kosame offers strong typing and rust-analyzer auto-completion.

**Kosame is currently a prototype and not recommended for production use.**

- [Showcase](#showcase)
- [Planned features](#planned-features)
- [Declaring the schema](#declaring-the-schema)
   * [Column renaming and type overrides](#column-renaming-and-type-overrides)
   * [Relations](#relations)
- [Queries](#queries)
   * [Columns and relations](#columns-and-relations)
   * [Aliases and type overrides](#aliases-and-type-overrides)
   * [Attributes](#attributes)
   * [Expressions](#expressions)
   * [Bind parameters](#bind-parameters)
   * [`where`, `order by`, `limit`, and `offset`](#where-order-by-limit-and-offset)
   * [Named vs. anonymous queries](#named-vs-anonymous-queries)
- [Statements](#statements)
   * [`select`](#select)
   * [`insert`](#insert)
   * [`update`](#update)
   * [`delete`](#delete)
- [Kosame CLI](#kosame-cli)
   * [Formatting Kosame macros](#formatting-kosame-macros)
      + [Editor integration](#editor-integration)
         - [Neovim with conform.nvim](#neovim-with-conformnvim)
- [Can Kosame handle all use cases well?](#can-kosame-handle-all-use-cases-well)

## Showcase

```rust
use kosame::prelude::*;

// Declare your database schema. You may split the schema into multiple Rust modules.
mod schema {
    use kosame::pg_table;

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

    // The `kosame::pg_table!` macro is a shorthand for `kosame::table!` with the driver
    // attribute `#![kosame(driver = "tokio-postgres")]` prefilled. The same applies to
    // `kosame::pg_statement!` and `kosame::pg_query!`.
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect().await;

    // Let's start by clearing the tables using `DELETE FROM` statements.
    kosame::pg_statement! { delete from schema::posts }
        .exec(&mut client)
        .await?;
    kosame::pg_statement! { delete from schema::comments }
        .exec(&mut client)
        .await?;

    // Insert some demo data using `INSERT INTO`.
    kosame::pg_statement! {
        insert into
            schema::posts
        values
            (0, "my post", "hi, this is a post"),
            (1, "another post", "very interesting content"),
            (2, "post without content", null),
    }
    .exec(&mut client)
    .await?;
    kosame::pg_statement! {
        insert into
            schema::comments
        values
            (0, 2, "wow very insightful"),
            (1, 1, "nice"),
            (2, 1, "didn't read lol"),
    }
    .exec(&mut client)
    .await?;

    // Upvote a comment using `UPDATE`.
    let comment_id = 2;
    let new_upvotes = kosame::pg_statement! {
        update
            schema::comments
        set
            upvotes = upvotes + 1,
        where
            // The `comment_id` variable above is used as a bind parameter in this expression.
            id = :comment_id
        returning
            // We can return the updated value. Kosame infers the result type of this statement
            // to be `struct Row { new_upvotes: i32 }` without a database connection.
            comments.upvotes as new_upvotes,
    }
    // With the `RETURNING` clause we can now use `query` instead of `exec` and retrieve data.
    .query_one(&mut client)
    .await?
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
                order by
                    upvotes desc,
                limit
                    5
            },

            // We can also query arbitrary SQL-like expressions, but we need to specify a name and
            // Rust type at the end as they cannot be inferred by Kosame.
            content is not null as has_content: bool,

            where
                id = :post_id
        }
    }
    .query_opt(&mut client)
    .await?;

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
        with
            posts_with_content as (
                select
                    posts.id,
                from
                    schema::posts
                where
                    content is not null
            ),
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
                    comments.id,
                from
                    schema::comments
                where
                    // We can access `posts_with_content` from the higher up scope here.
                    post_id = posts_with_content.id
                order by
                    upvotes desc,
                limit
                    1
            ) as top_comment on true
        group by
            posts_with_content.id,
            top_comment.id,
    }
    .query_vec(&mut client)
    .await?;
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
            eprintln!("connection error: {e}");
        }
    });

    client
}
```

## Planned features

Kosame is an early prototype. There are many features and performance optimizations left to implement, including but not limited to:

* Support for other database management systems. Currently, only PostgreSQL (using [`tokio_postgres`](https://docs.rs/tokio-postgres/latest/tokio_postgres/)) is supported.
* CLI for generating database migrations based on changes in the Kosame schema.
* CLI for generating a Kosame schema by introspecting a database.
* Support for more SQL expression syntax.
* Alternative query runners, similar to the [`relationLoadStrategy` that Prisma offers](https://www.prisma.io/blog/prisma-orm-now-lets-you-choose-the-best-join-strategy-preview).
* Type inference for bind parameters.

## Declaring the schema

Before you can write queries with Kosame, you must declare your database schema. Instead of inventing a new syntax, Kosame tries to follow the existing `CREATE TABLE` syntax closely.

```rust
kosame::pg_table! {
    create table posts (
        id uuid primary key default uuidv7(),
        title text not null,
        content text, // trailing comma is allowed
    );
}
```

This means declaring your schema may be as simple as copying a `pg_dump` into the Kosame macro. However, to enforce consistency, all SQL keywords must be lowercase. Kosame has a basic SQL expression parser, which allows you to define the `default` expression of a column.

### Column renaming and type overrides

Kosame converts database identifiers to snake_case by default. If you want to refer to a database column by a different name in Rust, you can rename it:

```rust
kosame::pg_table! {
    create table my_table (
        #[kosame(rename = cool_column)]
        my_column text not null,
    );
}
```

Kosame attempts to guess the Rust type of a column based on its database type. For example, a PostgreSQL column of type `text` will be represented by a Rust `String`. If you want to use a different type, or if the database type is unknown to Kosame (e.g., for PostgreSQL custom types), you can specify a type override:

```rust
use smol_str;

kosame::pg_table! {
    create table my_table (
        #[kosame(ty = smol_str::SmolStr)]
        my_column text not null,
    );
}
```

Note that the specified type must either be declared or `use`d in the scope of the `kosame::pg_table!` call or be a fully qualified path (e.g., `crate::MyType` or `::std::string::String`).

### Relations

Diverging from regular SQL syntax, you can declare relation fields. Relations tell Kosame how different tables can be queried together.

```rust
kosame::pg_table! {
    create table posts_table (
        id uuid primary key default uuidv7(),
        content text not null,
    );

    comments: (id) <= my_module::comments_table (post_id),
}

mod my_module {
    kosame::pg_table! {
        create table comments_table (
            id uuid primary key default uuidv7(),
            post_id int not null,
            content text not null,
        );

        post: (post_id) => super::posts_table (id),
    }
}
```

In this example, we have a `posts_table` and a `comments_table`. For each row in `posts_table`, we expect there to be zero or more comments. Conversely, each row in the `comments_table` has exactly one post associated with it, as defined by the `post_id` column.

The relation field declaration

```
comments: (id) <= my_module::comments_table (post_id)
```

describes a relation called `comments`. It specifies that the `post_id` column in `my_module::comments_table` "points to" the `id` column of `posts_table`. Although a Kosame relation does not have to map to a database foreign key, you can think of the `<=` as pointing in the direction of the foreign key "pointer". With this relation field, we can query all comments associated with a given post:

```rust
kosame::pg_query! {
    posts_table {
        id,
        content,
        comments {
            id,
            content,
        }
    }
}
```

In the comments table, we have the inverse relation:

```
post: (post_id) => super::posts_table (id),
```

This states that `post` is a row in `super::posts_table`, and it is linked by matching the `comments_table`'s `post_id` column with the `posts_table`'s `id` column. Note that the arrow (`=>`) points in the other direction here. In this case, Kosame expects there to be at most one post per comment.

```rust
kosame::pg_query! {
    my_module::comments_table {
        id,
        content,
        post {
            id,
            content,
        }
    }
}
```

## Queries

### Columns and relations

A basic Kosame query starts by defining the root table you want to read from. This can be a relative or absolute path to your table's declaration.

```rust
pub mod schema {
    ...
}

kosame::pg_query! {
    schema::posts {
        ...
    }
}

// or

kosame::pg_query! {
    crate::schema::posts {
        ...
    }
}
```

In the query, you can list the column and relation fields you want to read. Relations can be nested as often as desired.

```rust
kosame::pg_query! {
    schema::posts {
        id,
        title,

        // `comments` is a relation, as indicated by the curly braces.
        comments {
            id,
            content,
            
            author {
                name,
                email,
            }  
        },

        // You can mix the order of columns and relations.
        content,
    }
}
```

Instead of listing each column manually, you can also use `*` to select all columns of a table.

```rust
kosame::pg_query! {
    schema::posts {
        *,
        comments {
            *,
            author { * }  
        },
    }
}
```

### Aliases and type overrides

You can rename column or relation fields for each query using `as ...`. You can also change the Rust type of a column using `: ...`.

```rust
kosame::pg_query! {
    schema::posts {
        id as my_id,
        title: ::smol_str::SmolStr,
        content as my_content: ::std::string::String,
        comments {
            *
        } as all_comments,
    }
}
```

The row structs generated by Kosame will use the new aliases and data types.

### Attributes

Kosame allows you to annotate your query and its fields with Rust attributes. Attributes assigned to the top-level table will be applied to _all_ generated row structs, including those representing nested relations. Attributes above column or relation fields will be assigned only to the row struct field they correspond to. It is also possible to document your query with documentation comments. Enable the `serde` feature for automatic `serde` derives on all row structs.

```rust
kosame::pg_query! {
    #[serde(rename_all = "camelCase")]
    schema::posts {
        id as my_id,
        
        /// Rust documentation comments, like this one, are also attributes.
        /// This means you can easily document your query and its fields!
        content,

        comments {
            id as my_id,

            #[serde(rename = "cool_content")]
            content as comment_content,
        }
    }
}
```

Serializing the result of the query above using `serde_json` returns the following JSON string:

```json
{
  "myId": 5,
  "content": "hi this is a post",
  "comments": [
    {
      "myId": 19,
      "cool_content": "im another comment"
    },
    {
      "myId": 18,
      "cool_content": "im commenting something"
    }
  ]
}
```

### Expressions

Kosame can parse basic SQL expressions. Expressions can be used in various places, one of which is an expression field in your query:

```rust
kosame::pg_query! {
    posts {
        id,
        upvotes + 1 as reddit_upvotes: i32,
        cast(now() as text) as current_time: String,
        title is not null or content is not null as has_content: bool,
    }
}
```

Like in the table definition, SQL keywords must be lowercase. Expression fields in a query **must** be aliased **and** given a type override. Kosame makes no attempt to deduce the name or type of an expression automatically.

The main difference between the syntax of Kosame expressions and SQL expressions is the handling of string literals and identifiers. Unlike in PostgreSQL, you do not need to use double-quotes to make your identifiers case-sensitive. Strings are written using double-quoted Rust strings, as opposed to single quotes:

```rust
kosame::pg_query! {
    my_table {
        "Hello world!" as hello_world: ::std::string::String,
    }
}
```

### Bind parameters

Kosame uses the `:param_name` syntax for using bind parameters in expressions:

```rust
kosame::pg_query! {
    my_table {
        :my_param + 5 as add_5: i32,
    }
}
```

Kosame generates a `Params` struct containing a borrowed field for each parameter referenced in your query. When executing the query, the bind parameters are converted to the respective database management system's parameter syntax (e.g., `$1`, `$2`, etc., for PostgreSQL).

### `where`, `order by`, `limit`, and `offset`

Kosame uses the familiar syntax for `where`, `order by`, `limit`, and `offset`. You can use expressions for each of these:

```rust
kosame::pg_query! {
    posts {
        id,
        content,
        comments {
            content,
            
            order by upvotes desc, id asc nulls last
            limit 5
        },

        where title = :title and content is not null
        limit :page_size
        offset :page * :page_size
    }
}
```

`where`, `order by`, `limit`, and `offset` must be specified in this order. They must come at the end of a block in a query. Make sure your last query field has a trailing comma.

### Named vs. anonymous queries

Kosame supports both named and anonymous queries. Anonymous queries are defined inline and act as a Rust expression that can be executed immediately. They also allow capturing variables from the surrounding scope as bind parameters for the query (`:id` in this example):

```rust
let id = 5;

let rows = kosame::pg_query! {
    posts {
        content,
        where id = :id
    }
}
.exec(client, &mut RecordArrayRunner {})
.await?;
```

While they are concise, anonymous queries have the drawback that the row types generated by Kosame cannot be named. This makes it difficult to specify concrete return types. We can only resort to the `impl Trait` syntax.

```rust
async fn fetch_row(
    client: &mut tokio_postgres::Client,
    id: i32,
) -> Result<Vec<impl serde::Serialize + Debug>, Box<dyn Error>> {
    let rows = kosame::pg_query! {
        posts {
            content,
            where id = :id
        }
    }
    .query_vec(client)
    .await?;

    Ok(rows)
}
```

Named queries solve this problem by declaring the query upfront. To do this, give your query an alias that will be used as the module name generated by Kosame:

```rust
kosame::pg_query! {
    posts {
        content,
        where id = :id
    }
    as my_query
}
```

You can now refer to all generated types by name:

```rust
async fn fetch_row(
    client: &mut tokio_postgres::Client,
    id: i32,
) -> Result<Vec<my_query::Row>, Box<dyn Error>> {
    let rows = my_query::Query::new(my_query::Params { id: &id })
        .query_vec(client)
        .await?;

    Ok(rows)
}
```

## Statements

Kosame also supports an SQL-like syntax for `SELECT`, `INSERT`, `UPDATE`, and `DELETE` queries which make database mutations possible and allow for greater oversight and flexibility over what exactly your database does.

### `select`

A simple `select` statement works without a `from` clause.
```rust
let rows = kosame::pg_statement! {
    select
        5 as my_column: i32
}
.query_one(&mut client)
.await?;
```

You can also buid more complex queries with `where`, `group by`, `having`, `order by`, `limit`, and `offset`.
```rust
let rows = kosame::pg_statement! {
    select
        // Name and type of this column are inferred.
        posts.id,
        sum(comments.upvotes) as total_upvotes: i64,
    from
        schema::posts
        inner join schema::comments on posts.id = comments.post_id
    where
        comments.upvotes > 0
    group by
        posts.id
    having
        count(1) > 0
    order by
        posts.id
    limit
        5
}
.query_vec(&mut client)
.await?;
```

Common table expressions and (lateral) subqueries are also supported:
```rust
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
.query_vec(&mut client)
.await?;
```

Kosame also supports set operations for combining multiple `select` statements:
```rust
let rows = kosame::pg_statement! {
    select
        comments.content,
    from
        schema::comments
    union all
        select
            posts.content,
        from
            schema::posts
    order by
        1 desc,
    limit
        20
}
.query_vec(&mut client)
.await?;
```

The following set operations are supported:
- `union` - Combines results from multiple queries, removing duplicates
- `union all` - Combines results from multiple queries, keeping duplicates
- `intersect` - Returns only rows that appear in both queries
- `intersect all` - Returns rows that appear in both queries, keeping duplicates
- `except` - Returns rows from the first query that don't appear in the second
- `except all` - Returns rows from the first query that don't appear in the second, keeping duplicates

The Rust type is inferred from the first set of `select` fields in the chain of operations.

### `insert`

```rust
let new_post_ids = kosame::pg_statement! {
    insert into schema::posts
    values
        (0, "my post", "hi, this is a post"),
        (1, "another post", "very interesting content"),
        (2, "post without content", null),
    returning
        posts.id
}
// With the `RETURNING` clause we can now use `query` instead of `exec` and retrieve data.
.query_vec(&mut client)
.await?;
```

### `update`

```rust
let new_upvotes = kosame::pg_statement! {
    update
        schema::comments
    set
        upvotes = upvotes + 1
    where
        id = 5
    returning
        comments.upvotes as new_upvotes
}
.query_one(&mut client)
.await?;
```

### `delete`

```rust
kosame::pg_statement! {
    delete from
        schema::posts
    using
        schema::comments
    where
        posts.id = comments.post_id
}
.exec(&mut client)
.await?;
```

## Kosame CLI

Kosame provides a command-line tool for code formatting. In the future, it will also be used for database migrations and introspection. Install the CLI tool using:

```bash
cargo install kosame_cli
```

And make sure your `PATH` environment variable is configured correctly (see https://rust-lang.org/tools/install/).
Commands can be run either through the `kosame` binary or using `cargo kosame ...`.

### Formatting Kosame macros

The CLI includes a formatter that automatically reformats only the contents of `pg_table!`, `pg_query!`, and `pg_statement!` macros (and their non-`pg_` variants) with proper indentation and structure.

```bash
# Format a single file
kosame fmt src/main.rs

# Format multiple files
kosame fmt src/main.rs src/lib.rs

# Format all Rust files in a directory recursively
kosame fmt src/

# Use glob patterns
kosame fmt "src/**/*.rs"

# Read from stdin and write to stdout (useful for editor integrations)
kosame fmt --stdin < src/main.rs
```

#### Editor integration

##### Neovim with conform.nvim

Create a `Kosame.toml` file at the root of your Rust project. Then add the following `conform.nvim` setup to your Neovim configuration:

```lua
require("conform").setup({
    formatters = {
        kosame = {
            command = "kosame",
            args = { "fmt", "--stdin" },
            require_cwd = true,
            cwd = function(self, ctx)
                return require("conform.util").root_file({ "Kosame.toml" })(self, ctx)
            end,
        },
    }
    formatters_by_ft = {
        rust = { "kosame", lsp_format = "first" },
    },
})
```

## Can Kosame handle all use cases well?

No. Writing raw SQL directly will always give you more flexibility and control over what your database does, which may also allow you to optimize performance beyond what the Kosame supports. But that's okay! You can combine Kosame with another method to access the database. Use Kosame for situations in which you benefit from the relational query syntax and type inference. In more demanding situations, consider using a crate like [`sqlx`](https://github.com/launchbadge/sqlx).
