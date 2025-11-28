use std::fmt::Write;

use kosame_repr::schema::Relation;
use kosame_sql::FmtSql;

use crate::driver::Connection;

use super::{Field, Node, Params, Query, Runner};

pub struct RecordArrayRunner {}

impl RecordArrayRunner {
    pub fn query_to_sql<D: kosame_sql::Dialect>(
        &self,
        query: &(impl Query + ?Sized),
    ) -> Result<String, kosame_sql::Error> {
        let mut sql = String::new();
        let mut formatter = kosame_sql::Formatter::<D>::new(&mut sql);
        fmt_node_sql(&mut formatter, query.repr(), None)?;
        Ok(sql)
    }
}

impl Runner for RecordArrayRunner {
    async fn run<'a, C, Q>(&self, connection: &mut C, query: &Q) -> crate::Result<Vec<Q::Row>>
    where
        C: Connection,
        Q: Query + ?Sized,
        Q::Params: Params<C::Params<'a>>,
        for<'b> Q::Row: From<&'b C::Row>,
    {
        let sql = self.query_to_sql::<C::Dialect>(query)?;
        let rows = connection
            .query(&sql, &query.params().to_driver())
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(rows.iter().map(Q::Row::from).collect())
    }
}

fn fmt_node_sql<D: kosame_sql::Dialect>(
    formatter: &mut kosame_sql::Formatter<D>,
    node: &Node,
    relation: Option<&Relation>,
) -> std::fmt::Result {
    formatter.write_str("select ")?;

    if relation.is_some() {
        formatter.write_str("row(")?;
    }

    if node.star() {
        for (index, column) in node.table().columns().iter().enumerate() {
            formatter.write_ident(column.name())?;
            if index != node.table().columns().len() - 1 {
                formatter.write_str(", ")?;
            }
        }
        if !node.fields().is_empty() {
            formatter.write_str(", ")?;
        }
    }

    for (index, field) in node.fields().iter().enumerate() {
        match field {
            Field::Column { column, .. } => {
                formatter.write_ident(column.name())?;
            }
            Field::Relation { node, relation, .. } => {
                formatter.write_str("array(")?;
                fmt_node_sql::<D>(formatter, node, Some(relation))?;
                formatter.write_str(")")?;
            }
            Field::Expr { expr, .. } => {
                expr.fmt_sql(formatter)?;
            }
        }
        if index != node.fields().len() - 1 {
            formatter.write_str(", ")?;
        }
    }

    if relation.is_some() {
        formatter.write_str(")")?;
    }

    formatter.write_str(" from ")?;
    formatter.write_ident(node.table().name())?;

    if relation.is_some() || node.r#where().is_some() {
        formatter.write_str(" where ")?;
    }

    if relation.is_some() && node.r#where().is_some() {
        formatter.write_str("(")?;
    }

    if let Some(relation) = relation {
        for (index, (source_column, target_column)) in relation.column_pairs().enumerate() {
            formatter.write_ident(relation.source_table())?;
            formatter.write_str(".")?;
            formatter.write_ident(source_column.name())?;
            formatter.write_str(" = ")?;
            formatter.write_ident(relation.target_table())?;
            formatter.write_str(".")?;
            formatter.write_ident(target_column.name())?;
            if index != relation.source_columns().len() - 1 {
                formatter.write_str(" and ")?;
            }
        }
    }

    if relation.is_some() && node.r#where().is_some() {
        formatter.write_str(") and (")?;
    }

    if let Some(r#where) = &node.r#where() {
        r#where.expr().fmt_sql(formatter)?;
    }

    if relation.is_some() && node.r#where().is_some() {
        formatter.write_str(")")?;
    }

    if let Some(order_by) = &node.order_by() {
        order_by.fmt_sql(formatter)?;
    }

    if let Some(limit) = &node.limit() {
        limit.fmt_sql(formatter)?;
    }

    if let Some(offset) = &node.offset() {
        offset.fmt_sql(formatter)?;
    }

    Ok(())
}
