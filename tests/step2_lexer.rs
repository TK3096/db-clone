//! M1 step 2: lexer (mentor-owned tests).

use db_clone::error::Error;
use db_clone::sql::lexer::{Keyword, Spanned, Token, tokenize};

use Keyword::*;
use Token::*;

fn lex(input: &str) -> Vec<Spanned> {
    tokenize(input).unwrap_or_else(|e| panic!("tokenize({input:?}) failed: {e:?}"))
}

/// Tokens only, positions ignored.
fn tokens(input: &str) -> Vec<Token> {
    lex(input).into_iter().map(|s| s.token).collect()
}

fn at(token: Token, position: usize) -> Spanned {
    Spanned { token, position }
}

fn num(s: &str) -> Token {
    Number(s.to_string())
}

fn string(s: &str) -> Token {
    Token::String(s.to_string())
}

fn ident(s: &str) -> Token {
    Ident(s.to_string())
}

/// Asserts `input` fails with `Error::Syntax` at `position`.
fn assert_syntax_error(input: &str, position: usize) {
    match tokenize(input) {
        Err(Error::Syntax { position: p, .. }) => {
            assert_eq!(p, position, "wrong error position for {input:?}")
        }
        other => panic!("{input:?}: expected Error::Syntax at {position}, got {other:?}"),
    }
}

// --- basics -------------------------------------------------------------------------------

#[test]
fn empty_and_whitespace_only_input_give_no_tokens() {
    assert_eq!(lex(""), vec![]);
    assert_eq!(lex("  \t\n\r\n "), vec![]);
}

#[test]
fn select_one_with_positions() {
    assert_eq!(
        lex("SELECT 1"),
        vec![at(Keyword(Select), 1), at(num("1"), 8)]
    );
}

#[test]
fn whitespace_kinds_are_all_skipped() {
    // S E L E C T \n \t 1  →  '1' is the 9th char
    assert_eq!(
        lex("SELECT\n\t1"),
        vec![at(Keyword(Select), 1), at(num("1"), 9)]
    );
}

// --- keywords and identifiers ---------------------------------------------------------------

#[test]
fn keywords_are_case_insensitive() {
    assert_eq!(
        tokens("select SeLeCt SELECT true TRUE False"),
        vec![
            Keyword(Select),
            Keyword(Select),
            Keyword(Select),
            Keyword(True),
            Keyword(True),
            Keyword(False)
        ]
    );
}

#[test]
fn identifiers_are_lowercased() {
    assert_eq!(
        tokens("foo Foo FOO foo_bar1 _x"),
        vec![
            ident("foo"),
            ident("foo"),
            ident("foo"),
            ident("foo_bar1"),
            ident("_x")
        ]
    );
}

#[test]
fn keyword_prefix_is_still_an_identifier() {
    assert_eq!(
        tokens("selected trueish false_"),
        vec![ident("selected"), ident("trueish"), ident("false_")]
    );
}

// --- numbers --------------------------------------------------------------------------------

#[test]
fn numbers_stay_raw_strings() {
    // Too big for i64: the lexer must not try to parse it.
    assert_eq!(
        tokens("0 007 2147483648 99999999999999999999"),
        vec![
            num("0"),
            num("007"),
            num("2147483648"),
            num("99999999999999999999")
        ]
    );
}

#[test]
fn number_followed_by_letters_is_an_error() {
    // PG 15+: "trailing junk after numeric literal". Position = start of the number.
    assert_syntax_error("SELECT 123abc", 8);
}

// --- operators and punctuation --------------------------------------------------------------

#[test]
fn all_operators_and_punctuation() {
    assert_eq!(
        tokens("+ - * / ( ) , ;"),
        vec![Plus, Minus, Star, Slash, LParen, RParen, Comma, Semicolon]
    );
}

#[test]
fn no_spaces_needed_between_tokens() {
    assert_eq!(
        lex("1+2*(3-4);"),
        vec![
            at(num("1"), 1),
            at(Plus, 2),
            at(num("2"), 3),
            at(Star, 4),
            at(LParen, 5),
            at(num("3"), 6),
            at(Minus, 7),
            at(num("4"), 8),
            at(RParen, 9),
            at(Semicolon, 10),
        ]
    );
}

#[test]
fn minus_is_always_a_separate_token() {
    // Unary minus is the parser's job (step 3), not the lexer's.
    assert_eq!(tokens("-1"), vec![Minus, num("1")]);
}

// --- strings --------------------------------------------------------------------------------

#[test]
fn simple_and_empty_strings() {
    assert_eq!(tokens("'hi' ''"), vec![string("hi"), string("")]);
}

#[test]
fn doubled_quote_is_an_escaped_quote() {
    assert_eq!(tokens("'it''s'"), vec![string("it's")]);
    assert_eq!(tokens("''''"), vec![string("'")]);
}

#[test]
fn strings_keep_spaces_newlines_and_case() {
    assert_eq!(
        tokens("'  Hello,\nWorld; SELECT  '"),
        vec![string("  Hello,\nWorld; SELECT  ")]
    );
}

#[test]
fn adjacent_strings_are_separate_tokens() {
    // 'a' 'b' with a space: two tokens (PG would only concatenate across a newline; not in M1).
    assert_eq!(tokens("'a' 'b'"), vec![string("a"), string("b")]);
}

#[test]
fn non_ascii_inside_strings() {
    assert_eq!(tokens("'สวัสดี'"), vec![string("สวัสดี")]);
}

#[test]
fn positions_count_chars_not_bytes() {
    // "SELECT " = 7 chars; 'สวัสดี' = 8 chars (quote + 6 code points + quote) at 8..=15;
    // ',' at 16; ' ' at 17; '1' at 18. In bytes, '1' would be at 30.
    assert_eq!(
        lex("SELECT 'สวัสดี', 1"),
        vec![
            at(Keyword(Select), 1),
            at(string("สวัสดี"), 8),
            at(Comma, 16),
            at(num("1"), 18),
        ]
    );
}

#[test]
fn unterminated_string_is_an_error_at_its_opening_quote() {
    assert_syntax_error("SELECT 'abc", 8);
    assert_syntax_error("SELECT 'it''s", 8);
    assert_syntax_error("'", 1);
}

// --- errors ---------------------------------------------------------------------------------

#[test]
fn unknown_char_is_an_error_at_that_char() {
    assert_syntax_error("SELECT @", 8);
    assert_syntax_error("1 % 2", 3);
}

#[test]
fn error_position_after_non_ascii_counts_chars() {
    // S..T = 1..=6, ' ' 7, '\'' 8, 'ก' 9, '\'' 10, ' ' 11, '@' 12
    assert_syntax_error("SELECT 'ก' @", 12);
}

#[test]
fn out_of_scope_syntax_is_rejected() {
    assert_syntax_error("SELECT \"Foo\"", 8); // quoted identifier
    assert_syntax_error("SELECT 1.5", 9); // decimal: '.' is unknown in M1
    assert_syntax_error("ก", 1); // non-ASCII outside a string
}
