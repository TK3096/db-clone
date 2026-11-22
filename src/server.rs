//! Server entry point: bind, then serve.
//!
//! STUB (mentor): signatures agreed in step 1. Bodies are `todo!()`; fill them in.

use std::net::SocketAddr;
use std::path::PathBuf;

use crate::error::Error;

/// Server configuration.
pub struct Config {
    /// TCP port on 127.0.0.1. `0` lets the OS pick a free port.
    pub port: u16,
    /// Data directory. Stored only in M1; first used in M2.
    pub data_dir: PathBuf,
}

/// Shared engine state, handed to every connection as `Arc<Database>`. Empty in M1.
pub struct Database {}

/// A bound, not-yet-serving server.
pub struct Server {
    // Your fields go here (e.g. the listener and an `Arc<Database>`).
}

impl Server {
    /// Binds `127.0.0.1:<config.port>`. Does not accept connections yet.
    pub fn bind(config: Config) -> Result<Server, Error> {
        let _ = config;
        todo!()
    }

    /// The address actually bound (the real port when `config.port == 0`).
    pub fn local_addr(&self) -> SocketAddr {
        todo!()
    }

    /// Accept loop. Does not return unless an error occurs.
    pub fn serve(self) -> Result<(), Error> {
        todo!()
    }
}
