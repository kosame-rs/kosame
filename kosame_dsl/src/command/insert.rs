use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    clause::{Returning, Values},
    keyword,
    parse_option::ParseOption,
    part::TargetTable,
    pretty::{PrettyPrint, Printer},
    quote_option::QuoteOption,
    visit::Visit,
};

pub struct Insert {
    pub insert_keyword: keyword::insert,
    pub into_keyword: keyword::into,
    pub target_table: TargetTable,
    pub values: Values,
    pub returning: Option<Returning>,
}

impl Insert {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::insert)
    }
}

pub fn visit_insert<'a>(visit: &mut (impl Visit<'a> + ?Sized), insert: &'a Insert) {
    visit.visit_target_table(&insert.target_table);
    visit.visit_values(&insert.values);
    if let Some(inner) = &insert.returning {
        visit.visit_returning(inner);
    }
}

impl Parse for Insert {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            insert_keyword: input.parse()?,
            into_keyword: input.parse()?,
            target_table: input.parse()?,
            values: input.parse()?,
            returning: input.call(Returning::parse_option)?,
        })
    }
}

impl ToTokens for Insert {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let target_table = &self.target_table;
        let values = &self.values;
        let returning = QuoteOption::from(&self.returning);

        quote! {
            ::kosame::repr::command::Insert::new(
                #target_table,
                {
                    mod scope {}
                    #values
                },
                #returning,
            )
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for Insert {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.insert_keyword.pretty_print(printer);
        " ".pretty_print(printer);
        self.into_keyword.pretty_print(printer);
        printer.scan_indent(1);
        printer.scan_break();
        " ".pretty_print(printer);
        self.target_table.pretty_print(printer);
        printer.scan_indent(-1);
        self.values.pretty_print(printer);
        self.returning.pretty_print(printer);
    }
}
