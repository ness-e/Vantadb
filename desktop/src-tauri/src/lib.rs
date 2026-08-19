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

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use vantadb::config::VantaConfig;

/// Event name emitted (Rust → frontend) when a `vanta://` deep link arrives
/// while the app is already running (single-instance callback). The payload
/// is a `Vec<String>` of raw URLs, parsed by the frontend (`parseVantaUrl`).
pub const DEEP_LINK_EVENT: &str = "vanta-deep-link";

/// Shared managed state injected into every IPC command (DESK-03).
///
/// `manager` is the [`ConnectionManager`] registry (DESK-06): it holds every
/// open connection and the currently-active one the data commands target.
/// `config` holds the embedded-engine configuration the native adapters
/// (DESK-05) reuse for their opens. `pending_deep_links` buffers `vanta://`
/// URLs that arrived before the frontend was ready to consume them (VS-16).
pub struct AppState {
    /// Registry of live connections (native / server / …).
    pub manager: ConnectionManager,
    /// Embedded-engine configuration for native connections.
    pub config: VantaConfig,
    /// Raw `vanta://` URLs waiting for the frontend to take them (VS-16).
    pub pending_deep_links: Arc<Mutex<Vec<String>>>,
}

/// Drain any deep-link URLs buffered while the frontend was loading (VS-16).
///
/// Called once at frontend startup; returns the raw `vanta://` URLs the app
/// was started with (or that arrived before the listener mounted). The
/// frontend validates the format with `parseVantaUrl` before navigating
/// (official deep-link caution: never trust raw args).
#[tauri::command]
fn vanta_deep_link_take(state: tauri::State<'_, AppState>) -> Vec<String> {
    let mut guard = state
        .pending_deep_links
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    std::mem::take(&mut *guard)
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
///
/// Deep links (VS-16) follow the official `tauri-plugin-deep-link` v2 setup:
/// on Windows/Linux the OS delivers the URL as a CLI argument to a new
/// process, so we pair the plugin with `tauri-plugin-single-instance`
/// (feature `deep-link`, registered FIRST — documented requirement) and
/// route URLs through `pending_deep_links` + a Rust-emitted event.
/// Sources: https://v2.tauri.app/plugin/deep-linking/
///          https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/deep-link
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        manager: ConnectionManager::new(),
        config: VantaConfig::default(),
        pending_deep_links: Arc::new(Mutex::new(Vec::new())),
    };

    let mut builder = tauri::Builder::default();

    // Single-instance MUST be the first plugin registered so deep-link URLs
    // are routed to the running instance instead of spawning a second one
    // (official docs). The callback runs IN the primary instance with the new
    // argv: filter `vanta://` URLs, buffer them, and emit the event.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let urls: Vec<String> = argv
                .iter()
                .filter(|a| a.starts_with("vanta://"))
                .cloned()
                .collect();
            if urls.is_empty() {
                return;
            }
            if let Some(st) = app.try_state::<AppState>() {
                st.pending_deep_links
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .extend(urls.clone());
            }
            // Best-effort emit: the frontend may not be listening yet — the
            // buffer above covers that race (it drains via vanta_deep_link_take).
            let _ = app.emit(DEEP_LINK_EVENT, urls);
        }));
    }

    builder
        .manage(state)
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Register the configured `vanta://` scheme for the current
            // executable so dev builds and uninstalled binaries still handle
            // deep links (macOS forbids runtime registration — docs).
            #[cfg(any(windows, target_os = "linux"))]
            {
                if let Err(e) = app.deep_link().register_all() {
                    eprintln!("[vantadb-desktop] deep-link register_all: {e}");
                }
            }
            // Capture the URL the app was STARTED with, before the frontend
            // loads. get_current also updates on every runtime trigger, but
            // we only need the startup case here (runtime is the event above).
            if let Some(urls) = app.deep_link().get_current()? {
                let vanta: Vec<String> = urls
                    .iter()
                    .filter(|u| u.as_str().starts_with("vanta://"))
                    .map(|u| u.to_string())
                    .collect();
                if !vanta.is_empty() {
                    *app.state::<AppState>()
                        .pending_deep_links
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = vanta;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            vanta_deep_link_take,
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
            commands::data::vanta_get_version,
            commands::data::vanta_versions,
            commands::data::vanta_delete,
            commands::data::vanta_list,
            commands::data::vanta_query,
            commands::data::vanta_iql_autocomplete,
            commands::data::vanta_export_namespace,
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
