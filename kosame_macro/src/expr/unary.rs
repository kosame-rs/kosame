use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

use crate::{data_type::InferredType, keyword, scopes::ScopeId};

use super::{Expr, Visitor};

pub struct Unary {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
}

impl Unary {
    pub fn new(op: UnaryOp, operand: Expr) -> Self {
        Self {
            op,
            operand: Box::new(operand),
        }
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.operand.accept(visitor);
    }

    pub fn infer_name(&self) -> Option<&Ident> {
        None
    }

    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType> {
        None
    }
}

impl ToTokens for Unary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let op = &self.op;
        let operand = &self.operand;
        quote! {
            ::kosame::repr::expr::Unary::new(#op, &#operand)
        }
        .to_tokens(tokens);
    }
}

#[allow(unused)]
pub enum UnaryOp {
    Not(keyword::not),
}

impl UnaryOp {
    pub fn peek(input: ParseStream) -> bool {
        input.fork().parse::<UnaryOp>().is_ok()
    }

    pub fn precedence(&self) -> u32 {
        // Taken from https://www.postgresql.org/docs/18/sql-syntax-lexical.html#SQL-PRECEDENCE
        match self {
            Self::Not(_) => 3,
        }
    }
}

impl Parse for UnaryOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::not) {
            return Ok(Self::Not(input.parse()?));
        }

        Err(lookahead.error())
    }
}

impl ToTokens for UnaryOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(..) => quote! { ::kosame::repr::expr::UnaryOp::$variant }.to_tokens(tokens)),*
                }
            };
        }

        branches!(Not);
    }
}
