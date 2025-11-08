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

## Showcase

```rust
use std::{error::Error, fmt::Debug};

use kosame::query::{Query, RecordArrayRunner};

mod schema {
    kosame::table! {
        // Kosame uses the familiar SQL syntax to define tables.
        create table posts (
            id int primary key default uuidv7(),
            title text not null,
            content text,
        );

        // Define a relation to another table. This enables relational queries.
        comments: (id) <= comments (post_id),
    }

    kosame::table! {
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

async fn fetch_post(
    client: &mut tokio_postgres::Client,
    id: i32,
) -> Result<Option<impl serde::Serialize + Debug>, Box<dyn Error>> {
    let row = kosame::query! {
        schema::posts {
            *, // Select all columns from the posts table.

            // Include all related comments using the relation defined above.
            comments {
                id,
                content,
                upvotes,

                // Familiar syntax for `where`, `order by`, `limit`, and `offset`.
                order by upvotes desc
                limit 3
            },

            // The `fetch_post` function parameter `id: i32` is used as a query parameter here.
            where id = :id
        }
    }
    .exec_opt(
        client,
        // RecordArrayRunner describes the strategy to fetch rows from the database. In this case,
        // we run just a single SQL query that uses PostgreSQL's arrays and anonymous records.
        &mut RecordArrayRunner {},
    )
    .await?;

    Ok(row)
}
```

To execute the query, the Kosame macro automatically generates the following row structs (simplified):

```rust
struct Row {
    id: i32,
    title: String,
    content: Option<String>,
    comments: Vec<RowComments>,
}

struct RowComments {
    id: i32,
    content: String,
    upvotes: i32,
}
```

If the `serde` feature is enabled, the row structs implement `serde::Serialize`, making it trivial to return them from an API endpoint. Using `serde_json`, we can print the result of the `fetch_post` function for post ID `5`:

```json
{
  "id": 5,
  "title": "my post",
  "content": "hi this is a post",
  "comments": [
    {
      "id": 18,
      "content": "im commenting something",
      "upvotes": 4
    },
    {
      "id": 19,
      "content": "im another comment",
      "upvotes": 0
    }
  ]
}
```

## Planned features

Kosame is an early prototype. There are many features and performance optimizations left to implement, including but not limited to:

* Support for other database management systems. Currently, only PostgreSQL (using [`tokio_postgres`](https://docs.rs/tokio-postgres/latest/tokio_postgres/)) is supported.
* CLI for generating database migrations based on changes in the Kosame schema.
* CLI for generating a Kosame schema by introspecting a database.
* Formatter for table, query and statement macros
* Support for more SQL expression syntax.
* Alternative query runners, similar to the [`relationLoadStrategy` that Prisma offers](https://www.prisma.io/blog/prisma-orm-now-lets-you-choose-the-best-join-strategy-preview).
* Type inference for bind parameters.

## Defining the schema

Before you can write queries with Kosame, you must declare your database schema. Instead of inventing a new syntax, Kosame tries to follow the existing `CREATE TABLE` syntax closely.

```rust
kosame::table! {
    create table posts (
        id int primary key default uuidv7(),
        title text not null,
        content text, // trailing comma is allowed
    );
}
```

This means declaring your schema may be as simple as copying a `pg_dump` into the Kosame macro. However, to enforce consistency, all SQL keywords must be lowercase. Kosame has a basic SQL expression parser, which allows you to define the `default` expression of a column.

### Column aliases and type overrides

If you want to refer to a database column by a different name in Rust, you can use a column alias:

```rust
kosame::table! {
    create table my_table (
        MyColumn text not null,
    );

    MyColumn as my_column,
}
```

Kosame attempts to guess the Rust type of a column based on its database type. For example, a PostgreSQL column of type `text` will be represented by a Rust `String`. If you want to use a different type, or if the database type is unknown to Kosame (e.g., for PostgreSQL custom types), you can specify a type override:

```rust
use smol_str;

kosame::table! {
    create table my_table (
        my_column text not null,
    );

    my_column: smol_str::SmolStr,
}
```

Note that the specified type must either be declared or `use`d in the scope of the `kosame::table!` call or be a fully qualified path (e.g., `crate::MyType` or `::std::string::String`).

Aliases and type overrides can be combined as follows:

```rust
use smol_str::SmolStr;

kosame::table! {
    create table my_table (
        MyColumn text not null,
    );

    MyColumn as my_column: SmolStr,
}
```

### Relations

In addition to column aliases and type overrides, you can also declare relation fields. Relations tell Kosame how different tables can be queried together.

```rust
kosame::table! {
    create table posts_table (
        id int primary key default uuidv7(),
        content text not null,
    );

    comments: (id) <= my_module::comments_table (post_id),
}

mod my_module {
    kosame::table! {
        create table comments_table (
            id int primary key default uuidv7(),
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
kosame::query! {
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
kosame::query! {
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

kosame::query! {
    schema::posts {
        ...
    }
}

// or

kosame::query! {
    crate::schema::posts {
        ...
    }
}
```

In the query, you can list the column and relation fields you want to read. Relations can be nested as often as desired.

```rust
kosame::query! {
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
kosame::query! {
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

Just like in the table definition, you can also rename column or relation fields for each query. You can also change the Rust type of a column.

```rust
kosame::query! {
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

Kosame allows you to annotate your query and its fields with Rust attributes. Attributes assigned to the top-level table will be applied to _all_ generated row structs, including those representing nested relations. Attributes above column or relation fields will be assigned only to the row struct field they correspond to. It is also possible to document your query with documentation comments.

```rust
kosame::query! {
    #[derive(serde::Serialize)]
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

You can also enable the `serde` feature to automatically annotate all row structs with Serde derives.

### Expressions

Kosame can parse basic SQL expressions. Expressions can be used in various places, one of which is an expression field in your query:

```rust
kosame::query! {
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
kosame::query! {
    my_table {
        "Hello world!" as hello_world: ::std::string::String,
    }
}
```

### Bind parameters

Kosame uses the `:param_name` syntax for using bind parameters in expressions:

```rust
kosame::query! {
    my_table {
        :my_param + 5 as add_5: i32,
    }
}
```

Kosame generates a `Params` struct containing a borrowed field for each parameter referenced in your query. When executing the query, the bind parameters are converted to the respective database management system's parameter syntax (e.g., `$1`, `$2`, etc., for PostgreSQL).

### `where`, `order by`, `limit`, and `offset`

Kosame uses the familiar syntax for `where`, `order by`, `limit`, and `offset`. You can use expressions for each of these:

```rust
kosame::query! {
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

let rows = kosame::query! {
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
    let rows = kosame::query! {
        posts {
            content,
            where id = :id
        }
    }
    .exec(client, &mut RecordArrayRunner {})
    .await?;

    Ok(rows)
}
```

Named queries solve this problem by declaring the query upfront. To do this, give your query an alias that will be used as the module name generated by Kosame:

```rust
kosame::query! {
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
        .exec(client, &mut RecordArrayRunner {})
        .await?;

    Ok(rows)
}
```

## Can Kosame handle all use cases well?

No. Kosame chooses a syntax that works well when you just want to "fetch a thing and its things and their things." Writing SQL directly will always give you more flexibility and control over what your database does, which may also allow you to optimize performance beyond what the Kosame query runner can come up with.

But that's okay! You can combine Kosame with another method to access the database. Use Kosame for situations in which you benefit from the relational query syntax and auto-generated types. In more demanding situations, consider using a crate like [`sqlx`](https://github.com/launchbadge/sqlx).
