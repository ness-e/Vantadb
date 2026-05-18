use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use vantadb::console;
use vantadb::storage::StorageEngine;
use vantadb_server::server::{app, ServerState};

#[tokio::main]
async fn main() {
    // ── Initialize styled logging & banner ──────────────────────────────────
    console::init_logging();

    let args: Vec<String> = env::args().collect();
    let is_mcp = args.iter().any(|arg| arg == "--mcp");

    if !is_mcp {
        console::print_banner();
        console::progress("Initializing storage engine...", None);
    }

    // ── Load Configuration ──────────────────────────────────────────────────
    let config = vantadb::config::VantaConfig::from_env();

    // ── Open storage engine ─────────────────────────────────────────────────
    let storage = match StorageEngine::open_with_config(&config.storage_path, Some(config.clone()))
    {
        Ok(s) => {
            if !is_mcp {
                console::ok("Storage engine opened", Some(&config.storage_path));
            }
            Arc::new(s)
        }
        Err(e) => {
            console::error("Failed to open storage engine", Some(&e.to_string()));
            std::process::exit(1);
        }
    };

    // ── Bootstrap Invalidation Dispatcher ──────────────────────────────────
    #[cfg(feature = "governance")]
    let invalidation_tx = {
        let mut dispatcher = vantadb::governance::invalidations::InvalidationDispatcher::new(256);
        let tx = dispatcher.sender();
        if let Some(rx) = dispatcher.take_receiver() {
            tokio::spawn(async move {
                vantadb::governance::invalidations::invalidation_listener(rx).await;
            });
        }
        tx
    };

    // ── Background maintenance worker ───────────────────────────────────────
    #[cfg(feature = "governance")]
    {
        let maintenance_storage_ctx = storage.clone();
        tokio::spawn(async move {
            vantadb::governance::maintenance_worker::MaintenanceWorker::start(
                maintenance_storage_ctx,
                invalidation_tx,
            )
            .await;
        });
    }

    #[cfg(feature = "governance")]
    if !is_mcp {
        console::ok(
            "Background workers started",
            Some("maintenance_worker · invalidations"),
        );
    }

    // ── Serve MCP or HTTP ───────────────────────────────────────────────────
    if is_mcp {
        vantadb_server::mcp::run_stdio_server(storage).await;
    } else {
        let state = Arc::new(ServerState {
            storage: storage.clone(),
        });
        let router = app(state);

        let addr = format!("{}:{}", config.host, config.port);

        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => {
                console::ok("TCP listener bound", Some(&addr));
                l
            }
            Err(e) => {
                console::error("Failed to bind port", Some(&e.to_string()));
                std::process::exit(1);
            }
        };

        console::print_ready(&addr);

        if let Err(e) = axum::serve(listener, router).await {
            console::error("Server terminated unexpectedly", Some(&e.to_string()));
            std::process::exit(1);
        }
    }
}
