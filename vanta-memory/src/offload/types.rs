//! Data contracts for context offload.
//!
//! Scope for MEM-08b: only what the F4 cursor task (MEM-20) consumes —
//! `OffloadEntry`, `ToolPair` and `PluginState` (TDAM `offload/types.ts`,
//! reference only). The MMD / L1.5 / compaction contracts
//! (`MmdMetadata`, `MmdNode`, `TaskJudgment`, `L15Boundary`) land in F5
//! (MEM-22..24) next to their consumers — not invented ahead of need.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single offloaded tool call/result summary (one line of TDAM's
/// `offload.jsonl`; in VantaDB it maps to store records).
///
/// Source: TDAM `offload/types.ts:13-30`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OffloadEntry {
    /// ISO timestamp inherited from the original tool result.
    pub timestamp: String,
    /// Mermaid node ID assigned by L2; `None` until L2 runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Short description of the tool call command.
    pub tool_call: String,
    /// LLM-generated summary of the tool result.
    pub summary: String,
    /// Reference to the full tool result (store path/ref).
    pub result_ref: String,
    /// The original tool call ID from the provider.
    pub tool_call_id: String,
    /// Session key this entry belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    /// Replaceability score (0-10): higher = summary can better replace the
    /// original. Assigned by the L1 LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<u8>,
}

/// A buffered tool call + result pair waiting to be processed by L1
/// (produced by the after-tool-call hook).
///
/// Source: TDAM `offload/types.ts:33-41`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolPair {
    /// Tool name (e.g. `read_file`).
    pub tool_name: String,
    /// Tool call ID from the provider.
    pub tool_call_id: String,
    /// Tool parameters (object or JSON string).
    pub params: Value,
    /// Tool result payload.
    pub result: Value,
    /// Error message when the tool call failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// ISO timestamp of the tool call.
    pub timestamp: String,
    /// Tool execution duration (ms).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Persistent offload state saved per session.
///
/// The [`Self::last_offloaded_tool_call_id`] cursor marks how far the compact
/// context has summarized — the core of the MEM-20 cursor task.
///
/// Source: TDAM `offload/types.ts:44-57`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PluginState {
    /// Path to the currently active MMD file.
    pub active_mmd_file: Option<String>,
    /// Identifier/label for the active MMD.
    pub active_mmd_id: Option<String>,
    /// Counter for auto-incrementing MMD filenames.
    pub mmd_counter: u64,
    /// Last session key the plugin was active in.
    pub last_session_key: Option<String>,
    /// Last tool call ID successfully offloaded into compact context (L3
    /// cursor).
    pub last_offloaded_tool_call_id: Option<String>,
    /// ISO timestamp of the last successful L2 trigger.
    pub last_l2_trigger_time: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offload_entry_roundtrip_skips_optionals() {
        let entry = OffloadEntry {
            timestamp: "2026-08-20T10:00:00Z".into(),
            node_id: None,
            tool_call: "read_file(path=config.md)".into(),
            summary: "read config".into(),
            result_ref: "results/2026-08-20/1.md".into(),
            tool_call_id: "call_1".into(),
            session_key: None,
            score: Some(7),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"score\":7"));
        assert!(!json.contains("\"node_id\""));
        let back: OffloadEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn tool_pair_roundtrip() {
        let pair = ToolPair {
            tool_name: "edit".into(),
            tool_call_id: "call_2".into(),
            params: serde_json::json!({"path": "scene.md", "edits": [{"oldText": "a", "newText": "b"}]}),
            result: serde_json::json!({"success": true}),
            error: None,
            timestamp: "2026-08-20T10:00:00Z".into(),
            duration_ms: Some(12),
        };
        let json = serde_json::to_string(&pair).unwrap();
        let back: ToolPair = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pair);
    }

    #[test]
    fn plugin_state_cursor_roundtrip() {
        let state = PluginState {
            active_mmd_file: Some("mmds/001.md".into()),
            active_mmd_id: Some("001".into()),
            mmd_counter: 1,
            last_session_key: Some("session-a".into()),
            last_offloaded_tool_call_id: Some("call_42".into()),
            last_l2_trigger_time: None,
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: PluginState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
        assert_eq!(back.last_offloaded_tool_call_id.as_deref(), Some("call_42"));
    }
}
