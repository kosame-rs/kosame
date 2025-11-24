mod printer;
mod ring_buffer;
mod span;
mod text;
mod trivia;

pub use printer::*;
pub use ring_buffer::*;
pub use span::*;
pub use text::*;
pub use trivia::*;

use syn::parse::Parse;

pub trait PrettyPrint {
    fn pretty_print(&self, printer: &mut Printer<'_>);
}

pub fn pretty_print_macro_str<T>(
    source_text: &str,
    initial_space: usize,
    initial_indent: usize,
) -> syn::Result<String>
where
    T: Parse + PrettyPrint,
{
    let ast: T = syn::parse_str(source_text)?;
    let trivia = Lexer::new(source_text).collect::<Vec<_>>();

    let mut printer = Printer::new(&trivia, initial_space, initial_indent);
    printer.scan_begin(BreakMode::Consistent);
    printer.scan_text_with_mode(" ", TextMode::NoBreak);
    ast.pretty_print(&mut printer);
    printer.scan_text_with_mode(" ", TextMode::NoBreak);
    printer.scan_end();

    Ok(printer.eof())
}
