//! TTL-based garbage collection for expired nodes.
//!
//! [`GcWorker`] tracks node expiration timestamps via a [`BTreeMap`] and
//! evicts expired entries from the [`StorageEngine`] on each sweep.

use crate::error::{Error, Result};
use crate::storage::StorageEngine;
use std::collections::{BTreeMap, HashSet};
use web_time::{SystemTime, UNIX_EPOCH};

/// TTL-based garbage collector for expired nodes.
pub struct GcWorker<'a> {
    /// Reference to the storage engine.
    storage: &'a StorageEngine,
    /// Maps expiration timestamp (seconds) to node IDs.
    index_ttl: BTreeMap<u64, Vec<u128>>,
}

impl<'a> GcWorker<'a> {
    /// Create a new GC worker.
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self {
            storage,
            index_ttl: BTreeMap::new(),
        }
    }

    /// Registers a node to be automatically expired and cleared at `expiry_secs`
    pub fn register_ttl(&mut self, id: u128, expiry_secs: u64) {
        self.index_ttl.entry(expiry_secs).or_default().push(id);
    }

    /// Triggers a sweep that clears old items. Registered TTLs are owned by
    /// callers (e.g. `ThreadStore`), so this runs on demand; production
    /// memory-record expiry is driven by `spawn_memory_ttl_sweeper` (feature
    /// `server`), which purges the full memory/index surface via
    /// `Embedded::purge_expired`.
    pub fn sweep(&mut self) -> Result<usize> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Validation {
                field: "system_time".into(),
                reason: "System time before UNIX epoch".into(),
            })?
            .as_secs();

        // Split the BTreeMap, taking all nodes where expiration <= now
        let mut expired_count = 0;

        let mut keys_to_remove = Vec::new();
        for (expiry, ids) in self.index_ttl.iter_mut() {
            if *expiry <= now {
                // retain() serves as a retry mechanism:
                // - Ok: removed from TTL map (false)
                // - NodeNotFound: node was already manually deleted — remove (false)
                // - Other error: transient failure — keep (true) for retry next sweep
                ids.retain(|&id| match self.storage.delete(id, "GC TTL Expired") {
                    Ok(_) => {
                        expired_count += 1;
                        false
                    }
                    Err(Error::NodeNotFound(_)) => false,
                    Err(e) => {
                        tracing::error!("GC failed to delete node {id}: {e}");
                        true
                    }
                });
                if ids.is_empty() {
                    keys_to_remove.push(*expiry);
                }
            } else {
                break;
            }
        }

        for key in keys_to_remove {
            self.index_ttl.remove(&key);
        }

        Ok(expired_count)
    }

    /// Remove TTL entries for node IDs that are no longer in the active set.
    ///
    /// Call this after a manual (non-TTL) delete so the GC does not accumulate
    /// stale entries for already-deleted nodes, preventing unbounded TTL map
    /// growth. Entries whose ID sets become empty are removed entirely.
    pub fn purge_ttl_for_deleted(&mut self, active_ids: &HashSet<u128>) {
        self.index_ttl.retain(|_, ids| {
            ids.retain(|id| active_ids.contains(id));
            !ids.is_empty()
        });
    }
}

