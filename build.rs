//! Build script for VantaDB
//!
//! Generates shell completion scripts automatically during compilation.

#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

use clap::CommandFactory;
use std::fs::create_dir_all;
use std::path::PathBuf;

fn main() {
    // Tell Cargo to re-run this build script if src/cli.rs changes
    println!("cargo:rerun-if-changed=src/cli.rs");

    // We write completions to a subdirectory in the workspace.
    // To accommodate read-only production builders, we handle errors gracefully.
    let out_dir = match std::env::var_os("VANTA_OUT_DIR") {
        Some(out) => PathBuf::from(out),
        None => PathBuf::from("completions"),
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
