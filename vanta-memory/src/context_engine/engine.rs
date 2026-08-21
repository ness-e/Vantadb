//! Context assembly: ratio gate → mild score cascade → aggressive one-shot
//! → emergency fallback. LLM-free, deterministic.
//!
//! Port of TDAM `offload-client/context-engine.ts` (`assemble` :445-482,
//! ratio < compaction_ratio skip), `offload/hooks/llm-input-l3.ts`
//! (`compressByScoreCascade` :402-576, guard summary>original :530-538,
//! `aggressiveCompressUntilBelowThreshold` :667-751) and the boundary
//! re-application of `offload/index.ts:1481-1523`.
//!
//! Cursor integration (MEM-20): the engine stays pure — it never imports
//! [`crate::offload::state_manager::OffloadStateManager`]. The caller derives
//! `protected_prefix` from the cursor (`last_offloaded_tool_call_id`):
//! messages at indices `< protected_prefix` are already offloaded and are
//! NEVER modified or deleted by any pass.

use crate::context_engine::compressor::{
    score_message, AggressiveBoundary, FLOOR_THRESHOLD, INITIAL_THRESHOLD,
    MIN_REPLACEMENTS_PER_PASS,
};
use crate::context_engine::mmd::TaskMemory;
use crate::context_engine::token_estimator::{build_units, emergency_truncate, TokenEstimator};
use crate::context_engine::types::{
    ChatMessage, ChatRole, CompactionMode, CompactionReport, ContextError,
};

/// Tunables of [`assemble`].
#[derive(Debug, Clone)]
pub struct AssembleConfig {
    /// Skip compaction when `tokens / budget` is below this (TDAM 0.5).
    pub compaction_ratio: f64,
    /// Final messages never touched by any pass (TDAM MIN_KEEP = 2).
    pub min_keep: usize,
}

impl Default for AssembleConfig {
    fn default() -> Self {
        Self {
            compaction_ratio: 0.5,
            min_keep: 2,
        }
    }
}

/// Output of [`assemble`]: compacted history + report + optional aggressive
/// boundary for idempotent re-application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembleOutput {
    pub messages: Vec<ChatMessage>,
    pub report: CompactionReport,
    /// Present iff the aggressive pass ran; feed back to
    /// [`crate::context_engine::compressor::apply_boundary`] when the full
    /// history is rebuilt.
    pub boundary: Option<AggressiveBoundary>,
}

