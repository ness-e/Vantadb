//! Contract tests D19 for MEM-22 (context engine assemble + mild/aggressive
//! cascade). Source: plan file Task 5 contract, TDAM refs @97f9465.
//!
//! (a) ratio < 0.5 → skip untouched · (b) mild cascade top-score, pairs whole
//! · (c) summary>original guard reverts · (d) aggressive one-shot +
//! fingerprint-idempotent boundary · (e) report fields · (f) 100% LLM-free.

use vanta_memory::context_engine::{
    apply_boundary, assemble, msg_fingerprint, AssembleConfig, CompactionMode, TokenEstimator,
};
use vanta_memory::context_engine::{ChatMessage, ChatRole};

fn est() -> TokenEstimator {
    TokenEstimator::default()
}

fn cfg() -> AssembleConfig {
    AssembleConfig::default()
}

/// No orphan tool_results: every ToolResult must directly follow its ToolCall.
fn assert_no_orphan_results(msgs: &[ChatMessage]) {
    let mut expect_result = false;
    for m in msgs {
        match m.role {
            ChatRole::ToolCall => expect_result = true,
            ChatRole::ToolResult => {
                assert!(expect_result, "orphan tool_result without preceding call");
            }
            _ => expect_result = false,
        }
    }
}

/// (a) ratio < 0.5 → mode None, messages byte-identical.
#[test]
fn a_ratio_below_half_skips_compaction() {
    let msgs = vec![
        ChatMessage::new(ChatRole::User, "u".repeat(300)),
        ChatMessage::new(ChatRole::Assistant, "a".repeat(300)),
        ChatMessage::new(ChatRole::User, "final"),
    ];
    let original = msgs.clone();
    // ~200 tokens vs budget 1000 → ratio 0.2 < 0.5.
    let out = assemble(msgs, 1000, &est(), 0, &cfg()).expect("valid budget");
    assert_eq!(out.report.mode, CompactionMode::None);
    assert_eq!(out.messages, original);
    assert_eq!(out.boundary, None);
    assert_eq!(out.report.tokens_after, out.report.tokens_before);
}

/// (b) Mild cascade conserves top-score units first and never splits a
/// tool_call/tool_result pair.
#[test]
fn b_mild_cascade_top_score_whole_pairs() {
    // Old big tool units score highest (ToolResult base 6 + max age bonus).
    let mut msgs = Vec::new();
    for i in 0..3 {
        msgs.push(ChatMessage::new(
            ChatRole::ToolCall,
            format!("call{i} {}", "c".repeat(300)),
        ));
        msgs.push(ChatMessage::new(
            ChatRole::ToolResult,
            format!("res{i} {}", "r".repeat(300)),
        ));
    }
    // Recent low-score messages (new User = base 2 + bonus 0).
    for i in 0..4 {
        msgs.push(ChatMessage::new(
            ChatRole::User,
            format!("recent{i} {}", "u".repeat(300)),
        ));
    }
    msgs.push(ChatMessage::new(ChatRole::User, "final question"));

    // Total ≈ (301+6)*7 msgs / 3 ≈ 720 tokens. Budget forces stubbing the
    // three old tool units (~103 tokens each) but not everything.
    let out = assemble(msgs, 400, &est(), 0, &cfg()).expect("valid budget");
    assert_eq!(out.report.mode, CompactionMode::Mild);
    assert!(out.report.tokens_after <= out.report.tokens_before);

    // Top-score old units were stubbed...
    let stubbed: Vec<&ChatMessage> = out
        .messages
        .iter()
        .filter(|m| m.content.starts_with("[compacted "))
        .collect();
    assert!(!stubbed.is_empty(), "at least one unit stubbed");
    // ...while recent messages survive intact.
    assert!(out.messages.iter().any(|m| m.content.contains("recent")));
    assert_eq!(
        out.messages.last().map(|m| m.content.as_str()),
        Some("final question")
    );
    // Pair integrity.
    assert_no_orphan_results(&out.messages);
}

