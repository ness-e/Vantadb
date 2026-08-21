//! `after_tool_call` hook — decides whether a finished tool call's payload
//! is worth offloading (port of the decision core of TDAM
//! `MC/offload/hooks/after-tool-call.ts`, MEM-20).
//!
//! TDAM buffers every pair and lets an LLM (L1) summarize it later; the L3
//! compression loop then replaces in-context results with those summaries.
//! The LLM-free equivalent here: when the serialized result exceeds a size
//! threshold, persist an [`OffloadEntry`] whose `summary` is a deterministic
//! truncated placeholder (L1 upgrades it later) and advance the persistent
//! cursor [`PluginState::last_offloaded_tool_call_id`].
//!
//! Idempotency (D19): re-processing the same tool call is a no-op — guarded
//! by both the cursor and storage-level dedup by `tool_call_id`.

use crate::offload::state_manager::{OffloadError, OffloadStateManager};
use crate::offload::storage::OffloadStorage;
use crate::offload::types::{OffloadEntry, ToolPair};
use crate::utils::text_utils::truncate_with_suffix;

/// Placeholder summary length for entries awaiting L1 summarization.
const PLACEHOLDER_SUMMARY_CHARS: usize = 200;

/// Why a tool call was not offloaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Cursor or storage already recorded this `tool_call_id`.
    AlreadyProcessed,
    /// Serialized result is below the size threshold.
    BelowThreshold,
}

/// Outcome of one hook invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookOutcome {
    pub offloaded: bool,
    pub skip_reason: Option<SkipReason>,
}

impl HookOutcome {
    fn offloaded() -> Self {
        Self {
            offloaded: true,
            skip_reason: None,
        }
    }

    fn skipped(reason: SkipReason) -> Self {
        Self {
            offloaded: false,
            skip_reason: Some(reason),
        }
    }
}

/// Post-tool-call hook wiring the state manager (cursor) and the entry
/// storage. Borrows both; cheap to construct per call.
pub struct AfterToolCallHook<'a> {
    state: &'a OffloadStateManager,
    storage: &'a OffloadStorage,
}

impl<'a> AfterToolCallHook<'a> {
    pub fn new(state: &'a OffloadStateManager, storage: &'a OffloadStorage) -> Self {
        Self { state, storage }
    }

