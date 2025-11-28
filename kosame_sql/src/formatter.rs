use std::{fmt::Write, marker::PhantomData};

use crate::Dialect;

pub struct Formatter<'a, D> {
    buf: &'a mut (dyn Write + 'a),
    _dialect: PhantomData<D>,
}

impl<'a, D> Formatter<'a, D>
where
    D: Dialect,
{
    pub fn new(buf: &'a mut (dyn Write + 'a)) -> Self {
        Self {
            buf,
            _dialect: PhantomData,
        }
    }
}

impl<D> Write for Formatter<'_, D> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.write_str(s)
    }
}
