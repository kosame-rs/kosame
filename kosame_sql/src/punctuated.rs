use std::fmt::Write;

use crate::{Dialect, FmtSql, Formatter};

pub struct Punctuated<'a, T> {
    items: &'a [T],
    punctuation: &'a str,
}

impl<'a, T> Punctuated<'a, T> {
    #[inline]
    pub const fn new(items: &'a [T], punctuation: &'a str) -> Self {
        Self { items, punctuation }
    }

    #[inline]
    #[must_use]
    pub const fn items(&self) -> &'a [T] {
        self.items
    }

    #[inline]
    #[must_use]
    pub const fn punctuation(&self) -> &'a str {
        self.punctuation
    }
}

impl<T> FmtSql for Punctuated<'_, T>
where
    T: FmtSql,
{
    fn fmt_sql<D>(&self, formatter: &mut Formatter<D>) -> crate::Result
    where
        D: Dialect,
    {
        for (index, item) in self.items.iter().enumerate() {
            item.fmt_sql(formatter)?;
            if index != self.items.len() - 1 {
                formatter.write_str(self.punctuation)?;
                formatter.write_str(" ")?;
            }
        }
        Ok(())
    }
}
