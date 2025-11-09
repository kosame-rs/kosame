use crate::{command::Command, expr::BindParam, part::TablePath};

pub trait Visitor<'a> {
    fn visit_bind_param(&mut self, _bind_param: &'a BindParam) {}
    fn visit_table_path(&mut self, _table_path: &'a TablePath) {}
    fn visit_command(&mut self, _command: &'a Command) {}
}
