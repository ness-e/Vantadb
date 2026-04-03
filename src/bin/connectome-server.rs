use connectomedb::server::{app, ServerState};
use connectomedb::storage::StorageEngine;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!("Starting ConnectomeDB Protocol Daemon on port 8080...");
    
    // Initialize storage engine and wrap in Arc for Axum state sharing
    let storage = Arc::new(StorageEngine::open("connectome_data").unwrap());
    let state = Arc::new(ServerState { storage: storage.clone() });

    // Iniciar Mantenimiento Circadiano (Background Garbage Collector)
    let sleep_storage_ctx = storage.clone();
    tokio::spawn(async move {
        connectomedb::governance::sleep_worker::SleepWorker::start(sleep_storage_ctx).await;
    });

    let router = app(state);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("ConnectomeDB successfully bound to 127.0.0.1:8080");

    axum::serve(listener, router).await.unwrap();
}
