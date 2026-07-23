//! TTL-based garbage collection for expired nodes.
//!
//! [`GcWorker`] tracks node expiration timestamps via a [`BTreeMap`] and
//! evicts expired entries from the [`StorageEngine`] on each sweep.

use crate::error::{Result, VantaError};
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

    /// Triggers a sweep that clears old items. In production this runs in a `tokio::spawn` loop.
    pub fn sweep(&mut self) -> Result<usize> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| VantaError::ValidationError {
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
                    Err(VantaError::NodeNotFound(_)) => false,
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;
    use crate::node::UnifiedNode;
    use crate::storage::{BackendKind, StorageEngine};
    use tempfile::tempdir;

    fn setup_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = VantaConfig {
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
