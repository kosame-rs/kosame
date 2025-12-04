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
#[test] 
    col2 int default 5 + 5,

);


}",
        "{
    create table test (col int primary key, #[test] col2 int default 5 + 5);
}"
    );
}

#[test]
fn multi_column_break() {
    assert_pretty!(Table:
        "{
create  table test (

col int primary key,
#[test] 
    col2 int not null default 5
+ 5 + (now() + 9
            / 5 = 5 and false),


    col3 int not null
);


}",
        "{
    create table test (
        col int primary key,
        #[test]
        col2 int not null default 5 + 5 + (now() + 9 / 5 = 5 and false),

        col3 int not null,
    );
}"
    );
}

#[test]
fn relations() {
    assert_pretty!(Table:
        "{
create table test ( col int primary key);
rel1 : (col) =>test(col2),
rel2 : (col,col,col,col,col,col,col,col,col,col,col,col,col,col,col,) =>test(col,col,col,col,col,col,col,col,col,col,col,col,col,col,col,),
}",
        "{
    create table test (col int primary key);

    rel1: (col) => test (col2),
    rel2: (
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
    ) => test (
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
        col,
    ),
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

#[test]
fn block_comments_break() {
    assert_pretty!(Table:
        "{
create  table test (
/*1*/my_column_1 int
/*2*/,/*3*/
/*4*/ my_column_2 int not null/*5*/ /*6*/
);
}",
        "{
    create table test (
        /*1*/
        my_column_1 int /*2*/, /*3*/
        /*4*/
        my_column_2 int not null, /*5*/
        /*6*/
    );
}"
    );
}

#[test]
fn line_comments_simple() {
    assert_pretty!(Table:
        "{
// comment at start
create  table test (
col int, // after first column
 col2 int not null // after second column

);


}",
        "{
    // comment at start
    create table test (
        col int, // after first column
        col2 int not null, // after second column
    );
}"
    );
}

#[test]
fn line_comments_multiline() {
    assert_pretty!(Table:
        "{
// comment at start
// comment at start 2
create  table test ( // before first column

// before first column 2
col int,

// after first column
// after first column 2
 col2 int not null // after second column
 // end
);


}",
        "{
    // comment at start
    // comment at start 2
    create table test (
        // before first column

        // before first column 2
        col int,

        // after first column
        // after first column 2
        col2 int not null, // after second column
        // end
    );
}"
    );
}

#[test]
fn line_comments_with_attributes() {
    assert_pretty!(Table:
        "{
create  table test (
// before attribute
#[test] // after attribute
    col int primary key, // after constraint
#[kosame(rename = foo)] // after second attribute
col2 int default 5 // after default
);
}",
        "{
    create table test (
        // before attribute
        #[test] // after attribute
        col int primary key, // after constraint
        #[kosame(rename = foo)] // after second attribute
        col2 int default 5, // after default
    );
}"
    );
}

#[test]
fn line_comments_with_relations() {
    assert_pretty!(Table:
        "{
create table test ( col int primary key);

// relation comment 1
rel1 : (col) =>test(col2), // after relation

// relation comment 2

// relation comment 3
rel2 : (col) =>test(col3),
// end comment
}",
        "{
    create table test (col int primary key);

    // relation comment 1
    rel1: (col) => test (col2), // after relation

    // relation comment 2

    // relation comment 3
    rel2: (col) => test (col3), // end comment
}"
    );
}
