//! Deterministic token estimation and emergency context truncation.
//!
//! Port of TDAM `fast-token-estimate.ts` + `emergencyCompress`
//! (`offload/hooks/llm-input-l3.ts`), with the tool-call-pair guard from
//! `offload/mmd-injector.ts:231`.
//!
//! Decision D21: chars/3 heuristic, NO tiktoken, no new deps. Known ceiling:
//! underestimates CJK/code — factor is configurable; calibration deferred
//! until after MEM-22 benchmarks.

use crate::context_engine::types::{
    ChatMessage, ChatRole, CompactionMode, CompactionReport, ContextError,
};

/// Marker appended to content cut by [`truncate_content`].
const TRUNCATION_MARKER: &str = "…[truncated]";

/// Emergency ceiling for a single message's content, in characters
/// (TDAM ~600-token guard ≈ 2000 chars at 3 chars/token).
const MAX_CONTENT_CHARS: usize = 2000;

/// Deterministic token estimator: `chars / chars_per_token`.
#[derive(Debug, Clone)]
pub struct TokenEstimator {
    pub chars_per_token: usize,
}

impl Default for TokenEstimator {
    fn default() -> Self {
        Self { chars_per_token: 3 }
    }
}

impl TokenEstimator {
    /// # Errors
    /// [`ContextError::InvalidConfig`] if `chars_per_token == 0`.
    pub fn new(chars_per_token: usize) -> Result<Self, ContextError> {
        if chars_per_token == 0 {
            return Err(ContextError::InvalidConfig);
        }
        Ok(Self { chars_per_token })
    }

    /// Unicode-safe (counts chars, never bytes). Total and deterministic.
    pub fn estimate_tokens(&self, text: &str) -> u64 {
        (text.chars().count() / self.chars_per_token) as u64
    }

    /// Parity with TDAM `extractLlmVisibleText`: role line + content.
    pub fn estimate_message(&self, msg: &ChatMessage) -> u64 {
        let role = serde_json::to_string(&msg.role).unwrap_or_default();
        self.estimate_tokens(&format!("{role}\n{}", msg.content))
    }

    pub fn estimate_messages(&self, msgs: &[ChatMessage]) -> u64 {
        msgs.iter().map(|m| self.estimate_message(m)).sum()
    }
}

/// Cuts `content` to at most `max_chars` on a char boundary, appending the
/// truncation marker. Never panics on multi-byte boundaries.
pub fn truncate_content(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        return content.to_string();
    }
    let marker_len = TRUNCATION_MARKER.chars().count();
    if max_chars <= marker_len {
        // Not even room for the marker: hard-cut.
        return content.chars().take(max_chars).collect();
    }
    let mut out: String = content.chars().take(max_chars - marker_len).collect();
    out.push_str(TRUNCATION_MARKER);
    out
}

/// One atomic compaction unit: a standalone message, or a tool call plus its
/// contiguous results. The pair guard lives HERE — by construction a unit is
/// dropped whole, so a tool_call/tool_result pair can never be split.
fn build_units(msgs: Vec<ChatMessage>) -> Vec<Vec<ChatMessage>> {
    let mut units: Vec<Vec<ChatMessage>> = Vec::new();
    let mut iter = msgs.into_iter().peekable();
    while let Some(msg) = iter.next() {
        if msg.role == ChatRole::ToolCall {
            let mut unit = vec![msg];
            while matches!(iter.peek().map(|m| &m.role), Some(ChatRole::ToolResult)) {
                unit.push(iter.next().expect("peeked element exists"));
            }
            units.push(unit);
        } else {
            units.push(vec![msg]);
        }
    }
    units
}

