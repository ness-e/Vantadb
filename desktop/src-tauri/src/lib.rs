//! VantaDB desktop client crate.
//!
//! Tauri v2 shell over the multi-connection contract ([`VantaConnection`]) that
//! every backend adapter implements. This module tree owns both the typed HTTP
//! client (DESK-08) and the trait contract (DESK-04); the Tauri runtime + IPC
//! commands (DESK-02) live alongside them here.

pub mod connections;
pub mod error;

pub use connections::{
    Capability, ConnectionInfo, ConnectionStatus, HealthReport, HealthStatus, IngestItem,
    MemoryRecord, SearchQuery, SearchResult, VantaConnection,
};
pub use error::VantaError;

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
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}