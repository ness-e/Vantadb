use super::super::builder::VantaEmbedded;
use super::{memory_record_from_node, memory_record_from_node_include_expired, now_ms};
use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::node::UnifiedNode;
use crate::storage::StorageEngine;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tracing;

impl VantaEmbedded {
    pub(crate) fn ensure_text_index_current_with(
        &self,
        engine: &Arc<StorageEngine>,
        nodes: &[UnifiedNode],
    ) -> Result<()> {
        let state = match Self::load_text_index_state(engine) {
            Ok(state) => state,
            Err(_) => {
                crate::metrics::record_text_index_repair();
                self.rebuild_text_index_with_report()?;
                return Ok(());
            }
        };

        let expected = Self::expected_text_index_counts_from(nodes);
        let current = Self::current_text_index_counts(engine)?;

        let has_state = state.is_some();
        let needs_rebuild = match &state {
            Some(state) => {
                !Self::text_index_state_matches_spec(state)
                    || state.record_count != expected.record_count
                    || state.posting_entries != current.posting_entries
                    || state.posting_entries != expected.posting_entries
                    || state.doc_stats_entries != current.doc_stats_entries
                    || state.doc_stats_entries != expected.doc_stats_entries
                    || state.term_stats_entries != current.term_stats_entries
                    || state.term_stats_entries != expected.term_stats_entries
                    || state.namespace_stats_entries != current.namespace_stats_entries
                    || state.namespace_stats_entries != expected.namespace_stats_entries
                    || current.posting_entries != expected.posting_entries
                    || current.doc_stats_entries != expected.doc_stats_entries
                    || current.term_stats_entries != expected.term_stats_entries
                    || current.namespace_stats_entries != expected.namespace_stats_entries
                    || current.unknown_entries != 0
            }
            None => {
                expected.record_count > 0
                    || current.posting_entries > 0
                    || current.doc_stats_entries > 0
                    || current.term_stats_entries > 0
                    || current.namespace_stats_entries > 0
                    || current.unknown_entries > 0
            }
        };

        if needs_rebuild {
            crate::metrics::record_text_index_repair();
            self.rebuild_text_index_with_report()?;
        } else if !has_state {
            Self::write_text_index_state(engine, &Self::fresh_text_index_state(expected))?;
        }

        Ok(())
    }

    pub(crate) fn load_text_index_state(
        engine: &StorageEngine,
    ) -> Result<Option<super::TextIndexState>> {
        let Some(bytes) = engine.get_from_partition(
            BackendPartition::InternalMetadata,
            super::TEXT_INDEX_STATE_KEY,
        )?
        else {
            return Ok(None);
        };
        postcard::from_bytes(&bytes)
            .map(Some)
            .map_err(VantaError::serialization)
    }

