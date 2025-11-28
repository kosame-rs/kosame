use crate::Ident;

use super::Column;

pub struct Relation<'a> {
    name: Ident<'a>,
    source_table: Ident<'a>,
    source_columns: &'a [&'a Column<'a>],
    target_table: Ident<'a>,
    target_columns: &'a [&'a Column<'a>],
}

impl<'a> Relation<'a> {
    #[inline]
    #[must_use]
    pub const fn new(
        name: &'a str,
        source_table: &'a str,
        source_columns: &'a [&'a Column],
        target_table: &'a str,
        target_columns: &'a [&'a Column],
    ) -> Self {
        Self {
            name: Ident::new(name),
            source_table: Ident::new(source_table),
            source_columns,
            target_table: Ident::new(target_table),
            target_columns,
        }
    }

    #[inline]
    #[must_use]
    pub const fn name(&self) -> Ident<'a> {
        self.name
    }

    #[inline]
    #[must_use]
    pub const fn source_table(&self) -> Ident<'a> {
        self.source_table
    }

    #[inline]
    #[must_use]
    pub const fn source_columns(&self) -> &'a [&'a Column<'a>] {
        self.source_columns
    }

    #[inline]
    #[must_use]
    pub const fn target_table(&self) -> Ident<'a> {
        self.target_table
    }

    #[inline]
    #[must_use]
    pub const fn target_columns(&self) -> &'a [&'a Column<'_>] {
        self.target_columns
    }

    #[inline]
    pub fn column_pairs(&self) -> impl Iterator<Item = (&'a Column<'a>, &'a Column<'a>)> {
        self.source_columns
            .iter()
            .zip(self.target_columns)
            .map(|(a, b)| (*a, *b))
    }
}
