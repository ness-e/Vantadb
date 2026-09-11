//! `From` trait implementations for SDK types.

use super::super::types::*;
use super::graph_types::unified_to_record;
use crate::executor::ExecutionResult;
use crate::node::FieldValue;
use crate::node::LabelIntern;

impl From<crate::storage::IndexRebuildReport> for IndexRebuildReport {
    fn from(report: crate::storage::IndexRebuildReport) -> Self {
        Self {
            scanned_nodes: report.scanned_nodes,
            indexed_vectors: report.indexed_vectors,
            skipped_tombstones: report.skipped_tombstones,
            duration_ms: report.duration_ms,
            derived_rebuild_ms: 0,
            index_path: report.index_path.to_string_lossy().into_owned(),
            success: report.success,
        }
    }
}

impl From<crate::metrics::OperationalMetricsSnapshot> for OperationalMetrics {
    fn from(metrics: crate::metrics::OperationalMetricsSnapshot) -> Self {
        Self {
            startup_ms: metrics.startup_ms,
            wal_replay_ms: metrics.wal_replay_ms,
            wal_records_replayed: metrics.wal_records_replayed,
            ann_rebuild_ms: metrics.ann_rebuild_ms,
            ann_rebuild_scanned_nodes: metrics.ann_rebuild_scanned_nodes,
            derived_rebuild_ms: metrics.derived_rebuild_ms,
            text_index_rebuild_ms: metrics.text_index_rebuild_ms,
            text_postings_written: metrics.text_postings_written,
            text_index_repairs: metrics.text_index_repairs,
            text_lexical_queries: metrics.text_lexical_queries,
            text_lexical_query_ms: metrics.text_lexical_query_ms,
            text_candidates_scored: metrics.text_candidates_scored,
            text_consistency_audits: metrics.text_consistency_audits,
            text_consistency_audit_failures: metrics.text_consistency_audit_failures,
            hybrid_query_ms: metrics.hybrid_query_ms,
            hybrid_candidates_fused: metrics.hybrid_candidates_fused,
            planner_hybrid_queries: metrics.planner_hybrid_queries,
            planner_text_only_queries: metrics.planner_text_only_queries,
            planner_vector_only_queries: metrics.planner_vector_only_queries,
            records_exported: metrics.records_exported,
            records_imported: metrics.records_imported,
            import_errors: metrics.import_errors,
            derived_prefix_scans: metrics.derived_prefix_scans,
            derived_full_scan_fallbacks: metrics.derived_full_scan_fallbacks,
            process_rss_bytes: metrics.memory.process_rss_bytes,
            process_virtual_bytes: metrics.memory.process_virtual_bytes,
            hnsw_nodes_count: metrics.memory.hnsw_nodes_count,
            hnsw_logical_bytes: metrics.memory.hnsw_logical_bytes,
            mmap_resident_bytes: metrics.memory.mmap_resident_bytes,
            volatile_cache_entries: metrics.memory.volatile_cache_entries,
            volatile_cache_cap_bytes: metrics.memory.volatile_cache_cap_bytes,
            jemalloc_allocated_bytes: metrics.memory.jemalloc_allocated_bytes,
            jemalloc_active_bytes: metrics.memory.jemalloc_active_bytes,
            jemalloc_metadata_bytes: metrics.memory.jemalloc_metadata_bytes,
            jemalloc_resident_bytes: metrics.memory.jemalloc_resident_bytes,
            jemalloc_mapped_bytes: metrics.memory.jemalloc_mapped_bytes,
            jemalloc_retained_bytes: metrics.memory.jemalloc_retained_bytes,
            l1_extraction_latency_ms: metrics.l1_extraction_latency_ms,
            l1_dedup_latency_ms: metrics.l1_dedup_latency_ms,
            l2_extraction_latency_ms: metrics.l2_extraction_latency_ms,
            l2_llm_duration_ms: metrics.l2_llm_duration_ms,
            l3_generation_latency_ms: metrics.l3_generation_latency_ms,
            persona_length_before: metrics.persona_length_before,
            persona_length_after: metrics.persona_length_after,
            persona_drift_ratio: metrics.persona_drift_ratio,
            recall_hit_count: metrics.recall_hit_count,
            recall_top_score: metrics.recall_top_score,
            recall_latency_ms: metrics.recall_latency_ms,
            recall_strategy: metrics.recall_strategy,
            offload_latency_ms: metrics.offload_latency_ms,
        }
    }
}

