use kosame_dsl::statement::Statement;

use crate::pretty::assert_pretty;

#[test]
fn simple_one_line() {
    assert_pretty!(Statement:
        "{ update   schema::posts   set   title = \"new title\" }",
        "{ update schema::posts set title = \"new title\" }"
    );
}

#[test]
fn update_with_where() {
    assert_pretty!(Statement:
        "{ update schema::posts set title = \"new title\" where id = :post_id }",
        "{
    update
        schema::posts
    set
        title = \"new title\",
    where
        id = :post_id
}"
    );
}

#[test]
fn update_multi_set() {
    assert_pretty!(Statement:
        "{
update
    schema::comments
set
    upvotes = upvotes + 1,
where
    id = :comment_id
}",
        "{
    update
        schema::comments
    set
        upvotes = upvotes + 1,
    where
        id = :comment_id
}"
    );
}

#[test]
fn update_multi_columns() {
    assert_pretty!(Statement:
        "{
update schema::posts set title = :new_title, content = :new_content, updated_at = now()
}",
        "{
    update
        schema::posts
    set
        title = :new_title,
        content = :new_content,
        updated_at = now(),
}"
    );
}

#[test]
fn update_with_returning() {
    assert_pretty!(Statement:
        "{
update
    schema::comments
set
    upvotes = upvotes + 1,
where
    id = :comment_id
returning
    comments.upvotes as new_upvotes,
}",
        "{
    update
        schema::comments
    set
        upvotes = upvotes + 1,
    where
        id = :comment_id
    returning
        comments.upvotes as new_upvotes,
}"
    );
}

#[test]
fn update_returning_multiple() {
    assert_pretty!(Statement:
        "{
update schema::posts set title = :title, content = :content returning posts.id, posts.title, posts.updated_at,
}",
        "{
    update
        schema::posts
    set
        title = :title,
        content = :content,
    returning
        posts.id,
        posts.title,
        posts.updated_at,
}"
    );
}

#[test]
fn update_with_line_comments() {
    assert_pretty!(Statement:
        "{
// Update comment upvotes
update
    schema::comments
set
    // Increment by one
    upvotes = upvotes + 1,
where
    id = :comment_id
// Return the new value
returning
    comments.upvotes as new_upvotes,
}",
        "{
    // Update comment upvotes
    update
        schema::comments
    set
        // Increment by one
        upvotes = upvotes + 1,
    where
        id = :comment_id
    // Return the new value
    returning
        comments.upvotes as new_upvotes,
}"
    );
}

#[test]
fn update_with_block_comments() {
    assert_pretty!(Statement:
        "{
update /*table*/ schema::posts set /*title*/ title = :title, /*content*/ content = :content
}",
        "{
    update
        schema::posts
    /*table*/
    set
        /*title*/
        title = :title, /*content*/
        content = :content,
}"
    );
}

#[test]
fn update_complex_expressions() {
    assert_pretty!(Statement:
        "{
update
    schema::posts
set
    view_count = view_count + 1,
    last_viewed = now(),
    score = (upvotes * 2) - (downvotes / 2),
where
    id = :post_id and published = true
returning
    posts.view_count,
}",
        "{
    update
        schema::posts
    set
        view_count = view_count + 1,
        last_viewed = now(),
        score = (upvotes * 2) - (downvotes / 2),
    where
        id = :post_id and published = true
    returning
        posts.view_count,
}"
    );
}
