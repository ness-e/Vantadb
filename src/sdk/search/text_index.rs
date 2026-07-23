//! Text-index audit, repair, and query-readiness utilities.
//!
//! These functions wrap the deeper audit/rebuild infrastructure in
//! `crate::sdk::serialization::impl_rebuild` and `impl_text_index`
//! so that `search/mod.rs` remains routing-focused.

use crate::error::{Result, VantaError};
use crate::sdk::builder::VantaEmbedded;
use crate::sdk::serialization::validate_namespace;
use crate::sdk::types::TextIndexRebuildReport;
use crate::sdk::types::*;
use crate::storage::StorageEngine;

/// Check that the persistent text index is ready for BM25 queries.
///
/// Returns the stored index state on success, or an appropriate error
/// if the index is missing or its schema predates the required version.
pub(crate) fn ensure_text_index_query_ready(engine: &StorageEngine) -> Result<TextIndexState> {
    let state = VantaEmbedded::load_text_index_state(engine).map_err(|_| VantaError::NotFound {
        kind: "text_index_state".into(),
        id: "bm25".into(),
    })?;
    let Some(state) = state else {
        return Err(VantaError::NotFound {
            kind: "text_index".into(),
            id: "bm25".into(),
        });
    };
    if !VantaEmbedded::text_index_state_matches_spec(&state) {
        return Err(VantaError::ValidationError {
            field: "text_index_schema".into(),
            reason:
                "text_query requires text_index schema v3; reopen writable or run rebuild_index"
                    .into(),
        });
    }
    Ok(state)
}

/// Run a shallow structural audit of the persistent text index.
pub(crate) fn run_audit(
    engine: &StorageEngine,
    namespace: Option<&str>,
) -> Result<VantaTextIndexAuditReport> {
    if let Some(ns) = namespace {
        validate_namespace(ns)?;
    }
    VantaEmbedded::build_text_index_audit_report_shallow(engine, namespace)
}

/// Run a deep structural audit of the persistent text index.
pub(crate) fn run_audit_deep(
    engine: &StorageEngine,
    namespace: Option<&str>,
) -> Result<VantaTextIndexAuditReport> {
    if let Some(ns) = namespace {
        validate_namespace(ns)?;
    }
    VantaEmbedded::build_text_index_audit_report_deep(engine, namespace)
}

/// Build a `VantaTextIndexRepairReport` from a rebuild report.
pub(crate) fn run_repair(report: TextIndexRebuildReport) -> VantaTextIndexRepairReport {
    VantaTextIndexRepairReport {
        record_count: report.record_count,
        posting_entries: report.posting_entries,
        doc_stats_entries: report.doc_stats_entries,
        term_stats_entries: report.term_stats_entries,
        namespace_stats_entries: report.namespace_stats_entries,
        duration_ms: report.duration_ms,
        success: true,
    }
}
