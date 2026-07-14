//! Derived index state management for `VantaEmbedded`.

use super::super::builder::VantaEmbedded;
use super::super::types::*;
use super::{
    namespace_index_key, node_id_bytes, now_ms, payload_index_key, DERIVED_INDEX_SCHEMA_VERSION,
};
use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::node::UnifiedNode;
use crate::storage::StorageEngine;
use std::sync::Arc;

impl VantaEmbedded {
    pub(crate) fn ensure_indexes_current(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let nodes = engine.scan_nodes()?;

        self.ensure_derived_indexes_current_with(&engine, &nodes)?;
        self.ensure_text_index_current_with(&engine, &nodes)?;

        Ok(())
    }

    fn ensure_derived_indexes_current_with(
        &self,
        engine: &Arc<StorageEngine>,
        nodes: &[UnifiedNode],
    ) -> Result<()> {
        let state = match Self::load_derived_index_state(engine) {
            Ok(state) => state,
            Err(_) => {
                self.rebuild_derived_indexes_with_report()?;
                return Ok(());
            }
        };

        let canonical_records = Self::count_memory_records_from(nodes);
        let (namespace_entries, payload_entries) = Self::current_derived_index_counts(engine)?;

        let has_state = state.is_some();
        let needs_rebuild = match &state {
            Some(state) => {
                state.schema_version != DERIVED_INDEX_SCHEMA_VERSION
                    || state.record_count != canonical_records
                    || state.namespace_entries != namespace_entries
                    || state.payload_entries != payload_entries
                    || namespace_entries < canonical_records
            }
            None => canonical_records > 0 || namespace_entries > 0 || payload_entries > 0,
        };

        if needs_rebuild {
            self.rebuild_derived_indexes_with_report()?;
        } else if !has_state {
            Self::write_derived_index_state(
                engine,
                &DerivedIndexState {
                    schema_version: DERIVED_INDEX_SCHEMA_VERSION,
                    rebuilt_at_ms: now_ms(),
                    record_count: canonical_records,
                    namespace_entries,
                    payload_entries,
                },
            )?;
        }

        Ok(())
    }

    pub(crate) fn load_derived_index_state(
        engine: &StorageEngine,
    ) -> Result<Option<DerivedIndexState>> {
        let Some(bytes) = engine.get_from_partition(
            BackendPartition::InternalMetadata,
            super::DERIVED_INDEX_STATE_KEY,
        )?
        else {
            return Ok(None);
        };
        postcard::from_bytes(&bytes)
            .map(Some)
            .map_err(VantaError::serialization)
    }

    pub(crate) fn write_derived_index_state(
        engine: &StorageEngine,
        state: &DerivedIndexState,
    ) -> Result<()> {
        let bytes = postcard::to_allocvec(state).map_err(VantaError::serialization)?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            super::DERIVED_INDEX_STATE_KEY,
            &bytes,
        )
    }

    fn current_derived_index_counts(engine: &StorageEngine) -> Result<(u64, u64)> {
        let namespace_entries = engine
            .scan_partition(BackendPartition::NamespaceIndex)?
            .len() as u64;
        let payload_entries = engine.scan_partition(BackendPartition::PayloadIndex)?.len() as u64;
        Ok((namespace_entries, payload_entries))
    }

    pub(crate) fn derived_put_ops(record: &VantaMemoryRecord) -> Result<Vec<BackendWriteOp>> {
        let mut ops = Vec::new();
        ops.push(BackendWriteOp::Put {
            partition: BackendPartition::NamespaceIndex,
            key: namespace_index_key(&record.namespace, &record.key),
            value: node_id_bytes(record.node_id),
        });

        for (field, value) in &record.metadata {
            for val in value.to_index_values() {
                ops.push(BackendWriteOp::Put {
                    partition: BackendPartition::PayloadIndex,
                    key: payload_index_key(&record.namespace, field, &val, &record.key)?,
                    value: node_id_bytes(record.node_id),
                });
            }
        }

        Ok(ops)
    }

    pub(crate) fn derived_delete_ops(record: &VantaMemoryRecord) -> Result<Vec<BackendWriteOp>> {
        let mut ops = Vec::new();
        ops.push(BackendWriteOp::Delete {
            partition: BackendPartition::NamespaceIndex,
            key: namespace_index_key(&record.namespace, &record.key),
        });

        for (field, value) in &record.metadata {
            for val in value.to_index_values() {
                ops.push(BackendWriteOp::Delete {
                    partition: BackendPartition::PayloadIndex,
                    key: payload_index_key(&record.namespace, field, &val, &record.key)?,
                });
            }
        }

        Ok(ops)
    }

    pub(crate) fn replace_derived_indexes(
        &self,
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
    ) -> Result<()> {
        let mut ops = Vec::new();
        if let Some(previous) = previous {
            ops.extend(Self::derived_delete_ops(previous)?);
        }
        if let Some(current) = current {
            ops.extend(Self::derived_put_ops(current)?);
        }
        let (text_ops, text_report) = Self::text_index_ops_for_replace(engine, previous, current)?;
        ops.extend(text_ops);
        if ops.is_empty() {
            return Ok(());
        }
        engine.write_backend_batch(ops.clone())?;

        for op in &ops {
            match op {
                BackendWriteOp::Put {
                    partition: BackendPartition::TextIndex,
                    key,
                    value,
                } => {
                    if crate::text_index::is_term_stats_key(key) {
                        if let Some((ns, token)) = Self::parse_term_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_term_stats(value) {
                                let mut cache = engine.text_stats_cache.write();
                                cache.insert((ns, token), stats);
                            }
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_namespace_stats(value) {
                                let mut cache = engine.text_ns_cache.write();
                                cache.insert(ns, stats);
                            }
                        }
                    }
                }
                BackendWriteOp::Delete {
                    partition: BackendPartition::TextIndex,
                    key,
                } => {
                    if crate::text_index::is_term_stats_key(key) {
                        if let Some((ns, token)) = Self::parse_term_stats_key(key) {
                            let mut cache = engine.text_stats_cache.write();
                            cache.remove(&(ns, token));
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            let mut cache = engine.text_ns_cache.write();
                            cache.remove(&ns);
                        }
                    }
                }
                _ => {}
            }
        }

        Self::adjust_derived_index_state_after_replace(engine, previous, current)?;
        Self::adjust_text_index_state_after_replace(engine, previous, current, text_report)?;
        crate::metrics::record_text_postings_written(text_report.postings_written);
        Ok(())
    }

    fn adjust_derived_index_state_after_replace(
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
    ) -> Result<()> {
        let Some(mut state) = Self::load_derived_index_state(engine)? else {
            return Ok(());
        };
        if state.schema_version != DERIVED_INDEX_SCHEMA_VERSION {
            return Ok(());
        }

        match (previous, current) {
            (None, Some(current)) => {
                state.record_count = state.record_count.saturating_add(1);
                state.namespace_entries = state.namespace_entries.saturating_add(1);
                state.payload_entries = state
                    .payload_entries
                    .saturating_add(current.metadata.len() as u64);
            }
            (Some(previous), None) => {
                state.record_count = state.record_count.saturating_sub(1);
                state.namespace_entries = state.namespace_entries.saturating_sub(1);
                state.payload_entries = state
                    .payload_entries
                    .saturating_sub(previous.metadata.len() as u64);
            }
            (Some(previous), Some(current)) => {
                state.payload_entries = state
                    .payload_entries
                    .saturating_sub(previous.metadata.len() as u64)
                    .saturating_add(current.metadata.len() as u64);
            }
            (None, None) => {}
        }

        Self::write_derived_index_state(engine, &state)
    }
}
