use syn::Path;

use crate::{
    clause::{FromItem, WithItem},
    command::Command,
    expr::BindParam,
};

pub trait Visitor<'a> {
    fn visit_bind_param(&mut self, _bind_param: &'a BindParam) {}

    fn visit_table_ref(&mut self, _table_ref: &'a Path) {}

    fn visit_command(&mut self, _command: &'a Command) {}
    fn end_command(&mut self) {}

    fn visit_from_item(&mut self, _from_item: &'a FromItem) {}
    fn end_from_item(&mut self) {}

    fn visit_with_item(&mut self, _with_item: &'a WithItem) {}
    fn end_with_item(&mut self) {}
}
