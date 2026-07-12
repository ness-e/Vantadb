//! `From` trait implementations for SDK types.

use super::super::types::*;
use crate::executor::ExecutionResult;
use crate::node::FieldValue;

impl From<crate::storage::IndexRebuildReport> for VantaIndexRebuildReport {
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

impl From<crate::metrics::OperationalMetricsSnapshot> for VantaOperationalMetrics {
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
        }
    }
}

impl From<VantaValue> for FieldValue {
    fn from(value: VantaValue) -> Self {
        match value {
            VantaValue::String(value) => FieldValue::String(value),
            VantaValue::Int(value) => FieldValue::Int(value),
            VantaValue::Float(value) => FieldValue::Float(value),
            VantaValue::Bool(value) => FieldValue::Bool(value),
            VantaValue::DateTime(value) => FieldValue::DateTime(value),
            VantaValue::ListString(value) => FieldValue::ListString(value),
            VantaValue::ListInt(value) => FieldValue::ListInt(value),
            VantaValue::ListFloat(value) => FieldValue::ListFloat(value),
            VantaValue::ListBool(value) => FieldValue::ListBool(value),
            VantaValue::ListDateTime(value) => FieldValue::ListDateTime(value),
            VantaValue::Null => FieldValue::Null,
        }
    }
}

impl From<FieldValue> for VantaValue {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(value) => VantaValue::String(value),
            FieldValue::Int(value) => VantaValue::Int(value),
            FieldValue::Float(value) => VantaValue::Float(value),
            FieldValue::Bool(value) => VantaValue::Bool(value),
            FieldValue::DateTime(value) => VantaValue::DateTime(value),
            FieldValue::ListString(value) => VantaValue::ListString(value),
            FieldValue::ListInt(value) => VantaValue::ListInt(value),
            FieldValue::ListFloat(value) => VantaValue::ListFloat(value),
            FieldValue::ListBool(value) => VantaValue::ListBool(value),
            FieldValue::ListDateTime(value) => VantaValue::ListDateTime(value),
            FieldValue::Null => VantaValue::Null,
        }
    }
}

impl From<ExecutionResult> for VantaQueryResult {
    fn from(result: ExecutionResult) -> Self {
        match result {
            ExecutionResult::Read(nodes) => {
                VantaQueryResult::Read(nodes.into_iter().map(Into::into).collect())
            }
            ExecutionResult::Write {
                affected_nodes,
                message,
                node_id,
            } => VantaQueryResult::Write {
                affected_nodes,
                message,
                node_id,
            },
            ExecutionResult::StaleContext(node_id) => VantaQueryResult::StaleContext { node_id },
        }
    }
}
