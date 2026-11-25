//! Embeds the commit this binary was built from (FIND-741).
//!
//! A stale artifact under `target/` cannot say which commit it came from, so a
//! DoD verified against it passes for a feature that is not there (MCP-47 was
//! closed against a binary built before `2fb20c72`). `--version` prints the
//! values emitted here.
//!
//! Why a build script and not `git rev-parse` at runtime: the question a DoD
//! must answer is "did what I just ran come from this commit?", and only
//! something baked into the artifact can answer it. A runtime read reports the
//! `HEAD` of the caller's cwd — it always agrees with itself and can never
//! reveal a stale binary.

use std::path::Path;
use std::process::Command;

fn main() {
    emit_rerun_paths();

    let from_git = git(&["rev-parse", "--short", "HEAD"]);
    // Escape hatch for builds without `.git` (Docker, vendored tree): the SHA
    // is pinned explicitly instead of guessed. Ignored when git can answer —
    // the binary must declare where it really came from.
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

    println!("cargo:rustc-env=VANTADB_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=VANTADB_BUILD_WORKTREE={worktree}");
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

/// Where a rebuild must re-run this script: the ref moves, the index changes
/// (staged edits), or the inputs of the compiled artifact change — this
/// package's sources, the lockfile, the toolchain/config pins, and the sources
/// and manifests of the sub-crates it links (unstaged edits flip `clean` →
/// `dirty`).
///
/// Emitting `rerun-if-changed` disables Cargo's "any file in the package"
/// default, so every input above has to be listed explicitly.
///
/// ponytail: mtime-based, per path. Cost of a commit is one `git status` plus a
/// rebuild of THIS crate (~19 s measured) and zero of the 200k-LOC root crate:
/// `cargo:rustc-env` belongs to the emitting package only.
fn emit_rerun_paths() {
    // Resolve the git dir (works in linked worktrees, where `.git` is a file).
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        for name in ["HEAD", "index", "logs/HEAD"] {
            let path = Path::new(&git_dir).join(name);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    for path in [
        "src",
        "Cargo.toml",
        "build.rs",
        "../Cargo.toml",
        "../Cargo.lock",
        "../build.rs",
        "../rust-toolchain.toml",
        "../.cargo/config.toml",
        "../src",
        "../vantadb-mcp/src",
        "../vantadb-mcp/Cargo.toml",
        "../vanta-memory/src",
        "../vanta-memory/Cargo.toml",
        "../vantadb-ffi-core/src",
        "../vantadb-ffi-core/Cargo.toml",
    ] {
        if Path::new(path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }

    println!("cargo:rerun-if-env-changed=VANTADB_BUILD_COMMIT");
}
