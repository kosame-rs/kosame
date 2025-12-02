use syn::{
    Path, Token,
    parse::{Parse, ParseStream},
};

use crate::{
    parse_option::ParseOption,
    pretty::{PrettyPrint, Printer},
};

pub struct TypeOverride {
    pub colon_token: Token![:],
    pub type_path: Path,
}

impl ParseOption for TypeOverride {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![:])
    }
}

impl Parse for TypeOverride {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            colon_token: input.parse()?,
            type_path: input.parse()?,
        })
    }
}

impl PrettyPrint for TypeOverride {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.colon_token.pretty_print(printer);
        " ".pretty_print(printer);
        self.type_path.pretty_print(printer);
    }
}
