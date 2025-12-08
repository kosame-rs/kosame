use kosame_dsl::statement::Statement;

use crate::pretty::assert_pretty;

#[test]
fn simple_one_line() {
    assert_pretty!(Statement:
        "{ select   posts.id,   from   schema::posts }",
        "{ select posts.id from schema::posts }"
    );
}

#[test]
fn select_multi_field() {
    assert_pretty!(Statement:
        "{
//1
select   
//2
posts.id,
//3
posts.title,   posts.content,   
//4
from
schema::posts
}",
        "{
    //1
    select
        //2
        posts.id,
        //3
        posts.title,
        posts.content,
    //4
    from
        schema::posts
}"
    );
}

#[test]
fn select_with_where() {
    assert_pretty!(Statement:
        "{
select posts.id, from schema::posts where id = :post_id
}",
        "{ select posts.id from schema::posts where id = :post_id }"
    );
}

#[test]
fn select_with_joins() {
    assert_pretty!(Statement:
        "{
select
    posts.id,
    comments.content,
from
    schema::posts
    left join schema::comments on posts.id = comments.post_id
}",
        "{
    select
        posts.id,
        comments.content,
    from
        schema::posts
        left join schema::comments on posts.id = comments.post_id
}"
    );
}

#[test]
fn select_with_multiple_joins() {
    assert_pretty!(Statement:
        "{
select
    posts.id,
    comments.content,
    users.name,
from
    schema::posts
    inner join schema::comments on posts.id = comments.post_id
    left join schema::users on posts.author_id = users.id
}",
        "{
    select
        posts.id,
        comments.content,
        users.name,
    from
        schema::posts
        inner join schema::comments on posts.id = comments.post_id
        left join schema::users on posts.author_id = users.id
}"
    );
}

#[test]
fn select_with_order_limit() {
    assert_pretty!(Statement:
        "{
select
    posts.id,
from
    schema::posts
order by
    id desc nulls last,
    content asc,
limit
    10
}",
        "{
    select
        posts.id,
    from
        schema::posts
    order by
        id desc nulls last,
        content asc,
    limit
        10
}"
    );
}

#[test]
fn select_with_group_by() {
    assert_pretty!(Statement:
        "{
select
    posts.author_id,
    count(5) as cnt: i64,
from
    schema::posts
group by
    posts.author_id,
}",
        "{
    select
        posts.author_id,
        count(5) as cnt: i64,
    from
        schema::posts
    group by
        posts.author_id,
}"
    );
}

#[test]
fn select_with_cte() {
    assert_pretty!(Statement:
        "{
with posts_with_content as ( select posts.id, from schema::posts where content is not null)
select posts_with_content.id, from posts_with_content }",
        "{
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
        posts_with_content.id,
    from
        posts_with_content
}"
    );
}

#[test]
fn select_with_lateral_join() {
    assert_pretty!(Statement:
        "{
select posts.id, top_comment.id, from schema::posts
left join lateral ( select comments.id, from schema::comments where post_id = posts.id order by 1 desc, limit 1
) as top_comment on true
}",
        "{
    select
        posts.id,
        top_comment.id,
    from
        schema::posts
        left join lateral (
            select
                comments.id,
            from
                schema::comments
            where
                post_id = posts.id
            order by
                1 desc,
            limit
                1
        ) as top_comment on true
}"
    );
}

#[test]
fn union_all() {
    assert_pretty!(Statement:
        "{ select comments.content from schema::comments union all select posts.renamed_title, from schema::posts order by 1 desc limit 20 }",
        "{
    select
        comments.content,
    from
        schema::comments
    union all
        select
            posts.renamed_title,
        from
            schema::posts
    order by
        1 desc,
    limit
        20
}"
    );
}
