// use kosame::{pg_statement, sql::FmtSql, statement::Statement};
//
// #[test]
// fn basic_select() {
//     let statement = pg_statement! {
//         select 5 as first: i32, true as second: bool
//     };
//     assert_eq!(
//         "select 5, true",
//         statement
//             .repr()
//             .to_sql_string::<kosame::sql::postgres::Dialect>()
//             .unwrap()
//     );
// }
//
// mod schema {
//     use kosame::pg_table;
//
//     pg_table! {
//         create table table_a (
//             id int primary key,
//             content text,
//         );
//     }
//
//     pg_table! {
//         create table table_b (
//             id int primary key,
//             title text,
//         );
//     }
// }
//
// #[test]
// fn complex_select() {
//     let limit = 5;
//     let statement = pg_statement! {
//         select
//             table_a.id as id: i32,
//             table_b.title as title: ::std::string::String,
//         from schema::table_a
//             left join schema::table_b on table_a.id = table_b.id
//         where content = "test"
//         group by title
//         having sum(table_b.id) > 8
//         order by table_a.id asc, table_b.title desc nulls last
//         limit :limit
//     };
//     assert_eq!(
//         r#"select "table_a"."id", "table_b"."title" from "table_a" left join "table_b" on "table_a"."id" = "table_b"."id" where "content" = 'test' group by "title" having "sum"("table_b"."id") > 8 order by "table_a"."id" asc, "table_b"."title" desc nulls last limit $1"#,
//         statement
//             .repr()
//             .to_sql_string::<kosame::sql::postgres::Dialect>()
//             .unwrap()
//     );
// }
