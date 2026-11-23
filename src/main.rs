//! Binary entry point: parse args, bind, print the address, serve.
//!
//! - `--port <u16>` (optional, default 5433) and `--data-dir <path>` (required), via `std::env::args`.
//! - Bad or missing args: print a message containing "usage" to stderr, exit non-zero.
//! - After a successful bind, print exactly `listening on <addr>` (e.g. `listening on 127.0.0.1:5433`)
//!   as the first line on stderr, then serve.

use std::{path::PathBuf, process::ExitCode};

use db_clone::server::{Config, Server};

fn main() -> ExitCode {
    let config = match parse_args(std::env::args()) {
        Ok(config) => config,
        Err(msg) => {
            eprintln!("error: {msg}");
            eprintln!("usage: db-clone --data-dir <path> [--port <port>]");
            return ExitCode::FAILURE;
        }
    };

    let server = match Server::bind(config) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let addr = server.local_addr();
    eprintln!("listening on {addr}");

    match server.serve() {
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
        Ok(()) => ExitCode::SUCCESS,
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
    let mut args = args.skip(1);
    let mut port = 5433;
    let mut data_dir: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --port".to_string())?;
                port = value
                    .parse::<u16>()
                    .map_err(|_| format!("invalid port: {value}"))?;
            }
            "--data-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --data-dir".to_string())?;
                data_dir = Some(PathBuf::from(value));
            }
            _ => return Err(format!("unknown argument {arg}")),
        }
    }

    let data_dir = data_dir.ok_or_else(|| "--data-dir is required".to_string())?;

    Ok(Config { port, data_dir })
}
