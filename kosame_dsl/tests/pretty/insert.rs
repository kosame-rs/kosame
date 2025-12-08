use kosame_dsl::statement::Statement;

use crate::pretty::assert_pretty;

#[test]
fn simple_one_line() {
    assert_pretty!(Statement:
        "{ insert into   schema::posts   values   (1, \"title\") }",
        "{ insert into schema::posts values (1, \"title\") }"
    );
}

#[test]
fn insert_single_row() {
    assert_pretty!(Statement:
        "{
insert into schema::posts values (0, \"my post\", \"hi, this is a post\")
}",
        "{
    insert into
        schema::posts
    values
        (0, \"my post\", \"hi, this is a post\"),
}"
    );
}

#[test]
fn insert_multiple_rows() {
    assert_pretty!(Statement:
        "{
insert into
    schema::posts
values
    (0, \"my post\", \"hi, this is a post\"),
    (1, \"another post\", \"very interesting content\"),
    (2, \"post without content\", null),
}",
        "{
    insert into
        schema::posts
    values
        (0, \"my post\", \"hi, this is a post\"),
        (1, \"another post\", \"very interesting content\"),
        (2, \"post without content\", null),
}"
    );
}

#[test]
fn insert_with_bind_params() {
    assert_pretty!(Statement:
        "{
insert into schema::posts values (:id, :title, :content)
}",
        "{
    insert into
        schema::posts
    values
        (:id, :title, :content),
}"
    );
}

#[test]
fn insert_with_returning() {
    assert_pretty!(Statement:
        "{
insert into
    schema::posts
values
    (0, \"title\", \"content\"),
returning
    posts.id,
}",
        "{
    insert into
        schema::posts
    values
        (0, \"title\", \"content\"),
    returning
        posts.id,
}"
    );
}

#[test]
fn insert_returning_multiple() {
    assert_pretty!(Statement:
        "{
insert into schema::posts values (:id, :title, :content) returning posts.id, posts.title, posts.created_at,
}",
        "{
    insert into
        schema::posts
    values
        (:id, :title, :content),
    returning
        posts.id,
        posts.title,
        posts.created_at,
}"
    );
}

#[test]
fn insert_with_line_comments() {
    assert_pretty!(Statement:
        "{
// Insert demo data
insert into
    schema::comments
values
    // First comment
    (0, 2, \"wow very insightful\"),
    // Second comment
    (1, 1, \"nice\"),
    (2, 1, \"didn't read lol\"),
}",
        "{
    // Insert demo data
    insert into
        schema::comments
    values
        // First comment
        (0, 2, \"wow very insightful\"),
        // Second comment
        (1, 1, \"nice\"),
        (2, 1, \"didn't read lol\"),
}"
    );
}

#[test]
fn insert_with_block_comments() {
    assert_pretty!(Statement:
        "{
insert into /*table*/ schema::posts values /*row*/ (1, \"title\", \"content\")
}",
        "{
    insert into
        schema::posts
    /*table*/
    values
        /*row*/
        (1, \"title\", \"content\"),
}"
    );
}

#[test]
fn insert_complex_values() {
    assert_pretty!(Statement:
        "{
insert into
    schema::posts
values
    (0, concat(\"title \", \"concat\"), now()),
    (1, upper(:title), :content),
    (2, $\"'custom sql'\", null),
}",
        "{
    insert into
        schema::posts
    values
        (0, concat(\"title \", \"concat\"), now()),
        (1, upper(:title), :content),
        (2, $\"'custom sql'\", null),
}"
    );
}

#[test]
fn insert_long_row_breaks() {
    assert_pretty!(Statement:
        "{
insert into schema::posts values (1, \"very long title\", \"very long content\", now(), \"author name\", 100, true, \"tag1\", \"tag2\", \"tag3\")
}",
        "{
    insert into
        schema::posts
    values
        (
            1,
            \"very long title\",
            \"very long content\",
            now(),
            \"author name\",
            100,
            true,
            \"tag1\",
            \"tag2\",
            \"tag3\",
        ),
}"
    );
}

#[test]
fn insert_multiple_long_rows_break() {
    assert_pretty!(Statement:
        "{
insert into schema::posts values (1, \"very long title one\", \"very long content one\", now(), \"author\", 100, true, \"tag\"), (2, \"very long title two\", \"very long content two\", now(), \"author\", 200, false, \"tag\")
}",
        "{
    insert into
        schema::posts
    values
        (
            1,
            \"very long title one\",
            \"very long content one\",
            now(),
            \"author\",
            100,
            true,
            \"tag\",
        ),
        (
            2,
            \"very long title two\",
            \"very long content two\",
            now(),
            \"author\",
            200,
            false,
            \"tag\",
        ),
}"
    );
}
