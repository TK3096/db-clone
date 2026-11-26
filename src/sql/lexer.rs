//! Lexer: `&str` → tokens.
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

use std::{iter::Peekable, str::Chars};

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

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            pos: 0,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += 1;

        Some(c)
    }

    fn advance_if(&mut self, pred: impl Fn(char) -> bool) -> Option<char> {
        if self.peek().is_some_and(pred) {
            self.advance()
        } else {
            None
        }
    }

    fn lex_operator(&mut self, c: char) -> Option<Token> {
        let operator = match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '(' => Token::LParen,
            ')' => Token::RParen,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            _ => return None,
        };

        self.advance();

        Some(operator)
    }

    fn lex_number(&mut self, start: usize) -> Result<Token, Error> {
        let mut digits = String::new();

        while let Some(c) = self.advance_if(|c| c.is_ascii_digit()) {
            digits.push(c);
        }

        if self.peek().is_some_and(is_ident_char) {
            return Err(Error::Syntax {
                message: "trailing junk after numeric literal".to_string(),
                position: start + 1,
            });
        }

        Ok(Token::Number(digits))
    }

    fn lex_word(&mut self) -> Token {
        let mut word = String::new();

        while let Some(c) = self.advance_if(is_ident_char) {
            word.push(c);
        }

        let lower = word.to_lowercase();
        match lower.as_str() {
            "select" => Token::Keyword(Keyword::Select),
            "true" => Token::Keyword(Keyword::True),
            "false" => Token::Keyword(Keyword::False),
            _ => Token::Ident(lower),
        }
    }

    fn lex_string(&mut self, start: usize) -> Result<Token, Error> {
        self.advance();
        let mut value = String::new();

        loop {
            match self.advance() {
                None => {
                    return Err(Error::Syntax {
                        message: "unterminated quoted string".to_string(),
                        position: start + 1,
                    });
                }
                Some('\'') => {
                    if self.peek() != Some('\'') {
                        break;
                    }

                    self.advance();
                    value.push('\'');
                }
                Some(c) => value.push(c),
            }
        }

        Ok(Token::String(value))
    }

    fn peek_second(&self) -> Option<char> {
        self.chars.clone().nth(1)
    }

    fn skip_line_comment(&mut self) {
        while self.advance_if(|c| c != '\n').is_some() {}
    }

    fn run(&mut self) -> Result<Vec<Spanned>, Error> {
        let mut tokens: Vec<Spanned> = Vec::new();

        while let Some(c) = self.peek() {
            let start = self.pos;

            if c == '-' && self.peek_second() == Some('-') {
                self.skip_line_comment();
                continue;
            }

            if c.is_whitespace() {
                self.advance();
            } else {
                let token = match c {
                    '\'' => self.lex_string(start)?,
                    'a'..='z' | 'A'..='Z' | '_' => self.lex_word(),
                    '0'..='9' => self.lex_number(start)?,
                    _ => self.lex_operator(c).ok_or_else(|| Error::Syntax {
                        message: format!("unexpected character \"{c}\""),
                        position: start + 1,
                    })?,
                };

                tokens.push(Spanned {
                    token,
                    position: start + 1,
                });
            }
        }

        Ok(tokens)
    }
}

/// Can appear in an identifier after its first char.
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Splits `input` into tokens. Errors are `Error::Syntax` with the offending char's position
/// (for an unterminated string or `123abc`: the position where that token starts).
pub fn tokenize(input: &str) -> Result<Vec<Spanned>, Error> {
    let mut lexer = Lexer::new(input);
    lexer.run()
}
