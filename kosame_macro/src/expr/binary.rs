use crate::{inferred_type::InferredType, keyword, scopes::ScopeId};

use super::{Expr, Visitor};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

pub struct Binary {
    pub lhs: Box<Expr>,
    pub op: BinOp,
    pub rhs: Box<Expr>,
}

impl Binary {
    pub fn new(left: Expr, op: BinOp, right: Expr) -> Self {
        Self {
            lhs: Box::new(left),
            op,
            rhs: Box::new(right),
        }
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        self.lhs.accept(visitor);
        self.rhs.accept(visitor);
    }

    pub fn infer_name(&self) -> Option<&Ident> {
        None
    }

    pub fn infer_type(&self, _scope_id: ScopeId) -> Option<InferredType> {
        None
    }

    pub fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span()).expect("same file")
    }
}

impl ToTokens for Binary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let lhs = &self.lhs;
        let op = &self.op;
        let rhs = &self.rhs;
        quote! {
            ::kosame::repr::expr::Binary::new(&#lhs, #op, &#rhs)
        }
        .to_tokens(tokens);
    }
}

#[derive(PartialEq, Eq)]
pub enum Associativity {
    Left,
}

#[allow(unused)]
pub enum BinOp {
    // multiplication, division, modulo
    Multiply(Token![*]),
    Divide(Token![/]),
    Modulo(Token![%]),
    // addition, subtraction
    Add(Token![+]),
    Subtract(Token![-]),
    // comparison operators
    Eq(Token![=]),
    Uneq(Token![<], Token![>]),
    LessThan(Token![<]),
    GreaterThan(Token![>]),
    LessThanOrEq(Token![<], Token![=]),
    GreaterThanOrEq(Token![>], Token![=]),
    // is
    Is(keyword::is),
    IsNot(keyword::is, keyword::not),
    IsDistinctFrom(keyword::is, keyword::distinct, keyword::from),
    // logical
    And(keyword::and),
    Or(keyword::or),
}

impl BinOp {
    pub fn peek(input: ParseStream) -> Option<BinOp> {
        input.fork().parse::<BinOp>().ok()
    }

    pub fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    pub fn precedence(&self) -> u32 {
        // Taken from https://www.postgresql.org/docs/18/sql-syntax-lexical.html#SQL-PRECEDENCE
        match self {
            Self::Multiply(_) => 9,
            Self::Divide(_) => 9,
            Self::Modulo(_) => 9,
            Self::Add(_) => 8,
            Self::Subtract(_) => 8,
            Self::Eq(_) => 5,
            Self::Uneq(..) => 5,
            Self::LessThan(_) => 5,
            Self::GreaterThan(_) => 5,
            Self::LessThanOrEq(..) => 5,
            Self::GreaterThanOrEq(..) => 5,
            Self::Is(..) => 4,
            Self::IsNot(..) => 4,
            Self::IsDistinctFrom(..) => 4,
            Self::And(_) => 2,
            Self::Or(_) => 1,
        }
    }
}

impl Parse for BinOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![+]) {
            return Ok(Self::Add(input.parse()?));
        } else if lookahead.peek(Token![-]) {
            return Ok(Self::Subtract(input.parse()?));
        } else if lookahead.peek(Token![*]) {
            return Ok(Self::Multiply(input.parse()?));
        } else if lookahead.peek(Token![/]) {
            return Ok(Self::Divide(input.parse()?));
        } else if lookahead.peek(Token![%]) {
            return Ok(Self::Modulo(input.parse()?));
        } else if lookahead.peek(keyword::and) {
            return Ok(Self::And(input.parse()?));
        } else if lookahead.peek(keyword::or) {
            return Ok(Self::Or(input.parse()?));
        }

        if lookahead.peek(keyword::is) {
            if input.peek2(keyword::not) {
                return Ok(Self::IsNot(input.parse()?, input.parse()?));
            }
            if input.peek2(keyword::distinct) {
                return Ok(Self::IsDistinctFrom(
                    input.parse()?,
                    input.parse()?,
                    input.parse()?,
                ));
            }
            return Ok(Self::Is(input.parse()?));
        }

        if lookahead.peek(Token![=]) {
            return Ok(Self::Eq(input.parse()?));
        } else if lookahead.peek(Token![<]) {
            if input.peek2(Token![>]) {
                return Ok(Self::Uneq(input.parse()?, input.parse()?));
            } else if input.peek2(Token![=]) {
                return Ok(Self::LessThanOrEq(input.parse()?, input.parse()?));
            } else {
                return Ok(Self::LessThan(input.parse()?));
            }
        } else if lookahead.peek(Token![>]) {
            if input.peek2(Token![=]) {
                return Ok(Self::GreaterThanOrEq(input.parse()?, input.parse()?));
            } else {
                return Ok(Self::GreaterThan(input.parse()?));
            }
        }

        Err(lookahead.error())
    }
}

impl ToTokens for BinOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(..) => quote! { ::kosame::repr::expr::BinOp::$variant }.to_tokens(tokens)),*
                }
            };
        }

        branches!(
            Multiply
            Divide
            Modulo
            Add
            Subtract
            Eq
            Uneq
            LessThan
            GreaterThan
            LessThanOrEq
            GreaterThanOrEq
            Is
            IsNot
            IsDistinctFrom
            And
            Or
        );
    }
}
