//! `vanta-seed` — import a seed JSON (initial skills + persona) into a
//! VantaDB memory store (MEM-39).
//!
//! Usage:
//!
//! ```text
//! vanta-seed <seed.json> [--db <path>]
//! ```
//!
//! - Without `--db`, the import runs against an in-memory store (useful to
//!   validate a seed file; nothing is persisted).
//! - With `--db <path>`, records persist under that directory. Requires the
//!   `fjall` feature: `cargo run -p vanta-memory --features fjall --bin vanta-seed`.
//!
//! Seed schema: see `vanta_memory::seed::input` (JSON only).

use std::process::ExitCode;

use vanta_memory::seed::import_seed_file;
use vantadb::config::VantaConfig;
use vantadb::storage::BackendKind;

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
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let mut seed_path: Option<&str> = None;
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
            other if seed_path.is_none() && !other.starts_with('-') => seed_path = Some(other),
            other => return Err(format!("unexpected argument: {other}")),
        }
    }
    let seed_path = seed_path.ok_or_else(|| "missing seed file path".to_string())?;

    let db = match &db_path {
        Some(path) => {
            let config = VantaConfig {
                storage_path: path.clone(),
                backend_kind: BackendKind::Fjall,
                ..VantaConfig::default()
            };
            vantadb::sdk::VantaEmbedded::open_with_config(config)
                .map_err(|e| format!("failed to open database at {path}: {e}"))?
        }
        None => {
            eprintln!("warning: no --db given; importing into an in-memory store (not persisted)");
            let config = VantaConfig {
                backend_kind: BackendKind::InMemory,
                read_only: false,
                ..VantaConfig::default()
            };
            vantadb::sdk::VantaEmbedded::open_with_config(config)
                .map_err(|e| format!("failed to open in-memory database: {e}"))?
        }
    };

    let counts = import_seed_file(&db, std::path::Path::new(seed_path))
        .map_err(|e| format!("seed import failed: {e}"))?;
    Ok(match db_path {
        Some(path) => format!("seed imported into {path}: {counts}"),
        None => format!("seed imported (in-memory): {counts}"),
    })
}
