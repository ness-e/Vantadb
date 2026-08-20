//! Derived sparse-vector inverted index for `VantaEmbedded`.
//!
//! Sparse vectors are user-provided `(dim: u32, weight: f32)` maps. Today the
//! SDK persists them on memory record nodes, but search scored them with a
//! brute-force O(n) scan over every record in the namespace (`sparse_memory_search`).
//!
//! This module maintains a persistent inverted index (posting lists per
//! namespace+dim) in its own `BackendPartition::SparseIndex`, mirroring the
//! lexical `TextIndex` pattern. A query only touches postings for the dims it
//! contains, so a sparse query with `q` populated dims is O(sum of posting-list
//! lengths) instead of O(records in namespace).
//!
//! ## Design note: dedicated index, not a merge with the lexical index
//!
//! The lexical text index (`text_index.rs`) is built from `payload` text via a
//! string tokenizer; its keys are `namespace\0token\0key` where the token is a
//! UTF-8 string. Sparse dims are `u32` vocabulary ids and their weights are
//! floats — a completely different data domain. Reusing `BackendPartition::TextIndex`
//! would require the text audit/rebuild to classify sparse keys and would make
//! `scan_partition_prefix_iter` for lexical terms able to collide with sparse
//! keys. A dedicated partition keeps the two audits and rebuilds independent.

use super::super::builder::VantaEmbedded;
use super::super::types::*;
use super::{memory_record_from_node, now_ms, SPARSE_INDEX_SCHEMA_VERSION};
use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::node::UnifiedNode;
use crate::storage::StorageEngine;
use postcard;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Schema version of the sparse index key format.
pub(crate) const SPARSE_INDEX_KEY_PREFIX: &[u8] = b"v1";

/// A posting entry linking a node ID to a sparse dim with its weight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SparsePosting {
    /// Node ID referenced by this posting.
    pub node_id: u128,
    /// Term weight of the dim within the record.
    pub weight: f32,
}

// ─── Key helpers ────────────────────────────────────────────

/// Build a sparse posting key from namespace, dim, and record key.
pub(crate) fn sparse_posting_key(namespace: &str, dim: u32, key: &str) -> Vec<u8> {
    let mut index_key = Vec::with_capacity(namespace.len() + key.len() + 10);
    index_key.extend_from_slice(SPARSE_INDEX_KEY_PREFIX);
    index_key.push(0);
    index_key.extend_from_slice(namespace.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(&dim.to_le_bytes());
    index_key.push(0);
    index_key.extend_from_slice(key.as_bytes());
    index_key
}

/// Build a sparse posting prefix for scanning all entries of a namespace+dim.
pub(crate) fn sparse_posting_prefix(namespace: &str, dim: u32) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(namespace.len() + 10);
    prefix.extend_from_slice(SPARSE_INDEX_KEY_PREFIX);
    prefix.push(0);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix.extend_from_slice(&dim.to_le_bytes());
    prefix.push(0);
    prefix
}

// ─── Value encode/decode ────────────────────────────────────

fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    postcard::to_allocvec(value).map_err(VantaError::serialization)
}

fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T> {
    let val: T = postcard::from_bytes(bytes).map_err(|err| {
        VantaError::SerializationError(Box::new(crate::error::SerdeMsgError::new(
            format!("{label} decode error: {err}"),
            err,
        )))
    })?;
    Ok(val)
}

/// Encode a sparse posting entry into bytes.
pub(crate) fn sparse_posting_value(node_id: u128, weight: f32) -> Result<Vec<u8>> {
    serialize(&SparsePosting { node_id, weight })
}

/// Decode sparse posting bytes into a `SparsePosting` struct.
pub(crate) fn decode_sparse_posting(bytes: &[u8]) -> Result<SparsePosting> {
    deserialize(bytes, "sparse posting")
}

// ─── Write ops ──────────────────────────────────────────────

/// Build write operations to upsert sparse postings for a record.
pub(crate) fn sparse_put_ops(record: &VantaMemoryRecord) -> Result<Vec<BackendWriteOp>> {
    let Some(sparse) = record.sparse_vector.as_ref() else {
        return Ok(Vec::new());
    };
    sparse
        .0
        .iter()
        .map(|(dim, weight)| {
            Ok(BackendWriteOp::Put {
                partition: BackendPartition::SparseIndex,
                key: sparse_posting_key(&record.namespace, *dim, &record.key),
                value: sparse_posting_value(record.node_id, *weight)?,
            })
        })
        .collect()
}

