use connectomedb::server::{app, ServerState};
use connectomedb::storage::StorageEngine;
use std::sync::Arc;
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let is_mcp = args.iter().any(|arg| arg == "--mcp");

    if !is_mcp {
        println!("Starting ConnectomeDB Protocol Daemon on port 8080...");
    }

    // El StorageEngine al ser abierto arrancará el HardwareScout en $O(1)$ y los eprint logs irán a stderr.
    let storage = Arc::new(StorageEngine::open("connectome_data").unwrap());
    
    // Bootstrap Invalidation Dispatcher (Reactive Event Bus)
    let mut dispatcher = connectomedb::governance::invalidations::InvalidationDispatcher::new(256);
    let invalidation_tx = dispatcher.sender();
    if let Some(rx) = dispatcher.take_receiver() {
        tokio::spawn(async move {
            connectomedb::governance::invalidations::invalidation_listener(rx).await;
        });
    }

    // Iniciar Mantenimiento Circadiano (Background Garbage Collector / Inmune System)
    let sleep_storage_ctx = storage.clone();
    tokio::spawn(async move {
        connectomedb::governance::sleep_worker::SleepWorker::start(sleep_storage_ctx, invalidation_tx).await;
    });

    if is_mcp {
        connectomedb::api::mcp::run_stdio_server(storage).await;
    } else {
        let state = Arc::new(ServerState { storage: storage.clone() });
        let router = app(state);

        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        println!("ConnectomeDB successfully bound to 127.0.0.1:8080");

        axum::serve(listener, router).await.unwrap();
    }
}