    pub(crate) fn write_text_index_state(
        engine: &StorageEngine,
        state: &super::TextIndexState,
    ) -> Result<()> {
        let bytes = postcard::to_allocvec(state).map_err(VantaError::serialization)?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            super::TEXT_INDEX_STATE_KEY,
            &bytes,
        )
    }

    pub(crate) fn fresh_text_index_state(counts: super::TextIndexCounts) -> super::TextIndexState {
        let spec = crate::text_index::TextIndexSpec::default();
        super::TextIndexState {
            schema_version: spec.schema_version,
            tokenizer: spec.tokenizer.name.to_string(),
            tokenizer_version: spec.tokenizer.version,
            key_format: spec.key_format.to_string(),
            rebuilt_at_ms: now_ms(),
            record_count: counts.record_count,
            posting_entries: counts.posting_entries,
            doc_stats_entries: counts.doc_stats_entries,
            term_stats_entries: counts.term_stats_entries,
            namespace_stats_entries: counts.namespace_stats_entries,
        }
    }

    pub(crate) fn text_index_state_matches_spec(state: &super::TextIndexState) -> bool {
        let spec = crate::text_index::TextIndexSpec::default();
        state.schema_version == spec.schema_version
            && state.tokenizer == spec.tokenizer.name
            && state.tokenizer_version == spec.tokenizer.version
            && state.key_format == spec.key_format
    }

    fn expected_text_index_counts_from(nodes: &[UnifiedNode]) -> super::TextIndexCounts {
        let mut counts = super::TextIndexCounts::default();
        let mut terms = BTreeSet::new();
        let mut namespaces = BTreeSet::new();

        for node in nodes {
            if let Some(record) = memory_record_from_node_include_expired(node) {
                counts.record_count += 1;
                counts.posting_entries += crate::text_index::posting_count(&record.payload);
                counts.doc_stats_entries += 1;
                namespaces.insert(record.namespace.clone());
                for token in crate::text_index::unique_tokens(&record.payload) {
                    terms.insert((record.namespace.clone(), token));
                }
            }
        }

        counts.term_stats_entries = terms.len() as u64;
        counts.namespace_stats_entries = namespaces.len() as u64;
        counts
    }

    fn current_text_index_counts(engine: &StorageEngine) -> Result<super::TextIndexCounts> {
        let mut counts = super::TextIndexCounts::default();
        for (key, _value) in engine.scan_partition(BackendPartition::TextIndex)? {
            if !crate::text_index::is_internal_key(&key) {
                counts.posting_entries += 1;
                continue;
            }

            if crate::text_index::is_doc_stats_key(&key) {
                counts.doc_stats_entries += 1;
            } else if crate::text_index::is_term_stats_key(&key) {
                counts.term_stats_entries += 1;
            } else if crate::text_index::is_namespace_stats_key(&key) {
                counts.namespace_stats_entries += 1;
            } else {
                counts.unknown_entries += 1;
            }
        }
        Ok(counts)
    }

    pub(crate) fn load_text_term_stats(
        engine: &StorageEngine,
        namespace: &str,
        token: &str,
    ) -> Result<Option<crate::text_index::TextTermStats>> {
        // Cache-aside: check in-memory cache first
        if let Some(stats) = engine
            .text_stats_cache
            .read()
            .get(&(namespace.to_string(), token.to_string()))
        {
            return Ok(Some(stats.clone()));
        }
        let key = crate::text_index::term_stats_key(namespace, token);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &key)? else {
            return Ok(None);
        };
        let stats =
            crate::text_index::decode_term_stats(&bytes).map_err(VantaError::serialization)?;
        // Populate cache on miss
        engine
            .text_stats_cache
            .write()
            .insert((namespace.to_string(), token.to_string()), stats.clone());
        Ok(Some(stats))
    }

    pub(crate) fn load_text_namespace_stats(
        engine: &StorageEngine,
        namespace: &str,
    ) -> Result<Option<crate::text_index::TextNamespaceStats>> {
        // Cache-aside: check in-memory cache first
        if let Some(stats) = engine.text_ns_cache.read().get(namespace) {
            return Ok(Some(stats.clone()));
        }
        let key = crate::text_index::namespace_stats_key(namespace);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &key)? else {
            return Ok(None);
        };
        let stats =
            crate::text_index::decode_namespace_stats(&bytes).map_err(VantaError::serialization)?;
        // Populate cache on miss
        engine
            .text_ns_cache
            .write()
            .insert(namespace.to_string(), stats.clone());
        Ok(Some(stats))
    }

    pub(crate) fn load_text_doc_stats(
        engine: &StorageEngine,
        namespace: &str,
        key: &str,
    ) -> Result<Option<crate::text_index::TextDocStats>> {
        let dkey = crate::text_index::doc_stats_key(namespace, key);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &dkey)? else {
            return Ok(None);
        };
        crate::text_index::decode_doc_stats(&bytes)
            .map(Some)
            .map_err(VantaError::serialization)
    }

    pub(crate) fn apply_u64_delta(value: u64, delta: i64) -> u64 {
        if delta >= 0 {
            value.saturating_add(delta as u64)
        } else {
            value.saturating_sub(delta.unsigned_abs())
        }
    }

    pub(crate) fn checked_stats_value(value: i128, label: &str) -> Result<u64> {
        if value < 0 {
            return Err(VantaError::ValidationError {
                field: "stats".into(),
                reason: format!("text index {label} would go negative"),
            });
        }
        u64::try_from(value).map_err(|_| VantaError::ValidationError {
            field: "stats".into(),
            reason: format!("text index {label} exceeds supported range"),
        })
    }

    pub(crate) fn text_index_ops_for_replace(
        engine: &StorageEngine,
        previous: Option<&super::VantaMemoryRecord>,
        current: Option<&super::VantaMemoryRecord>,
    ) -> Result<(Vec<BackendWriteOp>, super::TextIndexMutationReport)> {
        let mut ops = Vec::new();
        let mut report = super::TextIndexMutationReport::default();
        let mut term_deltas: BTreeMap<(String, String), i64> = BTreeMap::new();
        let mut namespace_deltas: BTreeMap<String, (i64, i64)> = BTreeMap::new();

        if let Some(previous) = previous {
            let terms = crate::text_index::record_terms(&previous.payload);
            ops.extend(crate::text_index::posting_delete_ops(
                &previous.namespace,
                &previous.key,
                &previous.payload,
            ));
            ops.push(crate::text_index::doc_stats_delete_op(
                &previous.namespace,
                &previous.key,
            ));
            report.doc_stats_delta -= 1;

            for token in terms.token_counts.keys() {
                *term_deltas
                    .entry((previous.namespace.clone(), token.clone()))
                    .or_default() -= 1;
            }
            let namespace_delta = namespace_deltas
                .entry(previous.namespace.clone())
                .or_insert((0, 0));
            namespace_delta.0 -= 1;
            namespace_delta.1 -= i64::from(terms.doc_len);
        }

        if let Some(current) = current {
            let terms = crate::text_index::record_terms(&current.payload);
            let posting_ops = crate::text_index::posting_put_ops(
                &current.namespace,
                &current.key,
                &current.payload,
                current.node_id,
            )?;
            report.postings_written = posting_ops.len() as u64;
            ops.extend(posting_ops);
            ops.push(crate::text_index::doc_stats_put_op(
                &current.namespace,
                &current.key,
                &current.payload,
                current.node_id,
            )?);
            report.doc_stats_delta += 1;

            for token in terms.token_counts.keys() {
                *term_deltas
                    .entry((current.namespace.clone(), token.clone()))
                    .or_default() += 1;
            }
            let namespace_delta = namespace_deltas
                .entry(current.namespace.clone())
                .or_insert((0, 0));
            namespace_delta.0 += 1;
            namespace_delta.1 += i64::from(terms.doc_len);
        }

        for ((namespace, token), delta) in term_deltas {
            if delta == 0 {
                continue;
            }

            let existing = Self::load_text_term_stats(engine, &namespace, &token)?
                .map(|stats| stats.df)
                .unwrap_or(0);
            let next = Self::checked_stats_value(existing as i128 + delta as i128, "df")?;
            match (existing == 0, next == 0) {
                (true, false) => report.term_stats_delta += 1,
                (false, true) => report.term_stats_delta -= 1,
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
                (true, false) => report.namespace_stats_delta += 1,
                (false, true) => report.namespace_stats_delta -= 1,
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

        Ok((ops, report))
    }

    pub(crate) fn parse_term_stats_key(key: &[u8]) -> Option<(String, String)> {
        const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
        const TERM_STATS_TAG: &[u8] = b"term\0";
        let remainder = key
            .strip_prefix(INTERNAL_PREFIX)?
            .strip_prefix(TERM_STATS_TAG)?;
        let pos = remainder.iter().position(|&b| b == 0)?;
        let ns = match String::from_utf8(remainder[..pos].to_vec()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(?e, raw = ?&remainder[..pos], "Invalid UTF-8 namespace in term stats key");
                return None;
            }
        };
        let token = match String::from_utf8(remainder[pos + 1..].to_vec()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(?e, raw = ?&remainder[pos + 1..], "Invalid UTF-8 token in term stats key");
                return None;
            }
        };
        Some((ns, token))
    }

    pub(crate) fn parse_namespace_stats_key(key: &[u8]) -> Option<String> {
        const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
        const NAMESPACE_STATS_TAG: &[u8] = b"ns\0";
        let remainder = key
            .strip_prefix(INTERNAL_PREFIX)?
            .strip_prefix(NAMESPACE_STATS_TAG)?;
        match String::from_utf8(remainder.to_vec()) {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::warn!(?e, raw = ?remainder, "Invalid UTF-8 in namespace stats key");
                None
            }
        }
    }

    pub(crate) fn adjust_text_index_state_after_replace(
        engine: &StorageEngine,
        previous: Option<&super::VantaMemoryRecord>,
        current: Option<&super::VantaMemoryRecord>,
        report: super::TextIndexMutationReport,
    ) -> Result<()> {
        let Some(mut state) = Self::load_text_index_state(engine)? else {
            return Ok(());
        };
        if !Self::text_index_state_matches_spec(&state) {
            return Ok(());
        }

        match (previous, current) {
            (None, Some(current)) => {
                state.record_count = state.record_count.saturating_add(1);
                state.posting_entries = state
                    .posting_entries
                    .saturating_add(crate::text_index::posting_count(&current.payload));
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, report.doc_stats_delta);
            }
            (Some(previous), None) => {
                state.record_count = state.record_count.saturating_sub(1);
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(crate::text_index::posting_count(&previous.payload));
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, report.doc_stats_delta);
            }
            (Some(previous), Some(current)) => {
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(crate::text_index::posting_count(&previous.payload))
                    .saturating_add(crate::text_index::posting_count(&current.payload));
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, report.doc_stats_delta);
            }
            (None, None) => {}
        }
        state.term_stats_entries =
            Self::apply_u64_delta(state.term_stats_entries, report.term_stats_delta);
        state.namespace_stats_entries =
            Self::apply_u64_delta(state.namespace_stats_entries, report.namespace_stats_delta);

        Self::write_text_index_state(engine, &state)
    }

    pub(crate) fn count_memory_records_from(nodes: &[UnifiedNode]) -> u64 {
        let mut count = 0u64;
        for node in nodes {
            if memory_record_from_node(node).is_some() {
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::super::super::builder::VantaEmbedded;
    use crate::node::UnifiedNode;
    use crate::sdk::serialization::{
        FIELD_CREATED_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD, FIELD_UPDATED_AT_MS,
        FIELD_VERSION,
    };
    use crate::sdk::types::*;

    // ─── apply_u64_delta ───────────────────────────────────────

    #[test]
    fn test_apply_u64_delta_positive() {
        assert_eq!(VantaEmbedded::apply_u64_delta(10, 5), 15);
    }

    #[test]
    fn test_apply_u64_delta_negative() {
        assert_eq!(VantaEmbedded::apply_u64_delta(10, -3), 7);
    }

    #[test]
    fn test_apply_u64_delta_saturating_positive() {
        assert_eq!(VantaEmbedded::apply_u64_delta(u64::MAX, 100), u64::MAX);
    }

    #[test]
    fn test_apply_u64_delta_saturating_negative() {
        assert_eq!(VantaEmbedded::apply_u64_delta(2, -5), 0);
    }

    #[test]
    fn test_apply_u64_delta_zero() {
        assert_eq!(VantaEmbedded::apply_u64_delta(42, 0), 42);
    }

    // ─── checked_stats_value ───────────────────────────────────

    #[test]
    fn test_checked_stats_value_positive() {
        assert_eq!(VantaEmbedded::checked_stats_value(42, "test").unwrap(), 42);
    }

    #[test]
    fn test_checked_stats_value_zero() {
        assert_eq!(VantaEmbedded::checked_stats_value(0, "test").unwrap(), 0);
    }

    #[test]
    fn test_checked_stats_value_negative() {
        let err = VantaEmbedded::checked_stats_value(-1, "df").unwrap_err();
        assert!(err.to_string().contains("would go negative"));
    }

    #[test]
    fn test_checked_stats_value_i128_max_as_u64() {
        let val = i128::from(u64::MAX);
        assert_eq!(
            VantaEmbedded::checked_stats_value(val, "test").unwrap(),
            u64::MAX
        );
    }

    // ─── parse_term_stats_key ──────────────────────────────────

    #[test]
    fn test_parse_term_stats_key_valid() {
        let key = b"\xffvanta_text_v3\0term\0myns\0mytoken";
        assert_eq!(
            VantaEmbedded::parse_term_stats_key(key),
            Some(("myns".into(), "mytoken".into()))
        );
    }

    #[test]
    fn test_parse_term_stats_key_invalid_utf8() {
        let key = b"\xffvanta_text_v3\0term\0myns\0\xff\xfe";
        assert_eq!(VantaEmbedded::parse_term_stats_key(key), None);
    }

    #[test]
    fn test_parse_term_stats_key_truncated() {
        assert_eq!(
            VantaEmbedded::parse_term_stats_key(b"\xffvanta_text_v3\0term"),
            None
        );
    }

    #[test]
    fn test_parse_term_stats_key_no_match() {
        assert_eq!(VantaEmbedded::parse_term_stats_key(b"not_a_text_key"), None);
    }

    // ─── parse_namespace_stats_key ─────────────────────────────

    #[test]
    fn test_parse_namespace_stats_key_valid() {
        assert_eq!(
            VantaEmbedded::parse_namespace_stats_key(b"\xffvanta_text_v3\0ns\0myns"),
            Some("myns".into())
        );
    }

    #[test]
    fn test_parse_namespace_stats_key_invalid_utf8() {
        assert_eq!(
            VantaEmbedded::parse_namespace_stats_key(b"\xffvanta_text_v3\0ns\0\xff\xfe"),
            None
        );
    }

    #[test]
    fn test_parse_namespace_stats_key_truncated() {
        assert_eq!(
            VantaEmbedded::parse_namespace_stats_key(b"\xffvanta_text_v3\0ns"),
            None
        );
    }

    // ─── count_memory_records_from ─────────────────────────────

    #[test]
    fn test_count_memory_records_from_empty() {
        assert_eq!(VantaEmbedded::count_memory_records_from(&[]), 0);
    }

    fn make_memory_node(id: u128, namespace: &str, key: &str) -> UnifiedNode {
        use crate::node::FieldValue;
        let mut node = UnifiedNode::new(id);
        node.set_field(FIELD_NAMESPACE, FieldValue::String(namespace.to_string()));
        node.set_field(FIELD_KEY, FieldValue::String(key.to_string()));
        node.set_field(FIELD_PAYLOAD, FieldValue::String("payload".into()));
        node.set_field(FIELD_CREATED_AT_MS, FieldValue::Int(1000));
        node.set_field(FIELD_UPDATED_AT_MS, FieldValue::Int(1000));
        node.set_field(FIELD_VERSION, FieldValue::Int(1));
        node
    }

    #[test]
    fn test_count_memory_records_from_valid() {
        let nodes = vec![
            make_memory_node(1, "ns1", "k1"),
            make_memory_node(2, "ns1", "k2"),
        ];
        assert_eq!(VantaEmbedded::count_memory_records_from(&nodes), 2);
    }

    #[test]
    fn test_count_memory_records_from_mixed() {
        let mut dead = make_memory_node(2, "ns1", "k2");
        dead.mark_deleted();
        let nodes = vec![
            make_memory_node(1, "ns1", "k1"),
            dead,
            make_memory_node(3, "ns2", "k3"),
        ];
        assert_eq!(VantaEmbedded::count_memory_records_from(&nodes), 2);
    }

    // ─── fresh_text_index_state ────────────────────────────────

    #[test]
    fn test_fresh_text_index_state() {
        let counts = TextIndexCounts {
            record_count: 5,
            posting_entries: 20,
            doc_stats_entries: 5,
            term_stats_entries: 3,
            namespace_stats_entries: 2,
            ..Default::default()
        };
        let state = VantaEmbedded::fresh_text_index_state(counts);
        assert_eq!(state.record_count, 5);
        assert_eq!(state.posting_entries, 20);
        assert_eq!(state.doc_stats_entries, 5);
        assert_eq!(state.term_stats_entries, 3);
        assert_eq!(state.namespace_stats_entries, 2);
        assert!(state.rebuilt_at_ms > 0);
    }

    // ─── text_index_state_matches_spec ─────────────────────────

    #[test]
    fn test_text_index_state_matches_spec_true() {
        let counts = TextIndexCounts::default();
        let state = VantaEmbedded::fresh_text_index_state(counts);
        assert!(VantaEmbedded::text_index_state_matches_spec(&state));
    }

    #[test]
    fn test_text_index_state_matches_spec_bad_schema() {
        let counts = TextIndexCounts::default();
        let mut state = VantaEmbedded::fresh_text_index_state(counts);
        state.schema_version = 999;
        assert!(!VantaEmbedded::text_index_state_matches_spec(&state));
    }
}
