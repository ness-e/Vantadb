//! Rebuild operations and text index audit for `VantaEmbedded`.

use super::super::builder::VantaEmbedded;
use super::super::types::*;
use super::{
    memory_record_from_node, memory_record_from_node_include_expired, now_ms, TextIndexCounts,
    DERIVED_INDEX_SCHEMA_VERSION,
};
use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::storage::StorageEngine;
use std::collections::{BTreeMap, BTreeSet};
use web_time::Instant;

impl VantaEmbedded {
    pub(crate) fn rebuild_derived_indexes_with_report(&self) -> Result<DerivedIndexRebuildReport> {
        let started = Instant::now();
        let engine = self.engine_handle()?;
        let mut ops = Vec::new();
        let mut record_count = 0u64;
        let mut namespace_entries = 0u64;
        let mut payload_entries = 0u64;

        for (key, _value) in engine.scan_partition(BackendPartition::NamespaceIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::NamespaceIndex,
                key,
            });
        }
        for (key, _value) in engine.scan_partition(BackendPartition::PayloadIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::PayloadIndex,
                key,
            });
        }
        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(&node) {
                record_count += 1;
                namespace_entries += 1;
                payload_entries += record.metadata.len() as u64;
                ops.extend(Self::derived_put_ops(&record)?);
            }
        }

        if !ops.is_empty() {
            engine.write_backend_batch(ops)?;
        }

        Self::write_derived_index_state(
            &engine,
            &DerivedIndexState {
                schema_version: DERIVED_INDEX_SCHEMA_VERSION,
                rebuilt_at_ms: now_ms(),
                record_count,
                namespace_entries,
                payload_entries,
            },
        )?;

        let report = DerivedIndexRebuildReport {
            record_count,
            namespace_entries,
            payload_entries,
            duration_ms: started.elapsed().as_millis() as u64,
        };
        crate::metrics::record_derived_rebuild(report.duration_ms);
        Ok(report)
    }

    pub(crate) fn rebuild_derived_indexes(&self) -> Result<()> {
        self.rebuild_derived_indexes_with_report().map(|_| ())
    }

    pub(crate) fn rebuild_text_index_with_report(&self) -> Result<TextIndexRebuildReport> {
        let started = Instant::now();
        let engine = self.engine_handle()?;

        {
            let mut cache = engine.text_stats_cache.write();
            cache.clear();
        }
        {
            let mut cache = engine.text_ns_cache.write();
            cache.clear();
        }

        let mut ops = Vec::new();
        let mut counts = TextIndexCounts::default();
        let mut term_stats: BTreeMap<(String, String), u64> = BTreeMap::new();
        let mut namespace_stats: BTreeMap<String, crate::text_index::TextNamespaceStats> =
            BTreeMap::new();

        for (key, _value) in engine.scan_partition(BackendPartition::TextIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::TextIndex,
                key,
            });
        }

        // Build the advanced analyzer once for the whole batch; constructing
        // the stemming/stopwords pipeline per record would pay N setups.
        #[cfg(feature = "advanced-tokenizer")]
        let mut analyzer = crate::tokenizer::build_advanced_analyzer(
            &crate::tokenizer::AdvancedTokenizerConfig::default(),
        );

        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node_include_expired(&node) {
                counts.record_count += 1;
                let terms = {
                    #[cfg(feature = "advanced-tokenizer")]
                    {
                        crate::text_index::record_terms_with_analyzer(
                            &mut analyzer,
                            &record.payload,
                        )
                    }
                    #[cfg(not(feature = "advanced-tokenizer"))]
                    {
                        crate::text_index::record_terms(&record.payload)
                    }
                };
                let posting_ops = crate::text_index::posting_ops_from_terms(
                    &record.namespace,
                    &record.key,
                    &terms,
                    record.node_id,
                )?;
                counts.posting_entries += posting_ops.len() as u64;
                ops.extend(posting_ops);
                ops.push(crate::text_index::doc_stats_op_from_terms(
                    &record.namespace,
                    &record.key,
                    terms.doc_len,
                    record.node_id,
                )?);
                counts.doc_stats_entries += 1;

                for token in terms.token_counts.keys() {
                    *term_stats
                        .entry((record.namespace.clone(), token.clone()))
                        .or_default() += 1;
                }
                let namespace = namespace_stats.entry(record.namespace.clone()).or_insert(
                    crate::text_index::TextNamespaceStats {
                        doc_count: 0,
                        total_doc_len: 0,
                    },
                );
                namespace.doc_count += 1;
                namespace.total_doc_len += u64::from(terms.doc_len);
            }
        }

        for ((namespace, token), df) in &term_stats {
            ops.push(crate::text_index::term_stats_put_op(namespace, token, *df)?);
        }
        for (namespace, stats) in &namespace_stats {
            ops.push(crate::text_index::namespace_stats_put_op(namespace, stats)?);
        }
        counts.term_stats_entries = term_stats.len() as u64;
        counts.namespace_stats_entries = namespace_stats.len() as u64;

        if !ops.is_empty() {
            engine.write_backend_batch(ops)?;
        }

        Self::write_text_index_state(&engine, &Self::fresh_text_index_state(counts))?;

        let report = TextIndexRebuildReport {
            record_count: counts.record_count,
            posting_entries: counts.posting_entries,
            doc_stats_entries: counts.doc_stats_entries,
            term_stats_entries: counts.term_stats_entries,
            namespace_stats_entries: counts.namespace_stats_entries,
            duration_ms: started.elapsed().as_millis() as u64,
        };
        crate::metrics::record_text_index_rebuild(report.duration_ms, report.posting_entries);
        Ok(report)
    }

    pub(crate) fn rebuild_text_index(&self) -> Result<()> {
        self.rebuild_text_index_with_report().map(|_| ())
    }

    fn expected_text_index_entries(
        engine: &StorageEngine,
        namespace_filter: Option<&str>,
    ) -> Result<ExpectedTextIndexEntries> {
        let mut audit = ExpectedTextIndexEntries::default();
        let mut term_stats: BTreeMap<(String, String), u64> = BTreeMap::new();
        let mut namespace_stats: BTreeMap<String, crate::text_index::TextNamespaceStats> =
            BTreeMap::new();

        for node in engine.scan_nodes()? {
            audit.records_scanned += 1;
            if let Some(record) = memory_record_from_node_include_expired(&node) {
                if matches!(namespace_filter, Some(namespace) if record.namespace != namespace) {
                    continue;
                }
                audit.counts.record_count += 1;
                audit.namespaces.insert(record.namespace.clone());
                let terms = crate::text_index::record_terms(&record.payload);
                for (token, tf) in &terms.token_counts {
                    audit.entries.insert(
                        crate::text_index::posting_key(&record.namespace, token, &record.key),
                        crate::text_index::posting_value(
                            record.node_id,
                            *tf,
                            terms
                                .token_positions
                                .get(token)
                                .map(Vec::as_slice)
                                .unwrap_or(&[]),
                        )?,
                    );
                    audit.counts.posting_entries += 1;
                    *term_stats
                        .entry((record.namespace.clone(), token.clone()))
                        .or_default() += 1;
                }
                audit.entries.insert(
                    crate::text_index::doc_stats_key(&record.namespace, &record.key),
                    crate::text_index::doc_stats_value(record.node_id, terms.doc_len)?,
                );
                audit.counts.doc_stats_entries += 1;
                let namespace = namespace_stats.entry(record.namespace.clone()).or_insert(
                    crate::text_index::TextNamespaceStats {
                        doc_count: 0,
                        total_doc_len: 0,
                    },
                );
                namespace.doc_count += 1;
                namespace.total_doc_len += u64::from(terms.doc_len);
            }
        }

        for ((namespace, token), df) in term_stats {
            audit.entries.insert(
                crate::text_index::term_stats_key(&namespace, &token),
                crate::text_index::term_stats_value(df)?,
            );
        }
        for (namespace, stats) in namespace_stats {
            audit.entries.insert(
                crate::text_index::namespace_stats_key(&namespace),
                crate::text_index::namespace_stats_value(stats.doc_count, stats.total_doc_len)?,
            );
        }

        audit.counts.term_stats_entries = audit
            .entries
            .keys()
            .filter(|key| crate::text_index::is_term_stats_key(key))
            .count() as u64;
        audit.counts.namespace_stats_entries = audit
            .entries
            .keys()
            .filter(|key| crate::text_index::is_namespace_stats_key(key))
            .count() as u64;

        Ok(audit)
    }

    fn text_index_value_readable(key: &[u8], value: &[u8]) -> bool {
        if !crate::text_index::is_internal_key(key) {
            return crate::text_index::decode_posting(value).is_ok();
        }

        if crate::text_index::is_doc_stats_key(key) {
            crate::text_index::decode_doc_stats(value).is_ok()
        } else if crate::text_index::is_term_stats_key(key) {
            crate::text_index::decode_term_stats(value).is_ok()
        } else if crate::text_index::is_namespace_stats_key(key) {
            crate::text_index::decode_namespace_stats(value).is_ok()
        } else {
            false
        }
    }

    fn text_index_state_audit_status(
        engine: &StorageEngine,
        expected_counts: TextIndexCounts,
        namespace_filter: Option<&str>,
    ) -> (bool, String) {
        let state = match Self::load_text_index_state(engine) {
            Ok(Some(state)) => state,
            Ok(None) => return (false, "missing".to_string()),
            Err(err) => return (false, format!("decode_error: {err}")),
        };

        if !Self::text_index_state_matches_spec(&state) {
            return (false, "incompatible".to_string());
        }

        if namespace_filter.is_none()
            && (state.record_count != expected_counts.record_count
                || state.posting_entries != expected_counts.posting_entries
                || state.doc_stats_entries != expected_counts.doc_stats_entries
                || state.term_stats_entries != expected_counts.term_stats_entries
                || state.namespace_stats_entries != expected_counts.namespace_stats_entries)
        {
            return (false, "count_mismatch".to_string());
        }

        (true, "current".to_string())
    }

    pub(crate) fn build_text_index_audit_report_deep(
        engine: &StorageEngine,
        namespace_filter: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        let started = Instant::now();
        let spec = crate::text_index::TextIndexSpec::default();
        let expected = Self::expected_text_index_entries(engine, namespace_filter)?;
        let actual: BTreeMap<Vec<u8>, Vec<u8>> = engine
            .scan_partition(BackendPartition::TextIndex)?
            .into_iter()
            .filter(|(key, _value)| {
                namespace_filter
                    .map(|namespace| {
                        crate::text_index::text_index_key_belongs_to_namespace(key, namespace)
                    })
                    .unwrap_or(true)
            })
            .collect();

        let mut missing_entries = 0u64;
        let mut unexpected_entries = 0u64;
        let mut value_mismatches = 0u64;
        let mut unreadable_entries = 0u64;
        let mut position_errors = 0u64;
        let mut tf_errors = 0u64;
        let mut df_errors = 0u64;
        let mut doc_len_errors = 0u64;
        let mut logical_corruptions = 0u64;

        for (key, value) in &expected.entries {
            match actual.get(key) {
                Some(actual_value) if actual_value == value => {}
                Some(actual_value) => {
                    value_mismatches += 1;
                    if !Self::text_index_value_readable(key, actual_value) {
                        unreadable_entries += 1;
                    } else if crate::text_index::is_doc_stats_key(key) {
                        if let (Ok(expected_stats), Ok(actual_stats)) = (
                            crate::text_index::decode_doc_stats(value),
                            crate::text_index::decode_doc_stats(actual_value),
                        ) {
                            if expected_stats.doc_len != actual_stats.doc_len {
                                doc_len_errors += 1;
                            } else {
                                logical_corruptions += 1;
                            }
                        }
                    } else if crate::text_index::is_term_stats_key(key) {
                        if let (Ok(expected_stats), Ok(actual_stats)) = (
                            crate::text_index::decode_term_stats(value),
                            crate::text_index::decode_term_stats(actual_value),
                        ) {
                            if expected_stats.df != actual_stats.df {
                                df_errors += 1;
                            } else {
                                logical_corruptions += 1;
                            }
                        }
                    } else if !crate::text_index::is_internal_key(key) {
                        if let (Ok(expected_posting), Ok(actual_posting)) = (
                            crate::text_index::decode_posting(value),
                            crate::text_index::decode_posting(actual_value),
                        ) {
                            if expected_posting.tf != actual_posting.tf {
                                tf_errors += 1;
                            }
                            if expected_posting.positions != actual_posting.positions {
                                position_errors += 1;
                            }
                            if expected_posting.tf == actual_posting.tf
                                && expected_posting.positions == actual_posting.positions
                            {
                                logical_corruptions += 1;
                            }
                        }
                    } else {
                        logical_corruptions += 1;
                    }
                }
                None => missing_entries += 1,
            }
        }
        for key in actual.keys() {
            if !expected.entries.contains_key(key) {
                unexpected_entries += 1;
                if let Some(value) = actual.get(key) {
                    if !Self::text_index_value_readable(key, value) {
                        unreadable_entries += 1;
                    }
                }
            }
        }

        let (state_valid, state_status) =
            Self::text_index_state_audit_status(engine, expected.counts, namespace_filter);
        let state_mismatches = u64::from(!state_valid);
        let mismatches = missing_entries + unexpected_entries + value_mismatches + state_mismatches;
        let passed = mismatches == 0;
        let mut namespaces_audited: Vec<String> = expected.namespaces.into_iter().collect();
        if namespaces_audited.is_empty() {
            if let Some(namespace) = namespace_filter {
                namespaces_audited.push(namespace.to_string());
            }
        }

        let report = VantaTextIndexAuditReport {
            schema_version: spec.schema_version,
            tokenizer: spec.tokenizer.name.to_string(),
            tokenizer_version: spec.tokenizer.version,
            key_format: spec.key_format.to_string(),
            namespace_filter: namespace_filter.map(ToOwned::to_owned),
            namespaces_audited,
            records_scanned: expected.records_scanned,
            expected_entries: expected.entries.len() as u64,
            actual_entries: actual.len() as u64,
            missing_entries,
            unexpected_entries,
            value_mismatches,
            unreadable_entries,
            mismatches,
            deep_audit: true,
            position_errors,
            tf_errors,
            df_errors,
            doc_len_errors,
            logical_corruptions,
            state_valid,
            state_status,
            duration_ms: started.elapsed().as_millis() as u64,
            passed,
            status: if passed {
                "ok".to_string()
            } else {
                "repair_recommended".to_string()
            },
        };
        crate::metrics::record_text_consistency_audit(!report.passed);
        Ok(report)
    }

    pub(crate) fn build_text_index_audit_report_shallow(
        engine: &StorageEngine,
        namespace_filter: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        let started = Instant::now();
        let spec = crate::text_index::TextIndexSpec::default();
        let expected = Self::expected_text_index_entries(engine, namespace_filter)?;

        let (state_valid, state_status) =
            Self::text_index_state_audit_status(engine, expected.counts, namespace_filter);

        let actual: BTreeSet<Vec<u8>> = engine
            .scan_partition(BackendPartition::TextIndex)?
            .into_iter()
            .filter(|(key, _value)| {
                namespace_filter
                    .map(|namespace| {
                        crate::text_index::text_index_key_belongs_to_namespace(key, namespace)
                    })
                    .unwrap_or(true)
            })
            .map(|(key, _value)| key)
            .collect();

        let actual_entries = actual.len() as u64;
        let expected_keys: BTreeSet<&Vec<u8>> = expected.entries.keys().collect();
        let missing_entries = expected_keys
            .iter()
            .filter(|key| !actual.contains(**key))
            .count() as u64;
        let unexpected_entries = actual
            .iter()
            .filter(|key| !expected.entries.contains_key(*key))
            .count() as u64;
        let mismatches = missing_entries + unexpected_entries;

        let passed = state_valid && mismatches == 0;

        let mut namespaces_audited: Vec<String> = expected.namespaces.into_iter().collect();
        if namespaces_audited.is_empty() {
            if let Some(namespace) = namespace_filter {
                namespaces_audited.push(namespace.to_string());
            }
        }

        let report = VantaTextIndexAuditReport {
            schema_version: spec.schema_version,
            tokenizer: spec.tokenizer.name.to_string(),
            tokenizer_version: spec.tokenizer.version,
            key_format: spec.key_format.to_string(),
            namespace_filter: namespace_filter.map(ToOwned::to_owned),
            namespaces_audited,
            records_scanned: expected.records_scanned,
            expected_entries: expected.entries.len() as u64,
            actual_entries,
            missing_entries,
            unexpected_entries,
            value_mismatches: 0,
            unreadable_entries: 0,
            mismatches,
            deep_audit: false,
            position_errors: 0,
            tf_errors: 0,
            df_errors: 0,
            doc_len_errors: 0,
            logical_corruptions: 0,
            state_valid,
            state_status,
            duration_ms: started.elapsed().as_millis() as u64,
            passed,
            status: if passed {
                "ok".to_string()
            } else {
                "repair_recommended".to_string()
            },
        };
        crate::metrics::record_text_consistency_audit(!report.passed);
        Ok(report)
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use crate::backend::BackendPartition;
    use crate::sdk::types::{TextIndexCounts, TextIndexState};
    use crate::sdk::VantaEmbedded;

    // ─── text_index_value_readable ─────────────────────────────

    #[test]
    fn test_text_index_value_readable_posting() {
        // Non-internal key (posting) with valid value
        let key = b"ns\0token\0key";
        let value = crate::text_index::posting_value(42, 3, &[0, 5, 10]).unwrap();
        assert!(VantaEmbedded::text_index_value_readable(key, &value));
    }

    #[test]
    fn test_text_index_value_readable_posting_corrupted() {
        let key = b"ns\0token\0key";
        assert!(!VantaEmbedded::text_index_value_readable(
            key,
            b"not-valid-posting"
        ));
    }

    #[test]
    fn test_text_index_value_readable_term_stats() {
        let key = crate::text_index::term_stats_key("myns", "hello");
        let value = crate::text_index::term_stats_value(5).unwrap();
        assert!(VantaEmbedded::text_index_value_readable(&key, &value));
    }

    #[test]
    fn test_text_index_value_readable_term_stats_corrupted() {
        let key = crate::text_index::term_stats_key("myns", "hello");
        assert!(!VantaEmbedded::text_index_value_readable(&key, &[]));
    }

    #[test]
    fn test_text_index_value_readable_doc_stats() {
        let key = crate::text_index::doc_stats_key("myns", "mykey");
        let value = crate::text_index::doc_stats_value(100, 20).unwrap();
        assert!(VantaEmbedded::text_index_value_readable(&key, &value));
    }

    #[test]
    fn test_text_index_value_readable_namespace_stats() {
        let key = crate::text_index::namespace_stats_key("myns");
        let value = crate::text_index::namespace_stats_value(10, 500).unwrap();
        assert!(VantaEmbedded::text_index_value_readable(&key, &value));
    }

    #[test]
    fn test_text_index_value_readable_unknown_internal_key() {
        let key = b"\xffvanta_text_v3\0unknown\0data";
        let value = b"whatever";
        assert!(!VantaEmbedded::text_index_value_readable(key, value));
    }

    // ─── text_index_state_audit_status ─────────────────────────

    fn in_memory_engine() -> crate::storage::StorageEngine {
        crate::storage::StorageEngine::open_with_config(
            ":memory:",
            Some(crate::config::VantaConfig {
                backend_kind: crate::backend::BackendKind::InMemory,
                ..Default::default()
            }),
        )
        .expect("in-memory engine")
    }

    #[test]
    fn test_text_index_state_audit_status_missing() {
        let engine = in_memory_engine();
        let counts = TextIndexCounts::default();
        let (valid, status) = VantaEmbedded::text_index_state_audit_status(&engine, counts, None);
        assert!(!valid);
        assert_eq!(status, "missing");
    }

    #[test]
    fn test_text_index_state_audit_status_current() {
        let engine = in_memory_engine();
        let counts = TextIndexCounts::default();
        // Write a valid matching state
        let state = VantaEmbedded::fresh_text_index_state(counts);
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();

        let (valid, status) = VantaEmbedded::text_index_state_audit_status(&engine, counts, None);
        assert!(valid);
        assert_eq!(status, "current");
    }

    #[test]
    fn test_text_index_state_audit_status_count_mismatch() {
        let engine = in_memory_engine();
        let state_counts = TextIndexCounts {
            record_count: 5,
            ..Default::default()
        };
        let state = VantaEmbedded::fresh_text_index_state(state_counts);
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();

        // Pass different expected counts
        let expected_counts = TextIndexCounts {
            record_count: 10,
            ..Default::default()
        };
        let (valid, status) =
            VantaEmbedded::text_index_state_audit_status(&engine, expected_counts, None);
        assert!(!valid);
        assert_eq!(status, "count_mismatch");
    }

    #[test]
    fn test_text_index_state_audit_status_decode_error() {
        let engine = in_memory_engine();
        // Garbage bytes that cannot be deserialized as TextIndexState
        engine
            .put_to_partition(
                BackendPartition::InternalMetadata,
                crate::sdk::serialization::TEXT_INDEX_STATE_KEY,
                b"this-is-not-valid-postcard-data",
            )
            .expect("put garbage state");
        let counts = TextIndexCounts::default();
        let (valid, status) = VantaEmbedded::text_index_state_audit_status(&engine, counts, None);
        assert!(!valid);
        assert!(
            status.starts_with("decode_error"),
            "expected decode_error, got {status}"
        );
    }

    #[test]
    fn test_text_index_state_audit_status_incompatible() {
        let engine = in_memory_engine();
        // Write a state with mismatched schema_version/tokenizer
        let bad_state = TextIndexState {
            schema_version: 999,
            tokenizer: "wrong-tokenizer".into(),
            tokenizer_version: 0,
            key_format: "unknown".into(),
            rebuilt_at_ms: 0,
            record_count: 0,
            posting_entries: 0,
            doc_stats_entries: 0,
            term_stats_entries: 0,
            namespace_stats_entries: 0,
        };
        VantaEmbedded::write_text_index_state(&engine, &bad_state).unwrap();
        let counts = TextIndexCounts::default();
        let (valid, status) = VantaEmbedded::text_index_state_audit_status(&engine, counts, None);
        assert!(!valid);
        assert_eq!(status, "incompatible");
    }

    #[test]
    fn test_text_index_state_audit_status_namespace_filter_skips_count_check() {
        let engine = in_memory_engine();
        // State with record_count=5
        let state_counts = TextIndexCounts {
            record_count: 5,
            ..Default::default()
        };
        let state = VantaEmbedded::fresh_text_index_state(state_counts);
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();
        // Expected counts differ (10 vs 5), but namespace_filter is Some(...)
        // so the count_mismatch check is skipped → returns "current"
        let expected_counts = TextIndexCounts {
            record_count: 10,
            ..Default::default()
        };
        let (valid, status) =
            VantaEmbedded::text_index_state_audit_status(&engine, expected_counts, Some("myns"));
        assert!(
            valid,
            "expected valid when namespace_filter skips count check"
        );
        assert_eq!(status, "current");
    }

    // ─── text_index_value_readable (corrupted internal keys) ───

    #[test]
    fn test_text_index_value_readable_doc_stats_corrupted() {
        let key = crate::text_index::doc_stats_key("myns", "mykey");
        assert!(!VantaEmbedded::text_index_value_readable(&key, b""));
    }

    #[test]
    fn test_text_index_value_readable_namespace_stats_corrupted() {
        let key = crate::text_index::namespace_stats_key("myns");
        assert!(!VantaEmbedded::text_index_value_readable(&key, b""));
    }

    // ─── text_index_state_audit_status (individual count mismatches) ──

    #[test]
    fn test_text_index_state_audit_status_posting_entries_mismatch() {
        let engine = in_memory_engine();
        let state_counts = TextIndexCounts {
            posting_entries: 5,
            ..Default::default()
        };
        let state = VantaEmbedded::fresh_text_index_state(state_counts);
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();
        let expected_counts = TextIndexCounts {
            posting_entries: 10,
            ..Default::default()
        };
        let (valid, status) =
            VantaEmbedded::text_index_state_audit_status(&engine, expected_counts, None);
        assert!(!valid);
        assert_eq!(status, "count_mismatch");
    }

    #[test]
    fn test_text_index_state_audit_status_doc_stats_entries_mismatch() {
        let engine = in_memory_engine();
        let state_counts = TextIndexCounts {
            doc_stats_entries: 3,
            ..Default::default()
        };
        let state = VantaEmbedded::fresh_text_index_state(state_counts);
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();
        let expected_counts = TextIndexCounts {
            doc_stats_entries: 7,
            ..Default::default()
        };
        let (valid, status) =
            VantaEmbedded::text_index_state_audit_status(&engine, expected_counts, None);
        assert!(!valid);
        assert_eq!(status, "count_mismatch");
    }

    #[test]
    fn test_text_index_state_audit_status_term_stats_entries_mismatch() {
        let engine = in_memory_engine();
        let state_counts = TextIndexCounts {
            term_stats_entries: 2,
            ..Default::default()
        };
        let state = VantaEmbedded::fresh_text_index_state(state_counts);
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();
        let expected_counts = TextIndexCounts {
            term_stats_entries: 5,
            ..Default::default()
        };
        let (valid, status) =
            VantaEmbedded::text_index_state_audit_status(&engine, expected_counts, None);
        assert!(!valid);
        assert_eq!(status, "count_mismatch");
    }

    #[test]
    fn test_text_index_state_audit_status_namespace_stats_entries_mismatch() {
        let engine = in_memory_engine();
        let state_counts = TextIndexCounts {
            namespace_stats_entries: 1,
            ..Default::default()
        };
        let state = VantaEmbedded::fresh_text_index_state(state_counts);
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();
        let expected_counts = TextIndexCounts {
            namespace_stats_entries: 3,
            ..Default::default()
        };
        let (valid, status) =
            VantaEmbedded::text_index_state_audit_status(&engine, expected_counts, None);
        assert!(!valid);
        assert_eq!(status, "count_mismatch");
    }

    // ─── rebuild_text_index_with_report ──────────────────────

    #[test]
    fn test_rebuild_text_index_with_report_empty() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        let report = db.rebuild_text_index_with_report().unwrap();
        assert_eq!(report.record_count, 0);
        assert_eq!(report.posting_entries, 0);
        assert_eq!(report.doc_stats_entries, 0);
        assert_eq!(report.term_stats_entries, 0);
        assert_eq!(report.namespace_stats_entries, 0);
    }

    #[test]
    fn test_rebuild_text_index_with_report_with_data() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        db.put(crate::sdk::VantaMemoryInput::new(
            "ns1",
            "k1",
            "hello world",
        ))
        .unwrap();
        db.put(crate::sdk::VantaMemoryInput::new(
            "ns1",
            "k2",
            "foo bar baz",
        ))
        .unwrap();

        let report = db.rebuild_text_index_with_report().unwrap();
        assert_eq!(report.record_count, 2);
        assert!(report.posting_entries > 0);
        assert_eq!(report.doc_stats_entries, 2);
        assert!(report.term_stats_entries > 0);
        assert!(report.namespace_stats_entries > 0);
    }

    #[test]
    fn test_rebuild_text_index_with_report_checks_exact_counts() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        // "hello world" has 2 tokens → 2 postings, 2 term stats, 1 doc stats, 1 namespace
        db.put(crate::sdk::VantaMemoryInput::new(
            "ns1",
            "k1",
            "hello world",
        ))
        .unwrap();

        let report = db.rebuild_text_index_with_report().unwrap();
        assert_eq!(report.record_count, 1);
        assert_eq!(report.posting_entries, 2);
        assert_eq!(report.doc_stats_entries, 1);
        assert_eq!(report.term_stats_entries, 2);
        assert_eq!(report.namespace_stats_entries, 1);
    }

    // ─── rebuild_derived_indexes_with_report ─────────────────

    #[test]
    fn test_rebuild_derived_indexes_with_report_empty() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        let report = db.rebuild_derived_indexes_with_report().unwrap();
        assert_eq!(report.record_count, 0);
        assert_eq!(report.namespace_entries, 0);
        assert_eq!(report.payload_entries, 0);
    }

    #[test]
    fn test_rebuild_derived_indexes_with_report_with_data() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        db.put(crate::sdk::VantaMemoryInput::new("ns1", "k1", "hello"))
            .unwrap();
        db.put(crate::sdk::VantaMemoryInput::new("ns1", "k2", "world"))
            .unwrap();

        let report = db.rebuild_derived_indexes_with_report().unwrap();
        assert_eq!(report.record_count, 2);
        assert_eq!(report.namespace_entries, 2);
        // payload_entries is 0 because VantaMemoryInput::new has no custom metadata
        assert_eq!(report.payload_entries, 0);
    }

    // ─── build_text_index_audit_report_deep ──────────────────

    #[test]
    fn test_build_text_index_audit_report_deep_empty() {
        let engine = std::sync::Arc::new(in_memory_engine());
        // Pre-write a valid state so audit passes on an empty index
        let state = VantaEmbedded::fresh_text_index_state(TextIndexCounts::default());
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();
        let report = VantaEmbedded::build_text_index_audit_report_deep(&engine, None).unwrap();
        assert!(
            report.passed,
            "expected passed, got status: {}",
            report.status
        );
        assert_eq!(report.records_scanned, 0);
        assert!(report.deep_audit);
        assert_eq!(report.state_status, "current");
    }

    #[test]
    fn test_build_text_index_audit_report_deep_with_data() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        db.put(crate::sdk::VantaMemoryInput::new(
            "ns1",
            "k1",
            "hello world",
        ))
        .unwrap();
        db.put(crate::sdk::VantaMemoryInput::new(
            "ns1",
            "k2",
            "foo bar baz",
        ))
        .unwrap();

        // Rebuild to ensure fresh, consistent state
        db.rebuild_text_index_with_report().unwrap();

        let engine = db.engine_handle().unwrap();
        let report = VantaEmbedded::build_text_index_audit_report_deep(&engine, None).unwrap();
        assert!(
            report.passed,
            "expected passed, got status: {}",
            report.status
        );
        assert_eq!(report.records_scanned, 2);
        assert!(report.deep_audit);
    }

    #[test]
    fn test_build_text_index_audit_report_deep_with_namespace_filter() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        db.put(crate::sdk::VantaMemoryInput::new("ns1", "k1", "hello"))
            .unwrap();
        db.put(crate::sdk::VantaMemoryInput::new("ns2", "k2", "world"))
            .unwrap();

        db.rebuild_text_index_with_report().unwrap();
        let engine = db.engine_handle().unwrap();
        let report =
            VantaEmbedded::build_text_index_audit_report_deep(&engine, Some("ns1")).unwrap();
        // With namespace_filter count check is skipped → state stays valid
        assert!(
            report.passed,
            "expected passed, got status: {}",
            report.status
        );
        assert_eq!(report.namespace_filter.as_deref(), Some("ns1"));
    }

    // ─── build_text_index_audit_report_shallow ────────────────

    #[test]
    fn test_build_text_index_audit_report_shallow_empty() {
        let engine = std::sync::Arc::new(in_memory_engine());
        let state = VantaEmbedded::fresh_text_index_state(TextIndexCounts::default());
        VantaEmbedded::write_text_index_state(&engine, &state).unwrap();
        let report = VantaEmbedded::build_text_index_audit_report_shallow(&engine, None).unwrap();
        assert!(
            report.passed,
            "expected passed, got status: {}",
            report.status
        );
        assert!(!report.deep_audit);
        assert_eq!(report.state_status, "current");
    }

    #[test]
    fn test_build_text_index_audit_report_shallow_with_data() {
        let db = crate::sdk::VantaEmbedded::open_with_config(crate::config::VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        })
        .expect("in-memory db");
        db.put(crate::sdk::VantaMemoryInput::new(
            "ns1",
            "k1",
            "hello world",
        ))
        .unwrap();

        db.rebuild_text_index_with_report().unwrap();
        let engine = db.engine_handle().unwrap();
        let report = VantaEmbedded::build_text_index_audit_report_shallow(&engine, None).unwrap();
        assert!(
            report.passed,
            "expected passed, got status: {}",
            report.status
        );
        assert!(!report.deep_audit);
    }
}
