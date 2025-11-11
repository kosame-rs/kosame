mod field;
mod from;
mod group_by;
mod having;
mod limit;
mod offset;
mod order_by;
mod returning;
mod select;
mod set;
mod values;
mod r#where;
mod with;

pub use field::*;
pub use from::*;
pub use group_by::*;
pub use having::*;
pub use limit::*;
pub use offset::*;
pub use order_by::*;
pub use returning::*;
pub use select::*;
pub use set::*;
pub use values::*;
pub use r#where::*;
pub use with::*;

pub fn peek_clause(input: syn::parse::ParseStream) -> bool {
    With::peek(input)
        || From::peek(input)
        || Where::peek(input)
        || GroupBy::peek(input)
        || Having::peek(input)
        || OrderBy::peek(input)
        || Limit::peek(input)
        || Offset::peek(input)
        || Returning::peek(input)
        || Set::peek(input)
        || Values::peek(input)
}