/// Handle for the background memory TTL sweeper.
///
/// Dropping the handle stops the loop (best-effort abort);
/// [`MemoryTtlSweeper::shutdown`] performs a graceful stop and join.
#[cfg(feature = "server")]
pub struct MemoryTtlSweeper {
    shutdown: tokio::sync::watch::Sender<bool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

#[cfg(feature = "server")]
impl MemoryTtlSweeper {
    /// Ask the sweep loop to stop and wait for it to finish (join).
    pub async fn shutdown(mut self) {
        let _ = self.shutdown.send(true);
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

#[cfg(feature = "server")]
impl Drop for MemoryTtlSweeper {
    fn drop(&mut self) {
        // Signal first, then abort: with a live runtime the loop exits cleanly
        // at its next select; the abort guarantees no detached task outlives
        // the handle during shutdown.
        let _ = self.shutdown.send(true);
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

/// Spawn the background TTL sweeper: every `interval`, physically purges
/// expired memory records (nodes + derived/scalar/text indexes) via
/// `Embedded::purge_expired`.
///
/// Lazy read filtering hides expired records from `get`/`search` but leaves
/// the node and its index entries in storage; this loop runs the same complete
/// purge the maintenance endpoint uses, on a blocking thread, so storage
/// actually shrinks. The first sweep runs immediately (Tokio interval
/// semantics), so records that expired while the process was down are cleaned
/// on startup. `interval` must be non-zero — callers that allow disabling
/// (e.g. `ttl_sweep_interval_ms == 0`) skip this call entirely.
#[cfg(feature = "server")]
pub fn spawn_memory_ttl_sweeper(
    db: crate::sdk::Embedded,
    interval: std::time::Duration,
) -> MemoryTtlSweeper {
    debug_assert!(!interval.is_zero(), "sweep interval must be non-zero");
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    let handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                _ = ticker.tick() => {
                    let db = db.clone();
                    match tokio::task::spawn_blocking(move || db.purge_expired()).await {
                        Ok(Ok(purged)) if purged > 0 => {
                            tracing::info!(purged, "ttl sweeper purged expired records");
                        }
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            tracing::warn!(error = %e, "ttl sweeper purge failed; retrying next tick");
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "ttl sweeper purge task failed to join");
                        }
                    }
                }
            }
        }
    });
    MemoryTtlSweeper {
        shutdown: shutdown_tx,
        handle: Some(handle),
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::node::UnifiedNode;
    use crate::storage::{BackendKind, StorageEngine};
    use tempfile::tempdir;

    fn setup_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = Config {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");
        (storage, dir)
    }

    #[test]
    fn test_register_ttl_inserts_entry() {
        let (storage, _dir) = setup_storage();
        let mut worker = GcWorker::new(&storage);
        worker.register_ttl(42, 1_000_000);
        assert_eq!(worker.index_ttl.len(), 1);
        assert_eq!(worker.index_ttl.get(&1_000_000).unwrap(), &vec![42]);
    }

    #[test]
    fn test_register_ttl_multiple_ids_same_expiry() {
        let (storage, _dir) = setup_storage();
        let mut worker = GcWorker::new(&storage);
        worker.register_ttl(1, 100);
        worker.register_ttl(2, 100);
        assert_eq!(worker.index_ttl.get(&100).unwrap().len(), 2);
    }

    #[test]
    fn test_register_ttl_different_expiries() {
        let (storage, _dir) = setup_storage();
        let mut worker = GcWorker::new(&storage);
        worker.register_ttl(1, 100);
        worker.register_ttl(2, 200);
        assert_eq!(worker.index_ttl.len(), 2);
    }

    #[test]
    fn test_sweep_no_expired_nodes() {
        let (storage, _dir) = setup_storage();
        let mut worker = GcWorker::new(&storage);
        let far_future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 100_000;
        worker.register_ttl(42, far_future);
        let count = worker.sweep().unwrap();
        assert_eq!(count, 0);
        assert_eq!(worker.index_ttl.len(), 1);
    }

    #[test]
    fn test_sweep_empty_worker_returns_zero() {
        let (storage, _dir) = setup_storage();
        let mut worker = GcWorker::new(&storage);
        let count = worker.sweep().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_sweep_removes_expired_entries_from_map() {
        let (storage, _dir) = setup_storage();
        let node = UnifiedNode::new(99);
        storage.insert(&node).unwrap();
        let mut worker = GcWorker::new(&storage);
        let past = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 1;
        worker.register_ttl(99, past);
        let count = worker.sweep().unwrap();
        assert_eq!(count, 1);
        assert!(worker.index_ttl.is_empty());
    }

    #[test]
    fn test_multiple_sweeps_gradual_expiry() {
        let (storage, _dir) = setup_storage();
        for i in 0..5 {
            let node = UnifiedNode::new(i);
            storage.insert(&node).unwrap();
        }
        let mut worker = GcWorker::new(&storage);
        for i in 0..5 {
            let past = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - (5 - i);
            worker.register_ttl(i as u128, past);
        }
        let count = worker.sweep().unwrap();
        assert_eq!(count, 5);
        assert!(worker.index_ttl.is_empty());
    }

    #[test]
    fn test_purge_ttl_for_deleted_removes_stale_entries() {
        let (storage, _dir) = setup_storage();
        let mut worker = GcWorker::new(&storage);
        worker.register_ttl(1, 100);
        worker.register_ttl(2, 100);
        worker.register_ttl(3, 200);

        // Only IDs 1 and 3 are still active (2 was manually deleted)
        let active: HashSet<u128> = [1, 3].into();
        worker.purge_ttl_for_deleted(&active);

        // Entry for expiry 200 still has id=3 → kept
        // Entry for expiry 100 now only has id=1 (id=2 removed) → kept
        assert_eq!(worker.index_ttl.len(), 2);
        assert_eq!(worker.index_ttl.get(&100).unwrap(), &vec![1]);
        assert_eq!(worker.index_ttl.get(&200).unwrap(), &vec![3]);
    }

    #[test]
    fn test_purge_ttl_for_deleted_removes_empty_expiry_keys() {
        let (storage, _dir) = setup_storage();
        let mut worker = GcWorker::new(&storage);
        worker.register_ttl(1, 100);
        worker.register_ttl(2, 100);

        // Neither 1 nor 2 is active anymore
        let active: HashSet<u128> = HashSet::new();
        worker.purge_ttl_for_deleted(&active);

        // Expiry 100 should be removed entirely (all IDs removed)
        assert!(worker.index_ttl.is_empty());
    }

    #[test]
    fn test_sweep_with_mixed_expired_and_future() {
        let (storage, _dir) = setup_storage();
        let node = UnifiedNode::new(42);
        storage.insert(&node).unwrap();

        let mut worker = GcWorker::new(&storage);
        let past = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 10;
        let future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 100_000;

        worker.register_ttl(42, past);
        worker.register_ttl(99, future);

        let count = worker.sweep().unwrap();
        // 42 expired and was deleted; 99 is future and kept
        assert_eq!(count, 1);
        assert_eq!(worker.index_ttl.len(), 1);
        assert!(worker.index_ttl.contains_key(&future));
    }

    #[test]
    fn test_sweep_skips_non_expired_keys() {
        let (storage, _dir) = setup_storage();
        for i in 0..3 {
            let node = UnifiedNode::new(i);
            storage.insert(&node).unwrap();
        }
        let mut worker = GcWorker::new(&storage);
        let far_future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 1_000_000;
        // All registered with the same future expiry
        for i in 0..3 {
            worker.register_ttl(i, far_future);
        }
        let count = worker.sweep().unwrap();
        assert_eq!(count, 0, "no nodes should be expired yet");
        assert_eq!(worker.index_ttl.len(), 1, "one expiry key remains");
        assert_eq!(
            worker.index_ttl.get(&far_future).unwrap().len(),
            3,
            "all 3 IDs still tracked"
        );
    }

    #[test]
    fn test_sweep_node_already_deleted_by_user() {
        let (storage, _dir) = setup_storage();
        let node = UnifiedNode::new(42);
        storage.insert(&node).unwrap();

        let mut worker = GcWorker::new(&storage);
        let past = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 1;
        worker.register_ttl(42, past);

        // User manually deletes the node before GC runs
        storage.delete(42, "manual").unwrap();

        // GC sweep: delete(42) returns Ok(()) even for already-deleted nodes,
        // so the sweep succeeds and counts it as expired
        let count = worker.sweep().unwrap();
        // The delete returns Ok(()) for already-deleted nodes → expired_count incremented
        // (StorageEngine::delete does NOT return NodeNotFound for nonexistent nodes)
        assert_eq!(count, 1, "GC should process the expired entry");
        assert!(
            worker.index_ttl.is_empty(),
            "TTL entry should be cleaned up"
        );
    }
}

/// WIRE-04: the background TTL sweeper (production driver of
/// [`Embedded::purge_expired`]). Feature-gated because the loop needs Tokio.
#[cfg(all(test, feature = "server"))]
#[allow(missing_docs)]
mod sweeper_tests {
    use super::*;
    use crate::config::Config;
    use crate::sdk::serialization::FIELD_EXPIRES_AT_MS;
    use crate::sdk::{Embedded, MemoryInput};
    use crate::storage::BackendKind;
    use std::time::Duration;

    fn in_memory_db() -> Embedded {
        Embedded::open_with_config(Config {
            storage_path: ":memory:".into(),
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        })
        .expect("open in-memory Embedded")
    }

    /// Wait (bounded) until `check` holds; fails the test when the deadline lapses.
    async fn wait_until(timeout: Duration, mut check: impl FnMut() -> bool) {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if check() {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "condition not reached within {timeout:?}"
            );
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    #[tokio::test]
    async fn ttl_sweeper_physically_removes_expired_records_and_indexes() {
        let db = in_memory_db();
        let sweeper = spawn_memory_ttl_sweeper(db.clone(), Duration::from_millis(20));

        let mut input = MemoryInput::new("sweep", "k1", "physically purged");
        input.ttl_ms = Some(25);
        let record = db.put(input).expect("put");

        let engine = db.engine_handle().expect("engine");
        // While alive: the scalar index must know the expiry field (this is what
        // `purge_expired` scans — asserting it here proves the index assertion
        // below is not vacuous).
        assert!(
            engine
                .scalar_lookup_int_le(FIELD_EXPIRES_AT_MS, i64::MAX)
                .contains(&record.node_id),
            "fresh TTL record must be present in the scalar index"
        );

        // The sweeper — not a manual `purge_expired` call — must physically remove
        // the node (absent from the engine, not merely hidden by lazy read filtering).
        wait_until(Duration::from_secs(3), || {
            engine.get(record.node_id).expect("engine get").is_none()
        })
        .await;

        // Index-level proof: the expiry entry is gone from the scalar index too.
        assert!(
            !engine
                .scalar_lookup_int_le(FIELD_EXPIRES_AT_MS, i64::MAX)
                .contains(&record.node_id),
            "sweeper must purge the derived index entry, not only the node"
        );

        sweeper.shutdown().await;
    }

    #[tokio::test]
    async fn ttl_sweeper_shutdown_joins_and_stops_further_sweeps() {
        let db = in_memory_db();
        let sweeper = spawn_memory_ttl_sweeper(db.clone(), Duration::from_millis(20));

        // Graceful shutdown: the join must complete (no detached loop left behind).
        sweeper.shutdown().await;

        let mut input = MemoryInput::new("sweep", "k2", "survives shutdown");
        input.ttl_ms = Some(20);
        let record = db.put(input).expect("put");

        // Wait well past several sweep intervals: a still-running sweeper would
        // have purged the node by now.
        tokio::time::sleep(Duration::from_millis(150)).await;

        let engine = db.engine_handle().expect("engine");
        assert!(
            engine.get(record.node_id).expect("engine get").is_some(),
            "no sweep may run after shutdown() joins the loop"
        );
    }
}
