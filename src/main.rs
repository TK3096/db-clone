//! Binary entry point: parse args, bind, print the address, serve.
//!
//! STUB (mentor). Agreed behaviour (step 1):
//! - `--port <u16>` (optional, default 5433) and `--data-dir <path>` (required), via `std::env::args`.
//! - Bad or missing args: print a message containing "usage" to stderr, exit non-zero.
//! - After a successful bind, print exactly `listening on <addr>` (e.g. `listening on 127.0.0.1:5433`)
//!   as the first line on stderr, then serve.

use std::process::ExitCode;

fn main() -> ExitCode {
    todo!()
}
