use syn::{
    Ident, Path, PathSegment, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::visitor::Visitor;

pub struct TablePath {
    pub leading_colon: Option<Token![::]>,
    pub segments: Punctuated<Ident, Token![::]>,
}

impl TablePath {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_table_path(self);
    }

    pub fn get_ident(&self) -> Option<&Ident> {
        if self.leading_colon.is_some() || self.segments.len() > 1 {
            return None;
        }
        self.segments.last()
    }

    pub fn to_path(&self) -> Path {
        Path {
            leading_colon: self.leading_colon,
            segments: self
                .segments
                .iter()
                .map(|ident| PathSegment {
                    ident: ident.clone(),
                    arguments: syn::PathArguments::None,
                })
                .collect(),
        }
    }
}

impl Parse for TablePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let leading_colon = input.peek(Token![::]).then(|| input.parse()).transpose()?;
        let mut segments = Punctuated::new();
        segments.push_value(input.parse()?);
        while input.peek(Token![::]) {
            segments.push_punct(input.parse()?);
            segments.push_value(input.parse()?);
        }

        Ok(Self {
            leading_colon,
            segments,
        })
    }
}
