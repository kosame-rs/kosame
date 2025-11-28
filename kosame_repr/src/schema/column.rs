use crate::{Ident, expr::Expr};

pub struct Column<'a> {
    pub name: Ident<'a>,
    pub data_type: Ident<'a>,
    pub primary_key: bool,
    pub not_null: bool,
    pub default: Option<&'a Expr<'a>>,
}

impl<'a> Column<'a> {
    #[inline]
    #[must_use]
    pub const fn new(
        name: &'a str,
        data_type: &'a str,
        primary_key: bool,
        not_null: bool,
        default: Option<&'a Expr<'a>>,
    ) -> Self {
        Self {
            name: Ident::new(name),
            data_type: Ident::new(data_type),
            primary_key,
            not_null,
            default,
        }
    }

    #[inline]
    #[must_use]
    pub const fn name(&self) -> Ident<'a> {
        self.name
    }

    #[inline]
    #[must_use]
    pub const fn data_type(&self) -> Ident<'a> {
        self.data_type
    }

    #[inline]
    #[must_use]
    pub const fn primary_key(&self) -> bool {
        self.primary_key
    }

    #[inline]
    #[must_use]
    pub const fn not_null(&self) -> bool {
        self.not_null
    }

    #[inline]
    #[must_use]
    pub const fn default(&self) -> Option<&'a Expr<'_>> {
        self.default
    }
}
