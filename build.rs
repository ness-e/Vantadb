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
//! holds `<version> (commit <sha>)` and is `include_str!`d by
//! `src/bin/vanta-cli.rs` only. A two-month-old copy in `~/.cargo/bin` answered
//! `vanta-cli 0.9.0` exactly like a fresh build, so `scripts/check-cli-fresh.ps1`
//! needs something baked into the artifact to compare against `HEAD`.
//!
//! Two deliberate differences from the FIND-741 script in `vantadb-server`:
//!
//! 1. **A file, not `cargo:rustc-env`.** `rustc-env` belongs to the *package*,
//!    and this package is the 200k-LOC engine: every commit would invalidate the
//!    lib and every binding. A file in OUT_DIR only lands in the dep-info of the
//!    target that includes it, so a commit costs recompiling/relinking the CLI.
//! 2. **No worktree clean/dirty label.** That label is only true if this script
//!    re-runs whenever the working tree changes, i.e. on every edit under
//!    `src/` — which would recompile the engine every time the label flips.
//!    Watching only git metadata keeps the **commit** exact (a commit always
//!    moves the reflog and/or the ref) and says nothing it cannot observe.
//!    `vantadb-server` can afford the label because its crate is ~1.4k lines.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    emit_rerun_paths();

    let version = version_line();
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(dir) => dir,
        // Cargo always sets OUT_DIR for a build script; without it the bin's
        // include_str! would fail with a far worse message.
        Err(_) => panic!("FIND-98: OUT_DIR is not set; cannot emit the build identity"),
    };
    if let Err(e) = std::fs::write(Path::new(&out_dir).join("build_version.txt"), &version) {
        panic!("FIND-98: could not write build_version.txt: {e}");
    }
}

/// `<version> (commit <sha>)`, or the honest `commit unknown` form when there
/// is no git metadata (tarball, Dockerfile).
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

    match from_git.or(from_env) {
        Some(commit) => format!("{version} (commit {commit})"),
        None => format!("{version} (commit unknown, not a git checkout)"),
    }
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

/// Absolute path of a git-internal path (works in linked worktrees, where the
/// refs live in the common dir while `HEAD`/`logs/HEAD` live in the worktree's
/// gitdir).
fn git_path(name: &str) -> Option<PathBuf> {
    let root = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let raw = git(&["rev-parse", "--path-format=absolute", "--git-path", name])
        .or_else(|| git(&["rev-parse", "--git-path", name]))?;
    if raw.is_empty() {
        return None;
    }
    let path = PathBuf::from(&raw);
    Some(if path.is_absolute() {
        path
    } else {
        Path::new(&root).join(path)
    })
}

/// Where a re-run is required: the ref moves (`HEAD` in a detached checkout, the
/// branch's own ref file) or the reflog grows (a commit in any lane). `index` is
/// deliberately absent: it changes on every `git add` and only feeds the
/// clean/dirty label this script no longer claims, so watching it would relink
/// the binary for nothing.
///
/// Emitting `rerun-if-changed` disables Cargo's "any file in the package"
/// default. Source files are deliberately absent: the output does not depend on
/// them (rustc tracks them for the targets themselves), and listing them would
/// rewrite the identity on every edit.
fn emit_rerun_paths() {
    let mut watched: Vec<PathBuf> = Vec::new();
    watched.extend(git_path("HEAD"));
    watched.extend(git_path("logs/HEAD"));
    if let Some(reference) = git(&["rev-parse", "--symbolic-full-name", "HEAD"]) {
        if reference.starts_with("refs/") {
            watched.extend(git_path(&reference));
        }
    }

    for path in watched {
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    for path in ["Cargo.toml", "build.rs"] {
        if Path::new(path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }

    println!("cargo:rerun-if-env-changed=VANTADB_BUILD_COMMIT");
}
