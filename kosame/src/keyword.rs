macro_rules! custom_keyword {
    ($kw:ident) => {
        pub mod $kw {
            pub mod $kw {}
        }
    };
}

macro_rules! keyword_group {
    ($group:ident { $($kw:ident),* }) => {
        pub mod $group {
            $(
                pub use super::$kw::*;
            )*
        }
    };
}

custom_keyword!(all);
custom_keyword!(and);
custom_keyword!(asc);
custom_keyword!(by);
custom_keyword!(cast);
custom_keyword!(create);
custom_keyword!(cross);
custom_keyword!(default);
custom_keyword!(delete);
custom_keyword!(desc);
custom_keyword!(distinct);
custom_keyword!(driver);
custom_keyword!(except);
custom_keyword!(first);
custom_keyword!(from);
custom_keyword!(full);
custom_keyword!(group);
custom_keyword!(having);
custom_keyword!(inner);
custom_keyword!(insert);
custom_keyword!(intersect);
custom_keyword!(into);
custom_keyword!(is);
custom_keyword!(join);
custom_keyword!(key);
custom_keyword!(kosame);
custom_keyword!(last);
custom_keyword!(lateral);
custom_keyword!(left);
custom_keyword!(limit);
custom_keyword!(natural);
custom_keyword!(not);
custom_keyword!(null);
custom_keyword!(nulls);
custom_keyword!(offset);
custom_keyword!(on);
custom_keyword!(or);
custom_keyword!(order);
custom_keyword!(__pass);
custom_keyword!(primary);
custom_keyword!(references);
custom_keyword!(rename);
custom_keyword!(returning);
custom_keyword!(right);
custom_keyword!(select);
custom_keyword!(set);
custom_keyword!(__table);
custom_keyword!(table);
custom_keyword!(ty);
custom_keyword!(union);
custom_keyword!(update);
custom_keyword!(using);
custom_keyword!(values);
custom_keyword!(with);

keyword_group!(group_attribute { driver, rename, ty });
keyword_group!(group_column_constraint {
    not,
    default,
    primary,
    references
});
keyword_group!(group_command {
    select,
    insert,
    update,
    delete
});
keyword_group!(group_join {
    left,
    right,
    inner,
    full,
    natural,
    cross
});
keyword_group!(group_order_by_dir { asc, desc });
keyword_group!(group_order_by_nulls { first, last });
keyword_group!(group_set_op {
    union,
    intersect,
    except
});
