#![allow(dead_code)]
use crate::config::VantaConfig;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;

    fn make_gov(memory_limit: u64) -> MemoryGovernor {
        let config = VantaConfig {
            memory_limit: Some(memory_limit),
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        MemoryGovernor::new(&config)
    }

    #[test]
    fn test_memory_governor_allocated_freed() {
        let gov = make_gov(1_000_000);
        assert_eq!(gov.used_bytes(), 0);

        gov.allocated(100);
        assert_eq!(gov.used_bytes(), 100);

        gov.freed(40);
        assert_eq!(gov.used_bytes(), 60);
    }

    #[test]
    fn test_memory_governor_memory_limit() {
        let gov = make_gov(512_000);
        assert_eq!(gov.memory_limit(), 512_000);
    }

    #[test]
    fn test_memory_governor_should_evict() {
        let gov = make_gov(1000);
        // high_water_mark = 1000 * 0.75 = 750
        gov.set_used_bytes(800);
        assert!(gov.should_evict());

        gov.set_used_bytes(700);
        assert!(!gov.should_evict());
    }

    #[test]
    fn test_memory_governor_needs_urgent_eviction() {
        let gov = make_gov(1000);
        gov.set_used_bytes(500);
        assert!(!gov.needs_urgent_eviction());

        gov.set_used_bytes(1500);
        assert!(gov.needs_urgent_eviction());
    }

    #[test]
    fn test_memory_governor_above_low_water() {
        let gov = make_gov(1000);
        // low_water_mark = 1000 * 0.75 * 0.9 = 675
        gov.set_used_bytes(700);
        assert!(gov.above_low_water());

        gov.set_used_bytes(600);
        assert!(!gov.above_low_water());
    }

    #[test]
    fn test_memory_governor_eviction_running() {
        let gov = make_gov(1_000_000);
        assert!(!gov.eviction_running());
        assert!(gov.try_start_eviction());
        assert!(gov.eviction_running());
        // Second attempt should fail
        assert!(!gov.try_start_eviction());
    }

    #[test]
    fn test_memory_governor_finish_eviction() {
        let gov = make_gov(1_000_000);
        gov.try_start_eviction();
        gov.finish_eviction();
        assert!(!gov.eviction_running());
        assert!(gov.try_start_eviction()); // can start again
    }

    #[test]
    fn test_memory_governor_oom_count() {
        let gov = make_gov(1_000_000);
        assert_eq!(gov.oom_count(), 0);
        gov.record_oom();
        assert_eq!(gov.oom_count(), 1);
        gov.record_oom();
        assert_eq!(gov.oom_count(), 2);
    }

    #[test]
    fn test_memory_governor_set_used_bytes() {
        let gov = make_gov(1_000_000);
        gov.set_used_bytes(5000);
        assert_eq!(gov.used_bytes(), 5000);
    }
}
