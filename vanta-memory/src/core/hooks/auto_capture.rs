//! Auto-capture hook: the host-facing entry point that turns raw
//! conversation messages into persisted L0 records.
//!
//! This is the only public L0 entry point for hosts. It is LLM-free
//! (Principle 4): filtering, sanitizing and recording never block on an LLM
//! and never lose data.

use std::collections::HashSet;
use std::str::FromStr;

use vantadb::sdk::VantaEmbedded;

use super::super::conversation::l0_recorder::now_ms;
use super::super::conversation::{L0Capture, L0Error, L0Message, L0Recorder, L0Role};

/// A raw message as delivered by the host conversation stream.
///
/// `role` is the host's own string; roles outside [`L0Role`] (e.g. "system")
/// are filtered out by the hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMessage {
    /// Stable message id when the host has one; `None` falls back to a
    /// derived `t{timestamp_ms}_{index}` key in the recorder.
    pub id: Option<String>,
    pub role: String,
    pub content: String,
    /// Unix-ms timestamp; `None` falls back to the recorder's clock.
    pub timestamp_ms: Option<u64>,
}

/// Configuration for the auto-capture hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoCaptureConfig {
    /// Roles that get recorded (default: user + assistant).
    pub capture_roles: HashSet<L0Role>,
    /// Strip fenced code blocks from assistant messages before recording.
    pub strip_code_blocks: bool,
    /// Minimum content length (after trim) for a message to be recorded.
    pub min_content_len: usize,
    /// Cursor floor when no persisted cursor exists yet. Avoids dumping the
    /// whole session on the first capture after a restart.
    pub plugin_start_timestamp_ms: Option<u64>,
}

impl Default for AutoCaptureConfig {
    fn default() -> Self {
        Self {
            capture_roles: HashSet::from([L0Role::User, L0Role::Assistant]),
            strip_code_blocks: true,
            min_content_len: 1,
            plugin_start_timestamp_ms: None,
        }
    }
}

/// Outcome of a capture pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoCaptureResult {
    pub recorded_count: usize,
    /// Messages dropped by role filter, empty-content filter, or cursor.
    pub filtered_messages: usize,
    /// New cursor value after the pass.
    pub cursor_ms: u64,
}

/// Host-facing auto-capture hook. Wraps an [`L0Recorder`] plus config.
pub struct AutoCaptureHook {
    recorder: L0Recorder,
    config: AutoCaptureConfig,
}

impl AutoCaptureHook {
    /// Build the hook over an open embedded database.
    pub fn new(db: VantaEmbedded, config: AutoCaptureConfig) -> Self {
        Self {
            recorder: L0Recorder::new(db),
            config,
        }
    }

    /// Capture a turn: filter roles → sanitize content → record via the
    /// idempotent L0 recorder, honoring the persisted cursor or the
    /// plugin-start floor.
    pub fn capture(
        &self,
        session_id: &str,
        messages: Vec<RawMessage>,
    ) -> Result<AutoCaptureResult, L0Error> {
        let mut filtered_messages = 0usize;
        let mut l0_messages: Vec<L0Message> = Vec::with_capacity(messages.len());

        for msg in messages {
            let role = match L0Role::from_str(&msg.role) {
                Ok(role) => role,
                Err(_) => {
                    filtered_messages += 1;
                    continue;
                }
            };
            if !self.config.capture_roles.contains(&role) {
                filtered_messages += 1;
                continue;
            }

            let content = sanitize_content(
                &msg.content,
                role,
                self.config.strip_code_blocks,
                self.config.min_content_len,
            );
            let Some(content) = content else {
                filtered_messages += 1;
                continue;
            };

            let timestamp_ms = msg.timestamp_ms.unwrap_or_else(now_ms);
            l0_messages.push(L0Message {
                id: msg.id,
                role,
                content,
                timestamp_ms,
            });
        }

        let capture = L0Capture {
            session_id: session_id.to_string(),
            messages: l0_messages,
        };
        let result = self
            .recorder
            .record_turn(&capture, self.config.plugin_start_timestamp_ms)?;

        Ok(AutoCaptureResult {
            recorded_count: result.recorded_count,
            filtered_messages: filtered_messages + (capture.messages.len() - result.recorded.len()),
            cursor_ms: result.cursor_ms,
        })
    }
}

/// Sanitize a message for L0: trim, enforce minimum length, and optionally
/// strip fenced code blocks (assistant messages only — TDAM does the same).
/// Returns `None` when the message should be filtered out.
fn sanitize_content(
    content: &str,
    role: L0Role,
    strip_code_blocks: bool,
    min_content_len: usize,
) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.chars().count() < min_content_len {
        return None;
    }
    let cleaned = if strip_code_blocks && role == L0Role::Assistant {
        strip_fenced_code_blocks(trimmed)
    } else {
        trimmed.to_string()
    };
    if cleaned.trim().is_empty() {
        return None;
    }
    Some(cleaned)
}

/// Remove fenced code blocks (```...```) from a string by toggling a flag on
/// fence lines. Unbalanced fences leave the tail visible (never panic).
pub(crate) fn strip_fenced_code_blocks(content: &str) -> String {
    let mut in_block = false;
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if !in_block {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fenced_code_blocks() {
        let input = "answer:\n```rust\nlet x = 1;\n```\ndone";
        assert_eq!(strip_fenced_code_blocks(input), "answer:\ndone");
    }

    #[test]
    fn strips_balanced_blocks_and_keeps_text() {
        let input = "before\n```\ncode\n```\nafter";
        assert_eq!(strip_fenced_code_blocks(input), "before\nafter");
    }
}
