//! Predictive cache warming — co-access tracking and HNSW top-layer prefetch.
//!
//! Tracks which node IDs are frequently accessed together so that when one
//! is fetched, its co-accessed partners are proactively loaded into the
//! volatile cache (prefetch). Also provides HNSW top-layer node discovery
//! for warming the graph entry point and its top-layer neighbors.
//!
//! # Co-access table
//!
//! `co_access[A][B]` = how many times B has been observed in the same access
//! batch as A (via `record_co_access`). When `get(A)` triggers a cache miss,
//! `suggest_warm_ids(A)` returns the B's whose count >= `min_accesses`,
//! sorted by descending frequency.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use parking_lot::RwLock;

/// Upper bound on distinct (A,B) co-access pairs retained by the warmer
/// (~64-90 bytes/pair in the nested HashMap). Prevents the O(n²) pair table
/// from exhausting the heap under long, high-cardinality workloads.
///
/// AUDIT-04: the 10K/128d/1000q hybrid benchmark grew `co_access` at
/// ~2 MB/query (each `get_many` records every candidate pair), reaching
/// ~2.5 GB and aborting the process with 0xC0000409 ("memory allocation of
/// 270352 bytes failed") once the heap was exhausted. Once the table hits
/// this cap it stops learning NEW pairs (existing pairs keep updating), so
/// prefetch behavior for already-tracked hot pairs is preserved.
pub const MAX_CO_ACCESS_PAIRS: usize = 1_000_000;

/// Tracks co-access patterns and predicts which nodes to prefetch.
pub(crate) struct CacheWarmer {
    /// co_access[A][B] = number of times B was accessed together with A.
    co_access: RwLock<HashMap<u128, HashMap<u128, u32>>>,
    /// Minimum co-access count before prefetch is triggered.
    min_accesses: u32,
    /// Maximum number of nodes to prefetch in a single trigger.
    max_prefetch: usize,
    /// Maximum number of distinct pairs before new-pair learning stops.
    max_pairs: usize,
    /// Total number of co-access recording events.
    total_events: AtomicU64,
    /// Number of times a prefetched node was actually accessed later.
    prefetch_hits: AtomicU64,
    /// Number of distinct (A,B) pairs currently in the table.
    pair_count: AtomicUsize,
    /// Set once `pair_count` reaches `max_pairs`; new pairs are no longer
    /// learned (monotonic — decay shrinking the table does not re-enable it).
    saturated: AtomicBool,
}

/// Snapshot of cache warmer metrics for telemetry.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(not(feature = "prometheus"), allow(dead_code))]
pub(crate) struct CacheWarmerMetrics {
    /// Number of distinct nodes in the co-access table.
    pub tracked_nodes: usize,
    /// Total number of co-access pairs across all tracked nodes.
    pub total_pairs: usize,
    /// Total co-access recording events.
    pub total_events: u64,
    /// Number of successful prefetch predictions (prefetch → subsequent access).
    pub prefetch_hits: u64,
}

impl CacheWarmer {
    /// Create a new cache warmer with default thresholds.
    pub fn new() -> Self {
        Self::with_config(3, 8)
    }

    /// Create a cache warmer with explicit thresholds.
    ///
    /// * `min_accesses` — minimum co-access frequency before suggesting a prefetch.
    /// * `max_prefetch` — maximum nodes to prefetch per trigger.
    pub fn with_config(min_accesses: u32, max_prefetch: usize) -> Self {
        Self::with_config_and_cap(min_accesses, max_prefetch, MAX_CO_ACCESS_PAIRS)
    }

    /// Create a cache warmer with explicit thresholds and a pair-table cap.
    ///
    /// Exposed for tests so the saturation path can be exercised cheaply;
    /// production callers use `with_config` (which applies `MAX_CO_ACCESS_PAIRS`).
    fn with_config_and_cap(min_accesses: u32, max_prefetch: usize, max_pairs: usize) -> Self {
        Self {
            co_access: RwLock::new(HashMap::new()),
            min_accesses,
            max_prefetch,
            max_pairs,
            total_events: AtomicU64::new(0),
            prefetch_hits: AtomicU64::new(0),
            pair_count: AtomicUsize::new(0),
            saturated: AtomicBool::new(false),
        }
    }

