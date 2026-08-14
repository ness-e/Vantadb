//! Process-level tests for the CLI argument handling of `vantadb-server`.
//!
//! Spawns the real binary via `CARGO_BIN_EXE_vantadb-server` so argument
//! validation is exercised end-to-end, not through the library API.
//!
//! The timeout helper exists because a regression that silently starts the
//! HTTP server (blocking forever) must fail the test, not hang the suite.

use std::io::Read;
use std::process::{Child, Command, Output};
use std::time::{Duration, Instant};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vantadb-server"))
}

/// Runs `cmd` to completion, killing it if it does not exit within `timeout`.
fn output_with_timeout(cmd: &mut Command, timeout: Duration) -> Output {
    let mut child: Child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn vantadb-server");

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait().expect("failed to poll child") {
            Some(status) => break status,
            None if Instant::now() > deadline => {
                child.kill().expect("failed to kill vantadb-server");
                let _ = child.wait();
                panic!("vantadb-server did not exit within {timeout:?}");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };

    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stdout {
        pipe.read_to_string(&mut stdout)
            .expect("failed to read stdout");
    }
    if let Some(mut pipe) = child.stderr {
        pipe.read_to_string(&mut stderr)
            .expect("failed to read stderr");
    }

    Output {
        status,
        stdout: stdout.into_bytes(),
        stderr: stderr.into_bytes(),
    }
}

#[test]
fn unknown_flag_fails_with_error() {
    let output = output_with_timeout(binary().args(["--por", "8080"]), Duration::from_secs(10));

    assert!(
        !output.status.success(),
        "unknown flag must fail, got status {:?}",
        output.status.code()
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unrecognized argument '--por'"),
        "stderr should name the unknown flag, got: {stderr}"
    );
}

#[test]
fn help_exits_zero() {
    let output = output_with_timeout(binary().arg("--help"), Duration::from_secs(10));

    assert!(
        output.status.success(),
        "--help must exit 0, got status {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("USAGE"),
        "--help should print usage, got: {stdout}"
    );
}