/// (c) Guard: a summary (stub) longer than the original is reverted — the
/// message keeps its content.
#[test]
fn c_summary_longer_than_original_reverts() {
    let tiny = "x".repeat(19); // "[compacted 19 chars]" = 20 chars ≥ 19 → revert
    let mut msgs = vec![
        ChatMessage::new(ChatRole::Assistant, "old ".to_string() + &"a".repeat(300)),
        ChatMessage::new(ChatRole::User, tiny.clone()),
        ChatMessage::new(ChatRole::User, "keep-final"),
    ];
    msgs.insert(0, ChatMessage::new(ChatRole::System, "sys"));

    // Budget tight enough that the cascade reaches the tiny message's
    // threshold but the big assistant stub alone gets under it.
    let out = assemble(msgs, 120, &est(), 0, &cfg()).expect("valid budget");
    let untouched = out
        .messages
        .iter()
        .find(|m| m.content == tiny)
        .expect("tiny message still present");
    assert_eq!(
        untouched.content, tiny,
        "stub longer than original → reverted"
    );
}

/// (d) Aggressive one-shot drops below threshold and the boundary makes
/// re-application idempotent by fingerprint.
#[test]
fn d_aggressive_one_shot_boundary_idempotent() {
    // 30 mid-size messages: even fully stubbed they can't fit → aggressive.
    let msgs: Vec<ChatMessage> = (0..30)
        .map(|i| ChatMessage::new(ChatRole::User, format!("m{i:02} {}", "y".repeat(90))))
        .collect();
    let original = msgs.clone();
    let out = assemble(msgs, 150, &est(), 0, &cfg()).expect("valid budget");
    assert_eq!(out.report.mode, CompactionMode::Aggressive);
    assert!(out.report.tokens_after <= 150);
    assert!(out.report.msgs_conserved < out.report.msgs_before);

    let boundary = out
        .boundary
        .as_ref()
        .expect("aggressive ran → boundary set");
    // Re-applied to the full rebuilt history → identical result (idempotent).
    let reapplied = apply_boundary(&original, boundary).expect("fingerprint matches full history");
    assert_eq!(reapplied, out.messages);
    // Re-applied to the already-compacted history → mismatch → None.
    assert_eq!(apply_boundary(&out.messages, boundary), None);
    // Fingerprint is role+200-chars based and stable.
    assert_eq!(
        boundary.fingerprint,
        msg_fingerprint(&original[boundary.original_index])
    );
    assert_eq!(boundary.kept_msg_count, out.messages.len());
}

/// (e) Report exposes mode / conserved msgs / tokens before-after.
#[test]
fn e_report_fields_consistent() {
    let msgs: Vec<ChatMessage> = (0..12)
        .map(|_| ChatMessage::new(ChatRole::User, "z".repeat(300)))
        .chain(std::iter::once(ChatMessage::new(ChatRole::User, "last")))
        .collect();
    let out = assemble(msgs, 200, &est(), 0, &cfg()).expect("valid budget");
    let r = &out.report;
    assert_eq!(r.mode, CompactionMode::Aggressive);
    assert_eq!(r.msgs_conserved, out.messages.len());
    assert!(r.msgs_conserved < r.msgs_before);
    assert!(r.tokens_after <= r.tokens_before);
    assert!(r.tokens_after > 0);
}

/// (f) 100% LLM-free: assemble runs with only pure data types — no runner,
/// no host handle, deterministic across calls.
#[test]
fn f_llm_free_deterministic() {
    let build = || -> Vec<ChatMessage> {
        (0..20)
            .map(|i| ChatMessage::new(ChatRole::Assistant, format!("a{i} {}", "q".repeat(120))))
            .chain(std::iter::once(ChatMessage::new(ChatRole::User, "end")))
            .collect()
    };
    let out1 = assemble(build(), 250, &est(), 0, &cfg()).expect("valid budget");
    let out2 = assemble(build(), 250, &est(), 0, &cfg()).expect("valid budget");
    assert_eq!(out1, out2, "deterministic, no LLM in the loop");
}

/// Cursor integration (MEM-20): messages inside the protected prefix are
/// never modified nor deleted, whatever the budget.
#[test]
fn protected_prefix_never_touched() {
    let mut msgs = vec![
        ChatMessage::new(
            ChatRole::ToolCall,
            format!("offloaded-call {}", "c".repeat(500)),
        ),
        ChatMessage::new(
            ChatRole::ToolResult,
            format!("offloaded-res {}", "r".repeat(500)),
        ),
    ];
    msgs.extend(
        (0..10).map(|i| ChatMessage::new(ChatRole::User, format!("live{i} {}", "u".repeat(300)))),
    );
    let prefix_snapshot = msgs[..2].to_vec();

    let out = assemble(msgs, 200, &est(), 2, &cfg()).expect("valid budget");
    assert_eq!(
        &out.messages[..2],
        &prefix_snapshot[..],
        "prefix byte-identical"
    );
    assert_no_orphan_results(&out.messages);
}
