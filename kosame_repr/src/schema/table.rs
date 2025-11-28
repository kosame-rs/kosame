use crate::Ident;

use super::{Column, Relation};

pub struct Table<'a> {
    name: Ident<'a>,
    columns: &'a [&'a Column<'a>],
    relations: &'a [&'a Relation<'a>],
}

impl<'a> Table<'a> {
    #[inline]
    #[must_use]
    pub const fn new(
        name: &'a str,
        columns: &'a [&'a Column],
        relations: &'a [&'a Relation],
    ) -> Self {
        Self {
            name: Ident::new(name),
            columns,
            relations,
        }
    }

    #[inline]
    #[must_use]
    pub const fn name(&self) -> Ident<'a> {
        self.name
    }

    #[inline]
    #[must_use]
    pub const fn columns(&self) -> &'a [&'a Column<'a>] {
        self.columns
    }

    #[inline]
    #[must_use]
    pub const fn relations(&self) -> &'a [&'a Relation<'a>] {
        self.relations
    }
}
