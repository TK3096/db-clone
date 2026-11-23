//! M1 step 1: the `db-clone` binary's argument handling (mentor-owned tests).

use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::time::Duration;

use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_db-clone");

fn assert_usage_error(args: &[&str]) {
    let out = Command::new(BIN).args(args).output().unwrap();
    assert!(!out.status.success(), "{args:?} must exit non-zero");
    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
    assert!(
        stderr.contains("usage"),
        "{args:?}: stderr must mention usage, got: {stderr}"
    );
}

#[test]
fn rejects_non_numeric_port() {
    assert_usage_error(&["--port", "abc", "--data-dir", "/tmp/x"]);
}

#[test]
fn rejects_out_of_range_port() {
    assert_usage_error(&["--port", "70000", "--data-dir", "/tmp/x"]);
}

#[test]
fn requires_data_dir() {
    assert_usage_error(&["--port", "5433"]);
}

#[test]
fn rejects_unknown_flag() {
    assert_usage_error(&["--data-dir", "/tmp/x", "--bogus"]);
}

#[test]
fn starts_and_reports_listening_address() {
    let dir = TempDir::new().unwrap();
    let mut child = Command::new(BIN)
        .args(["--port", "0", "--data-dir"])
        .arg(dir.path())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut line = String::new();
    BufReader::new(child.stderr.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let result = (|| {
        let addr: SocketAddr = line
            .trim()
            .strip_prefix("listening on ")
            .ok_or_else(|| {
                format!("first stderr line must be `listening on <addr>`, got {line:?}")
            })?
            .parse()
            .map_err(|e| format!("bad address in {line:?}: {e}"))?;
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).map_err(|e| e.to_string())
    })();

    let _ = child.kill();
    let _ = child.wait();
    result.unwrap();
}
