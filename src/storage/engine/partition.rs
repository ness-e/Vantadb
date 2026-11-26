//! Backend partition delegation methods.

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::storage::engine::StorageEngine;

impl StorageEngine {
    /// Write a value to a specific backend partition.
    ///
    /// FIND-651: acquires `insert_lock` — this is the choke point for every
    /// SDK-level backend write (label interner, text/sparse/derived indexes,
    /// version history, skills, wiki, migrations), and none of them may mutate
    /// the backend while `create_snapshot` holds the quiesce guard across its
    /// mirror. MUST NOT be called while holding `insert_lock` (non-reentrant —
    /// engine internals use `self.backend` directly inside their sections).
    pub fn put_to_partition(
        &self,
        partition: BackendPartition,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        self.ensure_writable()?;
        let _guard = self.acquire_insert_lock("acquire insert_lock in put_to_partition")?;
        self.backend.put(partition, key, value)
    }

    /// Execute a batch of write operations atomically against the backend.
    ///
    /// FIND-651: same `insert_lock` quiesce contract as
    /// [`Self::put_to_partition`]. MUST NOT be called while holding
    /// `insert_lock` (non-reentrant).
    pub(crate) fn write_backend_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        self.ensure_writable()?;
        let _guard = self.acquire_insert_lock("acquire insert_lock in write_backend_batch")?;
        self.backend.write_batch(ops)
    }

    /// Scan all key-value pairs in the given backend partition.
    pub(crate) fn scan_partition(
        &self,
        partition: BackendPartition,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.backend.scan(partition)
    }

    /// Scan key-value pairs matching the given prefix in the given backend partition.
    #[allow(dead_code)]
    pub(crate) fn scan_partition_prefix(
        &self,
        partition: BackendPartition,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.backend.scan_prefix(partition, prefix)
    }

    /// Streaming variant of `scan_partition_prefix`.
    ///
    /// Returns a `Box<dyn Iterator>` so callers that only iterate once
    /// can avoid materializing the full result set.
    pub(crate) fn scan_partition_prefix_iter<'a>(
        &'a self,
        partition: BackendPartition,
        prefix: &'a [u8],
    ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + 'a>> {
        self.backend.scan_prefix_iter(partition, prefix)
    }

    /// Retrieve a single value from the given backend partition.
    pub(crate) fn get_from_partition(
        &self,
        partition: BackendPartition,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        self.backend.get(partition, key)
    }

    /// Retrieve multiple values from the given backend partition in one
    /// batch call (SK-03 / FIND-607).
    ///
    /// Keys that do not exist are omitted from the result. Backends with a
    /// native multi-get override answer in a single round-trip; the rest
    /// degrade to sequential `get`s inside `backend.get_many`.
    ///
    /// Read-only path — takes no engine lock, so callers MUST NOT hold
    /// `insert_lock` or a `volatile_cache` write guard (Regla 8).
    pub(crate) fn get_many_from_partition(
        &self,
        partition: BackendPartition,
        keys: &[&[u8]],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.backend.get_many(partition, keys)
    }
}