/// Build write operations to delete sparse postings for a record.
pub(crate) fn sparse_delete_ops(record: &VantaMemoryRecord) -> Vec<BackendWriteOp> {
    let Some(sparse) = record.sparse_vector.as_ref() else {
        return Vec::new();
    };
    sparse
        .0
        .keys()
        .map(|dim| BackendWriteOp::Delete {
            partition: BackendPartition::SparseIndex,
            key: sparse_posting_key(&record.namespace, *dim, &record.key),
        })
        .collect()
}

/// Build write operations to replace sparse postings for a record
/// (delete previous, put current). No term/namespace stats are needed:
/// sparse scoring is a raw dot product.
pub(crate) fn sparse_index_ops_for_replace(
    previous: Option<&VantaMemoryRecord>,
    current: Option<&VantaMemoryRecord>,
) -> Result<Vec<BackendWriteOp>> {
    let mut ops = Vec::new();
    if let Some(previous) = previous {
        ops.extend(sparse_delete_ops(previous));
    }
    if let Some(current) = current {
        ops.extend(sparse_put_ops(current)?);
    }
    Ok(ops)
}

/// Number of sparse posting entries a record would contribute.
pub(crate) fn sparse_posting_count(record: &VantaMemoryRecord) -> u64 {
    record
        .sparse_vector
        .as_ref()
        .map(|sparse| sparse.len() as u64)
        .unwrap_or(0)
}

// ─── State, counts, rebuild ─────────────────────────────────

impl VantaEmbedded {
    pub(crate) fn ensure_sparse_index_current_with(
        &self,
        engine: &Arc<StorageEngine>,
        nodes: &[UnifiedNode],
    ) -> Result<()> {
        let state = match Self::load_sparse_index_state(engine) {
            Ok(state) => state,
            Err(_) => {
                self.rebuild_sparse_index_with_report()?;
                return Ok(());
            }
        };

        let expected = Self::expected_sparse_index_counts_from(nodes);
        let current = Self::current_sparse_index_counts(engine)?;

        let has_state = state.is_some();
        let needs_rebuild = match &state {
            Some(state) => {
                state.schema_version != SPARSE_INDEX_SCHEMA_VERSION
                    || state.record_count != expected.record_count
                    || state.posting_entries != current.posting_entries
                    || state.posting_entries != expected.posting_entries
                    || current.posting_entries != expected.posting_entries
            }
            None => {
                expected.record_count > 0
                    || current.record_count > 0
                    || expected.posting_entries > 0
                    || current.posting_entries > 0
            }
        };

        if needs_rebuild {
            self.rebuild_sparse_index_with_report()?;
        } else if !has_state {
            Self::write_sparse_index_state(engine, &Self::fresh_sparse_index_state(expected))?;
        }

        Ok(())
    }

    pub(crate) fn load_sparse_index_state(
        engine: &StorageEngine,
    ) -> Result<Option<SparseIndexState>> {
        let Some(bytes) = engine.get_from_partition(
            BackendPartition::InternalMetadata,
            super::SPARSE_INDEX_STATE_KEY,
        )?
        else {
            return Ok(None);
        };
        postcard::from_bytes(&bytes)
            .map(Some)
            .map_err(VantaError::serialization)
    }

