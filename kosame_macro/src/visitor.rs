use syn::Path;

use crate::{command::Command, expr::BindParam, part::TablePath};

pub trait Visitor<'a> {
    fn visit_bind_param(&mut self, _bind_param: &'a BindParam) {}
    fn visit_table_ref(&mut self, _table_ref: &'a Path) {}
    fn visit_table_path(&mut self, table_path: &'a TablePath) {}
    fn visit_command(&mut self, _command: &'a Command) {}
}

impl<'a, T> Visitor<'a> for T
where
    T: FnMut(&'a Path),
{
    fn visit_table_ref(&mut self, table_ref: &'a Path) {
        self(table_ref)
    }
}
