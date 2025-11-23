# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Kosame is a macro-based Rust ORM inspired by Prisma and Drizzle. It provides compile-time type inference for database queries without requiring an active database connection or build step. The project uses procedural macros to parse SQL-like syntax and generate type-safe Rust code.

**Key principle**: Kosame is currently a prototype. All features should prioritize correctness and type safety over performance optimizations.

## Build & Development Commands

### Building
```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p kosame_dsl
cargo build -p kosame_macro
cargo build -p kosame

# Check without building (faster iteration)
cargo check
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p kosame
cargo test -p kosame_dsl

# Run specific test
cargo test -p kosame basic_select
```

### Code Quality
```bash
# Run clippy linter
cargo clippy

# Format code
cargo fmt

# Format Kosame macros in Rust files (custom formatter)
cargo kosame fmt <file>
```

### Development Environment
```bash
# Start PostgreSQL for examples/testing (Docker Compose)
docker compose up -d

# Enter Nix development shell (if using Nix)
nix develop
```

### Running Examples
```bash
# Tokio-Postgres example
cargo run -p tokio_postgres

# Postgres (synchronous) example
cargo run -p postgres
```

## Architecture

### Workspace Structure

The project is organized as a Cargo workspace with the following crates:

**Core Crates** (compile-time → runtime flow):
- `kosame_macro` - Procedural macro entry points (`pg_table!`, `pg_query!`, `pg_statement!`)
- `kosame_dsl` - DSL parser, AST, type inference engine, and code generation
- `kosame_repr` - Runtime representation structures (query/statement metadata)
- `kosame_sql` - SQL dialect abstraction and formatting (`FmtSql` trait)
- `kosame` - Runtime library with query execution, drivers, and public API

**Supporting Crates**:
- `kosame_cli` - CLI tool for formatting macros (`cargo kosame fmt`)
- `examples/` - Example projects demonstrating usage

### Macro Expansion Pipeline

All three macros (`pg_table!`, `pg_query!`, `pg_statement!`) follow this pattern:

```
User Code (macro invocation)
    ↓
kosame_macro (entry point)
    ↓
kosame_dsl::parse() → AST
    ↓
Type inference (Correlations + Scopes)
    ↓
kosame_dsl::to_tokens() → TokenStream
    ↓
Generated Rust code (modules, structs, trait impls)
```

### Type Inference System

The type inference system operates at compile-time without database access:

**Correlations** (`kosame_dsl/src/correlations.rs`):
- Maps scope IDs to table schemas
- Resolves column references like `posts.title` to their Rust types
- Tracks join nullability (LEFT/RIGHT/FULL JOINs make columns `Option<T>`)
- Handles subqueries and CTEs (WITH clauses)

**Scopes** (`kosame_dsl/src/scopes.rs`):
- Maps aggregate/group-by contexts to input types
- Tracks scope-local variable types

**InferredType** (`kosame_dsl/src/inferred_type.rs`):
- Represents types during inference (before final resolution)
- Handles chains: `Correlation → TableColumn → RustType`

### Schema as Code

`pg_table!` macros generate Rust modules that encode schema information:

```rust
pg_table! {
    create table posts (
        id int primary key,
        title text,
    );
}
```

Generates:
```rust
pub mod posts {
    pub mod columns {
        pub mod id {
            pub const COLUMN_NAME: &str = "id";
            pub type Type = i32;              // non-null (primary key)
            pub type TypeNotNull = i32;
            pub type TypeNullable = Option<i32>;
        }
        pub mod title {
            pub const COLUMN_NAME: &str = "title";
            pub type Type = Option<String>;   // nullable by default
            pub type TypeNotNull = String;
            pub type TypeNullable = Option<String>;
        }
    }
    // ... TABLE constant, Select struct, etc.
}
```

This enables compile-time type resolution: `posts::columns::title::Type` → `Option<String>`.

## Key Files by Task

### Adding/Modifying SQL Syntax Support

**Expression parsing**:
- `kosame_dsl/src/expr/mod.rs` - Operator precedence parser
- `kosame_dsl/src/expr/*.rs` - Expression AST types (Binary, Call, Cast, etc.)

**Statement parsing**:
- `kosame_dsl/src/statement/mod.rs` - Statement parser entry point
- `kosame_dsl/src/statement/command.rs` - SELECT/INSERT/UPDATE/DELETE variants
- `kosame_dsl/src/statement/from.rs` - FROM clause parsing (joins, subqueries)

**Query parsing**:
- `kosame_dsl/src/query/mod.rs` - Relational query parser
- `kosame_dsl/src/query/node.rs` - Query field tree structure

**Schema parsing**:
- `kosame_dsl/src/schema/table.rs` - Table definition parser
- `kosame_dsl/src/schema/column.rs` - Column metadata and type mapping

