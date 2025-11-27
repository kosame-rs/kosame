use syn::spanned::Spanned;

use super::{PrettyPrint, Printer};

macro_rules! impl_token {
    ($($for:tt)*) => {
        impl PrettyPrint for $($for)* {
            fn pretty_print(&self, printer: &mut Printer<'_>) {
                self.token().pretty_print(printer);
            }
        }
    };
}

impl_token!(syn::LitBool);
impl_token!(syn::LitInt);
impl_token!(syn::LitFloat);
impl_token!(syn::LitStr);

impl PrettyPrint for syn::Attribute {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        if self.meta.path().is_ident("doc") {
            // Docs comments are treated as regular comments by the pretty printer.
            return;
        }

        self.pound_token.pretty_print(printer);
        if let syn::AttrStyle::Inner(not) = &self.style {
            not.pretty_print(printer);
        }
        if let Some(source_text) = self.bracket_token.span.span().source_text() {
            source_text.pretty_print(printer);
        }
        printer.scan_break(true);
    }
}
