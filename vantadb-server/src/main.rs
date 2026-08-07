#![warn(missing_docs)]

//! VantaDB server binary entrypoint. Selects the global allocator at compile
//! time and dispatches to the MCP stdio server or the HTTP CLI server.

/// Jemalloc global allocator (used on non-Windows when `jemalloc` feature is enabled).
#[cfg(all(feature = "jemalloc", not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// MiMalloc global allocator (used on Windows or when `custom-allocator` is
/// enabled without `jemalloc`).
#[cfg(all(
    feature = "custom-allocator",
    any(not(feature = "jemalloc"), target_os = "windows")
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::sync::Arc;
use vantadb::storage::StorageEngine;

/// Application entrypoint. Starts either the MCP stdio server (`--mcp`) or the
/// HTTP CLI server.
/// The single CLI flag this binary understands (alongside `-h`/`--help`).
const MCP_FLAG: &str = "--mcp";

#[tokio::main]
async fn main() {
    // Hand-rolled arg scan (the binary has exactly one boolean flag; clap is
    // only a dev-dependency here and adding it as a runtime dep would be
    // over-engineering). `skip(1)` drops the binary name so flags are matched
    // against real argv values only.
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    let is_mcp = args.iter().any(|a| a == MCP_FLAG);
    let config = vantadb::config::VantaConfig::from_env();

    if is_mcp {
        let storage_path = config.storage_path.clone();

        let storage = match StorageEngine::open_with_config(&storage_path, Some(config.clone())) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                eprintln!("Failed to open storage engine: {e}");
                std::process::exit(1);
            }
        };

        // Init telemetry after storage is open (needs config for log_format)
        vantadb::cli_server::init_telemetry(true, Some(config.log_format));

        vantadb_mcp::run_stdio_server(storage.clone()).await;

        tracing::info!("MCP server exited, flushing storage...");
        if let Err(e) = storage.flush() {
            tracing::error!("Flush failed: {e}");
        } else {
            tracing::info!("Storage flushed");
        }
    } else {
        if let Err(e) = vantadb::cli_server::run(config).await {
            eprintln!("Server error: {e}");
            std::process::exit(1);
        }
    }
}

/// Prints the usage message and exits (called on `-h`/`--help`).
fn print_help() {
    println!(
        "VantaDB server\n\n\
         USAGE:\n    \
         vantadb-server [OPTIONS]\n\n\
         OPTIONS:\n    \
         {MCP_FLAG}    Run as an MCP stdio server instead of the HTTP CLI server\n    \
         -h, --help    Print help information"
    );
}
