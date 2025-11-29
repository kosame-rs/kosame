use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    clause::{Fields, From, FromChain, GroupBy, Having, Where},
    keyword,
    parse_option::ParseOption,
    quote_option::QuoteOption,
    scopes::{ScopeId, Scoped},
    visit::Visit,
};

pub struct Select {
    pub select_keyword: keyword::select,
    pub fields: Fields,
}

pub fn visit_select<'a>(visit: &mut (impl Visit<'a> + ?Sized), select: &'a Select) {
    visit.visit_fields(&select.fields);
}

impl Parse for Select {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            select_keyword: input.call(keyword::select::parse_autocomplete)?,
            fields: input.parse()?,
        })
    }
}

impl ParseOption for Select {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::select)
    }
}

impl ToTokens for Select {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = &self.fields;
        quote! {
            ::kosame::repr::clause::Select::new(#fields)
        }
        .to_tokens(tokens);
    }
}

pub struct SelectCore {
    pub scope_id: ScopeId,
    pub select: Select,
    pub from: Option<From>,
    pub r#where: Option<Where>,
    pub group_by: Option<GroupBy>,
    pub having: Option<Having>,
}

pub fn visit_select_core<'a>(visit: &mut (impl Visit<'a> + ?Sized), select_core: &'a SelectCore) {
    visit.visit_select(&select_core.select);
    if let Some(inner) = select_core.from.as_ref() {
        visit.visit_from(inner);
    }
    if let Some(inner) = select_core.r#where.as_ref() {
        visit.visit_where(inner);
    }
    if let Some(inner) = select_core.group_by.as_ref() {
        visit.visit_group_by(inner);
    }
    if let Some(inner) = select_core.having.as_ref() {
        visit.visit_having(inner);
    }
}

impl Scoped for SelectCore {
    #[inline]
    fn scope_id(&self) -> ScopeId {
        self.scope_id
    }

    #[inline]
    fn from_chain(&self) -> Option<&FromChain> {
        self.from.as_ref().map(|from| &from.chain)
    }
}

impl Parse for SelectCore {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            scope_id: ScopeId::new(),
            select: input.parse()?,
            from: input.call(From::parse_option)?,
            r#where: input.call(Where::parse_option)?,
            group_by: input.call(GroupBy::parse_option)?,
            having: input.call(Having::parse_option)?,
        })
    }
}

impl ParseOption for SelectCore {
    fn peek(input: ParseStream) -> bool {
        Select::peek(input)
    }
}

impl ToTokens for SelectCore {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // self.scope_id.scope(|| {
        let select = &self.select;
        let from = QuoteOption::from(&self.from);
        let r#where = QuoteOption::from(&self.r#where);
        let group_by = QuoteOption::from(&self.group_by);
        let having = QuoteOption::from(&self.having);

        quote! {
            ::kosame::repr::clause::SelectCore::new(
                #select,
                #from,
                #r#where,
                #group_by,
                #having,
            )
        }
        .to_tokens(tokens);
        // });
    }
}