/// Emergency truncation: drop-from-front whole units until the history fits
/// `budget_tokens`, always keeping the last `min_keep` messages. If it still
/// exceeds, truncates the largest remaining message's content to
/// [`MAX_CONTENT_CHARS`]. LLM-free, deterministic.
pub fn emergency_truncate(
    msgs: Vec<ChatMessage>,
    budget_tokens: u64,
    estimator: &TokenEstimator,
    min_keep: usize,
) -> (Vec<ChatMessage>, CompactionReport) {
    let msgs_before = msgs.len();
    let tokens_before = estimator.estimate_messages(&msgs);
    let mode = CompactionMode::Emergency;

    if tokens_before <= budget_tokens {
        return (
            msgs,
            CompactionReport {
                mode,
                msgs_conserved: msgs_before,
                msgs_before,
                tokens_before,
                tokens_after: tokens_before,
            },
        );
    }

    let units = build_units(msgs);
    let total_msgs: usize = units.iter().map(Vec::len).sum();

    // Drop leading units while over budget, never eating into the last
    // `min_keep` messages.
    let mut kept_count = total_msgs;
    let mut idx = 0;
    while idx < units.len() && kept_count - units[idx].len() >= min_keep.max(1) {
        let candidate = &units[idx..];
        let candidate_tokens: u64 = candidate
            .iter()
            .flat_map(|u| u.iter())
            .map(|m| estimator.estimate_message(m))
            .sum();
        if candidate_tokens <= budget_tokens {
            break;
        }
        kept_count -= units[idx].len();
        idx += 1;
    }
    let mut kept: Vec<ChatMessage> = units[idx..].concat();

    // Still over? Truncate the largest remaining message's content.
    let mut tokens_after = estimator.estimate_messages(&kept);
    if tokens_after > budget_tokens {
        if let Some(largest) = kept.iter_mut().max_by_key(|m| m.content.chars().count()) {
            largest.content = truncate_content(&largest.content, MAX_CONTENT_CHARS);
        }
        tokens_after = estimator.estimate_messages(&kept);
    }

    (
        kept.clone(),
        CompactionReport {
            mode,
            msgs_conserved: kept.len(),
            msgs_before,
            tokens_before,
            tokens_after,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_engine::types::{ChatMessage, ChatRole};

    fn est() -> TokenEstimator {
        TokenEstimator::default()
    }

    #[test]
    fn estimate_tokens_empty_ascii_unicode_deterministic() {
        let e = est();
        assert_eq!(e.estimate_tokens(""), 0);
        assert_eq!(e.estimate_tokens("abcdef"), 2); // 6 chars / 3
                                                    // Unicode counts chars, not bytes ("ñáé" = 3 chars = 1 token).
        assert_eq!(e.estimate_tokens("ñáé"), 1);
        // Determinism.
        assert_eq!(
            e.estimate_tokens("héllo wörld 🚀"),
            e.estimate_tokens("héllo wörld 🚀")
        );
    }

    #[test]
    fn estimator_rejects_zero_factor() {
        assert!(TokenEstimator::new(0).is_err());
    }

    #[test]
    fn truncate_content_respects_char_boundary() {
        assert_eq!(truncate_content("short", 10), "short");
        let long = "ñ".repeat(50);
        let cut = truncate_content(&long, 30);
        assert!(cut.ends_with(TRUNCATION_MARKER));
        assert!(cut.chars().count() <= 30);
        // max smaller than the marker → hard cut, still char-boundary safe.
        let tiny = truncate_content(&long, 5);
        assert_eq!(tiny.chars().count(), 5);
    }

    #[test]
    fn emergency_noop_when_under_budget() {
        let msgs = vec![ChatMessage::new(ChatRole::User, "hi")];
        let (_, report) = emergency_truncate(msgs.clone(), 100, &est(), 1);
        assert_eq!(report.mode, CompactionMode::Emergency);
        assert_eq!(report.msgs_conserved, 1);
        assert_eq!(report.msgs_before, 1);
        assert_eq!(report.tokens_after, report.tokens_before);
    }

    #[test]
    fn tool_call_pair_never_split_by_truncation() {
        let e = est();
        let filler = "x".repeat(300); // 100 tokens each
        let msgs = vec![
            ChatMessage::new(ChatRole::User, filler.clone()),
            ChatMessage::new(ChatRole::Assistant, filler.clone()),
            ChatMessage::new(ChatRole::ToolCall, "{\"name\":\"search\"}"),
            ChatMessage::new(ChatRole::ToolResult, "[1,2,3]"),
            ChatMessage::new(ChatRole::ToolResult, "[4,5,6]"),
            ChatMessage::new(ChatRole::User, filler),
        ];
        // Budget forces drops but keeps the last message → the pair must die
        // or live together with whatever precedes it.
        let (kept, _) = emergency_truncate(msgs, 150, &e, 1);
        let has_call = kept.iter().any(|m| m.role == ChatRole::ToolCall);
        let results: usize = kept
            .iter()
            .filter(|m| m.role == ChatRole::ToolResult)
            .count();
        if has_call {
            assert_eq!(results, 2, "call present → both its results must survive");
        } else {
            assert_eq!(results, 0, "no call → no orphan results");
        }
        // min_keep respected.
        assert!(!kept.is_empty());
        assert_eq!(kept.last().map(|m| m.role), Some(ChatRole::User));
    }

    #[test]
    fn min_keep_final_messages_never_dropped() {
        let e = est();
        let msgs: Vec<ChatMessage> = (0..10)
            .map(|_| ChatMessage::new(ChatRole::User, "y".repeat(300)))
            .chain(std::iter::once(ChatMessage::new(ChatRole::User, "keep me")))
            .collect();
        let (kept, _) = emergency_truncate(msgs, 50, &e, 3);
        assert!(kept.len() >= 3);
        assert_eq!(kept.last().map(|m| m.content.as_str()), Some("keep me"));
    }

    #[test]
    fn oversized_single_message_content_gets_truncated() {
        let e = est();
        let huge = "z".repeat(10_000);
        let msgs = vec![ChatMessage::new(ChatRole::User, huge)];
        let (kept, report) = emergency_truncate(msgs, 10, &e, 1);
        assert_eq!(kept.len(), 1);
        assert!(kept[0].content.contains(TRUNCATION_MARKER));
        assert!(report.tokens_after < report.tokens_before);
    }
}
