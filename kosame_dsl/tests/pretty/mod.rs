mod query;
mod table;

macro_rules! assert_pretty {
    ($ty:ty: $before:literal, $after:literal) => {{
        use kosame_dsl::pretty::{Macro, pretty_print_macro_str};
        assert_eq!(
            pretty_print_macro_str::<Macro<$ty>>($before, 0, 0).unwrap(),
            $after
        );
    }};
}

pub(crate) use assert_pretty;
