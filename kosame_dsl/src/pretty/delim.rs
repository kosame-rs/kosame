use proc_macro2::extra::DelimSpan;

use crate::pretty::{BreakMode, PrettyPrint, Printer};

pub trait Delim {
    fn pretty_print(
        &self,
        printer: &mut Printer<'_>,
        break_mode: Option<BreakMode>,
        f: impl FnOnce(&mut Printer<'_>),
    ) {
        printer.move_cursor(self.span().open().start());
        printer.flush_trivia();
        self.open_text().pretty_print(printer);
        if let Some(break_mode) = break_mode {
            printer.scan_begin(break_mode);
        }
        printer.scan_indent(1);
        printer.scan_break(false);
        f(printer);
        printer.move_cursor(self.span().close().start());
        printer.flush_trivia();
        printer.scan_indent(-1);
        printer.scan_break(false);
        if break_mode.is_some() {
            printer.scan_end();
        }
        self.close_text().pretty_print(printer);
    }

    #[must_use]
    fn open_text(&self) -> &'static str;

    #[must_use]
    fn close_text(&self) -> &'static str;

    #[must_use]
    fn span(&self) -> DelimSpan;
}

impl Delim for syn::token::Paren {
    fn open_text(&self) -> &'static str {
        "("
    }

    fn close_text(&self) -> &'static str {
        ")"
    }

    fn span(&self) -> DelimSpan {
        self.span
    }
}

impl Delim for syn::token::Bracket {
    fn open_text(&self) -> &'static str {
        "["
    }

    fn close_text(&self) -> &'static str {
        "]"
    }

    fn span(&self) -> DelimSpan {
        self.span
    }
}

impl Delim for syn::token::Brace {
    fn open_text(&self) -> &'static str {
        "{"
    }

    fn close_text(&self) -> &'static str {
        "}"
    }

    fn span(&self) -> DelimSpan {
        self.span
    }
}
