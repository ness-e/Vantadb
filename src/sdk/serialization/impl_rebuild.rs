//! Rebuild operations and text index audit for `VantaEmbedded`.

use super::super::builder::VantaEmbedded;
use super::super::types::*;
use super::{memory_record_from_node, now_ms, TextIndexCounts, DERIVED_INDEX_SCHEMA_VERSION};
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
            if let Some(record) = memory_record_from_node(node) {
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

        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(node) {
                counts.record_count += 1;
                let posting_ops = crate::text_index::posting_put_ops(
                    &record.namespace,
                    &record.key,
                    &record.payload,
                    record.node_id,
                )?;
                counts.posting_entries += posting_ops.len() as u64;
                ops.extend(posting_ops);
                ops.push(crate::text_index::doc_stats_put_op(
                    &record.namespace,
                    &record.key,
                    &record.payload,
                    record.node_id,
                )?);
                counts.doc_stats_entries += 1;

                let terms = crate::text_index::record_terms(&record.payload);
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
            if let Some(record) = memory_record_from_node(node) {
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
