//! Parser: tokens → `Statement`.
//!
//! STUB (mentor): API agreed in M1 step 3. The body is `todo!()`; fill it in.
//!
//! Agreed behaviour:
//! - Grammar: `SELECT <expr> [, <expr>]... [;]`, then end of input.
//! - Precedence, low to high: `+ -` < `* /` < unary `-`. Binary operators are left-associative.
//! - Errors are `Error::Syntax` (SQLSTATE 42601), with messages matching Postgres:
//!   - an unexpected token: `syntax error at or near "<token>"`, at that token's position;
//!   - running out of tokens: `syntax error at end of input`, at position `<chars in input> + 1`.
//! - Lexer errors pass through unchanged.
//! - Not in M1 (syntax error): bare `SELECT` (PG allows an empty target list), column names,
//!   `*`, aliases, more than one statement per query.

use crate::error::Error;
use crate::sql::ast::Statement;

/// Parses one SQL statement.
pub fn parse(sql: &str) -> Result<Statement, Error> {
    let _ = sql;
    todo!()
}
