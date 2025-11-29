use std::fmt::Write;

pub enum Dialect {}

impl crate::Dialect for Dialect {
    fn fmt_ident(formatter: &mut impl Write, name: &str) -> std::fmt::Result {
        write!(formatter, "\"{name}\"")
    }

    fn fmt_bind_param(formatter: &mut impl Write, _name: &str, ordinal: u32) -> std::fmt::Result {
        write!(formatter, "${}", ordinal + 1)
    }
}
