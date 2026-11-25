//! Lexer: `&str` → tokens.
//!
//! STUB (mentor): API agreed in M1 step 2. Bodies are `todo!()`; fill them in.
//!
//! Agreed behaviour:
//! - Whitespace (`char::is_whitespace`) separates tokens and produces none.
//! - Keywords (`SELECT`, `TRUE`, `FALSE`) are case-insensitive.
//! - Identifiers: `[A-Za-z_][A-Za-z0-9_]*`, folded to lowercase (as Postgres does for unquoted names).
//! - Numbers: `[0-9]+`, kept as the raw digit string. The evaluator decides the type.
//!   A number immediately followed by an identifier char (`123abc`) is a syntax error (PG 15+).
//! - Strings: `'...'` with `''` as an escaped quote; may span lines; unterminated is an error.
//! - Positions are 1-based **char** offsets (not bytes) of the token's first char.
//! - Out of scope in M1 (syntax error): `"quoted identifiers"`, `--` comments, decimals like `1.5`.

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Select,
    True,
    False,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    /// Unquoted identifier, lowercased.
    Ident(String),
    /// Raw digits, e.g. `"007"`.
    Number(String),
    /// String literal contents with `''` already unescaped.
    String(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
    Semicolon,
}

/// A token plus the 1-based char position where it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned {
    pub token: Token,
    pub position: usize,
}

/// Splits `input` into tokens. Errors are `Error::Syntax` with the offending char's position
/// (for an unterminated string or `123abc`: the position where that token starts).
pub fn tokenize(input: &str) -> Result<Vec<Spanned>, Error> {
    let _ = input;
    todo!()
}
