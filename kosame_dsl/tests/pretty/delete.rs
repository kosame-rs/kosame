use kosame_dsl::statement::Statement;

use crate::pretty::assert_pretty;

#[test]
fn simple_one_line() {
    assert_pretty!(Statement:
        "{ delete   from   schema::posts }",
        "{ delete from schema::posts }"
    );
}

#[test]
fn delete_with_where() {
    assert_pretty!(Statement:
        "{ delete from schema::posts where id = :post_id }",
        "{ delete from schema::posts where id = :post_id }"
    );
}

#[test]
fn delete_with_where_break() {
    assert_pretty!(Statement:
        "{
delete from
    schema::posts
where
    id = :post_id
}",
        "{ delete from schema::posts where id = :post_id }"
    );
}

#[test]
fn delete_with_complex_where() {
    assert_pretty!(Statement:
        "{
delete from schema::posts where id > 100 and (published = false or created_at < now() - cast(\"30 days\" as interval))
}",
        "{
    delete from
        schema::posts
    where
        id > 100 and (published = false or created_at < now() - cast(\"30 days\"
                    as interval))
}"
    );
}

#[test]
fn delete_with_returning() {
    assert_pretty!(Statement:
        "{
delete from
    schema::posts
where
    id = :post_id
returning
    posts.id,
}",
        "{
    delete from
        schema::posts
    where
        id = :post_id
    returning
        posts.id,
}"
    );
}

#[test]
fn delete_returning_multiple() {
    assert_pretty!(Statement:
        "{
delete from schema::posts where id = :post_id returning posts.id, posts.title, posts.deleted_at,
}",
        "{
    delete from
        schema::posts
    where
        id = :post_id
    returning
        posts.id,
        posts.title,
        posts.deleted_at,
}"
    );
}

#[test]
fn delete_with_line_comments() {
    assert_pretty!(Statement:
        "{
// Delete old posts
delete from
    schema::posts
where
    // Older than 30 days
    created_at < now() - cast(\"30 days\" as interval)
}",
        "{
    // Delete old posts
    delete from
        schema::posts
    where
        // Older than 30 days
        created_at < now() - cast(\"30 days\" as interval)
}"
    );
}

#[test]
fn delete_with_block_comments() {
    assert_pretty!(Statement:
        "{
delete from /*table*/ schema::posts where /*condition*/ id = :post_id
}",
        "{
    delete from
        schema::posts
    /*table*/
    where
        /*condition*/
        id = :post_id
}"
    );
}

#[test]
fn delete_with_using() {
    assert_pretty!(Statement:
        "{
    delete from schema::posts
//1
using
//2
(select post_id from schema::flagged_posts where severity > 5) as flagged where id = flagged.post_id
}",
        "{
    delete from
        schema::posts
    //1
    using
        //2
        (select post_id from schema::flagged_posts where severity > 5) as flagged
    where
        id = flagged.post_id
}"
    );
}
