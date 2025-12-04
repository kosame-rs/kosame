mod field;
mod from;
mod group_by;
mod having;
mod limit;
mod offset;
mod order_by;
mod returning;
mod select;
mod set;
mod values;
mod r#where;
mod with;

pub use field::*;
pub use from::*;
pub use group_by::*;
pub use having::*;
pub use limit::*;
pub use offset::*;
pub use order_by::*;
pub use returning::*;
pub use select::*;
pub use set::*;
pub use values::*;
pub use r#where::*;
pub use with::*;

use crate::{
    command::SelectCombinator,
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
};

pub fn peek_clause(input: syn::parse::ParseStream) -> bool {
    With::peek(input)
        || From::peek(input)
        || Where::peek(input)
        || GroupBy::peek(input)
        || Having::peek(input)
        || OrderBy::peek(input)
        || Limit::peek(input)
        || Offset::peek(input)
        || Returning::peek(input)
        || Set::peek(input)
        || Values::peek(input)
        || SelectCombinator::peek(input)
}

pub struct Clause<'a> {
    keywords: &'a [&'a dyn PrettyPrint],
    body: &'a dyn PrettyPrint,
    first: bool,
}

impl<'a> Clause<'a> {
    pub fn new(keywords: &'a [&'a dyn PrettyPrint], body: &'a dyn PrettyPrint) -> Self {
        Self {
            keywords,
            body,
            first: false,
        }
    }

    pub fn new_first(keywords: &'a [&'a dyn PrettyPrint], body: &'a dyn PrettyPrint) -> Self {
        Self {
            keywords,
            body,
            first: true,
        }
    }
}

impl PrettyPrint for Clause<'_> {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        if !self.first {
            printer.scan_break();
            " ".pretty_print(printer);
        }
        printer.scan_trivia(!self.first, true);

        for (index, keyword) in self.keywords.iter().enumerate() {
            keyword.pretty_print(printer);
            if index < self.keywords.len() - 1 {
                " ".pretty_print(printer);
            }
        }
        printer.scan_indent(1);
        printer.scan_break();
        " ".pretty_print(printer);
        printer.scan_trivia(false, true);
        self.body.pretty_print(printer);
        printer.scan_indent(-1);
    }
}
