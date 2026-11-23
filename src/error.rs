//! Crate-wide error type. Maps to SQLSTATE codes in later steps.
//! `std::error::Error`, `From<std::io::Error>`, `sqlstate()` and `is_fatal()`
//! (see "Step 0 design decisions" on the M1 ticket).

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
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
        }
    }
}
