use crate::config::VantaConfig;
use crate::error::Result;
use crate::storage::EvictionReport;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use web_time::Instant;

pub(crate) struct MemoryGovernor {
    memory_limit: u64,
    target_ratio: f64,
    low_water_mark: u64,
    high_water_mark: u64,
    used_bytes: AtomicU64,
    eviction_running: AtomicBool,
    last_eviction_ms: AtomicU64,
    oom_count: AtomicU64,
}

impl MemoryGovernor {
    pub fn new(config: &VantaConfig) -> Self {
        let caps = crate::hardware::HardwareCapabilities::global();
        let memory_limit = config.memory_limit.unwrap_or(caps.total_memory);
        let target_ratio = 0.75;
        Self {
            memory_limit,
            target_ratio,
            low_water_mark: (memory_limit as f64 * target_ratio * 0.9) as u64,
            high_water_mark: (memory_limit as f64 * target_ratio) as u64,
            used_bytes: AtomicU64::new(0),
            eviction_running: AtomicBool::new(false),
            last_eviction_ms: AtomicU64::new(0),
            oom_count: AtomicU64::new(0),
        }
    }

    pub fn allocated(&self, bytes: u64) {
        self.used_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn freed(&self, bytes: u64) {
        self.used_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    pub fn used_bytes(&self) -> u64 {
        self.used_bytes.load(Ordering::Relaxed)
    }

    pub fn memory_limit(&self) -> u64 {
        self.memory_limit
    }

    pub fn should_evict(&self) -> bool {
        self.used_bytes.load(Ordering::Relaxed) > self.high_water_mark
    }

    pub fn needs_urgent_eviction(&self) -> bool {
        self.used_bytes.load(Ordering::Relaxed) > self.memory_limit
    }

    pub fn above_low_water(&self) -> bool {
        self.used_bytes.load(Ordering::Relaxed) > self.low_water_mark
    }

    pub fn try_start_eviction(&self) -> bool {
        self.eviction_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn finish_eviction(&self) {
        self.eviction_running.store(false, Ordering::Release);
        self.last_eviction_ms.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    pub fn oom_count(&self) -> u64 {
        self.oom_count.load(Ordering::Relaxed)
    }

    pub fn record_oom(&self) {
        self.oom_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn eviction_running(&self) -> bool {
        self.eviction_running.load(Ordering::Acquire)
    }

    pub fn set_used_bytes(&self, bytes: u64) {
        self.used_bytes.store(bytes, Ordering::Relaxed);
    }
}
