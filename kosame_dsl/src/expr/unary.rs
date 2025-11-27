use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

use crate::{
    inferred_type::InferredType,
    keyword,
    pretty::{PrettyPrint, Printer},
    scopes::ScopeId,
};

use super::{Expr, Visitor};

pub struct Unary {
    pub op: UnOp,
    pub operand: Box<Expr>,
}

impl Unary {
    #[must_use]
    pub fn new(op: UnOp, operand: Expr) -> Self {
        Self {
            op,
            operand: Box::new(operand),
        }
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.operand.accept(visitor);
    }

    #[must_use]
    pub fn infer_name(&self) -> Option<&Ident> {
        None
    }

    #[must_use]
    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType<'_>> {
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

impl PrettyPrint for Unary {
    fn pretty_print(&self, printer: &mut Printer) {
        self.op.pretty_print(printer);
        " ".pretty_print(printer);
        self.operand.pretty_print(printer);
    }
}

#[allow(unused)]
pub enum UnOp {
    Not(keyword::not),
}

impl UnOp {
    pub fn peek(input: ParseStream) -> bool {
        input.fork().parse::<UnOp>().is_ok()
    }

    #[must_use]
    pub fn precedence(&self) -> u32 {
        // Taken from https://www.postgresql.org/docs/18/sql-syntax-lexical.html#SQL-PRECEDENCE
        match self {
            Self::Not(_) => 3,
        }
    }
}

impl Parse for UnOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::not) {
            return Ok(Self::Not(input.parse()?));
        }

        Err(lookahead.error())
    }
}

impl ToTokens for UnOp {
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

impl PrettyPrint for UnOp {
    fn pretty_print(&self, printer: &mut Printer) {
        match self {
            Self::Not(inner) => inner.pretty_print(printer),
        }
    }
}
