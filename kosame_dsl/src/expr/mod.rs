mod binary;
mod bind_param;
mod call;
mod cast;
mod column_ref;
mod lit;
mod paren;
mod raw;
mod unary;

pub use binary::*;
pub use bind_param::*;
pub use call::*;
pub use cast::*;
pub use column_ref::*;
pub use lit::*;
pub use paren::*;
pub use raw::*;
pub use unary::*;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use crate::{
    inferred_type::InferredType,
    pretty::{PrettyPrint, Printer},
    scopes::ScopeId,
    visitor::Visitor,
};

pub enum Expr {
    Binary(Binary),
    BindParam(BindParam),
    Call(Call),
    Cast(Cast),
    ColumnRef(ColumnRef),
    Lit(Lit),
    Paren(Paren),
    Raw(Raw),
    Unary(Unary),
}

macro_rules! variants {
    ($macro:ident!()) => {
        $macro!(
            Binary
            BindParam
            Call
            Cast
            ColumnRef
            Lit
            Paren
            Raw
            Unary
        )
    };
}

impl Expr {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(inner) => inner.accept(visitor)),*
                }
            };
        }

        variants!(branches!());
    }

    #[must_use]
    pub fn infer_name(&self) -> Option<&Ident> {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(inner) => inner.infer_name()),*
                }
            };
        }

        variants!(branches!())
    }

    #[must_use]
    pub fn infer_type(&self, scope_id: ScopeId) -> Option<InferredType<'_>> {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(inner) => inner.infer_type(scope_id)),*
                }
            };
        }

        variants!(branches!())
    }

    fn parse_prefix(input: ParseStream) -> syn::Result<Expr> {
        if input.peek(syn::token::Paren) {
            Ok(Expr::Paren(input.parse()?))
        } else if BindParam::peek(input) {
            Ok(Expr::BindParam(input.parse()?))
        } else if Raw::peek(input) {
            Ok(Expr::Raw(input.parse()?))
        } else if UnOp::peek(input) {
            let op = input.parse::<UnOp>()?;
            let precedence = op.precedence();
            Ok(Expr::Unary(Unary::new(
                op,
                Self::parse_expr(input, precedence)?,
            )))
        } else if Cast::peek(input) {
            Ok(Expr::Cast(input.parse()?))
        } else if input.fork().parse::<Lit>().is_ok() {
            Ok(Expr::Lit(input.parse()?))
        } else if Call::peek(input) {
            Ok(Expr::Call(input.parse()?))
        } else if input.fork().parse::<ColumnRef>().is_ok() {
            Ok(Expr::ColumnRef(input.parse()?))
        } else {
            Err(syn::Error::new(input.span(), "expected expression"))
        }
    }

    fn parse_expr(input: ParseStream, min_precedence: u32) -> syn::Result<Expr> {
        let mut lhs = Self::parse_prefix(input)?;

        while let Some(bin_op) = BinOp::peek(input) {
            let precedence = bin_op.precedence();
            if precedence < min_precedence {
                break;
            }

            let next_precedence = if bin_op.associativity() == Associativity::Left {
                precedence + 1
            } else {
                precedence
            };

            let bin_op = input.parse()?;
            let rhs = Self::parse_expr(input, next_precedence)?;

            lhs = Expr::Binary(Binary::new(lhs, bin_op, rhs));
        }

        Ok(lhs)
    }

    #[must_use]
    pub fn span(&self) -> Span {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(inner) => inner.span()),*
                }
            };
        }

        variants!(branches!())
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_expr(input, 0)
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(inner) => quote! { ::kosame::repr::expr::Expr::$variant(#inner) }.to_tokens(tokens)),*
                }
            };
        }

        variants!(branches!());
    }
}

impl PrettyPrint for Expr {
    fn pretty_print(&self, printer: &mut Printer) {
        macro_rules! branches {
            ($($variant:ident)*) => {
                match self {
                    $(Self::$variant(inner) => inner.pretty_print(printer)),*
                }
            };
        }

        variants!(branches!());
    }
}
