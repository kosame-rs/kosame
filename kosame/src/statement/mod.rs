pub use kosame_repr::command::*;

pub trait Statement {
    type Params: std::fmt::Debug;
    type Row;

    const REPR: Command<'static>;

    fn params(&self) -> &Self::Params;
}
