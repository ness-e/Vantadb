/// Dynamic quantization governor
///
/// Tracks node access frequency and automatically transitions
/// cold f32 vectors to SQ8 to save memory, promoting back
/// to f32 when access frequency increases.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use web_time::Instant;

/// Access tracking entry for a single node
#[derive(Debug, Clone)]
struct AccessEntry {
    /// Monotonic access counter (batched)
    hits: u64,
    /// Last access timestamp
    last_access: Instant,
}

impl AccessEntry {
    fn new() -> Self {
        Self {
            hits: 1,
            last_access: Instant::now(),
        }
    }
}

/// Governor configuration
#[derive(Debug, Clone, Copy)]
pub struct QuantizationConfig {
    /// Number of ticks without access before considering node "cold" (default: 100)
    pub cold_threshold_ticks: u64,
    /// Tick interval in milliseconds (default: 1000ms = 1s)
    pub tick_interval_ms: u64,
    /// Minimum hits per tick to consider node "hot" for promotion (default: 5)
    pub hot_threshold: u64,
    /// Enable/disable auto-quantization (default: true)
    pub enabled: bool,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            cold_threshold_ticks: 100,
            tick_interval_ms: 1000,
            hot_threshold: 5,
            enabled: true,
        }
    }
}

/// Result of a quantization decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationAction {
    /// Node should be quantized to SQ8
    Quantize,
    /// Node should be restored to f32
    Promote,
    /// No action needed
    None,
}

/// Thread-safe quantization governor
pub struct QuantizationGovernor {
    /// Access tracking per node key
    access_map: Mutex<HashMap<String, AccessEntry>>,
    /// Global tick counter (incremented periodically)
    tick: AtomicU64,
    /// Configuration
    config: QuantizationConfig,
}

impl QuantizationGovernor {
    pub fn new(config: QuantizationConfig) -> Self {
        Self {
            access_map: Mutex::new(HashMap::new()),
            tick: AtomicU64::new(0),
            config,
        }
    }

    /// Record access to a node
    pub fn record_access(&self, key: &str) {
        if !self.config.enabled {
            return;
        }
        let mut map = self.access_map.lock().unwrap();
        match map.get_mut(key) {
            Some(entry) => {
                entry.hits += 1;
                entry.last_access = Instant::now();
            }
            None => {
                map.insert(key.to_string(), AccessEntry::new());
            }
        }
    }

    /// Evaluate what action should be taken for a node
    pub fn evaluate(&self, key: &str) -> QuantizationAction {
        if !self.config.enabled {
            return QuantizationAction::None;
        }
        let map = self.access_map.lock().unwrap();
        let current_tick = self.tick.load(Ordering::Relaxed);

        match map.get(key) {
            Some(entry) => {
                let ticks_since_access = current_tick.saturating_sub(entry.hits);
                if ticks_since_access > self.config.cold_threshold_ticks {
                    QuantizationAction::Quantize
                } else if entry.hits >= self.config.hot_threshold {
                    QuantizationAction::Promote
                } else {
                    QuantizationAction::None
                }
            }
            None => QuantizationAction::Quantize,
        }
    }

    /// Increment tick counter (called periodically)
    pub fn tick(&self) {
        self.tick.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset tracking for a key (after a quantization action)
    pub fn reset(&self, key: &str) {
        let mut map = self.access_map.lock().unwrap();
        map.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_governor() {
        let gov = QuantizationGovernor::new(QuantizationConfig::default());
        assert!(gov.config.enabled);
    }

    #[test]
    fn test_record_access() {
        let gov = QuantizationGovernor::new(QuantizationConfig::default());
        gov.record_access("test-key");
        gov.record_access("test-key");
        let map = gov.access_map.lock().unwrap();
        assert_eq!(map.get("test-key").unwrap().hits, 2);
    }

    #[test]
    fn test_cold_node_quantized() {
        let config = QuantizationConfig {
            cold_threshold_ticks: 1,
            tick_interval_ms: 100,
            hot_threshold: 5,
            enabled: true,
        };
        let gov = QuantizationGovernor::new(config);
        gov.tick();
        gov.tick();
        assert_eq!(
            gov.evaluate("never-accessed"),
            QuantizationAction::Quantize
        );
    }

    #[test]
    fn test_disabled_governor() {
        let config = QuantizationConfig {
            enabled: false,
            ..Default::default()
        };
        let gov = QuantizationGovernor::new(config);
        assert_eq!(gov.evaluate("any-key"), QuantizationAction::None);
    }

    #[test]
    fn test_reset_removes_entry() {
        let gov = QuantizationGovernor::new(QuantizationConfig::default());
        gov.record_access("key");
        gov.reset("key");
        let map = gov.access_map.lock().unwrap();
        assert!(map.get("key").is_none());
    }
}