    /// Record that a set of IDs was accessed together.
    ///
    /// Typically called after `get_many()` or when search results are returned.
    /// For each pair (A, B) in the slice, increments the co-access count.
    /// Auto-decays old patterns every 1000 events to prevent stale data buildup.
    ///
    /// Memory bound: once `max_pairs` distinct pairs are tracked, NEW pairs are
    /// no longer inserted — only already-tracked pairs get their count bumped.
    /// Without this cap the table grows O(n²) with distinct node pairs (AUDIT-04).
    pub fn record_co_access(&self, ids: &[u128]) {
        if ids.len() < 2 {
            return;
        }
        let prev = self.total_events.fetch_add(1, Ordering::Relaxed);
        // Auto-decay every 1000 events — halves all counts to age out stale patterns
        if prev > 0 && prev % 1000 == 0 {
            self.decay();
        }
        let mut table = self.co_access.write();
        if self.saturated.load(Ordering::Relaxed) {
            // Table at cap: refresh existing pairs only, do not learn new ones.
            for (i, &a) in ids.iter().enumerate() {
                if let Some(entry) = table.get_mut(&a) {
                    for &b in &ids[i + 1..] {
                        if let Some(count) = entry.get_mut(&b) {
                            *count = count.saturating_add(1);
                        }
                    }
                }
            }
            return;
        }
        let mut new_pairs = 0usize;
        for (i, &a) in ids.iter().enumerate() {
            let entry = table.entry(a).or_default();
            for &b in &ids[i + 1..] {
                if !entry.contains_key(&b) {
                    new_pairs += 1;
                }
                *entry.entry(b).or_insert(0) =
                    entry.get(&b).copied().unwrap_or(0).saturating_add(1);
            }
        }
        if new_pairs > 0 {
            let total = self.pair_count.fetch_add(new_pairs, Ordering::Relaxed) + new_pairs;
            if total >= self.max_pairs {
                self.saturated.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Return node IDs to prefetch for a given just-accessed node.
    ///
    /// `cache_contains` is called for each candidate to avoid re-fetching
    /// nodes already in the volatile cache. Only IDs whose co-access count
    /// >= `min_accesses` are returned.
    pub fn suggest_warm_ids(&self, id: u128, cache_contains: impl Fn(u128) -> bool) -> Vec<u128> {
        let table = self.co_access.read();
        let Some(related) = table.get(&id) else {
            return Vec::new();
        };

        // Collect candidates above threshold
        let mut candidates: Vec<(u128, u32)> = related
            .iter()
            .filter(|(_, &count)| count >= self.min_accesses)
            .map(|(&id, &count)| (id, count))
            .collect();

        if candidates.is_empty() {
            return Vec::new();
        }

        // Sort by co-access count descending, take top N
        candidates.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
        candidates
            .into_iter()
            .take(self.max_prefetch)
            .filter(|(id, _)| !cache_contains(*id))
            .map(|(id, _)| id)
            .collect()
    }

    /// Record that a prefetched node was subsequently accessed (a hit).
    pub fn record_prefetch_hit(&self) {
        self.prefetch_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Decay old co-access patterns by halving all counts.
    /// Entries that fall to 0 are removed.
    /// Called by `record_co_access()` every 1000 events.
    pub fn decay(&self) {
        let mut table = self.co_access.write();
        table.retain(|_, related| {
            related.retain(|_, count| {
                *count /= 2;
                *count > 0
            });
            !related.is_empty()
        });
        // Reconcile the pair counter with what actually survived the decay.
        let total: usize = table.values().map(|m| m.len()).sum();
        self.pair_count.store(total, Ordering::Relaxed);
    }

    /// Return the top-layer node IDs from an HNSW graph.
    ///
    /// The top layer of HNSW contains the entry point and its neighbors at
    /// the highest layer — these are the first nodes touched on every search
    /// and should be kept hot in cache.
    pub fn hnsw_top_layer_ids(hnsw: &crate::index::CPIndex) -> Vec<u128> {
        let ep = match hnsw.get_entry_point() {
            Some(id) => id,
            None => return Vec::new(),
        };
        let max_layer = hnsw.max_layer.load(Ordering::Relaxed);
        if max_layer == 0 {
            return vec![ep];
        }
        let mut ids = vec![ep];
        if let Some(neighbors) = hnsw.neighbor_index.get_neighbors_ref(ep, max_layer) {
            for &nid in neighbors.iter() {
                if !ids.contains(&nid) {
                    ids.push(nid);
                }
            }
        }
        ids
    }

    /// Read current metrics and update Prometheus gauges.
    /// ponytail: `#[allow(dead_code)]` — only called from tests; a periodic
    /// metrics sampler should call this from the engine's tick loop.
    #[allow(dead_code)]
    pub fn metrics(&self) -> CacheWarmerMetrics {
        let table = self.co_access.read();
        let total_pairs: usize = table.values().map(|m| m.len()).sum();
        let m = CacheWarmerMetrics {
            tracked_nodes: table.len(),
            total_pairs,
            total_events: self.total_events.load(Ordering::Relaxed),
            prefetch_hits: self.prefetch_hits.load(Ordering::Relaxed),
        };
        crate::metrics::record_cache_warmer_metrics(m);
        m
    }

    /// Clear all tracked co-access data.
    /// ponytail: kept `#[allow(dead_code)]` — no external caller yet, added for
    /// tests and manual reset; becomes live when cache warmer reset API is exposed.
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.co_access.write().clear();
        self.total_events.store(0, Ordering::Relaxed);
        self.prefetch_hits.store(0, Ordering::Relaxed);
        self.pair_count.store(0, Ordering::Relaxed);
        self.saturated.store(false, Ordering::Relaxed);
    }
}

impl Default for CacheWarmer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_suggest() {
        let warmer = CacheWarmer::with_config(2, 5);

        // Record co-access: [1,2,3] together 3 times
        for _ in 0..3 {
            warmer.record_co_access(&[1, 2, 3]);
        }

        // After 3 co-accesses of (1,2) and (1,3), suggesting for 1 should return 2 and 3
        let cache = |_| false; // nothing in cache
        let warm = warmer.suggest_warm_ids(1, cache);
        assert!(warm.contains(&2), "should suggest co-accessed node 2");
        assert!(warm.contains(&3), "should suggest co-accessed node 3");
        assert_eq!(warm.len(), 2);
    }

    #[test]
    fn test_min_accesses_threshold() {
        let warmer = CacheWarmer::with_config(5, 10);

        // Record co-access only 3 times — below threshold of 5
        for _ in 0..3 {
            warmer.record_co_access(&[1, 2]);
        }

        let warm = warmer.suggest_warm_ids(1, |_| false);
        assert!(
            warm.is_empty(),
            "should not prefetch below min_accesses threshold"
        );
    }

    #[test]
    fn test_cache_contains_filter() {
        let warmer = CacheWarmer::with_config(1, 10);

        for _ in 0..3 {
            warmer.record_co_access(&[1, 2, 3]);
        }

        // Node 2 is already in cache, should only suggest node 3
        let warm = warmer.suggest_warm_ids(1, |id| id == 2);
        assert_eq!(warm, vec![3], "should skip already-cached node 2");
    }

    #[test]
    fn test_max_prefetch_limit() {
        let warmer = CacheWarmer::with_config(1, 2); // max 2

        for _ in 0..3 {
            warmer.record_co_access(&[1, 2, 3, 4, 5]);
        }

        let warm = warmer.suggest_warm_ids(1, |_| false);
        assert!(warm.len() <= 2, "should respect max_prefetch limit");
    }

    #[test]
    fn test_decay() {
        let warmer = CacheWarmer::with_config(1, 10);

        for _ in 0..4 {
            warmer.record_co_access(&[1, 2]);
        }

        // Before decay: count is 4
        let warm_before = warmer.suggest_warm_ids(1, |_| false);
        assert_eq!(warm_before.len(), 1);

        // After decay: count becomes 2, still >= 1
        warmer.decay();
        let warm_after = warmer.suggest_warm_ids(1, |_| false);
        assert_eq!(warm_after.len(), 1, "decayed but still above threshold");

        // Second decay: count becomes 1, still >= 1
        warmer.decay();
        let warm_after2 = warmer.suggest_warm_ids(1, |_| false);
        assert_eq!(warm_after2.len(), 1);

        // Third decay: count becomes 0, removed
        warmer.decay();
        let warm_after3 = warmer.suggest_warm_ids(1, |_| false);
        assert!(warm_after3.is_empty(), "decayed to 0, should be removed");
    }

    #[test]
    fn test_metrics() {
        let warmer = CacheWarmer::with_config(1, 10);
        warmer.record_co_access(&[1, 2, 3]);
        warmer.record_co_access(&[4, 5]);

        let m = warmer.metrics();
        assert_eq!(m.total_events, 2);
        assert!(m.tracked_nodes > 0);
        assert!(m.total_pairs > 0);
    }

    #[test]
    fn test_pair_cap_saturates_and_stops_learning() {
        // Cap of 6 pairs: [1,2,3] inserts 3 pairs, [4,5] inserts 1 more = 4.
        // [6,7,8] would insert 3 more (total 7 > 6) → saturates mid-call.
        let warmer = CacheWarmer::with_config_and_cap(1, 10, 6);
        warmer.record_co_access(&[1, 2, 3]);
        warmer.record_co_access(&[4, 5]);
        assert!(!warmer.saturated.load(Ordering::Relaxed));
        assert_eq!(warmer.metrics().total_pairs, 4);

        // Cross the cap: [6,7,8] adds pairs (6,7),(6,8),(7,8) → total 7 ≥ 6.
        warmer.record_co_access(&[6, 7, 8]);
        assert!(warmer.saturated.load(Ordering::Relaxed));

        // A brand-new pair (9,10) must NOT be learned once saturated.
        let before = warmer.metrics().total_pairs;
        warmer.record_co_access(&[9, 10]);
        assert_eq!(
            warmer.metrics().total_pairs,
            before,
            "saturated warmer must not grow"
        );

        // Existing pairs still get refreshed (count bump, no new memory).
        warmer.record_co_access(&[1, 2]);
        assert_eq!(
            warmer.metrics().total_pairs,
            before,
            "refresh must not grow table"
        );
    }

    #[test]
    fn test_clear_resets_saturation() {
        let warmer = CacheWarmer::with_config_and_cap(1, 10, 1);
        warmer.record_co_access(&[1, 2]); // 1 pair ≥ cap 1 → saturated
        assert!(warmer.saturated.load(Ordering::Relaxed));
        warmer.clear();
        assert!(!warmer.saturated.load(Ordering::Relaxed));
        assert_eq!(warmer.metrics().total_pairs, 0);
    }

    #[test]
    fn test_hnsw_top_layer_no_nodes() {
        let hnsw = crate::index::CPIndex::new();
        let ids = CacheWarmer::hnsw_top_layer_ids(&hnsw);
        assert!(ids.is_empty(), "empty HNSW should return empty");
    }
}
