//! Message and report types for the context engine.
//!
//! Host-neutral wire types consumed by MEM-22 (context compaction). Typed
//! roles instead of raw JSON so the tool-call-pair guard can discriminate
//! without parsing. Serde snake_case, matching crate conventions
//! (`core/abstractions/types.rs`).

use serde::{Deserialize, Serialize};

/// Role of a chat-history message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    /// Assistant tool invocation. Must always travel with its [`ChatRole::ToolResult`]s.
    ToolCall,
    /// Result of a preceding [`ChatRole::ToolCall`].
    ToolResult,
}

/// One message of the chat history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

/// How aggressive the last compaction pass was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompactionMode {
    None,
    Mild,
    Aggressive,
    Emergency,
}

/// Outcome of a compaction pass (contract expected by MEM-22 / Task 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionReport {
    pub mode: CompactionMode,
    /// Messages that survived compaction.
    pub msgs_conserved: usize,
    pub msgs_before: usize,
    pub tokens_before: u64,
    pub tokens_after: u64,
}

/// Errors of the context engine.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ContextError {
    #[error("chars_per_token must be greater than zero")]
    InvalidConfig,
    #[error("vantadb store error: {0}")]
    Store(#[from] vantadb::error::VantaError),
    #[error("malformed task-memory payload: {0}")]
    Payload(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_role_serde_snake_case_roundtrip() {
        for (role, tag) in [
            (ChatRole::System, "\"system\""),
            (ChatRole::User, "\"user\""),
            (ChatRole::Assistant, "\"assistant\""),
            (ChatRole::ToolCall, "\"tool_call\""),
            (ChatRole::ToolResult, "\"tool_result\""),
        ] {
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(json, tag);
            let back: ChatRole = serde_json::from_str(&json).unwrap();
            assert_eq!(back, role);
        }
    }

    #[test]
    fn compaction_report_serde_roundtrip() {
        let report = CompactionReport {
            mode: CompactionMode::Emergency,
            msgs_conserved: 3,
            msgs_before: 10,
            tokens_before: 900,
            tokens_after: 250,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"msgs_conserved\""));
        assert!(json.contains("\"emergency\""));
        let back: CompactionReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back, report);
    }
}
