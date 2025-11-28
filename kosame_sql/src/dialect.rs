use std::fmt::Write;

pub trait Dialect {
    fn fmt_ident(formatter: &mut impl Write, name: &str) -> std::fmt::Result;
    fn fmt_bind_param(formatter: &mut impl Write, name: &str, ordinal: u32) -> std::fmt::Result;
}
