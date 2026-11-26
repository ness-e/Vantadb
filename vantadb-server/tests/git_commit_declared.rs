// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! FIND-741: the binary must declare the commit it was built from.
//!
//! A stale `target/` artifact otherwise keeps answering with the old shape of a
//! response and a DoD passes for a feature that is not there (MCP-47 was closed
//! against a binary built before `2fb20c72` and `memory:{captured,queued}` was
//! missing from a `{"ok":true}` that looked fine).
//!
//! Process-level on purpose (`CARGO_BIN_EXE_*`): the contract is what
//! `--version` prints, not what an internal helper returns.

use std::path::Path;
use std::process::Command;

/// Runs `git -C <vantadb-server>`; `None` when git is absent or the command
/// fails (not a checkout, broken repo, no git binary).
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[test]
fn version_declares_the_head_commit() {
    let manifest = env!("CARGO_MANIFEST_DIR");

    let Some(head) = git(&["rev-parse", "--short", "HEAD"]) else {
        // Skipping is legitimate only when there really is no git metadata
        // (source tarball / vendored tree). A `.git` that exists while
        // `rev-parse` fails is a broken checkout, not an excuse to skip.
        assert!(
            !Path::new(manifest).join("../.git").exists(),
            "git rev-parse failed inside a checkout — refusing to skip silently"
        );
        eprintln!("SKIP version_declares_the_head_commit: no git metadata under {manifest}");
        return;
    };

    let out = Command::new(env!("CARGO_BIN_EXE_vantadb-server"))
        .arg("--version")
        .output()
        .expect("failed to run vantadb-server --version");
    assert!(
        out.status.success(),
        "--version must exit 0, got {:?}",
        out.status.code()
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("commit {head}")),
        "`--version` must declare the commit it was built from (HEAD = {head}), got: {stdout}"
    );
    assert!(
        stdout.contains("worktree clean") || stdout.contains("worktree dirty"),
        "`--version` must also say whether the build tree was dirty, got: {stdout}"
    );
}
