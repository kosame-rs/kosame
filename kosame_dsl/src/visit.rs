use crate::{
    clause::{
        Field, Fields, From, FromChain, FromCombinator, FromItem, GroupBy, Having, Limit, Offset,
        OrderBy, Returning, Set, SetItem, Values, ValuesItem, ValuesRow, Where, With, WithItem,
    },
    command::{
        Command, CommandType, Delete, Insert, Select as SelectCommand, SelectChain,
        SelectCombinator, SelectItem, Update, Using,
    },
    expr::{Binary, BindParam, Call, Cast, ColumnRef, Expr, Lit, Paren, Raw, Unary},
    part::{TablePath, TargetTable},
    query::Node,
    statement::Statement,
};

pub use crate::{
    clause::{
        visit_field, visit_fields, visit_from, visit_from_chain, visit_from_combinator,
        visit_from_item, visit_group_by, visit_having, visit_limit, visit_offset, visit_order_by,
        visit_returning, visit_select, visit_select_core, visit_set, visit_set_item, visit_values,
        visit_values_item, visit_values_row, visit_where, visit_with, visit_with_item,
    },
    command::{
        visit_command, visit_command_type, visit_delete, visit_insert, visit_select_chain,
        visit_select_combinator, visit_select_command, visit_select_item, visit_update,
        visit_using,
    },
    expr::{
        visit_binary, visit_bind_param, visit_call, visit_cast, visit_column_ref, visit_expr,
        visit_lit, visit_paren, visit_raw, visit_unary,
    },
    part::{visit_table_path, visit_target_table},
    query::visit_node,
    statement::visit_statement,
};

pub trait Visit<'a> {
    // Expression nodes
    fn visit_expr(&mut self, expr: &'a Expr) {
        visit_expr(self, expr);
    }

    fn visit_binary(&mut self, binary: &'a Binary) {
        visit_binary(self, binary);
    }

    fn visit_bind_param(&mut self, bind_param: &'a BindParam) {
        visit_bind_param(self, bind_param);
    }

    fn visit_call(&mut self, call: &'a Call) {
        visit_call(self, call);
    }

    fn visit_cast(&mut self, cast: &'a Cast) {
        visit_cast(self, cast);
    }

    fn visit_column_ref(&mut self, column_ref: &'a ColumnRef) {
        visit_column_ref(self, column_ref);
    }

    fn visit_lit(&mut self, lit: &'a Lit) {
        visit_lit(self, lit);
    }

    fn visit_paren(&mut self, paren: &'a Paren) {
        visit_paren(self, paren);
    }

    fn visit_raw(&mut self, raw: &'a Raw) {
        visit_raw(self, raw);
    }

    fn visit_unary(&mut self, unary: &'a Unary) {
        visit_unary(self, unary);
    }

    // Clause nodes
    fn visit_field(&mut self, field: &'a Field) {
        visit_field(self, field);
    }

    fn visit_fields(&mut self, fields: &'a Fields) {
        visit_fields(self, fields);
    }

    fn visit_from(&mut self, from: &'a From) {
        visit_from(self, from);
    }

    fn visit_from_chain(&mut self, from_chain: &'a FromChain) {
        visit_from_chain(self, from_chain);
    }

    fn visit_from_item(&mut self, from_item: &'a FromItem) {
        visit_from_item(self, from_item);
    }

    fn visit_from_combinator(&mut self, from_combinator: &'a FromCombinator) {
        visit_from_combinator(self, from_combinator);
    }

    fn visit_group_by(&mut self, group_by: &'a GroupBy) {
        visit_group_by(self, group_by);
    }

    fn visit_having(&mut self, having: &'a Having) {
        visit_having(self, having);
    }

    fn visit_limit(&mut self, limit: &'a Limit) {
        visit_limit(self, limit);
    }

    fn visit_offset(&mut self, offset: &'a Offset) {
        visit_offset(self, offset);
    }

    fn visit_order_by(&mut self, order_by: &'a OrderBy) {
        visit_order_by(self, order_by);
    }

    fn visit_returning(&mut self, returning: &'a Returning) {
        visit_returning(self, returning);
    }

    fn visit_select(&mut self, select: &'a crate::clause::Select) {
        visit_select(self, select);
    }

    fn visit_select_core(&mut self, select_core: &'a crate::clause::SelectCore) {
        visit_select_core(self, select_core);
    }

    fn visit_set(&mut self, set: &'a Set) {
        visit_set(self, set);
    }

    fn visit_set_item(&mut self, set_item: &'a SetItem) {
        visit_set_item(self, set_item);
    }

    fn visit_values(&mut self, values: &'a Values) {
        visit_values(self, values);
    }

    fn visit_values_row(&mut self, values_row: &'a ValuesRow) {
        visit_values_row(self, values_row);
    }

    fn visit_values_item(&mut self, values_item: &'a ValuesItem) {
        visit_values_item(self, values_item);
    }

    fn visit_where(&mut self, r#where: &'a Where) {
        visit_where(self, r#where);
    }

    fn visit_with(&mut self, with: &'a With) {
        visit_with(self, with);
    }

    fn visit_with_item(&mut self, with_item: &'a WithItem) {
        visit_with_item(self, with_item);
    }

    // Command nodes
    fn visit_command(&mut self, command: &'a Command) {
        visit_command(self, command);
    }

    fn visit_command_type(&mut self, command_type: &'a CommandType) {
        visit_command_type(self, command_type);
    }

    fn visit_delete(&mut self, delete: &'a Delete) {
        visit_delete(self, delete);
    }

    fn visit_insert(&mut self, insert: &'a Insert) {
        visit_insert(self, insert);
    }

    fn visit_select_command(&mut self, select: &'a SelectCommand) {
        visit_select_command(self, select);
    }

    fn visit_select_chain(&mut self, select_chain: &'a SelectChain) {
        visit_select_chain(self, select_chain);
    }

    fn visit_select_item(&mut self, select_item: &'a SelectItem) {
        visit_select_item(self, select_item);
    }

    fn visit_select_combinator(&mut self, select_combinator: &'a SelectCombinator) {
        visit_select_combinator(self, select_combinator);
    }

    fn visit_update(&mut self, update: &'a Update) {
        visit_update(self, update);
    }

    fn visit_using(&mut self, using: &'a Using) {
        visit_using(self, using);
    }

    // Part nodes
    fn visit_table_path(&mut self, table_path: &'a TablePath) {
        visit_table_path(self, table_path);
    }

    fn visit_target_table(&mut self, target_table: &'a TargetTable) {
        visit_target_table(self, target_table);
    }

    // Query nodes
    fn visit_node(&mut self, node: &'a Node) {
        visit_node(self, node);
    }

    // Statement node
    fn visit_statement(&mut self, statement: &'a Statement) {
        visit_statement(self, statement);
    }
}
