//! Server entry point: bind, then serve.

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::Error;

/// Server configuration.
pub struct Config {
    /// TCP port on 127.0.0.1. `0` lets the OS pick a free port.
    pub port: u16,
    /// Data directory. Accepted but unused in M1; first used in M2.
    pub data_dir: PathBuf,
}

/// Shared engine state, handed to every connection as `Arc<Database>`. Empty in M1.
pub struct Database {}

/// A bound, not-yet-serving server.
pub struct Server {
    listener: TcpListener,
    addr: SocketAddr,
    db: Arc<Database>,
}

impl Server {
    /// Binds `127.0.0.1:<config.port>`. Does not accept connections yet.
    pub fn bind(config: Config) -> Result<Server, Error> {
        let listener = TcpListener::bind(("127.0.0.1", config.port))?;
        let addr = listener.local_addr()?;

        Ok(Server {
            listener,
            addr,
            db: Arc::new(Database {}),
        })
    }

    /// The address actually bound (the real port when `config.port == 0`).
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// Accepts connections forever and handles each one inline (no threads until M11).
    ///
    /// A failed `accept` is logged to stderr and skipped; it never stops the server.
    /// In M1 this function never returns; the `Result` is kept for future fatal errors
    /// (e.g. shutdown in M11).
    pub fn serve(self) -> Result<(), Error> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_connection(stream, Arc::clone(&self.db));
                }
                Err(e) => eprintln!("accept error: {e}"),
            }
        }

        Ok(())
    }
}

fn handle_connection(_stream: TcpStream, _db: Arc<Database>) {}
