use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::pretty::{PrettyPrint, pretty_print_ast};

pub struct Doc<'a, T> {
    inner: &'a T,
}

impl<'a, T> Doc<'a, T> {
    pub fn new(inner: &'a T) -> Self {
        Self { inner }
    }
}

impl<T> ToTokens for Doc<'_, T>
where
    T: PrettyPrint,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let doc = pretty_print_ast(self.inner);
        quote! { #[doc = #doc] }.to_tokens(tokens);
    }
}
