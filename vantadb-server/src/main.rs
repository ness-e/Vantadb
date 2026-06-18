#[cfg(feature = "custom-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::sync::Arc;
use vantadb::storage::StorageEngine;

#[tokio::main]
async fn main() {
    let is_mcp = std::env::args().any(|a| a == "--mcp");

    if is_mcp {
        let config = vantadb::config::VantaConfig::from_env();
        let storage_path = config.storage_path.clone();

        let storage = match StorageEngine::open_with_config(&storage_path, Some(config)) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                eprintln!("Failed to open storage engine: {e}");
                std::process::exit(1);
            }
        };

        // Install SIGTERM handler for MCP mode
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            vantadb::cli_server::wait_for_shutdown_signal().await;
            eprintln!("[vantadb-server MCP] Shutdown signal received, flushing storage...");
            if let Err(e) = storage_clone.flush() {
                eprintln!("[vantadb-server MCP] Flush failed: {e}");
            } else {
                eprintln!("[vantadb-server MCP] Storage flushed");
            }
            std::process::exit(0);
        });

        vantadb::cli_server::init_telemetry(true);
        vantadb_mcp::run_stdio_server(storage).await;
    } else {
        let config = vantadb::config::VantaConfig::from_env();
        if let Err(e) = vantadb::cli_server::run(config).await {
            eprintln!("Server error: {e}");
            std::process::exit(1);
        }
    }
}
