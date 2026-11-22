//! Crate-wide error type.
//!
//! STUB (mentor): the variant set is a placeholder. The user writes `Display`,
//! `std::error::Error`, `From<std::io::Error>`, `sqlstate()` and `is_fatal()`
//! (see "Step 0 design decisions" on the M1 ticket).

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
