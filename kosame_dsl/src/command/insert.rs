use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{clause::*, keyword, part::TargetTable, quote_option::QuoteOption, visitor::Visitor};

pub struct Insert {
    pub _insert_keyword: keyword::insert,
    pub _into_keyword: keyword::into,
    pub target_table: TargetTable,
    pub values: Values,
    pub returning: Option<Returning>,
}

impl Insert {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::insert)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.target_table.accept(visitor);
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
            target_table: input.parse()?,
            values: input.parse()?,
            returning: input.call(Returning::parse_optional)?,
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