    /// Handle one finished tool call. `result_size_threshold_bytes` is the
    /// minimum serialized-result size that triggers offload.
    pub fn handle(
        &self,
        session_id: &str,
        pair: &ToolPair,
        result_size_threshold_bytes: usize,
    ) -> Result<HookOutcome, OffloadError> {
        // Idempotency guard 1: cursor already at/after this call.
        if self
            .state
            .last_offloaded_tool_call_id(session_id)?
            .as_deref()
            == Some(pair.tool_call_id.as_str())
        {
            return Ok(HookOutcome::skipped(SkipReason::AlreadyProcessed));
        }
        // Idempotency guard 2: entry already stored (out-of-order replays).
        if self.storage.has_entry(session_id, &pair.tool_call_id)? {
            return Ok(HookOutcome::skipped(SkipReason::AlreadyProcessed));
        }

        let serialized = serde_json::to_string(&pair.result)?;
        if serialized.len() < result_size_threshold_bytes {
            return Ok(HookOutcome::skipped(SkipReason::BelowThreshold));
        }

        let entry = OffloadEntry {
            timestamp: pair.timestamp.clone(),
            node_id: None, // assigned later by L2
            tool_call: truncate_with_suffix(&pair.tool_name, 80, "…"),
            summary: truncate_with_suffix(&serialized, PLACEHOLDER_SUMMARY_CHARS, "…"),
            result_ref: format!("offload/{}/{}", session_id, pair.tool_call_id),
            tool_call_id: pair.tool_call_id.clone(),
            session_key: Some(session_id.to_string()),
            score: None, // assigned later by L1
        };

        // Storage dedup is authoritative: a concurrent writer between the
        // probe above and this put still cannot duplicate the record.
        if !self.storage.append_entry(session_id, &entry)? {
            return Ok(HookOutcome::skipped(SkipReason::AlreadyProcessed));
        }

        self.state
            .set_last_offloaded_tool_call_id(session_id, Some(&pair.tool_call_id))?;
        Ok(HookOutcome::offloaded())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    fn open_db() -> vantadb::sdk::VantaEmbedded {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        vantadb::sdk::VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    fn pair(id: &str, result_size: usize) -> ToolPair {
        ToolPair {
            tool_name: "read_file".into(),
            tool_call_id: id.into(),
            params: serde_json::json!({"path": "config.md"}),
            result: serde_json::json!({"body": "x".repeat(result_size)}),
            error: None,
            timestamp: "2026-08-20T10:00:00Z".into(),
            duration_ms: Some(5),
        }
    }

    fn managers(db: vantadb::sdk::VantaEmbedded) -> (OffloadStateManager, OffloadStorage) {
        (
            OffloadStateManager::new(db.clone()),
            OffloadStorage::new(db),
        )
    }

    #[test]
    fn below_threshold_is_skipped_without_cursor_move() {
        let (state, storage) = managers(open_db());
        let hook = AfterToolCallHook::new(&state, &storage);
        let outcome = hook
            .handle("s1", &pair("call_1", 10), 1000)
            .expect("handle");
        assert_eq!(outcome.skip_reason, Some(SkipReason::BelowThreshold));
        assert_eq!(
            state.last_offloaded_tool_call_id("s1").expect("cursor"),
            None
        );
    }

    #[test]
    fn above_threshold_offloads_and_advances_cursor() {
        let (state, storage) = managers(open_db());
        let hook = AfterToolCallHook::new(&state, &storage);
        let outcome = hook
            .handle("s1", &pair("call_1", 2000), 1000)
            .expect("handle");
        assert!(outcome.offloaded);
        assert_eq!(
            state
                .last_offloaded_tool_call_id("s1")
                .expect("cursor")
                .as_deref(),
            Some("call_1")
        );
        let entries = storage.read_entries("s1").expect("entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tool_call_id, "call_1");
        assert_eq!(entries[0].session_key.as_deref(), Some("s1"));
    }

    #[test]
    fn reprocessing_same_tool_call_is_idempotent() {
        let (state, storage) = managers(open_db());
        let hook = AfterToolCallHook::new(&state, &storage);
        let p = pair("call_1", 2000);
        assert!(hook.handle("s1", &p, 1000).expect("first").offloaded);
        // Replay: neither duplicates nor rewinds.
        let replay = hook.handle("s1", &p, 1000).expect("replay");
        assert_eq!(replay.skip_reason, Some(SkipReason::AlreadyProcessed));
        assert_eq!(storage.read_entries("s1").expect("entries").len(), 1);
        assert_eq!(
            state
                .last_offloaded_tool_call_id("s1")
                .expect("cursor")
                .as_deref(),
            Some("call_1")
        );
    }

    #[test]
    fn cursor_persists_across_hook_sessions() {
        let db = open_db();
        {
            let state = OffloadStateManager::new(db.clone());
            let storage = OffloadStorage::new(db.clone());
            let hook = AfterToolCallHook::new(&state, &storage);
            hook.handle("s1", &pair("call_7", 2000), 1000)
                .expect("handle");
        }
        // Fresh managers over the same DB: cursor survives.
        let (state, storage) = managers(db);
        assert_eq!(
            state
                .last_offloaded_tool_call_id("s1")
                .expect("cursor")
                .as_deref(),
            Some("call_7")
        );
        // And the replayed call is still deduped after reopen.
        let hook = AfterToolCallHook::new(&state, &storage);
        let replay = hook
            .handle("s1", &pair("call_7", 2000), 1000)
            .expect("replay");
        assert_eq!(replay.skip_reason, Some(SkipReason::AlreadyProcessed));
    }

    #[test]
    fn placeholder_summary_is_truncated_and_entry_has_no_score_yet() {
        let (state, storage) = managers(open_db());
        let hook = AfterToolCallHook::new(&state, &storage);
        hook.handle("s1", &pair("call_big", 5000), 100)
            .expect("handle");
        let entry = &storage.read_entries("s1").expect("entries")[0];
        assert!(entry.summary.chars().count() <= PLACEHOLDER_SUMMARY_CHARS + 1);
        assert_eq!(entry.score, None);
        assert_eq!(entry.node_id, None);
    }
}
