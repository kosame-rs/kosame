use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{clause::*, keyword, part::TargetTable, quote_option::QuoteOption, visitor::Visitor};

pub struct Delete {
    pub _delete_keyword: keyword::delete,
    pub _from_keyword: keyword::from,
    pub target_table: TargetTable,
    pub using: Option<Using>,
    pub r#where: Option<Where>,
    pub returning: Option<Returning>,
}

impl Delete {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::delete)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.target_table.accept(visitor);
        if let Some(inner) = &self.using {
            inner.accept(visitor)
        }
        if let Some(inner) = &self.r#where {
            inner.accept(visitor)
        }
        if let Some(inner) = &self.returning {
            inner.accept(visitor)
        }
    }
}

impl Parse for Delete {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _delete_keyword: input.parse()?,
            _from_keyword: input.parse()?,
            target_table: input.parse()?,
            using: input.call(Using::parse_optional)?,
            r#where: input.call(Where::parse_optional)?,
            returning: input.call(Returning::parse_optional)?,
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

pub struct Using {
    pub _using_keyword: keyword::using,
    pub chain: FromChain,
}

impl Using {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::using)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.chain.accept(visitor);
    }
}

impl Parse for Using {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _using_keyword: input.parse()?,
            chain: input.parse()?,
        })
    }
}

impl ToTokens for Using {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.chain.to_tokens(tokens);
    }
}
