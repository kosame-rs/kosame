use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use crate::{clause::peek_clause, keyword, part::ColumnList, quote_option::QuoteOption};

pub struct TableAlias {
    pub _as_token: Option<Token![as]>,
    pub name: Ident,
    pub columns: Option<ColumnList>,
}

impl TableAlias {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        if input.is_empty() || peek_clause(input) {
            return Ok(None);
        }
        macro_rules! check {
            ($kw:expr) => {
                if input.peek($kw) {
                    return Ok(None);
                }
            };
        }
        check!(keyword::inner);
        check!(keyword::left);
        check!(keyword::right);
        check!(keyword::full);
        check!(keyword::on);

        check!(keyword::natural);
        check!(keyword::cross);

        Ok(Some(input.parse()?))
    }
}

impl Parse for TableAlias {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _as_token: input.peek(Token![as]).then(|| input.parse()).transpose()?,
            name: input.parse()?,
            columns: {
                let columns = input
                    .peek(syn::token::Paren)
                    .then(|| input.parse::<ColumnList>())
                    .transpose()?;
                if let Some(columns) = columns {
                    return Err(syn::Error::new(
                        columns._paren_token.span.span(),
                        "column aliases are currently unsupported",
                    ));
                }

                columns
            },
        })
    }
}

impl ToTokens for TableAlias {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name.to_string();
        let columns = QuoteOption::from(&self.columns);
        quote! {
            ::kosame::repr::part::TableAlias::new(#name, #columns)
        }
        .to_tokens(tokens);
    }
}
