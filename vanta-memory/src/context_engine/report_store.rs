//! Per-run persistence for [`crate::context_engine::CompactionReport`] (MEM-64).
//!
//! The post-L3 context assembly in
//! [`crate::services::pipeline_worker::run_context_assembly`] already writes
//! the full [`crate::context_engine::IntegratedContext`] to
//! `context/{session}/__assembled`. MEM-64 adds a sibling record under
//! `context/{session}/compaction_reports/{run_id}` carrying just the
//! `CompactionReport` plus metadata (capture time, run id, recall/mmd
//! injection flags). The split exists for one reason: the assembled context
//! is intentionally **mutable** (the next run overwrites `__assembled`),
//! while the compaction history must be **append-only** so every run is
//! auditable.
//!
//! ponytail: O(1) write per session per run. `run_id` is supplied by the
//! caller (deterministic counter) so retries don't duplicate records.

use serde::{Deserialize, Serialize};

use crate::context_engine::types::CompactionReport;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};

use super::engine::IntegratedContext;

/// Prefix for per-session compaction-report records (MEM-64).
pub const COMPACTION_REPORT_PREFIX: &str = "compaction_reports";

/// Wire shape persisted under `context/{session}/{COMPACTION_REPORT_PREFIX}/{run_id}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedCompactionReport {
    pub run_id: String,
    pub captured_at_ms: u64,
    pub report: CompactionReport,
    pub mmd_injected: bool,
    pub recall_injected: bool,
    /// Token counts mirror `CompactionReport`, kept here for convenience
    /// (callers that only fetch the report row don't have to deserialize the
    /// full `__assembled` payload).
    pub tokens_before: u64,
    pub tokens_after: u64,
    /// Messages the user can fetch from the report alone — no need to load
    /// the full `IntegratedContext` to learn the compaction outcome.
    pub msgs_before: usize,
    pub msgs_conserved: usize,
}

impl PersistedCompactionReport {
    /// Lift an [`IntegratedContext`] + `run_id` + capture time into the
    /// persisted shape.
    pub fn from_context(
        run_id: impl Into<String>,
        captured_at_ms: u64,
        ctx: &IntegratedContext,
    ) -> Self {
        Self {
            run_id: run_id.into(),
            captured_at_ms,
            report: ctx.report.clone(),
            mmd_injected: ctx.mmd_injected,
            recall_injected: ctx.recall_injected,
            tokens_before: ctx.report.tokens_before,
            tokens_after: ctx.report.tokens_after,
            msgs_before: ctx.report.msgs_before,
            msgs_conserved: ctx.report.msgs_conserved,
        }
    }
}

/// Persist a [`PersistedCompactionReport`] under
/// `context/{session}/{COMPACTION_REPORT_PREFIX}/{run_id}`.
///
/// Errors are surfaced verbatim — the caller (the worker) decides whether
/// to fail-fast or warn-and-continue.
pub fn record_compaction_report(
    db: &VantaEmbedded,
    session_id: &str,
    record: &PersistedCompactionReport,
) -> Result<(), String> {
    let namespace = format!(
        "context/{}/{}",
        crate::utils::sanitize::sanitize_component(session_id, 128, false),
        COMPACTION_REPORT_PREFIX,
    );
    let key = crate::utils::sanitize::sanitize_key(&record.run_id);
    let payload = serde_json::to_string(record).map_err(|e| e.to_string())?;
    let mut metadata = VantaMemoryMetadata::new();
    metadata.insert(
        "kind".into(),
        vantadb::sdk::VantaValue::String("compaction_report".into()),
    );
    metadata.insert(
        "mode".into(),
        vantadb::sdk::VantaValue::String(format!("{:?}", record.report.mode)),
    );
    db.put(VantaMemoryInput {
        namespace,
        key,
        payload,
        metadata,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Read every persisted [`PersistedCompactionReport`] for `session_id`,
/// oldest first (sorted by `captured_at_ms`).
pub fn list_compaction_reports(
    db: &VantaEmbedded,
    session_id: &str,
) -> Result<Vec<PersistedCompactionReport>, String> {
    let namespace = format!(
        "context/{}/{}",
        crate::utils::sanitize::sanitize_component(session_id, 128, false),
        COMPACTION_REPORT_PREFIX,
    );
    let opts = vantadb::sdk::VantaMemoryListOptions {
        limit: 10_000,
        ..vantadb::sdk::VantaMemoryListOptions::default()
    };
    let page = db
        .list(&namespace, opts)
        .map_err(|e| format!("compaction report list failed: {e}"))?;
    let mut reports: Vec<PersistedCompactionReport> = page
        .records
        .into_iter()
        .map(|e| serde_json::from_str(&e.payload))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("compaction report decode failed: {e}"))?;
    reports.sort_by_key(|r| r.captured_at_ms);
    Ok(reports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_engine::types::{ChatMessage, ChatRole, CompactionMode};
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    fn db() -> VantaEmbedded {
        VantaEmbedded::open_with_config(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        })
        .expect("open in-memory db")
    }

    fn ctx() -> IntegratedContext {
        IntegratedContext {
            messages: vec![ChatMessage::new(ChatRole::User, "hi")],
            report: CompactionReport {
                mode: CompactionMode::Mild,
                msgs_conserved: 1,
                msgs_before: 4,
                tokens_before: 800,
                tokens_after: 250,
            },
            mmd_injected: true,
            recall_injected: false,
        }
    }

    #[test]
    fn round_trips_compaction_report() {
        let db = db();
        let record = PersistedCompactionReport::from_context(
            "run-2026-08-30T00-001",
            1_700_000_000_000,
            &ctx(),
        );
        record_compaction_report(&db, "session-A", &record).expect("write");
        let reports = list_compaction_reports(&db, "session-A").expect("list");
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0], record);
    }

    #[test]
    fn multiple_runs_kept_in_capture_order() {
        let db = db();
        for (run_id, ts) in [("r1", 1), ("r2", 2), ("r3", 3)] {
            let r = PersistedCompactionReport::from_context(run_id, ts, &ctx());
            record_compaction_report(&db, "session-B", &r).expect("write");
        }
        let reports = list_compaction_reports(&db, "session-B").expect("list");
        assert_eq!(reports.len(), 3);
        assert_eq!(
            reports
                .iter()
                .map(|r| r.run_id.as_str())
                .collect::<Vec<_>>(),
            vec!["r1", "r2", "r3"],
        );
    }
}
