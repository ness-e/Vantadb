//! Connection-facing IPC commands (DESK-03).
//!
//! `vanta_health` proves the core [`VantaEmbedded`] engine works from the Tauri
//! shell: it opens the database in a throwaway temp dir, probes capabilities,
//! reports the backend, and closes. Duplicate-open lock handling is deliberately
//! covered by DESK-05 NativeConnection, so the health probe uses a unique dir
//! per call (two probes never collide).

use std::time::{SystemTime, UNIX_EPOCH};

use tauri::State;
use vantadb::VantaEmbedded;

use crate::error::VantaError;
use crate::{AppState, HealthReport, HealthStatus};

/// Unique temp subdirectory for a health probe, safe across calls / processes.
fn probe_dir() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir()
        .join(format!("vantadb-desktop-health-{}-{:x}", std::process::id(), ts))
        .to_string_lossy()
        .into_owned()
}

/// Map a core `vantadb` engine error onto the desktop contract error.
fn map_core_error(err: vantadb::VantaError) -> VantaError {
    use vantadb::VantaError as Core;
    match err {
        Core::DatabaseBusy(msg) => VantaError::Lock(msg),
        Core::IoError(io) => VantaError::Io(io.to_string()),
        other => VantaError::Native(other.to_string()),
    }
}

/// Round-trip health probe of the native embedded engine.
///
/// Opens `vantadb` (fjall backend) in a throwaway temp dir, confirms it opens,
/// reports the backend, and closes — never persisting data.
#[tauri::command]
pub fn vanta_health(_app_state: State<AppState>) -> Result<HealthReport, VantaError> {
    let started = SystemTime::now();
    let dir = probe_dir();

    let db = VantaEmbedded::open(&dir).map_err(map_core_error)?;
    // capabilities() is the cheapest proof the engine is fully initialized.
    let caps = db.capabilities();
    db.close().map_err(map_core_error)?;

    // ponytail: leave the probe dir behind rather than churn best-effort cleanup;
    // it's OS temp space. Add dir removal if temp-space pressure becomes real.
    let _dir = dir;

    Ok(HealthReport {
        status: HealthStatus::Healthy,
        backend: "fjall".to_string(),
        latency_ms: started.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0),
        checked_at_ms: started
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        message: Some(format!(
            "native backend upl; persistence={}, vector_search={}",
            caps.persistence, caps.vector_search
        )),
    })
}