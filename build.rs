//! Build script for VantaDB.
//!
//! Shell completions are intentionally NOT generated here: they come from the
//! built binary (`vanta-cli completions --shell <sh>` for static scripts,
//! `COMPLETE=<shell> vanta-cli` for dynamic completion) and the checked-in
//! snapshot under `completions/` is verified by `dev-tools/check_completions.ps1`.
//! Generating into OUT_DIR (the old behavior) left a second, unverified copy and
//! forced `src/cli.rs` to stay build-script-compatible, which blocked
//! engine-backed value completion (`src/cli_complete.rs`).
//!
//! It does write the build identity (FIND-98): `OUT_DIR/build_version.txt`
//! holds `<version> (commit <sha>, worktree <state>)` and is `include_str!`d by
//! `src/bin/vanta-cli.rs` only. A two-month-old copy in `~/.cargo/bin` answered
//! `vanta-cli 0.9.0` exactly like a fresh build, so `scripts/check-cli-fresh.ps1`
//! needs something baked into the artifact to compare against `HEAD`.
//!
//! Why a file and not `cargo:rustc-env` (the FIND-741 mechanism): `rustc-env`
//! belongs to the **package**, and this package is the 200k-LOC engine — every
//! commit would invalidate the lib and every binding built on it. A file in
//! OUT_DIR only lands in the dep-info of the target that includes it, so a
//! commit costs recompiling/relinking the CLI binary and nothing else.

use std::path::Path;
use std::process::Command;

fn main() {
    emit_rerun_paths();

    let version = version_line();
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(dir) => dir,
        // Cargo always sets OUT_DIR for a build script; without it there is
        // nowhere to put the identity and the bin's include_str! would fail
        // with a much worse message.
        Err(_) => panic!("FIND-98: OUT_DIR is not set; cannot emit the build identity"),
    };
    if let Err(e) = write_if_changed(&Path::new(&out_dir).join("build_version.txt"), &version) {
        panic!("FIND-98: could not write build_version.txt: {e}");
    }
}

/// `<version> (commit <sha>, worktree <clean|dirty>)`, or the honest
/// `commit unknown` form when there is no git metadata (tarball, Dockerfile).
fn version_line() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let from_git = git(&["rev-parse", "--short", "HEAD"]);
    // Escape hatch for builds without `.git` (Docker, vendored tree). Ignored
    // when git can answer: the binary must declare where it really came from.
    let from_env = std::env::var("VANTADB_BUILD_COMMIT")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let (Some(git_sha), Some(env_sha)) = (&from_git, &from_env) {
        if git_sha != env_sha {
            println!(
                "cargo:warning=VANTADB_BUILD_COMMIT={env_sha} ignored: git says {git_sha}, \
                 and the binary declares what it was really built from"
            );
        }
    }

    let commit = from_git
        .or(from_env)
        .unwrap_or_else(|| "unknown".to_string());
    let worktree = match git(&["status", "--porcelain", "--untracked-files=no"]) {
        Some(status) if status.is_empty() => "clean",
        Some(_) => "dirty",
        None => "unknown",
    };

    if commit == "unknown" && worktree == "unknown" {
        format!("{version} (commit unknown, not a git checkout)")
    } else {
        format!("{version} (commit {commit}, worktree {worktree})")
    }
}

/// Write only when the bytes differ: cargo compares dep-info by mtime, so
/// rewriting an identical file on every `git add` (index changes) would relink
/// the binary for nothing.
fn write_if_changed(path: &Path, contents: &str) -> std::io::Result<()> {
    if std::fs::read_to_string(path).is_ok_and(|current| current == contents) {
        return Ok(());
    }
    std::fs::write(path, contents)
}

/// `git -C <package root> <args>`, trimmed stdout; `None` when git is absent,
/// the command fails, or this is not a checkout.
fn git(args: &[&str]) -> Option<String> {
    let root = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let out = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Where a re-run is required: the ref moves (commit/checkout), the index
/// changes (staged edits), and the package version (the prefix of the line).
///
/// Emitting `rerun-if-changed` disables Cargo's "any file in the package"
/// default. Source files are deliberately absent: this script's output does not
/// depend on them (rustc tracks them for the targets themselves), and listing
/// them would rewrite the identity file on every edit and relink the bin.
fn emit_rerun_paths() {
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        for name in ["HEAD", "index", "logs/HEAD"] {
            let path = Path::new(&git_dir).join(name);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    for path in ["Cargo.toml", "build.rs"] {
        if Path::new(path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }

    println!("cargo:rerun-if-env-changed=VANTADB_BUILD_COMMIT");
}
