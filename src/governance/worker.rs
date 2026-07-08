use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use web_time::{Duration, Instant};

use super::admission::AdmissionFilter;
use super::conflict::ConflictResolver;
use super::consistency::ConsistencyBuffer;

const DEFAULT_INTERVAL_MS: u64 = 10_000;
const INACTIVITY_THRESHOLD_MS: u64 = 5_000;
const CONFLICT_GC_AGE_NANOS: u128 = 3_600_000_000_000; // 1 hour

/// Health status of the maintenance worker.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub last_maintenance: Option<Instant>,
    pub cycles_completed: u64,
    pub bloom_insert_count: u64,
    pub buffer_pending_count: usize,
    pub conflict_log_size: usize,
    pub healthy: bool,
}

impl HealthStatus {
    fn new() -> Self {
        Self {
            last_maintenance: None,
            cycles_completed: 0,
            bloom_insert_count: 0,
            buffer_pending_count: 0,
            conflict_log_size: 0,
            healthy: true,
        }
    }
}

/// Background maintenance worker that periodically processes bloom reset checks,
/// conflict garbage collection, and buffer flushes.
pub struct MaintenanceWorker {
    admission: Option<Arc<AdmissionFilter>>,
    conflict: Option<Arc<ConflictResolver>>,
    buffer: Option<Arc<ConsistencyBuffer<String>>>,
    interval: Duration,
    running: Arc<AtomicBool>,
    handle: RwLock<Option<JoinHandle<()>>>,
    health: Arc<RwLock<HealthStatus>>,
    last_activity: Arc<RwLock<Instant>>,
}

impl MaintenanceWorker {
    pub fn new(
        admission: Option<Arc<AdmissionFilter>>,
        conflict: Option<Arc<ConflictResolver>>,
        buffer: Option<Arc<ConsistencyBuffer<String>>>,
    ) -> Self {
        Self {
            admission,
            conflict,
            buffer,
            interval: Duration::from_millis(DEFAULT_INTERVAL_MS),
            running: Arc::new(AtomicBool::new(false)),
            handle: RwLock::new(None),
            health: Arc::new(RwLock::new(HealthStatus::new())),
            last_activity: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Start the background maintenance thread.
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let interval = self.interval;
        let inactivity = Duration::from_millis(INACTIVITY_THRESHOLD_MS);
        let health = self.health.clone();
        let last_activity = self.last_activity.clone();

        let admission = self.admission.clone();
        let conflict = self.conflict.clone();
        let buffer = self.buffer.clone();

        let handle = thread::spawn(move || loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            let now = Instant::now();
            let inactive = {
                let last = *last_activity.read().expect("RwLock poisoned");
                now.duration_since(last) >= inactivity
            };

            if inactive {
                if let Err(e) = Self::run_maintenance_cycle(&admission, &conflict, &buffer, &health)
                {
                    tracing::warn!("Maintenance cycle error: {}", e);
                }
            }

            thread::sleep(interval);
        });

        *self.handle.write().expect("RwLock poisoned") = Some(handle);
    }

    /// Stop the background maintenance thread.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.write().expect("RwLock poisoned").take() {
            let _ = handle.join();
        }
    }

    /// Manually trigger a maintenance cycle.
    pub fn trigger_maintenance(&self) -> Result<(), String> {
        Self::run_maintenance_cycle(&self.admission, &self.conflict, &self.buffer, &self.health)
    }

    /// Mark activity (resets inactivity timer).
    pub fn mark_activity(&self) {
        if let Ok(mut last) = self.last_activity.write() {
            *last = Instant::now();
        }
    }

    /// Get current health status.
    pub fn health(&self) -> HealthStatus {
        self.health.read().expect("RwLock poisoned").clone()
    }

    fn run_maintenance_cycle(
        admission: &Option<Arc<AdmissionFilter>>,
        conflict: &Option<Arc<ConflictResolver>>,
        buffer: &Option<Arc<ConsistencyBuffer<String>>>,
        health: &Arc<RwLock<HealthStatus>>,
    ) -> Result<(), String> {
        // 1. Bloom filter reset check
        if let Some(filter) = admission {
            let fp_rate = filter.estimated_fp_rate();
            let threshold = filter.config().reset_threshold;
            if fp_rate > threshold * 0.8 {
                // approaching threshold — log warning
                tracing::warn!(
                    "Bloom filter approaching reset threshold: {:.4} (threshold {:.4})",
                    fp_rate,
                    threshold
                );
            }
            if fp_rate > threshold {
                filter.reset_filter();
                tracing::info!("Bloom filter auto-reset at FP rate {:.4}", fp_rate);
            }
        }

        // 2. Conflict log GC
        if let Some(resolver) = conflict {
            let removed = resolver.gc_conflict_log(CONFLICT_GC_AGE_NANOS);
            if removed > 0 {
                tracing::debug!("GC'd {} conflict log entries", removed);
            }
        }

        // 3. Buffer flush
        if let Some(buf) = buffer {
            let expired = buf.expire_entries();
            if !expired.is_empty() {
                tracing::debug!("Expired {} stale buffer entries", expired.len());
            }
            if buf.should_flush() {
                let result = buf.flush_all();
                tracing::info!(
                    "Buffer flush: {} accepted, {} rejected, {} tombstones",
                    result.accepted.len(),
                    result.rejected.len(),
                    result.tombstones.len(),
                );
            }
        }

        // Update health
        if let Ok(mut h) = health.write() {
            h.last_maintenance = Some(Instant::now());
            h.cycles_completed += 1;
            if let Some(filter) = admission {
                h.bloom_insert_count = filter.insert_count();
            }
            if let Some(buf) = buffer {
                h.buffer_pending_count = buf.len();
            }
            if let Some(resolver) = conflict {
                h.conflict_log_size = resolver.conflict_log().len();
            }
            h.healthy = true;
        }

        Ok(())
    }
}

impl Drop for MaintenanceWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::admission::{AdmissionConfig, AdmissionFilter};

    #[test]
    fn test_health_status_after_trigger() {
        let admission = Arc::new(AdmissionFilter::new(AdmissionConfig::default()));
        let conflict = Arc::new(ConflictResolver::new(0.5, 100));
        let buffer = Arc::new(ConsistencyBuffer::new(
            100,
            Duration::from_secs(10),
            Duration::from_secs(60),
            50,
        ));
        let worker = MaintenanceWorker::new(Some(admission), Some(conflict), Some(buffer));

        worker.trigger_maintenance().unwrap();
        let health = worker.health();
        assert!(health.healthy);
        assert!(health.cycles_completed >= 1);
    }

    #[test]
    fn test_start_stop() {
        let worker = MaintenanceWorker::new(None, None, None);
        worker.start();
        thread::sleep(Duration::from_millis(50));
        worker.stop();
    }

    #[test]
    fn test_mark_activity() {
        let worker = MaintenanceWorker::new(None, None, None);
        worker.mark_activity();
    }
}
