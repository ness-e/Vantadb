//! Background self-healing maintenance service for VantaDB server (FIND-384).
//!
//! Monitores client activity and automatically runs background vacuum and
//! index consolidation during quiet periods to preserve HNSW graph cache locality
//! and reclaim space without manual operator intervention.

use crate::sdk::Embedded;
use crate::server::state::BackgroundService;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use web_time::{SystemTime, UNIX_EPOCH};

/// Global or per-server activity tracker.
#[derive(Debug)]
pub struct ActivityTracker {
    last_activity_secs: AtomicU64,
}

impl Default for ActivityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivityTracker {
    /// Create a new tracker initialized to current timestamp.
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            last_activity_secs: AtomicU64::new(now),
        }
    }

    /// Record client activity (e.g. read, write, query).
    pub fn record_activity(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_activity_secs.store(now, Ordering::Relaxed);
    }

    /// Return elapsed idle seconds since the last recorded activity.
    pub fn idle_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let last = self.last_activity_secs.load(Ordering::Relaxed);
        now.saturating_sub(last)
    }

    /// Reset activity timestamp to a specific epoch seconds value (for testing).
    #[cfg(test)]
    pub fn set_last_activity_for_test(&self, secs: u64) {
        self.last_activity_secs.store(secs, Ordering::Relaxed);
    }
}

/// Handle for the background self-healing maintenance service.
pub struct SelfHealingService {
    shutdown: watch::Sender<bool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl SelfHealingService {
    /// Gracefully stop the background maintenance loop and join its task.
    pub async fn shutdown(mut self) {
        let _ = self.shutdown.send(true);
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

impl BackgroundService for SelfHealingService {
    fn shutdown(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            SelfHealingService::shutdown(*self).await;
        })
    }
}

impl Drop for SelfHealingService {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

/// Spawn the background self-healing service.
///
/// Every `check_interval`, checks if `idle_seconds >= quiet_window_secs`.
/// When quiet, executes storage vacuum and index consolidation on a blocking thread.
pub fn spawn_self_healing_service(
    db: Embedded,
    activity_tracker: Arc<ActivityTracker>,
    quiet_window_secs: u64,
) -> SelfHealingService {
    let quiet_window_secs = std::env::var("VANTADB_SELF_HEALING_IDLE_SECS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(quiet_window_secs);

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    // Bounded check interval: check at least every 30s or half the quiet window (min 1s)
    let check_interval = Duration::from_secs(30.min(quiet_window_secs / 2).max(1));
    let has_performed_for_idle = Arc::new(AtomicBool::new(false));

    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(check_interval);
        // Skip first immediate tick
        interval.tick().await;

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let idle = activity_tracker.idle_seconds();
                    if idle >= quiet_window_secs {
                        if !has_performed_for_idle.load(Ordering::Relaxed) {
                            tracing::info!(
                                quiet_secs = idle,
                                threshold = quiet_window_secs,
                                "Starting automated self-healing maintenance in low-activity window"
                            );
                            let db_clone = db.clone();
                            let maintenance_res = tokio::task::spawn_blocking(move || {
                                db_clone.vacuum()
                            }).await;

                            match maintenance_res {
                                Ok(Ok(report)) => {
                                    tracing::info!(
                                        scanned = report.scanned_nodes,
                                        removed = report.removed_nodes,
                                        reclaimed_bytes = report.reclaimed_bytes,
                                        duration_ms = report.duration_ms,
                                        "Self-healing vacuum completed successfully"
                                    );
                                }
                                Ok(Err(e)) => {
                                    tracing::warn!(error = %e, "Self-healing vacuum encountered error");
                                }
                                Err(join_err) => {
                                    tracing::error!(error = %join_err, "Self-healing worker task join failed");
                                }
                            }

                            // Cooperative cancellation check: if user requests resumed during vacuum,
                            // defer index rebuild to prioritize live query latency.
                            if activity_tracker.idle_seconds() < quiet_window_secs {
                                tracing::info!(
                                    "User traffic resumed during vacuum; deferring index rebuild to next quiet window"
                                );
                            } else {
                                let db_rebuild = db.clone();
                                let rebuild_res = tokio::task::spawn_blocking(move || {
                                    db_rebuild.rebuild_index()
                                }).await;

                                match rebuild_res {
                                    Ok(Ok(report)) => {
                                        tracing::info!(
                                            duration_ms = report.duration_ms,
                                            "Self-healing index rebuild completed successfully"
                                        );
                                    }
                                    Ok(Err(e)) => {
                                        tracing::warn!(error = %e, "Self-healing index rebuild encountered error");
                                    }
                                    Err(join_err) => {
                                        tracing::error!(error = %join_err, "Self-healing index rebuild task join failed");
                                    }
                                }
                            }

                            has_performed_for_idle.store(true, Ordering::Relaxed);
                        }
                    } else {
                        // Reset latch when new activity occurs
                        has_performed_for_idle.store(false, Ordering::Relaxed);
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::debug!("Self-healing service received shutdown signal");
                        break;
                    }
                }
            }
        }
    });

    SelfHealingService {
        shutdown: shutdown_tx,
        handle: Some(handle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_tracker_idle_and_record() {
        let tracker = ActivityTracker::new();
        assert_eq!(tracker.idle_seconds(), 0);

        // Simulate 100 seconds ago
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        tracker.set_last_activity_for_test(now - 100);
        assert!(tracker.idle_seconds() >= 100);

        // Recording activity resets idle seconds
        tracker.record_activity();
        assert!(tracker.idle_seconds() < 2);
    }
}
