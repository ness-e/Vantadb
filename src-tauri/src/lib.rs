//! Tauri application entry point for VantaDB Desktop (MVP).
//!
//! Wires the [`NativeConnection`](connection::NativeConnection) contract
//! (backed by the in-process VantaDB core) into Tauri commands for the UI.
//! Tauri v2 command API — no plugin required for this MVP.

mod connection;

use connection::{ConnectionError, InProcessConnection, NativeConnection};
use std::sync::{Arc, Mutex};

/// Owns the live connection(s) for the app (DESKTOP-19 ConnectionManager,
/// MVP shape: one in-process connection, trait-abstracted so a future
/// server-backed variant can be added without touching commands/UI).
#[derive(Clone)]
pub struct ConnectionManager {
    conn: Arc<Mutex<dyn NativeConnection>>,
}

impl ConnectionManager {
    fn open_in_memory() -> Result<Self, ConnectionError> {
        Ok(Self {
            conn: Arc::new(Mutex::new(InProcessConnection::open_in_memory()?)),
        })
    }
}

/// Health check command — the MVP contract gate.
#[tauri::command]
fn ping(mgr: tauri::State<'_, ConnectionManager>) -> Result<serde_json::Value, String> {
    let guard = mgr.conn.lock().map_err(|e| format!("state lock poisoned: {e}"))?;
    let health = guard.ping().map_err(|e| e.to_string())?;
    serde_json::to_value(health).map_err(|e| format!("health serialization failed: {e}"))
}

/// Upsert a memory record (CRUD demo).
#[tauri::command]
fn put(
    mgr: tauri::State<'_, ConnectionManager>,
    namespace: String,
    key: String,
    payload: String,
) -> Result<String, String> {
    let guard = mgr.conn.lock().map_err(|e| format!("state lock poisoned: {e}"))?;
    guard.put(&namespace, &key, &payload).map_err(|e| e.to_string())
}

/// Read a memory record.
#[tauri::command]
fn get(
    mgr: tauri::State<'_, ConnectionManager>,
    namespace: String,
    key: String,
) -> Result<Option<String>, String> {
    let guard = mgr.conn.lock().map_err(|e| format!("state lock poisoned: {e}"))?;
    guard.get(&namespace, &key).map_err(|e| e.to_string())
}

/// Delete a memory record.
#[tauri::command]
fn delete(
    mgr: tauri::State<'_, ConnectionManager>,
    namespace: String,
    key: String,
) -> Result<bool, String> {
    let guard = mgr.conn.lock().map_err(|e| format!("state lock poisoned: {e}"))?;
    guard.delete(&namespace, &key).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let manager = match ConnectionManager::open_in_memory() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("FATAL: failed to open in-memory Vanta core: {e}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .manage(manager)
        .invoke_handler(tauri::generate_handler![ping, put, get, delete])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("FATAL: tauri run error: {e}");
            std::process::exit(1);
        });
}