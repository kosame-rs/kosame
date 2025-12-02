use syn::{
    Ident, Path,
    parse::{Parse, ParseStream},
};

use crate::{
    correlations::CorrelationId,
    pretty::{PrettyPrint, Printer},
    visit::Visit,
};

#[derive(Debug)]
pub struct TablePath {
    pub path: Path,
    pub correlation_id: CorrelationId,
}

impl TablePath {
    #[must_use]
    pub fn get_ident(&self) -> Option<&Ident> {
        self.path.get_ident()
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

pub fn visit_table_path<'a>(_visit: &mut (impl Visit<'a> + ?Sized), _table_path: &'a TablePath) {}

impl Parse for TablePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TablePath {
            path: input.parse()?,
            correlation_id: CorrelationId::new(),
        })
    }
}

impl PrettyPrint for TablePath {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.path.pretty_print(printer);
    }
}
