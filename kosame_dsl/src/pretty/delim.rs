use proc_macro2::extra::DelimSpan;

use crate::pretty::{BreakMode, PrettyPrint, Printer};

pub trait Delim {
    fn pretty_print(
        &self,
        printer: &mut Printer<'_>,
        break_mode: Option<BreakMode>,
        f: impl FnOnce(&mut Printer<'_>),
    ) {
        printer.flush_trivia(self.span().open().into());
        self.open_text().pretty_print(printer);
        if let Some(break_mode) = break_mode {
            printer.scan_begin(break_mode);
        }
        printer.scan_break(false);
        f(printer);
        printer.flush_trivia(self.span().close().into());
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