impl From<Value> for FieldValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(value) => FieldValue::String(value),
            Value::Int(value) => FieldValue::Int(value),
            Value::Float(value) => FieldValue::Float(value),
            Value::Bool(value) => FieldValue::Bool(value),
            Value::DateTime(value) => FieldValue::DateTime(value),
            Value::ListString(value) => FieldValue::ListString(value),
            Value::ListInt(value) => FieldValue::ListInt(value),
            Value::ListFloat(value) => FieldValue::ListFloat(value),
            Value::ListBool(value) => FieldValue::ListBool(value),
            Value::ListDateTime(value) => FieldValue::ListDateTime(value),
            Value::Null => FieldValue::Null,
        }
    }
}

impl From<FieldValue> for Value {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(value) => Value::String(value),
            FieldValue::Int(value) => Value::Int(value),
            FieldValue::Float(value) => Value::Float(value),
            FieldValue::Bool(value) => Value::Bool(value),
            FieldValue::DateTime(value) => Value::DateTime(value),
            FieldValue::ListString(value) => Value::ListString(value),
            FieldValue::ListInt(value) => Value::ListInt(value),
            FieldValue::ListFloat(value) => Value::ListFloat(value),
            FieldValue::ListBool(value) => Value::ListBool(value),
            FieldValue::ListDateTime(value) => Value::ListDateTime(value),
            FieldValue::Null => Value::Null,
        }
    }
}

impl From<ExecutionResult> for QueryResult {
    fn from(result: ExecutionResult) -> Self {
        // Use an empty interner — labels in edge records will show as "<unknown>"
        // when no interner context is available. The main code path (Embedded::query)
        // converts with the engine's interner directly.
        let fallback_intern = LabelIntern::new();
        match result {
            ExecutionResult::Read(nodes) => QueryResult::Read(
                nodes
                    .into_iter()
                    .map(|n| unified_to_record(n, &fallback_intern))
                    .collect(),
            ),
            ExecutionResult::Write {
                affected_nodes,
                message,
                node_id,
            } => QueryResult::Write {
                affected_nodes,
                message,
                node_id,
            },
            ExecutionResult::StaleContext(node_id) => QueryResult::StaleContext { node_id },
        }
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::executor::ExecutionResult;
    use crate::metrics::{MemoryBreakdownSnapshot, OperationalMetricsSnapshot};
    use crate::node::UnifiedNode;
    use crate::storage::IndexRebuildReport;
    use std::path::PathBuf;

    // ─── storage::IndexRebuildReport → sdk::IndexRebuildReport ───────────

    #[test]
    fn test_index_rebuild_report_conversion() {
        let report = IndexRebuildReport {
            scanned_nodes: 100,
            indexed_vectors: 80,
            skipped_tombstones: 20,
            duration_ms: 1500,
            index_path: PathBuf::from("/tmp/index.hnsw"),
            success: true,
        };
        let vanta: crate::sdk::types::IndexRebuildReport = report.into();
        assert_eq!(vanta.scanned_nodes, 100);
        assert_eq!(vanta.indexed_vectors, 80);
        assert_eq!(vanta.skipped_tombstones, 20);
        assert_eq!(vanta.duration_ms, 1500);
        assert!(vanta.index_path.contains("index.hnsw"));
        assert!(vanta.success);
    }

    #[test]
    fn test_index_rebuild_report_conversion_failure() {
        let report = IndexRebuildReport {
            scanned_nodes: 0,
            indexed_vectors: 0,
            skipped_tombstones: 0,
            duration_ms: 0,
            index_path: PathBuf::from("/dev/null"),
            success: false,
        };
        let vanta: crate::sdk::types::IndexRebuildReport = report.into();
        assert_eq!(vanta.scanned_nodes, 0);
        assert!(!vanta.success);
    }

    // ─── OperationalMetricsSnapshot → OperationalMetrics ──

    #[test]
    fn test_operational_metrics_conversion() {
        let memory = MemoryBreakdownSnapshot {
            process_rss_bytes: 1_000_000,
            process_virtual_bytes: 2_000_000,
            hnsw_nodes_count: 500,
            hnsw_logical_bytes: 50_000_000,
            mmap_resident_bytes: Some(8_000_000),
            volatile_cache_entries: 100,
            volatile_cache_cap_bytes: 100_000_000,
            jemalloc_allocated_bytes: Some(10_000_000),
            jemalloc_active_bytes: Some(8_000_000),
            jemalloc_metadata_bytes: Some(1_000_000),
            jemalloc_resident_bytes: Some(10_000_000),
            jemalloc_mapped_bytes: Some(12_000_000),
            jemalloc_retained_bytes: Some(2_000_000),
        };
        let metrics = OperationalMetricsSnapshot {
            startup_ms: 100,
            wal_replay_ms: 50,
            wal_records_replayed: 2000,
            ann_rebuild_ms: 500,
            ann_rebuild_scanned_nodes: 1000,
            derived_rebuild_ms: 300,
            text_index_rebuild_ms: 400,
            text_postings_written: 15000,
            text_index_repairs: 2,
            text_lexical_queries: 50,
            text_lexical_query_ms: 250,
            text_candidates_scored: 10000,
            text_consistency_audits: 10,
            text_consistency_audit_failures: 1,
            hybrid_query_ms: 600,
            hybrid_candidates_fused: 500,
            planner_hybrid_queries: 30,
            planner_text_only_queries: 20,
            planner_vector_only_queries: 10,
            index_routing_flat: 1,
            index_routing_ivf: 2,
            index_routing_hnsw: 3,
            records_exported: 100,
            records_imported: 200,
            import_errors: 3,
            derived_prefix_scans: 40,
            derived_full_scan_fallbacks: 5,
            evictions_total: 50,
            eviction_scanned_total: 5000,
            eviction_cycles_total: 5,
            eviction_bytes_total: 10_000_000,
            quantized_nodes_total: 100,
            promoted_nodes_total: 20,
            current_quantized_nodes: 80,
            l1_extraction_latency_ms: 15,
            l1_dedup_latency_ms: 5,
            l2_extraction_latency_ms: 25,
            l2_llm_duration_ms: 80,
            l3_generation_latency_ms: 45,
            persona_length_before: 120,
            persona_length_after: 150,
            persona_drift_ratio: 2_500,
            recall_hit_count: 7,
            recall_top_score: 9_500,
            recall_latency_ms: 30,
            recall_strategy: 3,
            offload_latency_ms: 60,
            memory,
        };
        let vanta: OperationalMetrics = metrics.into();
        assert_eq!(vanta.startup_ms, 100);
        assert_eq!(vanta.wal_replay_ms, 50);
        assert_eq!(vanta.process_rss_bytes, 1_000_000);
        assert_eq!(vanta.mmap_resident_bytes, Some(8_000_000));
        assert_eq!(vanta.jemalloc_allocated_bytes, Some(10_000_000));
        assert_eq!(vanta.hnsw_nodes_count, 500);
        assert!(vanta.hnsw_logical_bytes > 0);
        assert_eq!(vanta.l1_extraction_latency_ms, 15);
        assert_eq!(vanta.l2_llm_duration_ms, 80);
        assert_eq!(vanta.l3_generation_latency_ms, 45);
        assert_eq!(vanta.persona_length_before, 120);
        assert_eq!(vanta.persona_length_after, 150);
        assert_eq!(vanta.persona_drift_ratio, 2_500);
        assert_eq!(vanta.recall_hit_count, 7);
        assert_eq!(vanta.recall_top_score, 9_500);
        assert_eq!(vanta.recall_strategy, 3);
        assert_eq!(vanta.offload_latency_ms, 60);
    }

    // ─── Value ↔ FieldValue ───────────────────────────────

    #[test]
    fn test_vanta_value_to_field_value_string() {
        let vv = Value::String("hello".into());
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::String("hello".into()));
    }

