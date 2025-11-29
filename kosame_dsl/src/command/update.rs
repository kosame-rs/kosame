use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    clause::{From, Returning, Set, Where},
    keyword,
    parse_option::ParseOption,
    part::TargetTable,
    quote_option::QuoteOption,
    visit::Visit,
};

pub struct Update {
    pub update_keyword: keyword::update,
    pub target_table: TargetTable,
    pub set: Set,
    pub from: Option<From>,
    pub r#where: Option<Where>,
    pub returning: Option<Returning>,
}

impl Update {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::update)
    }
}

pub fn visit_update<'a>(visit: &mut (impl Visit<'a> + ?Sized), update: &'a Update) {
    visit.visit_target_table(&update.target_table);
    visit.visit_set(&update.set);
    if let Some(inner) = &update.from {
        visit.visit_from(inner);
    }
    if let Some(inner) = &update.r#where {
        visit.visit_where(inner);
    }
    if let Some(inner) = &update.returning {
        visit.visit_returning(inner);
    }
}

impl Parse for Update {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            update_keyword: input.call(keyword::update::parse_autocomplete)?,
            target_table: input.parse()?,
            set: input.parse()?,
            from: input.call(From::parse_option)?,
            r#where: input.call(Where::parse_option)?,
            returning: input.call(Returning::parse_option)?,
        })
    }
}

impl ToTokens for Update {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let target_table = &self.target_table;
        let set = &self.set;
        let from = QuoteOption::from(&self.from);
        let r#where = QuoteOption::from(&self.r#where);
        let returning = QuoteOption::from(&self.returning);

        quote! {
            ::kosame::repr::command::Update::new(
                #target_table,
                #set,
                #from,
                #r#where,
                #returning,
            )
        }
        .to_tokens(tokens);
    }
}
