use proc_macro2::Span;
use syn::{Ident, Path, PathSegment, Token, punctuated::Punctuated};

pub trait PathExt {
    fn is_absolute(&self) -> bool;
    #[allow(unused)]
    fn is_relative(&self) -> bool;
    fn is_primitive_type(&self) -> bool;
    fn to_call_site(&self, nesting_levels: usize) -> Path;
}

impl PathExt for Path {
    fn is_absolute(&self) -> bool {
        self.leading_colon.is_some()
            || self
                .segments
                .iter()
                .next()
                .is_some_and(|segment| segment.ident == "crate")
    }

    fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    fn is_primitive_type(&self) -> bool {
        if self.leading_colon.is_some() || self.segments.len() != 1 {
            false
        } else {
            let segment = &self.segments[0];
            matches!(
                segment.ident.to_string().as_ref(),
                "u8" | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "isize"
                    | "f32"
                    | "f64"
                    | "char"
                    | "bool",
            )
        }
    }

    fn to_call_site(&self, nesting_levels: usize) -> Path {
        if self.is_absolute() || self.is_primitive_type() {
            self.clone()
        } else {
            let mut result = Path {
                leading_colon: None,
                segments: Punctuated::<PathSegment, Token![::]>::new(),
            };
            result.segments.extend(std::iter::repeat_n(
                PathSegment::from(Ident::new("super", Span::call_site())),
                nesting_levels,
            ));
            result.segments.extend(self.segments.iter().cloned());
            result
        }
    }
}
