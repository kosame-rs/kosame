use syn::{
    Ident, Path,
    parse::{Parse, ParseStream},
};

use crate::{correlations::CorrelationId, visitor::Visitor};

#[derive(Debug)]
pub struct TablePath {
    pub path: Path,
    pub correlation_id: CorrelationId,
}

impl TablePath {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_table_path(self);
    }

    #[must_use]
    pub fn get_ident(&self) -> Option<&Ident> {
        self.path.get_ident()
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

impl Parse for TablePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TablePath {
            path: input.parse()?,
            correlation_id: CorrelationId::new(),
        })
    }
}
