use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub struct QuoteOption<T>(pub Option<T>);

impl<T> ToTokens for QuoteOption<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(inner) = &self.0 {
            quote! { ::core::option::Option::Some(#inner) }
        } else {
            quote! { ::core::option::Option::None }
        }
        .to_tokens(tokens);
    }
}

impl<'a, T> From<&'a Option<T>> for QuoteOption<&'a T> {
    fn from(value: &'a Option<T>) -> Self {
        QuoteOption(value.as_ref())
    }
}
