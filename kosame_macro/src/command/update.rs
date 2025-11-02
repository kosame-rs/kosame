use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Path,
    parse::{Parse, ParseStream},
};

use crate::{
    clause::*, keyword, path_ext::PathExt, quote_option::QuoteOption, scope_module::ScopeModule,
    visitor::Visitor,
};

pub struct Update {
    pub _update_keyword: keyword::update,
    pub table: Path,
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
        visitor.visit_table_ref(&self.table);
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
            table: input.parse()?,
            set: input.parse()?,
            from: input.call(From::parse_optional)?,
            r#where: input.call(Where::parse_optional)?,
            returning: input.call(Returning::parse_optional)?,
        })
    }
}

impl ToTokens for Update {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table = &self.table.to_call_site(1);
        let set = &self.set;
        let from = QuoteOption(self.from.as_ref());
        let r#where = QuoteOption(self.r#where.as_ref());
        let returning = QuoteOption(self.returning.as_ref());

        quote! {
            ::kosame::repr::command::Update::new(
                &#table::TABLE,
                #set,
                #from,
                #r#where,
                #returning,
            )
        }
        .to_tokens(tokens);
    }
}
