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
//! MEM-48: messages linked to persisted L1 memories (via `ChatMessage::id`
//! matching `MemoryRecord::source_message_ids`) are scored from the REAL
//! memory priority instead of the heuristic — see [`build_memory_scores`] and
//! [`score_message`]. Messages without linked memories keep the heuristic.
//!
//! LLM-free by construction.

use std::collections::HashMap;

use crate::context_engine::types::{ChatMessage, ChatRole};
use crate::core::abstractions::MemoryRecord;

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

/// Index message_id → max priority among the L1 memories linked to it
/// (MEM-48). Precomputed once per assembly so the join is O(records +
/// links), never O(messages × records).
pub type MemoryScoreMap = HashMap<String, i32>;

/// Builds [`MemoryScoreMap`] from persisted L1 records. The map stores the
/// RAW max priority (i32, `-1` = strict global instruction); conversion to a
/// replaceability score happens at lookup in [`score_message`].
pub fn build_memory_scores(records: &[MemoryRecord]) -> MemoryScoreMap {
    let mut map = MemoryScoreMap::new();
    for record in records {
        for msg_id in &record.source_message_ids {
            let entry = map.entry(msg_id.clone()).or_insert(record.priority);
            if record.priority > *entry {
                *entry = record.priority;
            }
        }
    }
    map
}

/// Replaceability (0-10) derived from a linked-memory priority. Inverse of
/// the heuristic's semantics: HIGH memory priority ⇒ LESS safely replaced.
///
/// `priority -1` (strict global instruction) clamps to 100 ⇒ score 0, which
/// is below [`FLOOR_THRESHOLD`] — a strict instruction is never compressed.
fn replaceability_from_priority(priority: i32) -> u8 {
    10u8.saturating_sub((priority.clamp(0, 100) / 10) as u8)
}

/// Deterministic replaceability score (0-10): base by role + age bonus,
/// overridden by real L1 memory scores when the message id is linked to
/// memories ([`MemoryScoreMap`]). Higher = more safely replaced by a summary.
/// `System` is never scored (returns [`None`] — excluded from all compaction
/// passes), linked or not.
///
/// Heuristic base: ToolResult=6 > ToolCall=5 > Assistant=4 > User=2.
/// Age bonus 0..=4: older messages (lower `position`) score higher.
pub fn score_message(
    msg: &ChatMessage,
    position: usize,
    total: usize,
    memory_scores: Option<&MemoryScoreMap>,
) -> Option<u8> {
    if msg.role == ChatRole::System {
        return None;
    }
    if let (Some(id), Some(scores)) = (msg.id.as_deref(), memory_scores) {
        if let Some(&priority) = scores.get(id) {
            return Some(replaceability_from_priority(priority));
        }
    }
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
        let old_tool_result =
            score_message(&ChatMessage::new(ChatRole::ToolResult, "r"), 0, total, None);
        let new_user = score_message(
            &ChatMessage::new(ChatRole::User, "u"),
            total - 1,
            total,
            None,
        );
        assert_eq!(old_tool_result, Some(10)); // 6 + max age bonus 4
        assert_eq!(new_user, Some(2)); // 2 + 0
        assert!(score_message(&ChatMessage::new(ChatRole::System, "s"), 0, total, None).is_none());
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

    /// MEM-48 / D19: a message linked to a HIGH-priority memory scores LOWER
    /// replaceability than one linked to a LOW-priority memory — priority 90
    /// survives compaction before priority 10 does.
    #[test]
    fn memory_priority_90_survives_before_10() {
        let record = |msg_id: &str, priority: i32| MemoryRecord {
            id: format!("m_{msg_id}"),
            content: "c".into(),
            memory_type: crate::core::abstractions::MemoryType::Persona,
            priority,
            scene_name: "s".into(),
            source_message_ids: vec![msg_id.into()],
            metadata: serde_json::Value::Null,
            timestamps: vec![],
            created_at: String::new(),
            updated_at: String::new(),
            version: 1,
            session_key: "sk".into(),
            session_id: String::new(),
            task_id: None,
            team_id: None,
            user_id: None,
            agent_id: None,
            vector: None,
            heat: 0,
            superseded_by: None,
        };
        let scores = build_memory_scores(&[record("hi", 90), record("lo", 10)]);
        let total = 10;
        let high = score_message(
            &ChatMessage::new(ChatRole::Assistant, "a").with_id("hi"),
            0,
            total,
            Some(&scores),
        );
        let low = score_message(
            &ChatMessage::new(ChatRole::Assistant, "b").with_id("lo"),
            0,
            total,
            Some(&scores),
        );
        assert_eq!(high, Some(1)); // 10 - 90/10 → least replaceable
        assert_eq!(low, Some(9)); // 10 - 10/10 → most replaceable
        assert!(high < low);
    }

    /// MEM-48 / D19: without a score map (or with an unlinked message id) the
    /// deterministic heuristic fallback applies — role base + age bonus.
    #[test]
    fn fallback_heuristic_without_memory_scores() {
        let total = 10;
        let msg = || ChatMessage::new(ChatRole::ToolResult, "r");
        // No map at all.
        assert_eq!(
            score_message(&msg().with_id("x"), 0, total, None),
            Some(10) // ToolResult 6 + max age bonus 4
        );
        // Map present but id not linked → heuristic too.
        let scores = build_memory_scores(&[]);
        assert_eq!(
            score_message(&msg().with_id("unlinked"), 0, total, Some(&scores)),
            Some(10)
        );
        // Same position, no id at all.
        assert_eq!(score_message(&msg(), 0, total, Some(&scores)), Some(10));
    }
}
