use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

use crate::{
    keyword,
    pretty::{PrettyPrint, Printer},
};

pub enum SetOp {
    Union(keyword::union),
    Intersect(keyword::intersect),
    Except(keyword::except),
}

impl Parse for SetOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::union) {
            Ok(Self::Union(input.parse()?))
        } else if lookahead.peek(keyword::intersect) {
            Ok(Self::Intersect(input.parse()?))
        } else if lookahead.peek(keyword::except) {
            Ok(Self::Except(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for SetOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Union(_) => {
                quote! { ::kosame::repr::part::SetOp::Union }
            }
            Self::Intersect(_) => {
                quote! { ::kosame::repr::part::SetOp::Intersect }
            }
            Self::Except(_) => {
                quote! { ::kosame::repr::part::SetOp::Except }
            }
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for SetOp {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::Union(inner) => inner.pretty_print(printer),
            Self::Intersect(inner) => inner.pretty_print(printer),
            Self::Except(inner) => inner.pretty_print(printer),
        }
    }
}

pub enum SetQuantifier {
    All(keyword::all),
    Distinct,
}

impl Parse for SetQuantifier {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::all) {
            Ok(Self::All(input.parse()?))
        } else {
            Ok(Self::Distinct)
        }
    }
}

impl ToTokens for SetQuantifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::All(_) => {
                quote! { ::kosame::repr::part::SetQuantifier::All }
            }
            Self::Distinct => {
                quote! { ::kosame::repr::part::SetQuantifier::Distinct }
            }
        }
        .to_tokens(tokens);
    }
}

impl PrettyPrint for SetQuantifier {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        match self {
            Self::All(all) => {
                " ".pretty_print(printer);
                all.pretty_print(printer);
            }
            Self::Distinct => {}
        }
    }
}
