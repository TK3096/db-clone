//! M1 step 1: `Server::bind` / `local_addr` / `serve` (mentor-owned tests).

use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use db_clone::server::{Config, Server};
use tempfile::TempDir;

fn config(dir: &TempDir) -> Config {
    Config { port: 0, data_dir: dir.path().to_path_buf() }
}

#[test]
fn bind_port_zero_reports_real_port() {
    let dir = TempDir::new().unwrap();
    let server = Server::bind(config(&dir)).unwrap();
    let addr = server.local_addr();
    assert_ne!(addr.port(), 0, "local_addr must report the port the OS picked");
    assert!(addr.ip().is_loopback(), "must bind 127.0.0.1, got {addr}");
}

#[test]
fn two_servers_bind_side_by_side() {
    // No global state: two servers in one process must not collide.
    let (d1, d2) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let a = Server::bind(config(&d1)).unwrap();
    let b = Server::bind(config(&d2)).unwrap();
    assert_ne!(a.local_addr().port(), b.local_addr().port());
}

#[test]
fn bind_to_taken_port_is_an_error() {
    let d1 = TempDir::new().unwrap();
    let first = Server::bind(config(&d1)).unwrap();
    let d2 = TempDir::new().unwrap();
    let taken = Config { port: first.local_addr().port(), data_dir: d2.path().to_path_buf() };
    assert!(Server::bind(taken).is_err(), "binding a port already in use must return Err");
}

#[test]
fn serve_accepts_tcp_connections() {
    let dir = TempDir::new().unwrap();
    let server = Server::bind(config(&dir)).unwrap();
    let addr = server.local_addr();
    // Leaked on purpose: M1 has no shutdown; the thread dies with the test process.
    thread::spawn(move || server.serve());
    TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("connect to served address");
}
