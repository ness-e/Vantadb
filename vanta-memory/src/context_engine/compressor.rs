//! Compaction primitives: message fingerprinting, replaceability scoring,
//! and the aggressive-boundary re-application guard.
//!
//! Port of TDAM `offload/index.ts` (`simpleHash` :115-129,
//! `_msgFingerprint` role+200-chars) and the replaceability score semantics
//! of `offload/types.ts:28` (score 0-10, higher = a summary can replace it
//! better). The L1 LLM score is substituted by a deterministic heuristic —
//! documented ceiling, upgrade path post-MEM-24 (consume real offload-entry
//! scores).
//!
//! LLM-free by construction.

use crate::context_engine::types::{ChatMessage, ChatRole};

/// Max replacements per mild pass (TDAM MIN_COUNT, llm-input-l3.ts:113).
pub const MIN_REPLACEMENTS_PER_PASS: usize = 10;
/// Starting cascade threshold (TDAM INITIAL_THRESHOLD).
pub const INITIAL_THRESHOLD: u8 = 7;
/// Lowest cascade threshold (TDAM FLOOR_THRESHOLD).
pub const FLOOR_THRESHOLD: u8 = 1;
/// Chars of content included in the boundary fingerprint (TDAM :121-129).
pub const FINGERPRINT_CHARS: usize = 200;

/// Port of TDAM `simpleHash` (32-bit, `(hash << 5) - hash + char`).
fn simple_hash(s: &str) -> i32 {
    let mut hash: i32 = 0;
    for ch in s.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(ch as u32 as i32);
    }
    hash
}

fn role_tag(role: ChatRole) -> &'static str {
    match role {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::ToolCall => "tool_call",
        ChatRole::ToolResult => "tool_result",
    }
}

/// Boundary fingerprint: `simpleHash("role:{first 200 chars}")`. Two messages
/// with equal role and equal first-200-chars content collide by design
/// (TDAM parity); collisions only cause a conservative extra head-delete.
pub fn msg_fingerprint(msg: &ChatMessage) -> i32 {
    let head: String = msg.content.chars().take(FINGERPRINT_CHARS).collect();
    simple_hash(&format!("{}:{head}", role_tag(msg.role)))
}

/// Deterministic replaceability score (0-10): base by role + age bonus.
/// Higher = more safely replaced by a summary. `System` is never scored
/// (returns [`None`] — excluded from all compaction passes).
///
/// Base: ToolResult=6 > ToolCall=5 > Assistant=4 > User=2. Age bonus 0..=4:
/// older messages (lower `position`) score higher.
pub fn score_message(msg: &ChatMessage, position: usize, total: usize) -> Option<u8> {
    let base: u8 = match msg.role {
        ChatRole::System => return None,
        ChatRole::ToolResult => 6,
        ChatRole::ToolCall => 5,
        ChatRole::Assistant => 4,
        ChatRole::User => 2,
    };
    let denom = total.saturating_sub(1).max(1);
    let older = total.saturating_sub(1 + position.min(total.saturating_sub(1)));
    let bonus = ((older * 4) / denom).min(4) as u8;
    Some(base.saturating_add(bonus))
}

/// Marks where an aggressive one-shot cut happened, so the same head-delete
/// can be re-applied idempotently when the full history is rebuilt
/// (TDAM `_lastAggressiveBoundary`, state-manager.ts:96-101).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AggressiveBoundary {
    /// Number of leading messages removed by the cut.
    pub original_index: usize,
    /// Fingerprint of the first kept message at cut time.
    pub fingerprint: i32,
    /// Messages that survived the cut.
    pub kept_msg_count: usize,
}

impl AggressiveBoundary {
    /// Builds the boundary for a cut that removed `cut` leading messages,
    /// leaving `kept`.
    pub fn new(cut: usize, kept: &[ChatMessage]) -> Option<Self> {
        let first = kept.first()?;
        Some(Self {
            original_index: cut,
            fingerprint: msg_fingerprint(first),
            kept_msg_count: kept.len(),
        })
    }
}

/// Re-applies an aggressive head-delete, verifying the boundary fingerprint.
///
/// * Full history rebuilt → `Some(shortened)` (same result as the original cut).
/// * Already applied / history diverged → `None` (caller clears the boundary).
pub fn apply_boundary(
    msgs: &[ChatMessage],
    boundary: &AggressiveBoundary,
) -> Option<Vec<ChatMessage>> {
    let pivot = msgs.get(boundary.original_index)?;
    if msg_fingerprint(pivot) != boundary.fingerprint {
        return None;
    }
    let mut out = msgs.to_vec();
    out.drain(..boundary.original_index);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_stable_and_role_sensitive() {
        let a = ChatMessage::new(ChatRole::User, "hello world");
        let b = ChatMessage::new(ChatRole::User, "hello world");
        let c = ChatMessage::new(ChatRole::Assistant, "hello world");
        assert_eq!(msg_fingerprint(&a), msg_fingerprint(&b));
        assert_ne!(msg_fingerprint(&a), msg_fingerprint(&c));
    }

    #[test]
    fn fingerprint_uses_only_first_200_chars() {
        let head = "x".repeat(200);
        let a = ChatMessage::new(ChatRole::User, head.clone());
        let b = ChatMessage::new(ChatRole::User, format!("{head}tail-differs"));
        assert_eq!(msg_fingerprint(&a), msg_fingerprint(&b));
    }

    #[test]
    fn score_orders_roles_and_age() {
        let total = 10;
        let old_tool_result = score_message(&ChatMessage::new(ChatRole::ToolResult, "r"), 0, total);
        let new_user = score_message(&ChatMessage::new(ChatRole::User, "u"), total - 1, total);
        assert_eq!(old_tool_result, Some(10)); // 6 + max age bonus 4
        assert_eq!(new_user, Some(2)); // 2 + 0
        assert!(score_message(&ChatMessage::new(ChatRole::System, "s"), 0, total).is_none());
    }

    #[test]
    fn apply_boundary_roundtrip_and_mismatch() {
        let msgs: Vec<ChatMessage> = (0..6)
            .map(|i| ChatMessage::new(ChatRole::User, format!("m{i}-{}", "y".repeat(30))))
            .collect();
        let kept = msgs[2..].to_vec();
        let boundary = AggressiveBoundary::new(2, &kept).expect("non-empty kept");
        // Re-applied to the full history → same shortened output.
        assert_eq!(apply_boundary(&msgs, &boundary), Some(kept.clone()));
        // Re-applied to the already-shortened history → mismatch → None.
        assert_eq!(apply_boundary(&kept, &boundary), None);
    }
}