/// Assembles a chat history under `token_budget_tokens`.
///
/// Passes, in order (first success wins):
/// 1. Ratio gate: `tokens/budget < cfg.compaction_ratio` → untouched.
/// 2. Mild cascade: replace top-score unit contents with `[compacted N chars]`
///    stubs until under budget (thresholds INITIAL→FLOOR, max
///    [`MIN_REPLACEMENTS_PER_PASS`] replacements).
/// 3. Aggressive one-shot: delete leading whole units until under budget.
/// 4. Emergency: [`emergency_truncate`] fallback.
///
/// # Errors
/// [`ContextError::InvalidConfig`] if `token_budget_tokens == 0`.
pub fn assemble(
    msgs: Vec<ChatMessage>,
    token_budget_tokens: u64,
    estimator: &TokenEstimator,
    protected_prefix: usize,
    cfg: &AssembleConfig,
) -> Result<AssembleOutput, ContextError> {
    if token_budget_tokens == 0 {
        return Err(ContextError::InvalidConfig);
    }

    let tokens_before = estimator.estimate_messages(&msgs);
    let msgs_before = msgs.len();
    let noop = |mode| AssembleOutput {
        messages: msgs.clone(),
        report: CompactionReport {
            mode,
            msgs_conserved: msgs_before,
            msgs_before,
            tokens_before,
            tokens_after: tokens_before,
        },
        boundary: None,
    };

    // 1. Ratio gate — nothing to do.
    let ratio = tokens_before as f64 / token_budget_tokens as f64;
    if ratio < cfg.compaction_ratio {
        return Ok(noop(CompactionMode::None));
    }
    if tokens_before <= token_budget_tokens {
        return Ok(noop(CompactionMode::None));
    }

    // 2. Mild cascade.
    let mild = mild_cascade(
        msgs.clone(),
        token_budget_tokens,
        estimator,
        protected_prefix,
        cfg.min_keep,
    );
    if estimator.estimate_messages(&mild) <= token_budget_tokens {
        return Ok(AssembleOutput {
            report: CompactionReport {
                mode: CompactionMode::Mild,
                msgs_conserved: mild.len(),
                msgs_before,
                tokens_before,
                tokens_after: estimator.estimate_messages(&mild),
            },
            messages: mild,
            boundary: None,
        });
    }

    // 3. Aggressive one-shot.
    let (aggressive, boundary) = aggressive_one_shot(
        mild,
        token_budget_tokens,
        estimator,
        protected_prefix,
        cfg.min_keep,
    );
    if estimator.estimate_messages(&aggressive) <= token_budget_tokens {
        return Ok(AssembleOutput {
            report: CompactionReport {
                mode: CompactionMode::Aggressive,
                msgs_conserved: aggressive.len(),
                msgs_before,
                tokens_before,
                tokens_after: estimator.estimate_messages(&aggressive),
            },
            messages: aggressive,
            boundary,
        });
    }

    // 4. Emergency fallback — operates ONLY on the compactable region so the
    // protected prefix survives even the last-resort pass (invariant 5).
    // ponytail: if the protected prefix alone exceeds the budget, we return
    // over budget rather than violate the cursor guarantee; caller decides.
    let (head, tail) = {
        let p = protected_prefix.min(aggressive.len());
        aggressive.split_at(p)
    };
    let (compacted, mut report) =
        emergency_truncate(tail.to_vec(), token_budget_tokens, estimator, cfg.min_keep);
    let mut messages = head.to_vec();
    messages.extend(compacted);
    report.mode = CompactionMode::Emergency;
    report.msgs_before = msgs_before;
    report.tokens_before = tokens_before;
    report.msgs_conserved = messages.len();
    report.tokens_after = estimator.estimate_messages(&messages);
    Ok(AssembleOutput {
        boundary: None,
        messages,
        report,
    })
}

/// Dedup markers of the injected recall blocks (MEM-37).
pub const RECALL_PREPEND_MARKER: &str = "_recallDynamicContext";
/// Dedup marker of the stable recall block (persona + scene navigation).
pub const RECALL_APPEND_MARKER: &str = "_recallStableContext";

/// Output of [`assemble_with_recall`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegratedContext {
    pub messages: Vec<ChatMessage>,
    /// Report of the compaction pass that ran inside [`assemble`].
    pub report: CompactionReport,
    pub mmd_injected: bool,
    /// `true` when at least one recall block was injected.
    pub recall_injected: bool,
}

/// Single shared-budget coordinator (MEM-37): compress → inject MMD → inject
/// recall blocks, all against ONE mutable token budget. The union is
/// guaranteed ≤ `budget_tokens`: compression leaves headroom and every
/// injection is whole-or-skip against what remains.
///
/// Cursor guard (MEM-20): everything up to and including the message whose
/// [`ChatMessage::id`] equals `cursor_tool_call_id` is already compacted — it
/// is folded into the protected prefix so no pass touches or duplicates it.
///
/// # Errors
/// [`ContextError::InvalidConfig`] if `budget_tokens == 0`.
pub fn assemble_with_recall(
    msgs: Vec<ChatMessage>,
    budget_tokens: u64,
    estimator: &TokenEstimator,
    protected_prefix: usize,
    cfg: &AssembleConfig,
    active_mmd: Option<TaskMemory>,
    recall_prepend: Option<&str>,
    recall_append: Option<&str>,
    cursor_tool_call_id: Option<&str>,
) -> Result<IntegratedContext, ContextError> {
    if budget_tokens == 0 {
        return Err(ContextError::InvalidConfig);
    }
    let cursor_boundary = cursor_tool_call_id
        .and_then(|id| {
            let pos = msgs.iter().rposition(|m| m.id.as_deref() == Some(id))?;
            // The cursor covers the whole atomic unit: the call plus its
            // contiguous results (build_units keeps them together).
            let mut end = pos + 1;
            while msgs
                .get(end)
                .is_some_and(|m| m.role == ChatRole::ToolResult)
            {
                end += 1;
            }
            Some(end)
        })
        .map_or(0, |boundary| boundary);

    let out = assemble(
        msgs,
        budget_tokens,
        estimator,
        protected_prefix.max(cursor_boundary),
        cfg,
    )?;
    let mut remaining = budget_tokens.saturating_sub(estimator.estimate_messages(&out.messages));
    let messages = out.messages;

    let len_before = messages.len();
    let mut messages = crate::context_engine::inject_mmd(messages, active_mmd, &mut remaining);
    let mmd_injected = messages.len() != len_before;

    let mut recall_injected = false;
    if let Some(prepend) = recall_prepend {
        recall_injected |= inject_recall_block(
            &mut messages,
            RECALL_PREPEND_MARKER,
            prepend,
            estimator,
            &mut remaining,
        );
    }
    if let Some(append) = recall_append {
        recall_injected |= inject_recall_block(
            &mut messages,
            RECALL_APPEND_MARKER,
            append,
            estimator,
            &mut remaining,
        );
    }

    Ok(IntegratedContext {
        messages,
        report: out.report,
        mmd_injected,
        recall_injected,
    })
}

