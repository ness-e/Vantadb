//! Cache layer: hot-node volatile cache, BM25 text caches, cardinality stats, warmer.
//!
//! [`CacheLayer`] owns the five pure-cache fields extracted from
//! `StorageEngine` (C2S3b, SRP): volatile LRU-ish map, the two BM25
//! caches, the cardinality map and the predictive
//! [`CacheWarmer`](crate::cache_warmer::CacheWarmer).
//! No I/O, no WAL, no backend access — unit-testable without storage.
//! Delegation is 1:1, same lock discipline the engine used inline
//! (`try_write` fast path with `read` fallback on contention, ERR-036).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::node::UnifiedNode;

/// Pure cache state: volatile map + text caches + cardinality + warmer.
///
/// All methods are thin state operations with the same lock discipline the
/// engine used inline. Sealed `pub(crate)` (precedente C2S1/S3-txn, C-SEALED).
pub(crate) struct CacheLayer {
    /// Volatile LRU cache for hot (frequently accessed) nodes.
    pub(crate) volatile: RwLock<HashMap<u128, UnifiedNode>>,
    /// Counts write-path invalidations of [`Self::volatile`] (`FIND-222`).
    ///
    /// `prefetch_related` materializes a node, drops every lock, and only then
    /// inserts it — and the write path's own `remove` is what makes that insert
    /// land on a `Vacant` entry. The prefetch snapshots this counter before
    /// materializing and re-reads it under the cache write lock: a bump in
    /// between means a writer invalidated the cache after the node was read, so
    /// the insert is skipped. Precision is deliberately coarse (one global
    /// counter, not a per-node epoch): a skipped prefetch costs a cache miss and
    /// never correctness. `ponytail: global counter — per-node epochs only if a
    /// benchmark shows the false-negative rate hurts`.
    invalidation_generation: AtomicU64,
    /// In-memory cache for BM25 term stats to avoid redundant I/O during ingestion.
    pub(crate) text_stats: RwLock<HashMap<(String, String), crate::text_index::TextTermStats>>,
    /// In-memory cache for BM25 namespace stats.
    pub(crate) text_ns: RwLock<HashMap<String, crate::text_index::TextNamespaceStats>>,
    /// Lightweight cardinality statistics for query optimization.
    pub(crate) cardinality_stats: RwLock<HashMap<String, HashMap<String, usize>>>,
    /// Predictive cache warmer for co-access tracking and prefetch (OLD-20).
    pub(crate) warmer: crate::cache_warmer::CacheWarmer,
}

impl CacheLayer {
    /// Build an empty layer around a pre-built cardinality map.
    ///
    /// The map is built by `StorageEngine::initialize_cardinality_stats`
    /// (needs the backend), so the caller passes it in — the manager
    /// itself performs no I/O (Spec decisión 7).
    pub(crate) fn new(cardinality_stats: HashMap<String, HashMap<String, usize>>) -> Self {
        Self {
            volatile: RwLock::new(HashMap::new()),
            invalidation_generation: AtomicU64::new(0),
            text_stats: RwLock::new(HashMap::new()),
            text_ns: RwLock::new(HashMap::new()),
            cardinality_stats: RwLock::new(cardinality_stats),
            warmer: crate::cache_warmer::CacheWarmer::new(),
        }
    }

    /// Volatile-cache probe (ERR-036: never block the read hot path).
    /// `None` = miss; `Some(Some)` = hit; `Some(None)` = tombstoned hit.
    ///
    /// Moved 1:1 from `StorageEngine::lookup_volatile` (get.rs):
    /// same `try_write` fast path with `read` fallback, same
    /// hits/last_accessed bookkeeping on the write path only.
    pub(crate) fn lookup_volatile(&self, id: u128) -> Option<Option<UnifiedNode>> {
        match self.volatile.try_write() {
            Some(mut guard) => match guard.get_mut(&id) {
                Some(n) if n.flags.is_set(crate::node::NodeFlags::TOMBSTONE) => Some(None),
                Some(n) => {
                    n.hits += 1;
                    n.last_accessed = now_ms_epoch_millis();
                    Some(Some(n.clone()))
                }
                None => None,
            },
            None => match self.volatile.read().get(&id) {
                Some(n) if n.flags.is_set(crate::node::NodeFlags::TOMBSTONE) => Some(None),
                Some(n) => Some(Some(n.clone())),
                None => None,
            },
        }
    }

    /// Current invalidation generation (`FIND-222`). Sampled by the prefetch
    /// **before** it materializes a warm node and re-read under the cache write
    /// lock before inserting it.
    pub(crate) fn invalidation_generation(&self) -> u64 {
        self.invalidation_generation.load(Ordering::SeqCst)
    }

    /// Record that the cache is about to lose (or did lose) an entry the write
    /// path just changed. Callers that already hold the `volatile` write guard
    /// call this immediately before their `remove`, so the bump and the removal
    /// are one critical section for any concurrent prefetch.
    pub(crate) fn note_invalidation(&self) {
        self.invalidation_generation.fetch_add(1, Ordering::SeqCst);
    }

    /// Bump + remove in one critical section: the invalidation must be the
    /// **last** step of the writer, after the durable mutation that makes the
    /// old payload unreadable (`FIND-222`).
    pub(crate) fn invalidate(&self, id: u128) {
        let mut guard = self.volatile.write();
        self.note_invalidation();
        guard.remove(&id);
    }
}

/// Millis since epoch (single clock read for access bookkeeping).
///
/// Private duplicate of `get.rs::now_ms_epoch_millis` to keep this module
/// dependency-free (no sibling imports; manager puro estado).
fn now_ms_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node(id: u128) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
        node
    }

    #[test]
    fn cache_layer_new_starts_empty() {
        let layer = CacheLayer::new(HashMap::new());
        assert!(layer.volatile.read().is_empty());
        assert!(layer.text_stats.read().is_empty());
        assert!(layer.text_ns.read().is_empty());
        assert!(layer.cardinality_stats.read().is_empty());
        assert!(layer.lookup_volatile(1).is_none());
    }

    #[test]
    fn cache_layer_new_keeps_cardinality_map() {
        let mut stats = HashMap::new();
        stats.insert("f".to_string(), HashMap::from([("v".to_string(), 2)]));
        let layer = CacheLayer::new(stats);
        assert_eq!(
            layer.cardinality_stats.read().get("f").unwrap().get("v"),
            Some(&2)
        );
    }

    #[test]
    fn cache_layer_lookup_hit_and_miss() {
        let layer = CacheLayer::new(HashMap::new());
        assert!(layer.lookup_volatile(7).is_none());
        layer.volatile.write().insert(7, sample_node(7));
        assert!(matches!(layer.lookup_volatile(7), Some(Some(ref n)) if n.id == 7));
    }

    #[test]
    fn cache_layer_lookup_tombstone_maps_to_none() {
        let layer = CacheLayer::new(HashMap::new());
        let mut node = sample_node(9);
        node.flags.set(crate::node::NodeFlags::TOMBSTONE);
        layer.volatile.write().insert(9, node);
        assert!(matches!(layer.lookup_volatile(9), Some(None)));
    }
}
