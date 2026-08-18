//! VantaDB desktop client crate.
//!
//! Tauri v2 shell over the multi-connection contract ([`VantaConnection`]) that
//! every backend adapter implements. This module tree owns both the typed HTTP
//! client (DESK-08) and the trait contract (DESK-04); the Tauri runtime + IPC
//! commands (DESK-02/03) live alongside them here.

pub mod commands;
pub mod connections;
pub mod error;

pub use connections::{
    Capability, ConnectionInfo, ConnectionManager, ConnectionStatus, HealthReport, HealthStatus,
    IngestItem, MemoryRecord, SearchQuery, SearchResult, VantaConnection,
};
pub use error::VantaError;

use vantadb::config::VantaConfig;

/// Shared managed state injected into every IPC command (DESK-03).
///
/// `manager` is the [`ConnectionManager`] registry (DESK-06): it holds every
/// open connection and the currently-active one the data commands target.
/// `config` holds the embedded-engine configuration the native adapters
/// (DESK-05) reuse for their opens.
pub struct AppState {
    /// Registry of live connections (native / server / …).
    pub manager: ConnectionManager,
    /// Embedded-engine configuration for native connections.
    pub config: VantaConfig,
}

/// Health-probe IPC command: proves the Rust <-> JS invoke bridge round-trips.
///
/// The frontend calls `invoke("ping")` and receives `"pong"`.
/// Source: https://tauri.app/develop/calling-rust/
#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

/// Desktop app entry point — the real Tauri runtime (wired by DESK-02).
///
/// `main.rs` calls `vantadb_desktop_lib::run()`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        manager: ConnectionManager::new(),
        config: VantaConfig::default(),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::connection::vanta_health,
            commands::connection::vanta_connect,
            commands::connection::vanta_disconnect,
            commands::connection::vanta_list_connections,
            commands::connection::vanta_set_active,
            commands::data::vanta_ingest,
            commands::data::vanta_ingest_batch,
            commands::data::vanta_put,
            commands::data::vanta_search,
            commands::data::vanta_get,
            commands::data::vanta_delete,
            commands::data::vanta_list,
            commands::metrics::vanta_metrics,
            commands::audit::vanta_audit_events,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Shutdown lifecycle (DESKTOP-20): tear down every connection when the
            // app is about to exit. `ExitRequested` fires while the event loop is
            // still alive, so we can block on the async teardown.
            // Source: https://docs.rs/tauri/latest/tauri/enum.RunEvent.html
            if let tauri::RunEvent::ExitRequested { .. } = event {
                use tauri::Manager as _;
                let manager = &app.state::<AppState>().manager;
                let results = tauri::async_runtime::block_on(
                    manager.shutdown_all(ConnectionManager::SHUTDOWN_GRACE),
                );
                for (id, res) in results {
                    if let Err(e) = res {
                        eprintln!("[vantadb-desktop] shutdown: {id}: {e}");
                    }
                }
            }
        });
}