    pub(crate) fn write_sparse_index_state(
        engine: &StorageEngine,
        state: &SparseIndexState,
    ) -> Result<()> {
        let bytes = postcard::to_allocvec(state).map_err(VantaError::serialization)?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            super::SPARSE_INDEX_STATE_KEY,
            &bytes,
        )
    }

    pub(crate) fn fresh_sparse_index_state(counts: SparseIndexCounts) -> SparseIndexState {
        SparseIndexState {
            schema_version: SPARSE_INDEX_SCHEMA_VERSION,
            rebuilt_at_ms: now_ms(),
            record_count: counts.record_count,
            posting_entries: counts.posting_entries,
        }
    }

    pub(crate) fn expected_sparse_index_counts_from(nodes: &[UnifiedNode]) -> SparseIndexCounts {
        let mut counts = SparseIndexCounts::default();
        for node in nodes {
            if let Some(record) = memory_record_from_node(node) {
                counts.record_count += 1;
                counts.posting_entries += sparse_posting_count(&record);
            }
        }
        counts
    }

    pub(crate) fn current_sparse_index_counts(engine: &StorageEngine) -> Result<SparseIndexCounts> {
        let mut counts = SparseIndexCounts::default();
        for (_key, _value) in engine.scan_partition(BackendPartition::SparseIndex)? {
            counts.posting_entries += 1;
        }
        Ok(counts)
    }

    pub(crate) fn rebuild_sparse_index_with_report(&self) -> Result<SparseIndexRebuildReport> {
        let started = web_time::Instant::now();
        let engine = self.engine_handle()?;

        let mut ops = Vec::new();
        let mut counts = SparseIndexCounts::default();

        for (key, _value) in engine.scan_partition(BackendPartition::SparseIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::SparseIndex,
                key,
            });
        }

        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(&node) {
                counts.record_count += 1;
                let put_ops = sparse_put_ops(&record)?;
                counts.posting_entries += put_ops.len() as u64;
                ops.extend(put_ops);
            }
        }

        if !ops.is_empty() {
            engine.write_backend_batch(ops)?;
        }

        Self::write_sparse_index_state(&engine, &Self::fresh_sparse_index_state(counts))?;

        let report = SparseIndexRebuildReport {
            record_count: counts.record_count,
            posting_entries: counts.posting_entries,
            duration_ms: started.elapsed().as_millis() as u64,
        };
        Ok(report)
    }

    pub(crate) fn adjust_sparse_index_state_after_replace(
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
    ) -> Result<()> {
        let Some(mut state) = Self::load_sparse_index_state(engine)? else {
            return Ok(());
        };
        if state.schema_version != SPARSE_INDEX_SCHEMA_VERSION {
            return Ok(());
        }

        match (previous, current) {
            (None, Some(current)) => {
                state.record_count = state.record_count.saturating_add(1);
                state.posting_entries = state
                    .posting_entries
                    .saturating_add(sparse_posting_count(current));
            }
            (Some(previous), None) => {
                state.record_count = state.record_count.saturating_sub(1);
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(sparse_posting_count(previous));
            }
            (Some(previous), Some(current)) => {
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(sparse_posting_count(previous))
                    .saturating_add(sparse_posting_count(current));
            }
            (None, None) => {}
        }

        Self::write_sparse_index_state(engine, &state)
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::super::super::types::*;
    use super::*;
    use crate::node::SparseVector;

    fn record(namespace: &str, key: &str, sparse: Option<SparseVector>) -> VantaMemoryRecord {
        VantaMemoryRecord {
            namespace: namespace.into(),
            key: key.into(),
            payload: "payload".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 100,
            updated_at_ms: 100,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id(namespace, key),
            vector: None,
            sparse_vector: sparse,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        }
    }

    #[test]
    fn sparse_posting_key_roundtrip() {
        let ns = "ns";
        let dim = 42;
        let key = "record-1";
        let index_key = sparse_posting_key(ns, dim, key);
        // The key is the scan prefix for (ns, dim) followed by the record key.
        let prefix = sparse_posting_prefix(ns, dim);
        assert!(index_key.starts_with(&prefix));
        assert_eq!(&index_key[prefix.len()..], key.as_bytes());
        // A key for another dim must not share the same prefix.
        assert!(!index_key.starts_with(&sparse_posting_prefix(ns, dim + 1)));
    }

    #[test]
    fn sparse_posting_value_roundtrip() {
        let value = sparse_posting_value(7, 1.25).expect("encode");
        let posting = decode_sparse_posting(&value).expect("decode");
        assert_eq!(posting.node_id, 7);
        assert_eq!(posting.weight, 1.25);
    }

    #[test]
    fn sparse_put_ops_emit_one_posting_per_dim() {
        let mut sparse = SparseVector::new();
        sparse.insert(1, 0.5);
        sparse.insert(2, 1.0);
        let rec = record("ns", "k", Some(sparse));
        let ops = sparse_put_ops(&rec).unwrap();
        assert_eq!(ops.len(), 2);
        for op in &ops {
            match op {
                BackendWriteOp::Put { partition, .. } => {
                    assert_eq!(*partition, BackendPartition::SparseIndex);
                }
                _ => panic!("expected Put"),
            }
        }
    }

    #[test]
    fn sparse_ops_empty_without_sparse_vector() {
        let rec = record("ns", "k", None);
        assert!(sparse_put_ops(&rec).unwrap().is_empty());
        assert!(sparse_delete_ops(&rec).is_empty());
    }

    #[test]
    fn sparse_index_ops_for_replace_deletes_previous_then_puts_current() {
        let mut prev = SparseVector::new();
        prev.insert(1, 0.5);
        let mut curr = SparseVector::new();
        curr.insert(1, 0.9);
        curr.insert(2, 0.3);
        let ops = sparse_index_ops_for_replace(
            Some(&record("ns", "k", Some(prev))),
            Some(&record("ns", "k", Some(curr))),
        )
        .unwrap();
        assert_eq!(ops.len(), 3); // 1 delete + 2 puts
    }
}