    #[test]
    fn test_vanta_value_to_field_value_int() {
        let vv = Value::Int(42);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::Int(42));
    }

    #[test]
    fn test_vanta_value_to_field_value_float() {
        let vv = Value::Float(1.5);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::Float(1.5));
    }

    #[test]
    fn test_vanta_value_to_field_value_bool() {
        let vv = Value::Bool(true);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::Bool(true));
    }

    #[test]
    fn test_vanta_value_to_field_value_datetime() {
        let dt = chrono::Utc::now();
        let vv = Value::DateTime(dt);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::DateTime(dt));
    }

    #[test]
    fn test_vanta_value_to_field_value_list_string() {
        let vv = Value::ListString(vec!["a".into(), "b".into()]);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::ListString(vec!["a".into(), "b".into()]));
    }

    #[test]
    fn test_vanta_value_to_field_value_list_int() {
        let vv = Value::ListInt(vec![1, 2]);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::ListInt(vec![1, 2]));
    }

    #[test]
    fn test_vanta_value_to_field_value_list_float() {
        let vv = Value::ListFloat(vec![1.0, 2.0]);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::ListFloat(vec![1.0, 2.0]));
    }

    #[test]
    fn test_vanta_value_to_field_value_list_bool() {
        let vv = Value::ListBool(vec![true, false]);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::ListBool(vec![true, false]));
    }

    #[test]
    fn test_vanta_value_to_field_value_list_datetime() {
        let dt = chrono::Utc::now();
        let vv = Value::ListDateTime(vec![dt]);
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::ListDateTime(vec![dt]));
    }

    #[test]
    fn test_vanta_value_to_field_value_null() {
        let vv = Value::Null;
        let fv: FieldValue = vv.into();
        assert_eq!(fv, FieldValue::Null);
    }

    // ─── FieldValue → Value ───────────────────────────────

    #[test]
    fn test_field_value_to_vanta_value_string() {
        let fv = FieldValue::String("world".into());
        let vv: Value = fv.into();
        assert_eq!(vv, Value::String("world".into()));
    }

    #[test]
    fn test_field_value_to_vanta_value_int() {
        let fv = FieldValue::Int(-7);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::Int(-7));
    }

    #[test]
    fn test_field_value_to_vanta_value_float() {
        let fv = FieldValue::Float(1.5);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::Float(1.5));
    }

    #[test]
    fn test_field_value_to_vanta_value_bool() {
        let fv = FieldValue::Bool(false);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::Bool(false));
    }

    #[test]
    fn test_field_value_to_vanta_value_datetime() {
        let dt = chrono::Utc::now();
        let fv = FieldValue::DateTime(dt);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::DateTime(dt));
    }

    #[test]
    fn test_field_value_to_vanta_value_list_string() {
        let fv = FieldValue::ListString(vec!["x".into()]);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::ListString(vec!["x".into()]));
    }

    #[test]
    fn test_field_value_to_vanta_value_list_int() {
        let fv = FieldValue::ListInt(vec![100]);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::ListInt(vec![100]));
    }

    #[test]
    fn test_field_value_to_vanta_value_list_float() {
        let fv = FieldValue::ListFloat(vec![-1.0]);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::ListFloat(vec![-1.0]));
    }

    #[test]
    fn test_field_value_to_vanta_value_list_bool() {
        let fv = FieldValue::ListBool(vec![false]);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::ListBool(vec![false]));
    }

    #[test]
    fn test_field_value_to_vanta_value_list_datetime() {
        let dt = chrono::Utc::now();
        let fv = FieldValue::ListDateTime(vec![dt]);
        let vv: Value = fv.into();
        assert_eq!(vv, Value::ListDateTime(vec![dt]));
    }

    #[test]
    fn test_field_value_to_vanta_value_null() {
        let fv = FieldValue::Null;
        let vv: Value = fv.into();
        assert_eq!(vv, Value::Null);
    }

    // ─── Value ↔ FieldValue roundtrip ─────────────────────

    #[test]
    fn test_value_roundtrip_string() {
        let original = Value::String("roundtrip".into());
        let fv: FieldValue = original.clone().into();
        let back: Value = fv.into();
        assert_eq!(original, back);
    }

    #[test]
    fn test_value_roundtrip_int() {
        let original = Value::Int(-99);
        let fv: FieldValue = original.clone().into();
        let back: Value = fv.into();
        assert_eq!(original, back);
    }

    #[test]
    fn test_value_roundtrip_null() {
        let original = Value::Null;
        let fv: FieldValue = original.clone().into();
        let back: Value = fv.into();
        assert_eq!(original, back);
    }

    // ─── ExecutionResult → QueryResult ────────────────────

    #[test]
    fn test_execution_result_read_conversion() {
        let nodes = vec![UnifiedNode::new(1), UnifiedNode::new(2)];
        let result = ExecutionResult::Read(nodes);
        let vqr: QueryResult = result.into();
        match vqr {
            QueryResult::Read(records) => {
                assert_eq!(records.len(), 2);
                assert!(records.iter().any(|r| r.id == 1));
                assert!(records.iter().any(|r| r.id == 2));
            }
            _ => panic!("expected Read variant"),
        }
    }

    #[test]
    fn test_execution_result_read_empty() {
        let result = ExecutionResult::Read(vec![]);
        let vqr: QueryResult = result.into();
        match vqr {
            QueryResult::Read(records) => assert!(records.is_empty()),
            _ => panic!("expected Read variant"),
        }
    }

    #[test]
    fn test_execution_result_write_conversion() {
        let result = ExecutionResult::Write {
            affected_nodes: 3,
            message: "inserted 3 records".into(),
            node_id: Some(42),
        };
        let vqr: QueryResult = result.into();
        match vqr {
            QueryResult::Write {
                affected_nodes,
                message,
                node_id,
            } => {
                assert_eq!(affected_nodes, 3);
                assert_eq!(message, "inserted 3 records");
                assert_eq!(node_id, Some(42));
            }
            _ => panic!("expected Write variant"),
        }
    }

    #[test]
    fn test_execution_result_write_no_node_id() {
        let result = ExecutionResult::Write {
            affected_nodes: 1,
            message: "updated".into(),
            node_id: None,
        };
        let vqr: QueryResult = result.into();
        match vqr {
            QueryResult::Write { node_id, .. } => assert_eq!(node_id, None),
            _ => panic!("expected Write variant"),
        }
    }

    #[test]
    fn test_execution_result_stale_context_conversion() {
        let result = ExecutionResult::StaleContext(999);
        let vqr: QueryResult = result.into();
        match vqr {
            QueryResult::StaleContext { node_id } => assert_eq!(node_id, 999),
            _ => panic!("expected StaleContext variant"),
        }
    }
}
