//! Build script for VantaDB.
//!
//! Intentionally a no-op: shell completions are generated from the built binary
//! (`vanta-cli completions --shell <sh>` for static scripts,
//! `COMPLETE=<shell> vanta-cli` for dynamic completion) and the checked-in
//! snapshot under `completions/` is verified by `dev-tools/check_completions.ps1`.
//! Generating into OUT_DIR from here (the old behavior) left a second,
//! unverified copy and forced `src/cli.rs` to stay build-script-compatible,
//! which blocked engine-backed value completion (`src/cli_complete.rs`).

fn main() {}
