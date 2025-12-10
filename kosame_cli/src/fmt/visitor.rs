use kosame_dsl::pretty::{MARGIN, Macro, pretty_print_str};
use proc_macro2::LineColumn;
use syn::{spanned::Spanned, visit::Visit};

pub(super) struct Replace {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) replacement: String,
}

#[derive(Default)]
pub(super) struct Visitor {
    pub(super) indent: isize,
    pub(super) replacements: Vec<Replace>,
    pub(super) errors: Vec<Error>,
}

pub(super) struct Error {
    pub(super) start: LineColumn,
    pub(super) inner: syn::Error,
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.indent += 1;
        syn::visit::visit_item_mod(self, node);
        self.indent -= 1;
    }
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.indent += 1;
        syn::visit::visit_item_impl(self, node);
        self.indent -= 1;
    }
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.indent += 1;
        syn::visit::visit_item_trait(self, node);
        self.indent -= 1;
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.indent += 1;
        syn::visit::visit_item_fn(self, node);
        self.indent -= 1;
    }
    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        let name = &i.path.segments.last().expect("paths cannot be empty").ident;
        let span = i.delimiter.span().span();
        let source_text = span.source_text().unwrap();
        let initial_space = MARGIN - isize::try_from(span.start().column).unwrap();
        let initial_indent = self.indent;

        let result = match name.to_string().as_ref() {
            "table" | "pg_table" => Some(pretty_print_str::<Macro<kosame_dsl::schema::Table>>(
                &source_text,
                initial_space,
                initial_indent,
            )),
            "query" | "pg_query" => Some(pretty_print_str::<Macro<kosame_dsl::query::Query>>(
                &source_text,
                initial_space,
                initial_indent,
            )),
            "statement" | "pg_statement" => Some(pretty_print_str::<
                Macro<kosame_dsl::statement::Statement>,
            >(
                &source_text, initial_space, initial_indent
            )),
            _ => None,
        };

        match result {
            Some(Ok(replacement)) => self.replacements.push(Replace {
                start: span.byte_range().start,
                end: span.byte_range().end,
                replacement,
            }),
            Some(Err(error)) => {
                let start = LineColumn {
                    line: error.span().start().line + span.start().line - 1,
                    column: match error.span().start().line {
                        1 => error.span().start().column + span.start().column,
                        _ => error.span().start().column,
                    },
                };
                self.errors.push(Error {
                    start,
                    inner: error,
                });
            }
            None => {}
        }
    }
}