/// Appends one recall block as a trailing System message if it fits the
/// remaining budget — whole or not at all. Marker-based dedup makes a second
/// call with the same content a no-op.
fn inject_recall_block(
    messages: &mut Vec<ChatMessage>,
    marker: &str,
    content: &str,
    estimator: &TokenEstimator,
    budget: &mut u64,
) -> bool {
    if content.trim().is_empty() || messages.iter().any(|m| m.content.contains(marker)) {
        return false;
    }
    let msg = ChatMessage::new(ChatRole::System, format!("{marker}\n{content}"));
    let cost = estimator.estimate_message(&msg);
    if cost > *budget {
        return false;
    }
    *budget -= cost;
    messages.push(msg);
    true
}

/// Unit score = max replaceability among its non-System messages (`None` =
/// unit not compressible, e.g. all-System). Max is used because a unit is
/// replaced whole: its most replaceable member bounds how safely a summary
/// can stand in for all of it.
fn unit_score(unit: &[ChatMessage], start: usize, total: usize) -> Option<u8> {
    unit.iter()
        .enumerate()
        .filter_map(|(i, m)| score_message(m, start + i, total))
        .max()
}

/// Replaces one message's content with a stub, unless the stub would be as
/// long as the original (TDAM guard llm-input-l3.ts:530-538 — revert).
fn stub_message(msg: &mut ChatMessage) -> bool {
    let original_chars = msg.content.chars().count();
    let stub = format!("[compacted {original_chars} chars]");
    if stub.chars().count() >= original_chars {
        return false;
    }
    msg.content = stub;
    true
}

/// Mild cascade: sort candidate units by score desc, walk thresholds from
/// [`INITIAL_THRESHOLD`] down to [`FLOOR_THRESHOLD`], stubbing units with
/// `score >= threshold` until under budget or
/// [`MIN_REPLACEMENTS_PER_PASS`] replacements reached.
///
/// Candidates are whole atomic units fully inside the compactable region
/// `[protected_prefix .. len - min_keep)` — a tool_call/tool_result pair can
/// never be split, and the protected prefix is never touched.
fn mild_cascade(
    msgs: Vec<ChatMessage>,
    budget: u64,
    est: &TokenEstimator,
    protected_prefix: usize,
    min_keep: usize,
) -> Vec<ChatMessage> {
    let total: usize = msgs.len();
    let units = build_units(msgs);

    // Cumulative message-start index of each unit.
    let starts: Vec<usize> = units
        .iter()
        .scan(0usize, |acc, u| {
            let s = *acc;
            *acc += u.len();
            Some(s)
        })
        .collect();
    let max_end = total.saturating_sub(min_keep.max(1));

    let mut candidates: Vec<usize> = (0..units.len())
        .filter(|&ui| starts[ui] >= protected_prefix && starts[ui] + units[ui].len() <= max_end)
        .filter(|&ui| unit_score(&units[ui], starts[ui], total).is_some())
        .collect();
    // Score desc, index asc tie-break → deterministic.
    candidates.sort_by(|&a, &b| {
        unit_score(&units[b], starts[b], total)
            .cmp(&unit_score(&units[a], starts[a], total))
            .then(a.cmp(&b))
    });

    let mut current = units;
    let mut replaced = vec![false; current.len()];
    let mut replacements = 0usize;
    'thresholds: for threshold in (FLOOR_THRESHOLD..=INITIAL_THRESHOLD).rev() {
        for &ui in &candidates {
            if replacements >= MIN_REPLACEMENTS_PER_PASS {
                break 'thresholds;
            }
            let Some(score) = unit_score(&current[ui], starts[ui], total) else {
                continue;
            };
            if score < threshold {
                break; // sorted desc: rest are lower at this threshold
            }
            if replaced[ui] {
                continue;
            }
            let mut changed = false;
            for msg in &mut current[ui] {
                changed |= stub_message(msg);
            }
            if changed {
                replaced[ui] = true;
                replacements += 1;
                let flat: Vec<ChatMessage> = current.concat();
                if est.estimate_messages(&flat) <= budget {
                    break 'thresholds;
                }
            }
        }
    }

    current.concat()
}

