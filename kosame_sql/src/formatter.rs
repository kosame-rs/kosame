use std::fmt::Write;

use crate::Dialect;

pub struct Formatter<'a, D> {
    buf: &'a mut (dyn Write + 'a),
    _dialect: std::marker::PhantomData<D>,
}

impl<'a, D> Formatter<'a, D>
where
    D: Dialect,
{
    pub fn new(buf: &'a mut (dyn Write + 'a)) -> Self {
        Self {
            buf,
            _dialect: Default::default(),
        }
    }

    pub fn write_ident(&mut self, ident: &str) -> crate::Result {
        let (prefix, suffix) = D::ident_esc();
        write!(self, "{prefix}{ident}{suffix}")
    }

    pub fn write_bind_param(&mut self, name: &str, ordinal: u32) -> crate::Result {
        D::fmt_bind_param(self, name, ordinal)
    }
}

impl<D> Write for Formatter<'_, D> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.write_str(s)
    }
}
