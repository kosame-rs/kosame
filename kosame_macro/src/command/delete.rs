use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Path,
    parse::{Parse, ParseStream},
};

use crate::{clause::*, keyword, path_ext::PathExt, quote_option::QuoteOption, visitor::Visitor};

pub struct Delete {
    pub _delete_keyword: keyword::delete,
    pub _from_keyword: keyword::from,
    pub table: Path,
    pub using: Option<Using>,
    pub r#where: Option<Where>,
    pub returning: Option<Returning>,
}

impl Delete {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::delete)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_table_ref(&self.table);
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
            table: input.parse()?,
            using: input.call(Using::parse_optional)?,
            r#where: input.call(Where::parse_optional)?,
            returning: input.call(Returning::parse_optional)?,
        })
    }
}

impl ToTokens for Delete {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table = &self.table.to_call_site(1);
        let using = QuoteOption(self.using.as_ref());
        let r#where = QuoteOption(self.r#where.as_ref());
        let returning = QuoteOption(self.returning.as_ref());

        quote! {
            ::kosame::repr::command::Delete::new(
                &#table::TABLE,
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
    pub item: FromItem,
}

impl Using {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::using)
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.item.accept(visitor);
    }
}

impl Parse for Using {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _using_keyword: input.parse()?,
            item: input.parse()?,
        })
    }
}

impl ToTokens for Using {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);
    }
}