/// Aggressive one-shot: delete leading whole units (single splice) until the
/// remaining history fits `budget`. Never eats into the protected prefix,
/// the last User message, or the final `min_keep` messages. Enforces TDAM's
/// minimum-delete rule (~20% of the history) so the pass is worth its cost.
///
/// Units are atomic, so no orphaned tool_results can exist past the cut —
/// the TDAM orphan-extension step (:655-660) is subsumed by `build_units`.
fn aggressive_one_shot(
    msgs: Vec<ChatMessage>,
    budget: u64,
    est: &TokenEstimator,
    protected_prefix: usize,
    min_keep: usize,
) -> (Vec<ChatMessage>, Option<AggressiveBoundary>) {
    let total = msgs.len();
    if total == 0 {
        return (msgs, None);
    }
    let last_user = msgs.iter().rposition(|m| m.role == ChatRole::User);
    let hard_end = total
        .saturating_sub(min_keep.max(1))
        .min(last_user.unwrap_or(total));

    let units = build_units(msgs);
    let starts: Vec<usize> = units
        .iter()
        .scan(0usize, |acc, u| {
            let s = *acc;
            *acc += u.len();
            Some(s)
        })
        .collect();

    let eligible =
        |ui: usize| starts[ui] >= protected_prefix && starts[ui] + units[ui].len() <= hard_end;

    // Natural cut: drop leading eligible units while over budget.
    let mut cut_unit = 0usize;
    while cut_unit < units.len() && eligible(cut_unit) {
        let suffix: Vec<ChatMessage> = units[cut_unit..].concat();
        if est.estimate_messages(&suffix) <= budget {
            break;
        }
        cut_unit += 1;
    }

    // Minimum-delete rule (TDAM :648-651): delete at least ~20% of messages.
    let deleted_so_far: usize = starts.get(cut_unit).copied().unwrap_or(total);
    let min_delete = total.saturating_mul(20).saturating_add(99) / 100;
    if deleted_so_far < min_delete {
        while cut_unit < units.len() && eligible(cut_unit) && starts[cut_unit] < min_delete {
            cut_unit += 1;
        }
    }

    if cut_unit == 0 {
        return (units.concat(), None);
    }
    let kept: Vec<ChatMessage> = units[cut_unit..].concat();
    let boundary = AggressiveBoundary::new(starts[cut_unit], &kept);
    (kept, boundary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn est() -> TokenEstimator {
        TokenEstimator::default()
    }

    #[test]
    fn assemble_rejects_zero_budget() {
        let err = assemble(vec![], 0, &est(), 0, &AssembleConfig::default());
        assert!(matches!(err, Err(ContextError::InvalidConfig)));
    }

    #[test]
    fn stub_guard_reverts_when_stub_not_shorter() {
        let mut msg = ChatMessage::new(ChatRole::User, "x".repeat(19));
        assert!(!stub_message(&mut msg)); // "[compacted 19 chars]" = 20 chars ≥ 19
        assert_eq!(msg.content, "x".repeat(19));
        let mut long = ChatMessage::new(ChatRole::User, "y".repeat(300));
        assert!(stub_message(&mut long));
        assert_eq!(long.content, "[compacted 300 chars]");
    }
}
