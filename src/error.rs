//! Crate-wide error type. Maps to SQLSTATE codes in later steps.

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    /// SQLSTATE 42601. `position` is a 1-based char offset into the query (ErrorResponse field `P`).
    Syntax {
        message: String,
        position: usize,
    },
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Syntax { .. } => todo!(), // STUB (mentor): your message format
        }
    }
}
