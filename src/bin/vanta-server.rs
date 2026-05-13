use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use vantadb::console;
use vantadb::server::{app, ServerState};
use vantadb::storage::StorageEngine;

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

    // ── Open storage engine ─────────────────────────────────────────────────
    let data_dir = env::var("VANTADB_STORAGE_PATH").unwrap_or_else(|_| "vantadb_data".to_string());
    let storage = match StorageEngine::open(&data_dir) {
        Ok(s) => {
            if !is_mcp {
                console::ok("Storage engine opened", Some(&data_dir));
            }
            Arc::new(s)
        }
        Err(e) => {
            console::error("Failed to open storage engine", Some(&e.to_string()));
            std::process::exit(1);
        }
    };

    // ── Bootstrap Invalidation Dispatcher ──────────────────────────────────
    let mut dispatcher = vantadb::governance::invalidations::InvalidationDispatcher::new(256);
    let invalidation_tx = dispatcher.sender();
    if let Some(rx) = dispatcher.take_receiver() {
        tokio::spawn(async move {
            vantadb::governance::invalidations::invalidation_listener(rx).await;
        });
    }

    // ── Background maintenance worker ───────────────────────────────────────
    let maintenance_storage_ctx = storage.clone();
    tokio::spawn(async move {
        vantadb::governance::maintenance_worker::MaintenanceWorker::start(
            maintenance_storage_ctx,
            invalidation_tx,
        )
        .await;
    });

    if !is_mcp {
        console::ok(
            "Background workers started",
            Some("maintenance_worker · invalidations"),
        );
    }

    // ── Serve MCP or HTTP ───────────────────────────────────────────────────
    if is_mcp {
        vantadb::api::mcp::run_stdio_server(storage).await;
    } else {
        let state = Arc::new(ServerState {
            storage: storage.clone(),
        });
        let router = app(state);

        let host = env::var("VANTADB_HOST")
            .or_else(|_| env::var("HOST"))
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("VANTADB_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "8080".to_string());
        let addr = format!("{}:{}", host, port);

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
