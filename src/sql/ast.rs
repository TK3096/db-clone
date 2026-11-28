//! Abstract syntax tree produced by the parser.
//!
//! Agreed in M1 step 3. Parentheses leave no node: the tree's shape encodes grouping.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    /// `SELECT <expr>[, <expr>...]`
    Select(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Literal(Literal),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    /// Raw digits; the evaluator picks the type.
    Number(String),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
