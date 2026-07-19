//! Backend partition delegation methods.

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::storage::engine::StorageEngine;

impl StorageEngine {
    /// Write a value to a specific backend partition.
    pub fn put_to_partition(
        &self,
        partition: BackendPartition,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        self.ensure_writable()?;
        self.backend.put(partition, key, value)
    }

    /// Execute a batch of write operations atomically against the backend.
    pub(crate) fn write_backend_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        self.ensure_writable()?;
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
}
