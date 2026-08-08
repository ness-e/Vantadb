//! Operational metrics IPC command (ADMIN-01).
//!
//! `vanta_metrics` returns the core engine's point-in-time operational
//! metrics snapshot. The low-level `vantadb::metrics` module is crate-private
//! (and `OperationalMetricsSnapshot` has no serde derives), so we read through
//! the public SDK surface instead: [`vantadb::VantaEmbedded::operational_metrics`]
//! returns the already-serializable [`vantadb::VantaOperationalMetrics`], which
//! carries every counter the dashboard needs (including `derived_prefix_scans`).

use tauri::State;
use vantadb::config::VantaConfig;
use vantadb::{VantaEmbedded, VantaOperationalMetrics};

use crate::error::VantaError;
use crate::AppState;

/// Snapshot of the core's operational metrics.
///
/// The counters are process-global atomics, so a non-open handle is
/// sufficient: `operational_metrics()` returns the same snapshot whether or
/// not an engine is currently open (memory-breakdown statics reflect the last
/// engine that recorded them).
#[tauri::command]
pub fn vanta_metrics(_app_state: State<AppState>) -> Result<VantaOperationalMetrics, VantaError> {
    // ponytail: reuse the SDK's empty-handle constructor instead of opening a
    // throwaway DB per poll (as vanta_health does) — metrics are process-global
    // atomics, so an engine open would only add I/O. If per-connection engine
    // metrics are ever needed, expose the engine from the active
    // NativeConnection instead.
    let db = VantaEmbedded::test_empty(VantaConfig::default());
    Ok(db.operational_metrics())
}
