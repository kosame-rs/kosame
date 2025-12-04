use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    clause::{FromChain, Returning, Where},
    keyword,
    parse_option::ParseOption,
    part::TargetTable,
    pretty::{PrettyPrint, Printer},
    quote_option::QuoteOption,
    visit::Visit,
};

pub struct Delete {
    pub delete_keyword: keyword::delete,
    pub from_keyword: keyword::from,
    pub target_table: TargetTable,
    pub using: Option<Using>,
    pub r#where: Option<Where>,
    pub returning: Option<Returning>,
}

impl Delete {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::delete)
    }
}

pub fn visit_delete<'a>(visit: &mut (impl Visit<'a> + ?Sized), delete: &'a Delete) {
    visit.visit_target_table(&delete.target_table);
    if let Some(inner) = &delete.using {
        visit.visit_using(inner);
    }
    if let Some(inner) = &delete.r#where {
        visit.visit_where(inner);
    }
    if let Some(inner) = &delete.returning {
        visit.visit_returning(inner);
    }
}

impl Parse for Delete {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            delete_keyword: input.parse()?,
            from_keyword: input.parse()?,
            target_table: input.parse()?,
            using: input.call(Using::parse_option)?,
            r#where: input.call(Where::parse_option)?,
            returning: input.call(Returning::parse_option)?,
        })
    }
}

impl ToTokens for Delete {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let target_table = &self.target_table;
        let using = QuoteOption::from(&self.using);
        let r#where = QuoteOption::from(&self.r#where);
        let returning = QuoteOption::from(&self.returning);

        quote! {
            ::kosame::repr::command::Delete::new(
                #target_table,
                #using,
                #r#where,
                #returning,
            )
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for Delete {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.delete_keyword.pretty_print(printer);
        " ".pretty_print(printer);
        self.from_keyword.pretty_print(printer);
        self.target_table.pretty_print(printer);
        self.using.pretty_print(printer);
        self.r#where.pretty_print(printer);
        self.returning.pretty_print(printer);
    }
}

pub struct Using {
    pub using_keyword: keyword::using,
    pub chain: FromChain,
}

impl ParseOption for Using {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::using)
    }
}

pub fn visit_using<'a>(visit: &mut (impl Visit<'a> + ?Sized), using: &'a Using) {
    visit.visit_from_chain(&using.chain);
}

impl Parse for Using {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            using_keyword: input.parse()?,
            chain: input.parse()?,
        })
    }
}

impl ToTokens for Using {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.chain.to_tokens(tokens);
    }
}

impl PrettyPrint for Using {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        printer.scan_break();
        printer.scan_trivia(true, true);
        " ".pretty_print(printer);
        self.using_keyword.pretty_print(printer);
        printer.scan_indent(1);
        printer.scan_break();
        " ".pretty_print(printer);
        self.chain.pretty_print(printer);
        printer.scan_indent(-1);
    }
}
