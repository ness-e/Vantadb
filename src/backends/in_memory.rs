//! In-memory implementation of `StorageBackend`.
//!
//! Provides a fully functional KV store backed by `BTreeMap`s in memory.
//! Intended for:
//! - Fast, isolated unit tests that don't need disk I/O.
//! - Decoupling `StorageEngine` logic from the persistence layer during testing.
//!
//! ## Important clarification
//!
//! "InMemoryBackend" means **in-memory KV backend only**. When used with
//! `StorageEngine`, VantaFile (vector store) and WAL are still initialized
//! on disk at the provided path. This backend replaces only the RocksDB
//! key-value layer, not the entire storage stack.

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::error::{Result, VantaError};
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// In-memory storage backend using `BTreeMap` per partition.
///
/// Thread-safe via `RwLock`. All data is lost when the backend is dropped.
pub(crate) struct InMemoryBackend {
    #[allow(clippy::type_complexity)]
    partitions: RwLock<HashMap<BackendPartition, BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryBackend {
    /// Create a new in-memory backend with all partitions initialized empty.
    pub(crate) fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(BackendPartition::Default, BTreeMap::new());
        map.insert(BackendPartition::TombstoneStorage, BTreeMap::new());
        map.insert(BackendPartition::CompressedArchive, BTreeMap::new());
        map.insert(BackendPartition::Tombstones, BTreeMap::new());
        map.insert(BackendPartition::NamespaceIndex, BTreeMap::new());
        map.insert(BackendPartition::PayloadIndex, BTreeMap::new());
        map.insert(BackendPartition::TextIndex, BTreeMap::new());
        map.insert(BackendPartition::InternalMetadata, BTreeMap::new());
        Self {
            partitions: RwLock::new(map),
        }
    }
}

impl StorageBackend for InMemoryBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        let mut parts = self.partitions.write();
        let btree = parts.entry(partition).or_default();
        btree.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let parts = self.partitions.read();
        Ok(parts
            .get(&partition)
            .and_then(|btree| btree.get(key).cloned()))
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        let mut parts = self.partitions.write();
        if let Some(btree) = parts.get_mut(&partition) {
            btree.remove(key);
        }
        Ok(())
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        let mut parts = self.partitions.write();
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    let btree = parts.entry(partition).or_default();
                    btree.insert(key, value);
                }
                BackendWriteOp::Delete { partition, key } => {
                    if let Some(btree) = parts.get_mut(&partition) {
                        btree.remove(&key);
                    }
                }
            }
        }
        Ok(())
    }

    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let parts = self.partitions.read();
        Ok(parts
            .get(&partition)
            .map(|btree| btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default())
    }

    fn scan_prefix(
        &self,
        partition: BackendPartition,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let parts = self.partitions.read();
        let Some(btree) = parts.get(&partition) else {
            return Ok(Vec::new());
        };

        let mut result = Vec::new();
        for (key, value) in btree.range(prefix.to_vec()..) {
            if !key.starts_with(prefix) {
                break;
            }
            result.push((key.clone(), value.clone()));
        }
        Ok(result)
    }

    fn flush(&self) -> Result<()> {
        // No-op: all data is already in memory.
        Ok(())
    }

    fn checkpoint(&self, _path: &Path) -> Result<()> {
        Err(VantaError::Execution(
            "Checkpoint not supported by InMemoryBackend".to_string(),
        ))
    }

    // compact() inherits the default no-op from the trait.

    fn capabilities(&self) -> crate::backend::BackendCapabilities {
        crate::backend::BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: false,
            kind: crate::backend::BackendKind::InMemory,
        }
    }
}

// ─── Unit Tests ─────────────────────────────────────────────
//
// These tests validate InMemoryBackend directly through the trait.
// They live here (inside the crate) because StorageBackend is pub(crate).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{BackendPartition, BackendWriteOp};

    #[test]
    fn test_backend_in_memory_basic_crud() {
        let backend = InMemoryBackend::new();

        // Put
        backend
            .put(BackendPartition::Default, b"key1", b"value1")
            .unwrap();

        // Get
        let val = backend
            .get(BackendPartition::Default, b"key1")
            .unwrap()
            .expect("key1 should exist");
        assert_eq!(val, b"value1");

        // Get non-existent
        assert!(backend
            .get(BackendPartition::Default, b"missing")
            .unwrap()
            .is_none());

        // Delete
        backend.delete(BackendPartition::Default, b"key1").unwrap();
        assert!(backend
            .get(BackendPartition::Default, b"key1")
            .unwrap()
            .is_none());

        // Scan on different partition
        backend
            .put(BackendPartition::Tombstones, b"t1", b"tombval")
            .unwrap();
        let entries = backend.scan(BackendPartition::Tombstones).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, b"t1");
    }

    #[test]
    fn test_backend_in_memory_batch() {
        let backend = InMemoryBackend::new();

        // Seed a value that will be deleted in the batch
        backend
            .put(BackendPartition::Default, b"to_delete", b"val")
            .unwrap();

        let ops = vec![
            BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: b"batch_key1".to_vec(),
                value: b"batch_val1".to_vec(),
            },
            BackendWriteOp::Put {
                partition: BackendPartition::TombstoneStorage,
                key: b"batch_key2".to_vec(),
                value: b"batch_val2".to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: b"to_delete".to_vec(),
            },
        ];

        backend.write_batch(ops).unwrap();

        assert!(backend
            .get(BackendPartition::Default, b"batch_key1")
            .unwrap()
            .is_some());
        assert!(backend
            .get(BackendPartition::TombstoneStorage, b"batch_key2")
            .unwrap()
            .is_some());
        assert!(backend
            .get(BackendPartition::Default, b"to_delete")
            .unwrap()
            .is_none());
    }
}
