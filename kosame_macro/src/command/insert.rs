use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Path,
    parse::{Parse, ParseStream},
};

use crate::{clause::*, keyword, path_ext::PathExt, quote_option::QuoteOption, visitor::Visitor};

pub struct Insert {
    pub _insert_keyword: keyword::insert,
    pub _into_keyword: keyword::into,
    pub table: Path,
    pub values: Values,
    pub returning: Option<Returning>,
}

impl Insert {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::insert)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_table_ref(&self.table);
        self.values.accept(visitor);
        if let Some(inner) = &self.returning {
            inner.accept(visitor)
        }
    }
}

impl Parse for Insert {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _insert_keyword: input.parse()?,
            _into_keyword: input.parse()?,
            table: input.parse()?,
            values: input.parse()?,
            returning: input.call(Returning::parse_optional)?,
        })
    }
}

impl ToTokens for Insert {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table = &self.table.to_call_site(1);
        let values = &self.values;
        let returning = QuoteOption(self.returning.as_ref());

        quote! {
            ::kosame::repr::command::Insert::new(
                &#table::TABLE,
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
