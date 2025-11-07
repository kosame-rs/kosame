use syn::{Ident, Path, parse_quote};

use crate::{
    correlations::{CorrelationId, Correlations},
    part::TablePath,
    scopes::{ScopeId, Scopes},
};

#[derive(Debug)]
pub enum InferredType<'a> {
    RustType(&'a Path),
    Scope {
        scope_id: ScopeId,
        table: Option<&'a Ident>,
        column: &'a Ident,
    },
    Correlation {
        correlation_id: CorrelationId,
        column: &'a Ident,
        nullable: bool,
    },
    TableColumn {
        table_path: &'a TablePath,
        column: &'a Ident,
    },
}

pub fn resolve_type(
    correlations: &Correlations<'_>,
    scopes: &Scopes<'_>,
    correlation_id: CorrelationId,
    column: &Ident,
) -> Option<Path> {
    let mut inferred_type = correlations.infer_type(correlation_id, column)?;
    let mut combined_nullable = false;
    let mut counter = 0;
    loop {
        counter += 1;
        println!("{:#?}", inferred_type);
        if counter > 100 {
            panic!("rip");
        }
        match inferred_type {
            InferredType::RustType(rust_type) => return Some(rust_type.clone()),
            InferredType::Scope {
                scope_id,
                table,
                column,
            } => {
                inferred_type = scopes.infer_type(scope_id, table, column)?;
            }
            InferredType::Correlation {
                correlation_id,
                column,
                nullable,
            } => {
                combined_nullable = combined_nullable || nullable;
                inferred_type = correlations.infer_type(correlation_id, column)?;
            }
            InferredType::TableColumn { table_path, column } => {
                let table_path = table_path.as_path();
                match combined_nullable {
                    true => return Some(parse_quote!(#table_path::columns::#column::TypeNullable)),
                    false => return Some(parse_quote!(#table_path::columns::#column::Type)),
                }
            }
        }
    }
}
