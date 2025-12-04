use kosame_dsl::query::Query;

use crate::pretty::assert_pretty;

#[test]
fn simple() {
    assert_pretty!(Query:
        "{ schema::posts { * } }",
        "{ schema::posts { * } }"
    );
}

#[test]
fn simple_fields() {
    assert_pretty!(Query:
        "{ schema::posts { id,  title,   content } }",
        "{ schema::posts { id, title, content } }"
    );
}

#[test]
fn with_where() {
    assert_pretty!(Query:
        "{ schema::posts { *, where id = :post_id } }",
        "{ schema::posts { *, where id = :post_id } }"
    );
}

#[test]
fn nested_simple() {
    assert_pretty!(Query:
        "{
schema::posts {
    *,
    comments { id, content }
}
}",
        "{ schema::posts { *, comments { id, content } } }"
    );
}

#[test]
fn nested_break() {
    assert_pretty!(Query:
        "{
schema::posts {
    *,
    comments { id, content, upvotes, post_id, very_long_field_name, another_long_field }
}
}",
        "{
    schema::posts {
        *,
        comments {
            id,
            content,
            upvotes,
            post_id,
            very_long_field_name,
            another_long_field,
        },
    }
}"
    );
}

#[test]
fn with_order_limit() {
    assert_pretty!(Query:
        "{
schema::posts {
    *,
    comments {
        id,
        content,
        order by upvotes desc,
        limit 5
    }
}
}",
        "{
    schema::posts { *, comments { id, content order by upvotes desc limit 5 } }
}"
    );
}

#[test]
fn with_attributes() {
    assert_pretty!(Query:
        "{
#[derive(Clone)]
schema::posts {
    *,
    #[serde(rename = \"serdeContent\")]
    content{*}
}
}",
        "{
    #[derive(Clone)]
    schema::posts { *, #[serde(rename = \"serdeContent\")] content { * } }
}"
    );
}

#[test]
fn line_comments_simple() {
    assert_pretty!(Query:
        "{
// comment at start
schema::posts {
    id,// after field
    title// after another field
}
}",
        "{
    // comment at start
    schema::posts {
        id, // after field
        title, // after another field
    }
}"
    );
}
