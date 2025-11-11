use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{clause::*, keyword, part::TargetTable, quote_option::QuoteOption, visitor::Visitor};

pub struct Update {
    pub _update_keyword: keyword::update,
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

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.target_table.accept(visitor);
        self.set.accept(visitor);
        if let Some(inner) = &self.r#where {
            inner.accept(visitor)
        }
        if let Some(inner) = &self.returning {
            inner.accept(visitor)
        }
    }
}

impl Parse for Update {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _update_keyword: input.call(keyword::update::parse_autocomplete)?,
            target_table: input.parse()?,
            set: input.parse()?,
            from: input.call(From::parse_optional)?,
            r#where: input.call(Where::parse_optional)?,
            returning: input.call(Returning::parse_optional)?,
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