### Type Inference & Code Generation

**Type inference**:
- `kosame_dsl/src/correlations.rs` - Scope-to-table mapping
- `kosame_dsl/src/scopes.rs` - Aggregate/grouping contexts
- `kosame_dsl/src/inferred_type.rs` - Type resolution logic
- `kosame_dsl/src/visitor.rs` - AST traversal trait

**Code generation** (via `ToTokens` implementations in each module):
- `kosame_dsl/src/query/node.rs::to_row_tokens()` - Generate Row structs
- `kosame_dsl/src/statement/command.rs::to_tokens()` - Generate Statement impls
- `kosame_dsl/src/schema/table.rs::to_tokens()` - Generate table modules

### Runtime Execution

**Query runners**:
- `kosame/src/record_array_runner.rs` - Relational query executor (marshals nested results)
- `kosame/src/query.rs` - Query trait definition
- `kosame/src/statement.rs` - Statement trait definition

**Database drivers**:
- `kosame/src/postgres.rs` - Synchronous Postgres driver
- `kosame/src/tokio_postgres.rs` - Async Postgres driver

**SQL formatting**:
- `kosame_sql/src/fmt.rs` - FmtSql trait and Formatter
- `kosame_sql/src/postgres.rs` - PostgreSQL dialect

### Pretty Printing (Error Messages)

- `kosame_dsl/src/pretty/mod.rs` - Pretty printer infrastructure
- `kosame_dsl/src/pretty/printer.rs` - Formatting logic
- `kosame_dsl/src/pretty/lexer.rs` - Trivia extraction (comments, whitespace)

## Development Patterns

### Adding New Expression Types

1. Add variant to `kosame_dsl/src/expr/expr.rs::Expr` enum
2. Implement parsing in `kosame_dsl/src/expr/mod.rs`
3. Implement `ToTokens` for code generation
4. Implement `FmtSql` in `kosame_repr/src/expr/*.rs` for runtime
5. Add tests in `kosame/tests/`

### Adding Type Inference for New Constructs

1. Update `Correlations::infer_type()` to handle new cases
2. Add `InferredType` variant if needed
3. Update `resolve_type()` to flatten inference chains
4. Handle nullability propagation (e.g., OUTER JOINs)

### Adding Support for New Database Dialects

1. Create new dialect in `kosame_sql/src/<dialect>.rs`
2. Implement `Dialect` trait
3. Update `FmtSql` impls in `kosame_repr/` if dialect needs special handling
4. Add driver in `kosame/src/<dialect>.rs` implementing `ToSql`/`FromSql`
5. Add macro shortcuts (e.g., `mysql_table!`, `mysql_query!`)

## Testing Strategy

**Unit tests**: Each crate has inline tests (`#[test]` in source files)

**Integration tests**: `kosame/tests/*.rs` - test macro expansion and SQL generation

**Example projects**: `examples/tokio_postgres/` and `examples/postgres/` - full end-to-end usage

When adding features:
- Add parsing tests in `kosame_dsl`
- Add SQL generation tests in `kosame/tests/`
- Test type inference correctness (check generated Row struct fields)
- Verify error messages are helpful (use pretty printer)

## Common Pitfalls

**Thread-local state**: Correlation/Scope IDs use thread-local counters. Never share state across macro invocations. Each `to_tokens()` call must reset counters.

**Nullability tracking**: LEFT/RIGHT/FULL JOINs change column nullability. Always check join types in `Correlations::infer_type()`.

**ToTokens hygiene**: Use `quote!` for code generation. Always use fully-qualified paths (`::`-prefixed) for types from `kosame` crates to avoid name resolution issues.

**SQL keyword case**: All SQL keywords in macros must be lowercase for consistency. Parser will error on uppercase keywords.

**Bind parameter lifetimes**: Generated `Params<'a>` structs must borrow from user code. Ensure lifetime bounds are correct in `to_tokens()`.

## Macro Formatting

The project includes a custom formatter for Kosame macros:

```bash
cargo kosame fmt path/to/file.rs
```

This reformats `pg_table!`, `pg_query!`, and `pg_statement!` macro contents with proper indentation and structure. The formatter is implemented in `kosame_cli/src/fmt.rs` and uses the pretty printer from `kosame_dsl/src/pretty/`.

## Project Roadmap Context

Kosame is an early prototype. Planned features include:
- Additional DBMS support (MySQL, SQLite, etc.)
- CLI for migration generation
- CLI for schema introspection (generate Kosame schema from existing DB)
- Alternative query runners (different join strategies)
- Type inference for bind parameters

When implementing features, prioritize correctness and maintain the "zero build step" philosophy (no database connection required during development).
