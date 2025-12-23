use std::cell::Cell;

use crate::driver::Driver;

thread_local! {
    static CONTEXT: Cell<Option<MacroContext>> = const { Cell::new(None) };
}

#[derive(Clone, Copy)]
pub struct MacroContext {
    driver: Driver,
}

impl MacroContext {
    pub fn scope(&self, f: impl FnOnce()) {
        let previous = CONTEXT.with(|cell| cell.replace(Some(*self)));
        f();
        CONTEXT.with(|cell| cell.replace(previous));
    }

    #[must_use]
    pub fn of_scope() -> Self {
        CONTEXT.get().expect(
            "`MacroContext::of_scope` was called outside of a call to `MacroContext::scope`",
        )
    }
}
