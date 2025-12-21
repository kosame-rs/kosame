//! Wrapper around the [`::proc_macro_error`] crate that makes it possible to disable the `emit`
//! or `abort` macros. This is useful for formatting, in which an AST can still be formatted even if
//! it contains warnings or errors.

use std::cell::Cell;

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(true) };
}

#[must_use]
pub fn is_enabled() -> bool {
    ENABLED.get()
}

pub fn set_enabled(enabled: bool) {
    ENABLED.set(enabled);
}

macro_rules! emit_error {
    ($($tt:tt)*) => {{
        if $crate::proc_macro_error::is_enabled() {
            ::proc_macro_error::emit_error!($($tt)*)
        }
    }}
}
pub(crate) use emit_error;

macro_rules! emit_call_site_error {
    ($($tt:tt)*) => {{
        if $crate::proc_macro_error::is_enabled() {
            ::proc_macro_error::emit_call_site_error!($($tt)*)
        }
    }}
}
pub(crate) use emit_call_site_error;

#[allow(unused)]
macro_rules! emit_warning {
    ($($tt:tt)*) => {{
        if $crate::proc_macro_error::is_enabled() {
            ::proc_macro_error::emit_warning!($($tt)*)
        }
    }}
}
#[allow(unused)]
pub(crate) use emit_warning;

macro_rules! abort {
    ($($tt:tt)*) => {{
        if $crate::proc_macro_error::is_enabled() {
            ::proc_macro_error::abort!($($tt)*)
        }
    }}
}
pub(crate) use abort;

pub mod dummy {
    use proc_macro2::TokenStream;

    #[allow(clippy::must_use_candidate)]
    pub fn set_dummy(dummy: TokenStream) -> Option<TokenStream> {
        if super::is_enabled() {
            return ::proc_macro_error::set_dummy(dummy);
        }
        None
    }
}
