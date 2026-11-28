//! Derived index state management for `Embedded`.

use super::super::builder::Embedded;
use super::super::types::*;
use super::{
    namespace_index_key, node_id_bytes, now_ms, payload_index_key, DERIVED_INDEX_SCHEMA_VERSION,
    SPARSE_INDEX_SCHEMA_VERSION,
};
use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Error, Result};
use crate::node::UnifiedNode;
use crate::storage::StorageEngine;
use std::collections::BTreeMap;
use std::sync::Arc;

impl Embedded {
    /// Reconcile derived, text, and sparse index state against the current
    /// node set. Idempotent: rebuilds only when state is missing, schema
    /// mismatched, or counts disagree; writes fresh empty state otherwise.
    ///
    /// Used by `open_with_config` and by server entrypoints that open a
    /// raw `StorageEngine` (e.g. the MCP stdio server) so that query paths
    /// (`text_query`, hybrid search, text filters) work on fresh databases.
    pub fn ensure_indexes_current(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        // FIND-812: the row set, not `scan_nodes()` — a row whose side storage
        // is corrupt is still a row, and the counts below it decide whether a
        // rebuild runs. Counting it as absent made `get()` and the rebuild
        // disagree forever.
        let nodes = engine.scan_rows_for_rebuild()?.nodes;

        self.ensure_derived_indexes_current_with(&engine, &nodes)?;
        self.ensure_text_index_current_with(&engine, &nodes)?;
        self.ensure_sparse_index_current_with(&engine, &nodes)?;

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
            .map_err(Error::serialization)
    }

