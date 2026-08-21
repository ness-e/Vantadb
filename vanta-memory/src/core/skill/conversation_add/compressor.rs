//! SkillMessageCompressor (MEM-17) — compress oversized tool payloads before
//! they enter the skill buffer.
//!
//! Port of TDAM `conversation-add/message-compressor.ts`: only
//! `tool_call`/`tool_result` are compressible; user/assistant/system never
//! are. Content over the byte threshold keeps head + tail with a placeholder.
//! Slicing is char-boundary safe (TDAM tolerated U+FFFD on raw Buffer slices;
//! Rust makes the correct behaviour free).

use serde::{Deserialize, Serialize};

/// Compression tuning (TDAM defaults).
#[derive(Debug, Clone)]
pub struct CompressOptions {
    /// Tool messages whose content exceeds this many bytes get compressed.
    pub tool_content_threshold_bytes: usize,
    /// Bytes kept from the start.
    pub head_bytes: usize,
    /// Bytes kept from the end.
    pub tail_bytes: usize,
}

/// TDAM `DEFAULT_COMPRESS_OPTIONS`.
pub const COMPRESS_DEFAULTS: CompressOptions = CompressOptions {
    tool_content_threshold_bytes: 2_048,
    head_bytes: 1_024,
    tail_bytes: 1_024,
};

/// One buffered conversation message (role + content; roles use the TDAM set:
/// `user` | `assistant` | `tool_call` | `tool_result` | `system`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMessage {
    pub role: String,
    pub content: String,
}

impl SkillMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    fn is_tool(&self) -> bool {
        matches!(self.role.as_str(), "tool_call" | "tool_result")
    }
}

/// Compress one message when it qualifies; returns `None` when unchanged
/// (identity-preserving so callers can detect real compression).
pub fn compress_message(msg: &SkillMessage, opts: &CompressOptions) -> Option<SkillMessage> {
    if !msg.is_tool() || msg.content.len() <= opts.tool_content_threshold_bytes {
        return None;
    }
    let bytes = msg.content.as_bytes();
    // Char-boundary-safe slice bounds (never split a multi-byte sequence).
    let mut head_end = opts.head_bytes.min(bytes.len());
    while head_end < bytes.len() && !msg.content.is_char_boundary(head_end) {
        head_end += 1;
    }
    let mut tail_start = bytes.len().saturating_sub(opts.tail_bytes);
    while tail_start > 0 && !msg.content.is_char_boundary(tail_start) {
        tail_start -= 1;
    }
    let head = &msg.content[..head_end];
    let tail = &msg.content[tail_start..];
    Some(SkillMessage {
        role: msg.role.clone(),
        content: format!("{head}\n\n[middle content too long — compressed to head/tail]\n\n{tail}"),
    })
}

/// Compress a batch; returns a new vector (unchanged messages cloned as-is).
pub fn compress_messages(messages: &[SkillMessage], opts: &CompressOptions) -> Vec<SkillMessage> {
    messages
        .iter()
        .map(|m| compress_message(m, opts).unwrap_or_else(|| m.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_and_non_tool_passthrough() {
        let user = SkillMessage::new("user", "x".repeat(9_999));
        assert!(compress_message(&user, &COMPRESS_DEFAULTS).is_none());
        let tool = SkillMessage::new("tool_result", "short");
        assert!(compress_message(&tool, &COMPRESS_DEFAULTS).is_none());
    }

    #[test]
    fn long_tool_compressed_to_head_tail() {
        let tool = SkillMessage::new("tool_call", "H".repeat(4_096));
        let out = compress_message(&tool, &COMPRESS_DEFAULTS).expect("compressed");
        assert!(out.content.starts_with("HHH"));
        assert!(out.content.ends_with("HHH"));
        assert!(out.content.contains("[middle content too long"));
        assert!(out.content.len() < 4_096);
    }

    #[test]
    fn multibyte_boundaries_never_panic() {
        // 'é' is 2 bytes in UTF-8; an odd byte cut would panic without the
        // boundary adjustment.
        let tool = SkillMessage::new("tool_result", "é".repeat(2_000));
        let out = compress_message(&tool, &COMPRESS_DEFAULTS).expect("compressed");
        assert!(!out.content.contains('\u{FFFD}'));
    }
}
