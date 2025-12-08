mod delete;
mod insert;
mod query;
mod r#select;
mod table;
mod update;

macro_rules! assert_pretty {
    ($ty:ty: $before:literal, $after:literal) => {{
        use kosame_dsl::pretty::{Macro, pretty_print_str};
        assert_eq!(
            pretty_print_str::<Macro<$ty>>($before, 0, 0).unwrap(),
            $after
        );
    }};
}

pub(crate) use assert_pretty;
