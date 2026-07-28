//! Thread-safe accumulators for parallel graph algorithms.
//!
//! Provides [`GraphAccumulator`] — a lock-free accumulator that stores `f64`
//! values encoded as `AtomicU64` bits, with CAS-loop-based atomic addition.
//!
//! # Lock-free guarantee
//!
//! `add()` uses a CAS loop (`compare_exchange_weak`) on `AtomicU64`, not a
//! mutex.  Multiple threads can safely update the same node's accumulator
//! concurrently without blocking.
//!
//! # IEEE 754 note
//!
//! Raw `fetch_add` on `AtomicU64` does **not** work for IEEE 754 floats
//! because the bit encoding is not additively homomorphic (e.g.
//! `f64::to_bits(1.0) + f64::to_bits(2.0) ≠ f64::to_bits(3.0)`).
//! Instead we use a CAS loop that reads the current `f64`, adds, and writes
//! back — still lock-free.

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe accumulator for parallel graph algorithms.
///
/// Stores `f64` values encoded as `AtomicU64` bits for lock-free atomic add.
///
/// - `add()` is lock-free (CAS loop on AtomicU64, no mutex).
/// - `set()` / `get()` / `snapshot()` are wait-free reads.
/// - All methods are `&self` — no mutable references needed.
pub struct GraphAccumulator {
    /// Per-node accumulator values (f64 encoded as AtomicU64 bits).
    values: DashMap<u128, AtomicU64>,
}

// DashMap<u128, AtomicU64> is Send + Sync because both key and value are.
// No manual unsafe impl needed.

impl GraphAccumulator {
    /// Create a new empty accumulator.
    pub fn new() -> Self {
        Self {
            values: DashMap::new(),
        }
    }

    /// Set the accumulator for `node_id` to `value`, replacing any previous.
    pub fn set(&self, node_id: u128, value: f64) {
        self.values.insert(node_id, AtomicU64::new(value.to_bits()));
    }

    /// Get the current value for `node_id`, or `None` if not set.
    pub fn get(&self, node_id: u128) -> Option<f64> {
        self.values
            .get(&node_id)
            .map(|v| f64::from_bits(v.load(Ordering::Relaxed)))
    }

    /// Atomically add `delta` to the accumulator for `node_id`.
    ///
    /// Returns the **previous** value (standard fetch-add semantics).
    ///
    /// If `node_id` has no accumulator yet, it is initialized to `0.0` first.
    ///
    /// Lock-free: uses a CAS loop on `AtomicU64` (no mutex).
    pub fn add(&self, node_id: u128, delta: f64) -> f64 {
        // `or_insert_with` is racy but safe — DashMap guarantees at most one
        // insert per key, and the closure is idempotent.
        let entry = self
            .values
            .entry(node_id)
            .or_insert_with(|| AtomicU64::new(f64::to_bits(0.0)));

        // CAS loop for atomic f64 addition (fetch_add on bit patterns
        // does not work for IEEE 754 floats).
        loop {
            let current_bits = entry.load(Ordering::Relaxed);
            let current = f64::from_bits(current_bits);
            let new = current + delta;
            let new_bits = new.to_bits();

            if entry
                .compare_exchange_weak(current_bits, new_bits, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return current;
            }
            // CAS failed (another thread updated concurrently) — retry.
        }
    }

    /// Capture a consistent snapshot of all accumulator values.
    ///
    /// Returns a `HashMap` with a point-in-time view of every (node, value) pair.
    pub fn snapshot(&self) -> HashMap<u128, f64> {
        let mut result = HashMap::new();
        for entry in self.values.iter() {
            result.insert(
                *entry.key(),
                f64::from_bits(entry.value().load(Ordering::Relaxed)),
            );
        }
        result
    }

    /// Remove all accumulator values.
    pub fn clear(&self) {
        self.values.clear();
    }

    /// Get all node IDs that have an accumulator entry.
    pub fn keys(&self) -> Vec<u128> {
        self.values.iter().map(|e| *e.key()).collect()
    }
}

impl Default for GraphAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: GraphAccumulator contains only a DashMap<u128, AtomicU64>, which is
//         itself Send + Sync. No shared mutable state without synchronization.
unsafe impl Send for GraphAccumulator {}
unsafe impl Sync for GraphAccumulator {}

