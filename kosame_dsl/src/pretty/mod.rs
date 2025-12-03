mod delim;
mod r#macro;
mod printer;
mod ring_buffer;
mod rust;
mod span;
mod text;
mod token;
mod trivia;

pub use delim::*;
pub use r#macro::*;
pub use printer::*;
pub use ring_buffer::*;
pub use span::*;
pub use token::*;
pub use trivia::*;

use syn::parse::Parse;

pub fn pretty_print_macro_str<T>(
    source_text: &str,
    initial_space: isize,
    initial_indent: isize,
) -> syn::Result<String>
where
    T: Parse + PrettyPrint,
{
    let ast: T = syn::parse_str(source_text)?;
    let trivia = Lexer::new(source_text).collect::<Vec<_>>();

    let mut printer = Printer::new(&trivia, initial_space, initial_indent);
    ast.pretty_print(&mut printer);
    Ok(printer.eof())
}

pub trait PrettyPrint {
    fn pretty_print(&self, printer: &mut Printer<'_>);
}

impl<T> PrettyPrint for Option<T>
where
    T: PrettyPrint,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        if let Some(inner) = self {
            inner.pretty_print(printer);
        }
    }
}

impl<T> PrettyPrint for [T]
where
    T: PrettyPrint,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        for item in self {
            item.pretty_print(printer);
        }
    }
}

impl<T> PrettyPrint for syn::punctuated::Punctuated<T, syn::Token![,]>
where
    T: PrettyPrint,
{
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        for (index, item) in self.pairs().enumerate() {
            item.value().pretty_print(printer);
            if item.punct().is_some() {
                printer.scan_no_break_trivia();
            }
            if index == self.len() - 1 {
                printer.scan_text(",".into(), TextMode::Break);
                printer.scan_trivia();
            } else {
                item.punct().unwrap().pretty_print(printer);
                printer.scan_trivia();
                printer.scan_break(false);
                " ".pretty_print(printer);
            }
        }
    }
}