    pub(crate) fn write_derived_index_state(
        engine: &StorageEngine,
        state: &DerivedIndexState,
    ) -> Result<()> {
        let bytes = postcard::to_allocvec(state).map_err(Error::serialization)?;
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

    pub(crate) fn derived_put_ops(record: &MemoryRecord) -> Result<Vec<BackendWriteOp>> {
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

    pub(crate) fn derived_delete_ops(record: &MemoryRecord) -> Result<Vec<BackendWriteOp>> {
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
        previous: Option<&MemoryRecord>,
        current: Option<&MemoryRecord>,
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
        let sparse_ops = super::impl_sparse_index::sparse_index_ops_for_replace(previous, current)?;
        ops.extend(sparse_ops);
        if ops.is_empty() {
            return Ok(());
        }
        engine.write_backend_batch(ops.clone())?;

        Self::apply_text_stats_cache_updates(engine, &ops);

        Self::adjust_derived_index_state_after_replace(engine, previous, current)?;
        Self::adjust_text_index_state_after_replace(engine, previous, current, text_report)?;
        Self::adjust_sparse_index_state_after_replace(engine, previous, current)?;
        crate::metrics::record_text_postings_written(text_report.postings_written);
        Ok(())
    }

    /// Reflect committed TextIndex write ops in the in-memory stats caches
    /// (`engine.cache.text_stats` / `text_ns`), with the watermark eviction
    /// policy. Shared by `replace_derived_indexes`, its batch variant
    /// (FIND-640) and `purge_expired` — the cache update must run AFTER the
    /// backend batch succeeded (a failed write must not poison the cache).
    pub(crate) fn apply_text_stats_cache_updates(engine: &StorageEngine, ops: &[BackendWriteOp]) {
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition: BackendPartition::TextIndex,
                    key,
                    value,
                } => {
                    if crate::text_index::is_term_stats_key(key) {
                        if let Some((ns, token)) = Self::parse_term_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_term_stats(value) {
                                let mut guard = engine.cache.text_stats.write();
                                guard.insert((ns, token), stats);
                                // ponytail: watermark eviction — drop first half if over limit
                                if guard.len() > crate::config::MAX_TEXT_STATS_CACHE {
                                    let keys: Vec<_> =
                                        guard.keys().take(guard.len() / 2).cloned().collect();
                                    for k in keys {
                                        guard.remove(&k);
                                    }
                                }
                            }
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_namespace_stats(value) {
                                let mut guard = engine.cache.text_ns.write();
                                guard.insert(ns, stats);
                                // ponytail: watermark eviction — drop first half if over limit
                                if guard.len() > crate::config::MAX_TEXT_NS_CACHE {
                                    let keys: Vec<_> =
                                        guard.keys().take(guard.len() / 2).cloned().collect();
                                    for k in keys {
                                        guard.remove(&k);
                                    }
                                }
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
                            let mut guard = engine.cache.text_stats.write();
                            guard.remove(&(ns, token));
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            let mut guard = engine.cache.text_ns.write();
                            guard.remove(&ns);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// FIND-640: batch counterpart of [`Self::replace_derived_indexes`] —
    /// computes the derived/text/sparse ops for a whole `put_batch` and
    /// commits them in **one** `write_backend_batch`, so the index layer is
    /// atomic across the batch: a failure can no longer leave a prefix of
    /// the records' index entries applied while the rest are missing.
    ///
    /// Stats resolution mirrors `purge_expired`: term/namespace deltas are
    /// netted across ALL (previous, current) pairs before resolving against
    /// the stored stats. This is equivalent to the sequential per-record
    /// replaces the old chunk loop ran — every intermediate write landed in
    /// the same atomic batch anyway — and correctly nets in-batch duplicate
    /// keys (delete-side + put-side cancel where fields overlap).
    ///
    /// `previous[i]` is the delete side for `records[i]` (`None` = fresh
    /// insert). The engine node commit must already have happened; this
    /// method only touches the derived index partitions.
    pub(crate) fn replace_derived_indexes_batch(
        &self,
        engine: &StorageEngine,
        previous: &[Option<MemoryRecord>],
        records: &[MemoryRecord],
    ) -> Result<()> {
        debug_assert_eq!(
            previous.len(),
            records.len(),
            "previous/records parallel arrays must match"
        );
        if records.is_empty() {
            return Ok(());
        }

        let mut ops = Vec::new();
        // Number of genuinely-new records (no delete side) — the
        // record_count/namespace_entries delta for the index states.
        let mut fresh_records = 0u64;
        let mut payload_entries_delta: i64 = 0;
        let mut postings_written = 0u64;
        let mut posting_entries_delta: i64 = 0;
        let mut doc_stats_delta: i64 = 0;
        let mut sparse_postings_delta: i64 = 0;
        let mut term_deltas: BTreeMap<(String, String), i64> = BTreeMap::new();
        let mut namespace_deltas: BTreeMap<String, (i64, i64)> = BTreeMap::new();

        for (prev, record) in previous.iter().zip(records.iter()) {
            if let Some(prev) = prev {
                ops.extend(Self::derived_delete_ops(prev)?);
                ops.extend(crate::text_index::posting_delete_ops(
                    &prev.namespace,
                    &prev.key,
                    &prev.payload,
                ));
                ops.push(crate::text_index::doc_stats_delete_op(
                    &prev.namespace,
                    &prev.key,
                ));
                doc_stats_delta -= 1;
                posting_entries_delta -= crate::text_index::posting_count(&prev.payload) as i64;
                payload_entries_delta -= prev.metadata.len() as i64;
                sparse_postings_delta -=
                    super::impl_sparse_index::sparse_posting_count(prev) as i64;
                let terms = crate::text_index::record_terms(&prev.payload);
                for token in terms.token_counts.keys() {
                    *term_deltas
                        .entry((prev.namespace.clone(), token.clone()))
                        .or_default() -= 1;
                }
                let ns_delta = namespace_deltas
                    .entry(prev.namespace.clone())
                    .or_insert((0, 0));
                ns_delta.0 -= 1;
                ns_delta.1 -= i64::from(terms.doc_len);
            } else {
                fresh_records += 1;
            }

            ops.extend(Self::derived_put_ops(record)?);
            let posting_ops = crate::text_index::posting_put_ops(
                &record.namespace,
                &record.key,
                &record.payload,
                record.node_id,
            )?;
            postings_written += posting_ops.len() as u64;
            ops.extend(posting_ops);
            ops.push(crate::text_index::doc_stats_put_op(
                &record.namespace,
                &record.key,
                &record.payload,
                record.node_id,
            )?);
            doc_stats_delta += 1;
            posting_entries_delta += crate::text_index::posting_count(&record.payload) as i64;
            payload_entries_delta += record.metadata.len() as i64;
            sparse_postings_delta += super::impl_sparse_index::sparse_posting_count(record) as i64;
            let terms = crate::text_index::record_terms(&record.payload);
            for token in terms.token_counts.keys() {
                *term_deltas
                    .entry((record.namespace.clone(), token.clone()))
                    .or_default() += 1;
            }
            let ns_delta = namespace_deltas
                .entry(record.namespace.clone())
                .or_insert((0, 0));
            ns_delta.0 += 1;
            ns_delta.1 += i64::from(terms.doc_len);

            ops.extend(super::impl_sparse_index::sparse_index_ops_for_replace(
                prev.as_ref(),
                Some(record),
            )?);
        }

        // Netted term stats → ops (identical resolution to
        // `text_index_ops_for_replace` and `purge_expired`).
        let mut term_stats_delta: i64 = 0;
        for ((namespace, token), delta) in term_deltas {
            if delta == 0 {
                continue;
            }
            let existing = Self::load_text_term_stats(engine, &namespace, &token)?
                .map(|stats| stats.df)
                .unwrap_or(0);
            let next = Self::checked_stats_value(existing as i128 + delta as i128, "df")?;
            match (existing == 0, next == 0) {
                (true, false) => term_stats_delta += 1,
                (false, true) => term_stats_delta -= 1,
                _ => {}
            }
            if next == 0 {
                ops.push(crate::text_index::term_stats_delete_op(&namespace, &token));
            } else {
                ops.push(crate::text_index::term_stats_put_op(
                    &namespace, &token, next,
                )?);
            }
        }

        let mut namespace_stats_delta: i64 = 0;
        for (namespace, (doc_delta, len_delta)) in namespace_deltas {
            if doc_delta == 0 && len_delta == 0 {
                continue;
            }
            let existing = Self::load_text_namespace_stats(engine, &namespace)?.unwrap_or(
                crate::text_index::TextNamespaceStats {
                    doc_count: 0,
                    total_doc_len: 0,
                },
            );
            let next_doc_count = Self::checked_stats_value(
                existing.doc_count as i128 + doc_delta as i128,
                "doc_count",
            )?;
            let next_total_doc_len = Self::checked_stats_value(
                existing.total_doc_len as i128 + len_delta as i128,
                "total_doc_len",
            )?;
            match (existing.doc_count == 0, next_doc_count == 0) {
                (true, false) => namespace_stats_delta += 1,
                (false, true) => namespace_stats_delta -= 1,
                _ => {}
            }
            if next_doc_count == 0 {
                ops.push(crate::text_index::namespace_stats_delete_op(&namespace));
            } else {
                ops.push(crate::text_index::namespace_stats_put_op(
                    &namespace,
                    &crate::text_index::TextNamespaceStats {
                        doc_count: next_doc_count,
                        total_doc_len: next_total_doc_len,
                    },
                )?);
            }
        }

        // ── Single atomic commit for the whole batch's index ops ──
        if !ops.is_empty() {
            engine.write_backend_batch(ops.clone())?;
        }
        // Cache only after the write succeeded (a failed batch must not
        // poison the cache — see `apply_text_stats_cache_updates`).
        Self::apply_text_stats_cache_updates(engine, &ops);

        if let Some(mut state) = Self::load_derived_index_state(engine)? {
            if state.schema_version == DERIVED_INDEX_SCHEMA_VERSION {
                state.record_count = state.record_count.saturating_add(fresh_records);
                state.namespace_entries = state.namespace_entries.saturating_add(fresh_records);
                state.payload_entries =
                    Self::apply_u64_delta(state.payload_entries, payload_entries_delta);
                Self::write_derived_index_state(engine, &state)?;
            }
        }

        if let Some(mut state) = Self::load_text_index_state(engine)? {
            if Self::text_index_state_matches_spec(&state) {
                state.record_count = state.record_count.saturating_add(fresh_records);
                state.posting_entries =
                    Self::apply_u64_delta(state.posting_entries, posting_entries_delta);
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, doc_stats_delta);
                state.term_stats_entries =
                    Self::apply_u64_delta(state.term_stats_entries, term_stats_delta);
                state.namespace_stats_entries =
                    Self::apply_u64_delta(state.namespace_stats_entries, namespace_stats_delta);
                Self::write_text_index_state(engine, &state)?;
            }
        }

        if let Some(mut state) = Self::load_sparse_index_state(engine)? {
            if state.schema_version == SPARSE_INDEX_SCHEMA_VERSION {
                state.record_count = state.record_count.saturating_add(fresh_records);
                state.posting_entries =
                    Self::apply_u64_delta(state.posting_entries, sparse_postings_delta);
                Self::write_sparse_index_state(engine, &state)?;
            }
        }

        crate::metrics::record_text_postings_written(postings_written);
        Ok(())
    }

    fn adjust_derived_index_state_after_replace(
        engine: &StorageEngine,
        previous: Option<&MemoryRecord>,
        current: Option<&MemoryRecord>,
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use crate::backend::{BackendPartition, BackendWriteOp};
    use crate::sdk::serialization::{namespace_index_key, node_id_bytes};
    use crate::sdk::types::*;
    use crate::sdk::Embedded;

    fn sample_record(namespace: &str, key: &str) -> MemoryRecord {
        MemoryRecord {
            namespace: namespace.into(),
            key: key.into(),
            payload: "test".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 100,
            updated_at_ms: 100,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id(namespace, key),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        }
    }

    fn record_with_metadata(namespace: &str, key: &str) -> MemoryRecord {
        let mut meta = MemoryMetadata::new();
        meta.insert("color".into(), Value::String("blue".into()));
        meta.insert("size".into(), Value::Int(42));
        MemoryRecord {
            metadata: meta,
            ..sample_record(namespace, key)
        }
    }

    // ─── derived_put_ops ───────────────────────────────────────

    #[test]
    fn test_derived_put_ops_no_metadata() {
        let record = sample_record("ns", "k");
        let ops = Embedded::derived_put_ops(&record).unwrap();

        assert_eq!(ops.len(), 1);
        match &ops[0] {
            BackendWriteOp::Put {
                partition,
                key,
                value,
            } => {
                assert_eq!(*partition, BackendPartition::NamespaceIndex);
                assert_eq!(key, &namespace_index_key("ns", "k"));
                assert_eq!(value, &node_id_bytes(record.node_id));
            }
            _ => panic!("expected Put op"),
        }
    }

    #[test]
    fn test_derived_put_ops_with_metadata() {
        let record = record_with_metadata("ns", "k");
        let ops = Embedded::derived_put_ops(&record).unwrap();

        // 1 namespace + 2 payload entries
        assert_eq!(ops.len(), 3);

        // Verify namespace entry
        match &ops[0] {
            BackendWriteOp::Put { partition, .. } => {
                assert_eq!(*partition, BackendPartition::NamespaceIndex);
            }
            _ => panic!("first op should be namespace Put"),
        }

        // Verify payload entries: both should be PayloadIndex partition
        for op in &ops[1..] {
            match op {
                BackendWriteOp::Put { partition, .. } => {
                    assert_eq!(*partition, BackendPartition::PayloadIndex);
                }
                _ => panic!("payload ops should be Put"),
            }
        }
    }

    #[test]
    fn test_derived_put_ops_list_metadata() {
        let mut meta = MemoryMetadata::new();
        meta.insert(
            "tags".into(),
            Value::ListString(vec!["a".into(), "b".into()]),
        );
        let record = MemoryRecord {
            metadata: meta,
            ..sample_record("ns", "k")
        };
        let ops = Embedded::derived_put_ops(&record).unwrap();
        // 1 namespace + 2 flattened list entries
        assert_eq!(ops.len(), 3);
    }

    // ─── derived_delete_ops ────────────────────────────────────

    #[test]
    fn test_derived_delete_ops_no_metadata() {
        let record = sample_record("ns", "k");
        let ops = Embedded::derived_delete_ops(&record).unwrap();

        assert_eq!(ops.len(), 1);
        match &ops[0] {
            BackendWriteOp::Delete { partition, key } => {
                assert_eq!(*partition, BackendPartition::NamespaceIndex);
                assert_eq!(key, &namespace_index_key("ns", "k"));
            }
            _ => panic!("expected Delete op"),
        }
    }

    #[test]
    fn test_derived_delete_ops_with_metadata() {
        let record = record_with_metadata("ns", "k");
        let ops = Embedded::derived_delete_ops(&record).unwrap();

        assert_eq!(ops.len(), 3);
        match &ops[0] {
            BackendWriteOp::Delete { partition, .. } => {
                assert_eq!(*partition, BackendPartition::NamespaceIndex);
            }
            _ => panic!("first op should be namespace Delete"),
        }
        for op in &ops[1..] {
            match op {
                BackendWriteOp::Delete { partition, .. } => {
                    assert_eq!(*partition, BackendPartition::PayloadIndex);
                }
                _ => panic!("payload ops should be Delete"),
            }
        }
    }

    // ─── derived index state (load/write) ──────────────────────

    #[test]
    fn test_load_derived_index_state_none() {
        let engine = crate::storage::StorageEngine::open_with_config(
            ":memory:",
            Some(crate::config::Config {
                backend_kind: crate::backend::BackendKind::InMemory,
                ..Default::default()
            }),
        )
        .unwrap();
        let state = Embedded::load_derived_index_state(&engine).unwrap();
        assert!(state.is_none());
    }

    #[test]
    fn test_write_then_load_derived_index_state() {
        let engine = crate::storage::StorageEngine::open_with_config(
            ":memory:",
            Some(crate::config::Config {
                backend_kind: crate::backend::BackendKind::InMemory,
                ..Default::default()
            }),
        )
        .unwrap();
        let state = DerivedIndexState {
            schema_version: 1,
            rebuilt_at_ms: 1234,
            record_count: 10,
            namespace_entries: 10,
            payload_entries: 25,
        };
        Embedded::write_derived_index_state(&engine, &state).unwrap();
        let loaded = Embedded::load_derived_index_state(&engine).unwrap();
        assert_eq!(loaded, Some(state));
    }

    #[test]
    fn test_overwrite_derived_index_state() {
        let engine = crate::storage::StorageEngine::open_with_config(
            ":memory:",
            Some(crate::config::Config {
                backend_kind: crate::backend::BackendKind::InMemory,
                ..Default::default()
            }),
        )
        .unwrap();
        let state1 = DerivedIndexState {
            schema_version: 1,
            rebuilt_at_ms: 100,
            record_count: 5,
            namespace_entries: 5,
            payload_entries: 5,
        };
        let state2 = DerivedIndexState {
            schema_version: 1,
            rebuilt_at_ms: 200,
            record_count: 20,
            namespace_entries: 20,
            payload_entries: 50,
        };
        Embedded::write_derived_index_state(&engine, &state1).unwrap();
        Embedded::write_derived_index_state(&engine, &state2).unwrap();
        let loaded = Embedded::load_derived_index_state(&engine).unwrap();
        assert_eq!(loaded, Some(state2));
    }
}
