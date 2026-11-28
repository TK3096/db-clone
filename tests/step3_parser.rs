//! M1 step 3: parser (mentor-owned tests).

use db_clone::error::Error;
use db_clone::sql::ast::{BinaryOp, Expr, Literal, Statement, UnaryOp};
use db_clone::sql::parser::parse;

// --- helpers --------------------------------------------------------------------------------

fn select(sql: &str) -> Vec<Expr> {
    match parse(sql) {
        Ok(Statement::Select(exprs)) => exprs,
        Err(e) => panic!("parse({sql:?}) failed: {e:?}"),
    }
}

/// Parses `SELECT <expr>` and returns the single expression.
fn expr(e: &str) -> Expr {
    let sql = format!("SELECT {e}");
    let mut exprs = select(&sql);
    assert_eq!(exprs.len(), 1, "{sql:?} should have exactly one expression");
    exprs.remove(0)
}

fn num(s: &str) -> Expr {
    Expr::Literal(Literal::Number(s.to_string()))
}

fn text(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.to_string()))
}

fn boolean(b: bool) -> Expr {
    Expr::Literal(Literal::Bool(b))
}

fn neg(e: Expr) -> Expr {
    Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(e),
    }
}

fn bin(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn add(l: Expr, r: Expr) -> Expr {
    bin(BinaryOp::Add, l, r)
}
fn sub(l: Expr, r: Expr) -> Expr {
    bin(BinaryOp::Sub, l, r)
}
fn mul(l: Expr, r: Expr) -> Expr {
    bin(BinaryOp::Mul, l, r)
}
fn div(l: Expr, r: Expr) -> Expr {
    bin(BinaryOp::Div, l, r)
}

const END_OF_INPUT: &str = "syntax error at end of input";

/// Asserts a syntax error at `position`. With `Some(msg)` the message must equal it;
/// with `None` it must be a Postgres-style "syntax error at or near ...".
fn assert_syntax_error(sql: &str, position: usize, message: Option<&str>) {
    match parse(sql) {
        Err(Error::Syntax {
            message: m,
            position: p,
        }) => {
            assert_eq!(
                p, position,
                "wrong error position for {sql:?} (message: {m:?})"
            );
            match message {
                Some(expected) => assert_eq!(m, expected, "wrong message for {sql:?}"),
                None => assert!(
                    m.starts_with("syntax error at or near "),
                    "{sql:?}: expected `syntax error at or near ...`, got {m:?}"
                ),
            }
        }
        other => panic!("{sql:?}: expected Error::Syntax at {position}, got {other:?}"),
    }
}

// --- literals and lists ---------------------------------------------------------------------

#[test]
fn select_one_number() {
    assert_eq!(
        parse("SELECT 1").unwrap(),
        Statement::Select(vec![num("1")])
    );
}

#[test]
fn trailing_semicolon_is_optional() {
    assert_eq!(select("SELECT 1;"), vec![num("1")]);
    assert_eq!(select("select 1 ;"), vec![num("1")]);
}

#[test]
fn all_literal_kinds() {
    assert_eq!(
        select("SELECT 1, 'hi', TRUE, false, ''"),
        vec![
            num("1"),
            text("hi"),
            boolean(true),
            boolean(false),
            text("")
        ]
    );
}

#[test]
fn numbers_stay_raw() {
    assert_eq!(expr("99999999999999999999"), num("99999999999999999999"));
}

#[test]
fn comments_and_newlines_are_fine() {
    assert_eq!(select("SELECT 1, -- first\n  2"), vec![num("1"), num("2")]);
}

// --- precedence and associativity -----------------------------------------------------------

#[test]
fn mul_binds_tighter_than_add() {
    assert_eq!(expr("1 + 2 * 3"), add(num("1"), mul(num("2"), num("3"))));
    assert_eq!(expr("1 * 2 + 3"), add(mul(num("1"), num("2")), num("3")));
    assert_eq!(expr("1 - 6 / 3"), sub(num("1"), div(num("6"), num("3"))));
}

#[test]
fn same_precedence_is_left_associative() {
    // PG: 1 - 2 - 3 = -4, which is (1 - 2) - 3.
    assert_eq!(expr("1 - 2 - 3"), sub(sub(num("1"), num("2")), num("3")));
    assert_eq!(expr("8 / 4 / 2"), div(div(num("8"), num("4")), num("2")));
    assert_eq!(expr("1 + 2 - 3"), sub(add(num("1"), num("2")), num("3")));
    assert_eq!(expr("2 * 3 / 4"), div(mul(num("2"), num("3")), num("4")));
}

#[test]
fn long_chain_mixes_both_levels() {
    // 1 + 2 * 3 - 4 / 2  →  (1 + (2 * 3)) - (4 / 2)
    assert_eq!(
        expr("1 + 2 * 3 - 4 / 2"),
        sub(
            add(num("1"), mul(num("2"), num("3"))),
            div(num("4"), num("2"))
        )
    );
}

// --- parentheses ----------------------------------------------------------------------------

#[test]
fn parentheses_override_precedence_and_leave_no_node() {
    assert_eq!(expr("(1 + 2) * 3"), mul(add(num("1"), num("2")), num("3")));
    assert_eq!(expr("1 - (2 - 3)"), sub(num("1"), sub(num("2"), num("3"))));
    assert_eq!(expr("((1))"), num("1"));
}

// --- unary minus ----------------------------------------------------------------------------

#[test]
fn unary_minus_binds_tighter_than_mul() {
    // PG: -2 * 3 = -6, parsed as (-2) * 3.
    assert_eq!(expr("-2 * 3"), mul(neg(num("2")), num("3")));
    assert_eq!(expr("2 * -3"), mul(num("2"), neg(num("3"))));
}

#[test]
fn unary_minus_nests_and_applies_to_groups() {
    assert_eq!(expr("- - 2"), neg(neg(num("2"))));
    assert_eq!(expr("-(1 + 2)"), neg(add(num("1"), num("2"))));
    assert_eq!(expr("1 - -2"), sub(num("1"), neg(num("2"))));
    assert_eq!(expr("-TRUE"), neg(boolean(true))); // type error is the evaluator's job
}

// --- errors ---------------------------------------------------------------------------------

#[test]
fn running_out_of_tokens_is_end_of_input() {
    // Position = chars in input + 1 (PG puts the caret just past the end).
    assert_syntax_error("SELECT 1 +", 11, Some(END_OF_INPUT));
    assert_syntax_error("SELECT (1", 10, Some(END_OF_INPUT));
    assert_syntax_error("SELECT 1,", 10, Some(END_OF_INPUT));
    assert_syntax_error("SELECT -", 9, Some(END_OF_INPUT));
}

#[test]
fn bare_select_is_rejected_in_m1() {
    // Divergence: PG accepts `SELECT` with an empty target list (one row, zero columns).
    assert_syntax_error("SELECT", 7, Some(END_OF_INPUT));
    assert_syntax_error("SELECT;", 7, None);
}

#[test]
fn unexpected_token_is_at_or_near_it() {
    assert_syntax_error("SELECT 1 2", 10, None);
    assert_syntax_error("SELECT 1)", 9, None);
    assert_syntax_error("SELECT 1,,2", 10, None);
    assert_syntax_error("SELECT (1 + 2", 14, Some(END_OF_INPUT));
    assert_syntax_error("SELECT * 2", 8, None);
}

#[test]
fn statement_must_start_with_select() {
    assert_syntax_error("1 + 2", 1, None);
    assert_syntax_error("TRUE", 1, None);
}

#[test]
fn not_in_m1_column_names_and_multiple_statements() {
    assert_syntax_error("SELECT foo", 8, None);
    assert_syntax_error("SELECT 1 AS x", 10, None); // `as` lexes as an identifier
    assert_syntax_error("SELECT 1; SELECT 2", 11, None);
    assert_syntax_error("SELECT 1;;", 10, None);
}

#[test]
fn error_positions_count_chars() {
    // S..T 1..=6, ' ' 7, 'ก' string 8..=10, ' ' 11, '2' 12
    assert_syntax_error("SELECT 'ก' 2", 12, None);
    // End of input after Thai: 10 chars + 1
    assert_syntax_error("SELECT 'ก' +", 13, Some(END_OF_INPUT));
}

#[test]
fn lexer_errors_pass_through() {
    match parse("SELECT 'abc") {
        Err(Error::Syntax { message, position }) => {
            assert_eq!(position, 8);
            assert_eq!(message, "unterminated quoted string");
        }
        other => panic!("expected the lexer's error, got {other:?}"),
    }
}
