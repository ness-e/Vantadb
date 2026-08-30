//! `vanta-seed` — import a seed JSON (initial skills + persona) into a
//! VantaDB memory store, or re-import a directory of Markdown files
//! produced by `vanta-cli export --format md` (MEM-62).
//!
//! Usage:
//!
//! ```text
//! vanta-seed <seed.json> [--db <path>]
//! vanta-seed import-md <dir> [--db <path>]
//! ```
//!
//! - Without `--db`, the import runs against an in-memory store (useful to
//!   validate a seed file; nothing is persisted).
//! - With `--db <path>`, records persist under that directory. Requires the
//!   `fjall` feature: `cargo run -p vanta-memory --features fjall --bin vanta-seed`.
//!
//! Seed schema: see `vanta_memory::seed::input` (JSON only).
//! MD schema: see `vanta_memory::seed::md_import` (one file per record with
//! JSON frontmatter).

use std::process::ExitCode;

use vanta_memory::seed::{import_md_dir, import_seed_file, SeedCounts};
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::BackendKind;

#[derive(Debug)]
enum Mode {
    SeedJson(String),
    ImportMd(String),
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!("usage: vanta-seed <seed.json> [--db <path>]");
            eprintln!("       vanta-seed import-md <dir> [--db <path>]");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let mut mode: Option<Mode> = None;
    let mut db_path: Option<String> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--db" => {
                db_path = Some(
                    iter.next()
                        .ok_or_else(|| "--db requires a path argument".to_string())?
                        .clone(),
                );
            }
            "import-md" => {
                let dir = iter
                    .next()
                    .ok_or_else(|| "import-md requires a directory argument".to_string())?
                    .clone();
                mode = Some(Mode::ImportMd(dir));
            }
            other if mode.is_none() && !other.starts_with('-') => {
                mode = Some(Mode::SeedJson(other.to_string()))
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
    }
    let mode = mode.ok_or_else(|| "missing seed file path or 'import-md <dir>'".to_string())?;

    let db = open_db(db_path.as_deref())?;

    let (counts, label) = match &mode {
        Mode::SeedJson(p) => {
            let counts = import_seed_file(&db, std::path::Path::new(p))
                .map_err(|e| format!("seed import failed: {e}"))?;
            (counts, format!("seed {}", p))
        }
        Mode::ImportMd(dir) => {
            let counts = import_md_dir(&db, std::path::Path::new(dir))
                .map_err(|e| format!("md import failed: {e}"))?;
            (counts, format!("md dir {}", dir))
        }
    };
    Ok(match db_path {
        Some(path) => format!("imported {label} into {path}: {counts}"),
        None => format!("imported {label} (in-memory): {counts}"),
    })
}

fn open_db(db_path: Option<&str>) -> Result<VantaEmbedded, String> {
    match db_path {
        Some(path) => {
            let config = VantaConfig {
                storage_path: path.to_string(),
                backend_kind: BackendKind::Fjall,
                ..VantaConfig::default()
            };
            VantaEmbedded::open_with_config(config)
                .map_err(|e| format!("failed to open database at {path}: {e}"))
        }
        None => {
            eprintln!("warning: no --db given; importing into an in-memory store (not persisted)");
            let config = VantaConfig {
                backend_kind: BackendKind::InMemory,
                read_only: false,
                ..VantaConfig::default()
            };
            VantaEmbedded::open_with_config(config)
                .map_err(|e| format!("failed to open in-memory database: {e}"))
        }
    }
}

// keep SeedCounts import used; the import path above uses it transitively.
#[allow(dead_code)]
fn _ensure_seed_counts_in_scope(_: SeedCounts) {}
