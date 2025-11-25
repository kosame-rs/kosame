use syn::spanned::Spanned;

use super::PrettyPrint;

impl PrettyPrint for syn::Attribute {
    fn pretty_print(&self, printer: &mut super::Printer<'_>) {
        if self.meta.path().is_ident("doc") {
            // Docs comments are treated as regular comments by the pretty printer.
            return;
        }

        self.pound_token.pretty_print(printer);
        if let syn::AttrStyle::Inner(not) = &self.style {
            not.pretty_print(printer)
        }
        if let Some(source_text) = self.bracket_token.span.span().source_text() {
            printer.scan_text(source_text);
        }
        printer.scan_break(true);
    }
}