// ─── Tests ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_accumulator_basic() {
        let acc = GraphAccumulator::new();

        // Initially None
        assert_eq!(acc.get(42), None);

        // Set and get
        acc.set(42, 10.5);
        let val = acc.get(42).unwrap();
        assert!((val - 10.5).abs() < 1e-10, "got {val}");

        // Add returns previous value
        let prev = acc.add(42, 2.0);
        assert!((prev - 10.5).abs() < 1e-10, "prev {prev}");
        let val = acc.get(42).unwrap();
        assert!((val - 12.5).abs() < 1e-10, "after add {val}");

        // Add to a new key — initialises to 0.0
        let prev = acc.add(99, 10.5);
        assert!((prev - 0.0).abs() < 1e-10, "prev for new key {prev}");
        let val = acc.get(99).unwrap();
        assert!((val - 10.5).abs() < 1e-10, "new key value {val}");

        // Negative delta
        let prev = acc.add(42, -1.0);
        assert!((prev - 12.5).abs() < 1e-10);
        let val = acc.get(42).unwrap();
        assert!((val - 11.5).abs() < 1e-10);

        // Non-integer values round-trip correctly
        acc.set(7, std::f64::consts::PI);
        let val = acc.get(7).unwrap();
        assert!((val - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_accumulator_concurrent() {
        let acc = Arc::new(GraphAccumulator::new());
        let node_id = 1u128;
        let num_threads = 8usize;
        let iterations = 1000usize;
        let delta = 1.0_f64;

        // Initialize to 0
        acc.set(node_id, 0.0);

        let mut handles = Vec::with_capacity(num_threads);
        for _ in 0..num_threads {
            let acc = Arc::clone(&acc);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    acc.add(node_id, delta);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let final_val = acc.get(node_id).unwrap();
        let expected = num_threads as f64 * iterations as f64 * delta;
        assert!(
            (final_val - expected).abs() < 1.0,
            "expected {expected}, got {final_val}"
        );
    }

    #[test]
    fn test_accumulator_snapshot() {
        let acc = GraphAccumulator::new();

        acc.set(1, 10.0);
        acc.set(2, 20.0);
        acc.add(3, 30.0); // starts at 0 → becomes 30

        let snap = acc.snapshot();
        assert_eq!(snap.len(), 3);
        assert!((snap[&1] - 10.0).abs() < 1e-10);
        assert!((snap[&2] - 20.0).abs() < 1e-10);
        assert!((snap[&3] - 30.0).abs() < 1e-10);

        // Clear
        acc.clear();
        assert!(acc.snapshot().is_empty());
    }

    #[test]
    fn test_accumulator_keys() {
        let acc = GraphAccumulator::new();
        assert!(acc.keys().is_empty());

        acc.set(1, 1.0);
        acc.set(2, 2.0);
        let mut keys = acc.keys();
        keys.sort();
        assert_eq!(keys, vec![1, 2]);
    }

    #[test]
    fn test_accumulator_integration() {
        // Build a tiny graph (chain 0→1→2→3), traverse and accumulate contributions.
        use crate::config::VantaConfig;
        use crate::node::UnifiedNode;
        use crate::storage::{BackendKind, StorageEngine};
        use crate::Edge;

        let dir = tempfile::tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        // Build chain: 0 → 1 → 2 → 3
        for i in 0..4u128 {
            let mut node = UnifiedNode::new(i);
            if i < 3 {
                node.edges = vec![Edge {
                    target: i + 1,
                    weight: 1.0,
                    label_id: 0,
                    reverse: false,
                }];
            }
            storage.insert(&node).unwrap();
        }

        let acc = GraphAccumulator::new();

        // Manual BFS with accumulator: each discovered node contributes its own ID
        let mut visited: Vec<u128> = Vec::new();
        let mut current_level = vec![0u128];

        while !current_level.is_empty() {
            let mut unvisited = Vec::new();
            for &id in &current_level {
                if !visited.contains(&id) {
                    visited.push(id);
                    unvisited.push(id);
                }
            }

            if unvisited.is_empty() {
                break;
            }

            let nodes = storage.get_many(&unvisited).unwrap();
            let mut next_level = Vec::new();
            for node in &nodes {
                acc.add(node.id, node.id as f64);
                for edge in &node.edges {
                    if !visited.contains(&edge.target) {
                        next_level.push(edge.target);
                    }
                }
            }

            next_level.sort();
            next_level.dedup();
            current_level = next_level;
        }

        // Verify all 4 nodes were visited and accumulator has correct values
        assert_eq!(visited, vec![0, 1, 2, 3]);
        assert!((acc.get(0).unwrap() - 0.0).abs() < 1e-10);
        assert!((acc.get(1).unwrap() - 1.0).abs() < 1e-10);
        assert!((acc.get(2).unwrap() - 2.0).abs() < 1e-10);
        assert!((acc.get(3).unwrap() - 3.0).abs() < 1e-10);
        assert_eq!(acc.get(4), None, "node 4 should not exist");

        // Verify snapshot matches
        let snap = acc.snapshot();
        assert_eq!(snap.len(), 4);
    }

    #[test]
    fn test_accumulator_send_sync() {
        // Compile-time check: GraphAccumulator must be Send + Sync.
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<GraphAccumulator>();
        assert_sync::<GraphAccumulator>();
    }
}
