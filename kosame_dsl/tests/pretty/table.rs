use kosame_dsl::schema::Table;

use crate::pretty::assert_pretty;

#[test]
fn empty() {
    assert_pretty!(Table:
        "{
create  table test();


}",
        "{ create table test (); }"
    );
}

#[test]
fn single_column() {
    assert_pretty!(Table:
        "{
create  table test (col int
primary
key

);


}",
        "{ create table test (col int primary key); }"
    );
}

#[test]
fn multi_column() {
    assert_pretty!(Table:
        "{
create  table test (

col int primary key,

    col2 int not null default 5 + 5

);


}",
        "{
    create table test (col int primary key, col2 int not null default 5 + 5);
}"
    );
}

#[test]
fn multi_column_break() {
    assert_pretty!(Table:
        "{
create  table test (

col int primary key,

    col2 int not null default 5
+ 5 + (now() + 9
            / 5 = 5 and false)

);


}",
        "{
    create table test (
        col int primary key,
        col2 int not null default 5 + 5 + (now() + 9 / 5 = 5 and false),
    );
}"
    );
}

#[test]
fn block_comments() {
    assert_pretty!(Table:
        "{
create  table test ( /*1*/col int /*2*/,/*3*/ col2 int not null/*4*/

);


}",
        "{
    create table test (/*1*/ col int /*2*/, /*3*/ col2 int not null /*4*/);
}"
    );
}
