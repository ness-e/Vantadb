//! Build script for VantaDB
//!
//! Generates shell completion scripts automatically during compilation.

#[cfg(feature = "cli")]
#[path = "src/cli.rs"]
mod cli;

#[cfg(feature = "cli")]
use clap::CommandFactory;

fn main() {
    generate_completions();
}

#[cfg(not(feature = "cli"))]
fn generate_completions() {}

#[cfg(feature = "cli")]
fn generate_completions() {
    use std::fs::create_dir_all;
    use std::path::PathBuf;

    // Tell Cargo to re-run this build script if src/cli.rs changes
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-env-changed=VANTA_OUT_DIR");

    // Publish-safe: default to OUT_DIR (outside the source tree) so
    // `cargo publish --verify` never sees the source dir modified
    // (cargo fails the publish when build.rs touches packaged files).
    // To refresh the checked-in snapshot under `completions/`, run once:
    //   VANTA_OUT_DIR=completions cargo build -p vantadb
    let out_dir = match std::env::var_os("VANTA_OUT_DIR") {
        Some(out) => PathBuf::from(out),
        None => match std::env::var_os("OUT_DIR") {
            Some(out) => PathBuf::from(out),
            None => return,
        },
    };

    if let Err(e) = create_dir_all(&out_dir) {
        println!(
            "cargo:warning=Failed to create completions directory '{:?}': {}",
            out_dir, e
        );
        return;
    }

    let mut cmd = cli::Cli::command();

    let shells = [
        clap_complete::Shell::Bash,
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Fish,
        clap_complete::Shell::PowerShell,
    ];

    for &shell in &shells {
        if let Err(e) = clap_complete::generate_to(shell, &mut cmd, "vanta-cli", &out_dir) {
            println!(
                "cargo:warning=Failed to generate completions for {:?}: {}",
                shell, e
            );
        }
    }
}
