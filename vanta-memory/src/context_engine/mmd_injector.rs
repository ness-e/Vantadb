//! MMD context injection into a chat history (MEM-24).
//!
//! Port of TDAM `mmd-injector.ts` (inject path): the active task memory is
//! inserted as one system message right after the leading protected prefix
//! (system prompt), wrapped in `<current_task_context>` tags and marked
//! `_mmdContextMessage`. The insertion point is always a
//! [`build_units`] boundary, so a tool_call/tool_result pair can never be
//! split. Re-injecting the same content is skipped via the MMD fingerprint.
//! The injected tokens are discounted from `budget`.

use crate::context_engine::mmd::{fingerprint, TaskMemory};
use crate::context_engine::token_estimator::{build_units, TokenEstimator};
use crate::context_engine::types::{ChatMessage, ChatRole};

/// Marker of an injected MMD context message.
pub const MMD_CONTEXT_MARKER: &str = "_mmdContextMessage";

/// Inject the active task memory into `messages`, discounting its cost from
/// `budget`. No-op when `active` is `None`/empty or the same content is
/// already present (fingerprint dedup). If the message does not fit the
/// remaining budget it is not injected at all — never partially.
pub fn inject_mmd(
    messages: Vec<ChatMessage>,
    active: Option<TaskMemory>,
    budget: &mut u64,
) -> Vec<ChatMessage> {
    let Some(memory) = active.filter(|m| !m.content.is_empty()) else {
        return messages;
    };
    let fp = fingerprint(&memory.content);
    let marker_line = format!("{MMD_CONTEXT_MARKER} fp={fp}");
    if messages.iter().any(|m| m.content.contains(&marker_line)) {
        return messages; // same context already injected → dedup
    }
    let wrapped = format!(
        "{marker_line}\n<current_task_context>\n{}\n</current_task_context>",
        memory.content
    );
    let msg = ChatMessage::new(ChatRole::System, wrapped);
    let cost = TokenEstimator::default().estimate_message(&msg);
    if cost > *budget {
        return messages; // no budget for context → skip entirely
    }

    // Insert after the leading run of System messages (the protected prefix).
    // Every unit boundary is pair-safe by construction of build_units.
    let mut units = build_units(messages);
    let insert_at = units
        .iter()
        .take_while(|u| u.first().is_some_and(|m| m.role == ChatRole::System))
        .count();
    let mut out: Vec<ChatMessage> = Vec::with_capacity(units.len());
    for (i, unit) in units.drain(..).enumerate() {
        if i == insert_at {
            out.push(msg.clone());
            *budget = budget.saturating_sub(cost);
        }
        out.extend(unit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn est() -> TokenEstimator {
        TokenEstimator::default()
    }

    fn sys(content: &str) -> ChatMessage {
        ChatMessage::new(ChatRole::System, content)
    }
    fn user(content: &str) -> ChatMessage {
        ChatMessage::new(ChatRole::User, content)
    }
    fn tool_call(id: &str) -> ChatMessage {
        ChatMessage::new(ChatRole::ToolCall, id)
    }
    fn tool_result(id: &str) -> ChatMessage {
        ChatMessage::new(ChatRole::ToolResult, id)
    }

    fn active(content: &str) -> Option<TaskMemory> {
        Some(TaskMemory {
            meta: crate::core::abstractions::SceneMeta {
                created: "2026-08-21T10:00:00.000Z".into(),
                updated: "2026-08-21T10:05:00.000Z".into(),
                summary: "s".into(),
                heat: 1,
            },
            content: content.into(),
        })
    }

    /// D19 (a): after an aggressive assemble pass, inject_mmd adds the MMD
    /// context message.
    #[test]
    fn a_mmd_injected_after_aggressive_assemble() {
        let msgs: Vec<ChatMessage> = std::iter::once(sys("protected"))
            .chain((0..40).map(|i| user(&format!("filler message number {i:03} with padding"))))
            .collect();
        let output = crate::context_engine::assemble(
            msgs,
            300,
            &est(),
            1,
            &crate::context_engine::AssembleConfig::default(),
            None,
        )
        .expect("assemble");
        let budget = 10_000;
        let mut b = budget;
        let injected = inject_mmd(
            output.messages,
            active("current task: build parser"),
            &mut b,
        );
        assert!(
            injected.iter().any(|m| {
                m.role == ChatRole::System
                    && m.content.contains(MMD_CONTEXT_MARKER)
                    && m.content.contains("<current_task_context>")
            }),
            "MMD must be present post-aggressive"
        );
        assert!(b < budget, "budget must be discounted");
    }

    /// D19 (b): fingerprint dedup — the same content is not re-injected.
    #[test]
    fn b_dedup_does_not_reinject_same_content() {
        let msgs = vec![sys("sys"), user("hi")];
        let mem = active("same task");
        let mut b = 10_000;
        let once = inject_mmd(msgs.clone(), mem.clone(), &mut b);
        assert_eq!(
            once.iter()
                .filter(|m| m.content.contains(MMD_CONTEXT_MARKER))
                .count(),
            1
        );
        let twice = inject_mmd(once, mem, &mut b);
        assert_eq!(
            twice
                .iter()
                .filter(|m| m.content.contains(MMD_CONTEXT_MARKER))
                .count(),
            1,
            "re-injection of identical content must be skipped"
        );
    }

    /// D19 (c): tool_call/tool_result pairs stay intact after injection.
    #[test]
    fn c_tool_pairs_intact_after_injection() {
        let msgs = vec![
            user("run the tool"),
            tool_call("tc1"),
            tool_result("tc1"),
            user("thanks"),
        ];
        let out = inject_mmd(msgs, active("ctx"), &mut 10_000);
        // No injection point may fall inside a pair: verify adjacency holds.
        for w in out.windows(2) {
            if w[0].role == ChatRole::ToolCall {
                assert_eq!(w[1].role, ChatRole::ToolResult, "pair split by injection");
            }
        }
        assert_eq!(out.len(), 5); // 4 original + 1 injected
    }

    #[test]
    fn no_active_or_empty_content_is_noop() {
        let msgs = vec![user("hi")];
        let mut b = 100;
        assert_eq!(inject_mmd(msgs.clone(), None, &mut b), msgs);
        assert_eq!(inject_mmd(msgs.clone(), active(""), &mut b), msgs);
        assert_eq!(b, 100, "budget untouched on no-op");
    }

    #[test]
    fn skips_entirely_when_budget_too_small() {
        let msgs = vec![user("hi")];
        let mut b = 1;
        let out = inject_mmd(msgs, active("some context"), &mut b);
        assert_eq!(out.len(), 1, "no partial injection");
        assert_eq!(b, 1);
    }
}
