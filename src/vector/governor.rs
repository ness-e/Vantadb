//! Dynamic quantization governor that tracks access frequency and transitions
//! cold f32 vectors to SQ8 to save memory.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;

/// Access tracking entry for a single node
#[derive(Debug, Clone)]
struct AccessEntry {
    /// Cumulative access count since last reset
    hits: u64,
    /// Tick when this node was last accessed
    last_access_tick: u64,
}

impl AccessEntry {
    fn new(tick: u64) -> Self {
        Self {
            hits: 1,
            last_access_tick: tick,
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
    /// Minimum hits to consider node "hot" for promotion (default: 5)
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
    /// Node should be quantized to SQ8.
    Quantize,
    /// Node should be restored to f32.
    Promote,
    /// No action needed.
    None,
}

/// Thread-safe quantization governor
pub struct QuantizationGovernor {
    /// Access tracking per node key (u128 node ID — matches HNSW)
    access_map: Mutex<HashMap<u128, AccessEntry>>,
    /// Global tick counter (incremented periodically)
    tick: AtomicU64,
    /// Configuration
    config: QuantizationConfig,
}

impl QuantizationGovernor {
    /// Create a new governor with the given configuration.
    pub fn new(config: QuantizationConfig) -> Self {
        Self {
            access_map: Mutex::new(HashMap::new()),
            tick: AtomicU64::new(0),
            config,
        }
    }

    /// Record access to a node.
    pub fn record_access(&self, node_id: u128) {
        if !self.config.enabled {
            return;
        }
        let current_tick = self.tick.load(Ordering::Relaxed);
        let mut map = self.access_map.lock();
        match map.get_mut(&node_id) {
            Some(entry) => {
                entry.hits += 1;
                entry.last_access_tick = current_tick;
            }
            None => {
                map.insert(node_id, AccessEntry::new(current_tick));
            }
        }
    }

    /// Evaluate what action should be taken for a node, given its current format.
    pub fn evaluate(&self, node_id: u128, is_quantized: bool) -> QuantizationAction {
        if !self.config.enabled {
            return QuantizationAction::None;
        }
        let map = self.access_map.lock();
        let current_tick = self.tick.load(Ordering::Relaxed);

        match map.get(&node_id) {
            Some(entry) => {
                let ticks_since_access = current_tick.saturating_sub(entry.last_access_tick);
                if !is_quantized && ticks_since_access > self.config.cold_threshold_ticks {
                    QuantizationAction::Quantize
                } else if is_quantized && entry.hits >= self.config.hot_threshold {
                    QuantizationAction::Promote
                } else {
                    QuantizationAction::None
                }
            }
            None => {
                if !is_quantized {
                    QuantizationAction::Quantize
                } else {
                    QuantizationAction::None
                }
            }
        }
    }

    /// Increment tick counter (called periodically).
    pub fn tick(&self) {
        self.tick.fetch_add(1, Ordering::Relaxed);
    }

    /// Current tick value.
    pub fn current_tick(&self) -> u64 {
        self.tick.load(Ordering::Relaxed)
    }

    /// Reset tracking for a node (after a quantization action).
    pub fn reset(&self, node_id: u128) {
        let mut map = self.access_map.lock();
        map.remove(&node_id);
    }

    /// Collect all tracked node IDs that need action.
    /// Returns a vec of `(node_id, action)` pairs.
    pub fn collect_actions(
        &self,
        current_format_fn: impl Fn(u128) -> Option<bool>,
    ) -> Vec<(u128, QuantizationAction)> {
        if !self.config.enabled {
            return Vec::new();
        }
        let map = self.access_map.lock();
        let current_tick = self.tick.load(Ordering::Relaxed);
        let mut actions = Vec::new();

        for (&node_id, entry) in map.iter() {
            let ticks_since_access = current_tick.saturating_sub(entry.last_access_tick);
            let is_cold = ticks_since_access > self.config.cold_threshold_ticks;

            if let Some(is_quantized) = current_format_fn(node_id) {
                if !is_quantized && is_cold {
                    actions.push((node_id, QuantizationAction::Quantize));
                } else if is_quantized && entry.hits >= self.config.hot_threshold {
                    actions.push((node_id, QuantizationAction::Promote));
                }
            }
        }

        actions
    }

    /// Quantize an f32 vector to SQ8.
    pub fn quantize_vector(data: &[f32]) -> (Box<[i8]>, f32) {
        crate::vector::quantization::sq8_quantize(data)
    }

    /// Promote an SQ8 vector back to f32.
    pub fn promote_vector(data: &[i8], max_abs: f32) -> Vec<f32> {
        let inv = max_abs / 127.0;
        data.iter().map(|&q| (q as f32) * inv).collect()
    }

    /// Returns the current config.
    pub fn config(&self) -> QuantizationConfig {
        self.config
    }
}

#[cfg(test)]
#[allow(missing_docs)]
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
        gov.record_access(42);
        gov.record_access(42);
        let map = gov.access_map.lock();
        assert_eq!(map.get(&42).unwrap().hits, 2);
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
        assert_eq!(gov.evaluate(99, false), QuantizationAction::Quantize);
    }

    #[test]
    fn test_disabled_governor() {
        let config = QuantizationConfig {
            enabled: false,
            ..Default::default()
        };
        let gov = QuantizationGovernor::new(config);
        assert_eq!(gov.evaluate(99, false), QuantizationAction::None);
    }

    #[test]
    fn test_reset_removes_entry() {
        let gov = QuantizationGovernor::new(QuantizationConfig::default());
        gov.record_access(7);
        gov.reset(7);
        let map = gov.access_map.lock();
        assert!(map.get(&7).is_none());
    }

    #[test]
    fn test_promote_hot_node() {
        let gov = QuantizationGovernor::new(QuantizationConfig {
            hot_threshold: 3,
            ..Default::default()
        });
        for _ in 0..3 {
            gov.record_access(1);
        }
        assert_eq!(gov.evaluate(1, true), QuantizationAction::Promote);
    }

    #[test]
    fn test_f32_node_not_quantized_if_recently_accessed() {
        let gov = QuantizationGovernor::new(QuantizationConfig {
            cold_threshold_ticks: 10,
            ..Default::default()
        });
        gov.record_access(1);
        assert_eq!(gov.evaluate(1, false), QuantizationAction::None);
    }

    #[test]
    fn test_quantize_vector_roundtrip() {
        let original = vec![0.12, 0.88, 0.54, 0.31, -0.22, 0.95, -0.11, 0.47];
        let (packed, scale) = QuantizationGovernor::quantize_vector(&original);

        let sq = crate::node::VectorRepresentations::SQ8(packed, scale);
        let roundtrip = sq.to_f32().unwrap();
        for (a, b) in original.iter().zip(roundtrip.iter()) {
            let err = (a - b).abs();
            assert!(
                err < 0.02,
                "quantization error {err} too high for {a} vs {b}"
            );
        }
    }
}
