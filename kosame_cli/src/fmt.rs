use std::io::Read;

use clap::Args;
use kosame_dsl::pretty::{MARGIN, Macro, pretty_print_macro_str};
use syn::spanned::Spanned;

#[derive(Args)]
#[command(version, about = "Format the content of Kosame macro invocations in Rust source files", long_about = None)]
pub struct Fmt {
    #[arg(short, long)]
    file: Option<std::path::PathBuf>,
}

impl Fmt {
    pub fn run(&self) -> anyhow::Result<()> {
        struct Replace {
            start: usize,
            end: usize,
            replacement: String,
        }

        #[derive(Default)]
        struct Visitor {
            indent: isize,
            replacements: Vec<Replace>,
            errors: Vec<syn::Error>,
        }

        use syn::visit::Visit;
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
                    "table" | "pg_table" => Some(pretty_print_macro_str::<
                        Macro<kosame_dsl::schema::Table>,
                    >(
                        &source_text, initial_space, initial_indent
                    )),
                    "query" | "pg_query" => Some(pretty_print_macro_str::<
                        Macro<kosame_dsl::query::Query>,
                    >(
                        &source_text, initial_space, initial_indent
                    )),
                    "statement" | "pg_statement" => {
                        Some(pretty_print_macro_str::<
                            Macro<kosame_dsl::statement::Statement>,
                        >(
                            &source_text, initial_space, initial_indent
                        ))
                    }
                    _ => None,
                };

                match result {
                    Some(Ok(replacement)) => self.replacements.push(Replace {
                        start: span.byte_range().start,
                        end: span.byte_range().end,
                        replacement,
                    }),
                    Some(Err(error)) => {
                        self.errors.push(error);
                    }
                    None => {}
                }
            }
        }

        let input = if let Some(file) = &self.file {
            std::fs::read_to_string(file)?
        } else {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        };
        let mut output = String::new();

        let file = syn::parse_file(&input)?;
        let mut visitor = Visitor::default();
        visitor.visit_file(&file);

        if !visitor.errors.is_empty() {
            anyhow::bail!(visitor.errors.into_iter().next().unwrap());
        }

        let mut current_index = 0;
        for replacement in visitor.replacements {
            output.push_str(&input[current_index..replacement.start]);
            output.push_str(&replacement.replacement);
            current_index = replacement.end;
        }

        output.push_str(&input[current_index..]);

        match &self.file {
            Some(file) => std::fs::write(file, output).unwrap(),
            None => print!("{output}"),
        }

        Ok(())
    }
}
