//! Claude Code client adapter (MEM-57, TDAM parity:
//! `agent-adapters/claude-code.ts` → `common/cc-request-classifier.ts` +
//! `common/user-text-extractor.ts`).
//!
//! Two concerns for CC traffic on `/v1/messages`:
//! 1. Classify each request as main / fork / sidequery so later stages
//!    (injection, mem interception, L0 capture) can route accordingly.
//! 2. Extract the text the user actually typed from a user-message content.
//!
//! Classification evidence (TDAM: CC source reverse-engineering + packet
//! captures, `2026-07-30-cc-request-routing-plan.md`):
//! - MAIN: `cache_control` marker on `messages[n-1]` (incl. single-message).
//! - FORK: marker on `messages[n-2]` — forked agents force
//!   `skip_cache_write`, which moves the marker back one slot.
//! - SIDEQUERY: no marker + empty tools + thinking disabled (standalone
//!   requests like TITLE / verify_api_key).
//!
//! Without a marker both a 3P-provider MAIN and a SIDEQUERY are possible, so
//! the fallback requires BOTH tools-empty AND thinking-disabled (`&&`, not
//! `||`: a main turn may disable just one). Anything malformed degrades to
//! [`CcRequestKind::Main`] — classification failure must behave like the
//! pre-adapter one-size-fits-all path.

use serde_json::Value;

/// Kind of a Claude Code request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CcRequestKind {
    /// Main conversation turn (cache_control marker on `messages[n-1]`).
    Main,
    /// Forked-agent request (SUGGESTION/RECAP/COMPACT/…): marker on
    /// `messages[n-2]`.
    Fork,
    /// Standalone side query (TITLE/verify_api_key/…): no marker, empty
    /// tools, thinking disabled.
    Sidequery,
}

/// Index of the LAST message whose content array holds any block carrying a
/// `cache_control` key (CC puts the marker on content blocks, never on the
/// message itself). Messages without an array content are skipped. `None`
/// when no message carries one.
pub fn find_last_cache_control_index(messages: &[Value]) -> Option<usize> {
    messages.iter().enumerate().rev().find_map(|(i, msg)| {
        let has_marker = msg
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|blocks| blocks.iter().any(|b| b.get("cache_control").is_some()));
        has_marker.then_some(i)
    })
}

/// Classify a parsed Anthropic `/v1/messages` body (TDAM `classifyCcRequest`).
///
/// Defensive narrowing throughout: unknown/malformed bodies fall back to
/// [`CcRequestKind::Main`].
pub fn classify_cc_request(body: &Value) -> CcRequestKind {
    let messages: &[Value] = body
        .get("messages")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let n = messages.len();

    if let Some(idx) = find_last_cache_control_index(messages) {
        // n-2 → fork (skipCacheWrite forced); any other position → main.
        return if Some(idx) == n.checked_sub(2) {
            CcRequestKind::Fork
        } else {
            CcRequestKind::Main
        };
    }

    let tools_empty = body
        .get("tools")
        .and_then(Value::as_array)
        .is_none_or(|tools| tools.is_empty());
    let thinking_off = body
        .get("thinking")
        .and_then(|t| t.get("type"))
        .and_then(Value::as_str)
        == Some("disabled");
    if tools_empty && thinking_off {
        return CcRequestKind::Sidequery;
    }
    CcRequestKind::Main
}

/// Text the user actually typed in a user-message `content`
/// (TDAM `extractLastUserText`): string content is itself; array content is
/// scanned BACKWARDS for the last `{"type":"text","text":string}` block.
///
/// CC prepends `<system-reminder>` metadata blocks before the real input, so
/// taking the last text block skips them naturally; tool_result/image/thinking
/// blocks are not user-typed text. `None` means nothing typed (e.g. an
/// all-tool_result content).
pub fn extract_last_user_text(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => blocks.iter().rev().find_map(|block| {
            if block.get("type").and_then(Value::as_str) != Some("text") {
                return None;
            }
            block
                .get("text")
                .and_then(Value::as_str)
                .map(str::to_string)
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn body(messages: Value, extra: Option<(&str, Value)>) -> Value {
        let mut b = json!({"model": "claude-x", "messages": messages});
        if let Some((k, v)) = extra {
            b[k] = v;
        }
        b
    }

    fn user_msg(content: Value) -> Value {
        json!({"role": "user", "content": content})
    }

    /// Message whose content array contains a cache_control-marked block.
    fn marked_msg(content: Value) -> Value {
        let mut m = user_msg(content);
        m["content"][0]["cache_control"] = json!({"type": "ephemeral"});
        m
    }

    #[test]
    fn marker_on_last_message_is_main() {
        let msgs = json!([
            user_msg(json!("hi")),
            user_msg(json!([{"type":"text","text":"turn"}])),
            marked_msg(json!([{"type":"text","text":"latest"}]))
        ]);
        assert_eq!(classify_cc_request(&body(msgs, None)), CcRequestKind::Main);
    }

    #[test]
    fn marker_at_n_minus_2_is_fork() {
        let msgs = json!([
            user_msg(json!("old")),
            marked_msg(json!([{"type":"text","text":"cached prefix"}])),
            user_msg(json!("new tail"))
        ]);
        assert_eq!(classify_cc_request(&body(msgs, None)), CcRequestKind::Fork);
    }

    #[test]
    fn marker_at_any_other_position_is_main() {
        // Marker on message 0 of 4: neither n-1 nor n-2.
        let msgs = json!([
            marked_msg(json!([{"type":"text","text":"deep history"}])),
            user_msg(json!("m2")),
            user_msg(json!("m3")),
            user_msg(json!("m4"))
        ]);
        assert_eq!(classify_cc_request(&body(msgs, None)), CcRequestKind::Main);
    }

    #[test]
    fn single_message_with_marker_is_main() {
        let msgs = json!([marked_msg(json!([{"type":"text","text":"only"}]))]);
        assert_eq!(
            classify_cc_request(&body(msgs, None)),
            CcRequestKind::Main,
            "n=1 boundary: marker sits at n-1"
        );
    }

    #[test]
    fn no_marker_tools_empty_thinking_disabled_is_sidequery() {
        let msgs = json!([user_msg(json!("title this"))]);
        assert_eq!(
            classify_cc_request(&body(msgs, Some(("thinking", json!({"type": "disabled"}))))),
            CcRequestKind::Sidequery,
            "tools absent counts as empty"
        );
    }

    #[test]
    fn no_marker_tools_present_stays_main_even_with_thinking_disabled() {
        let msgs = json!([user_msg(json!("real work"))]);
        assert_eq!(
            classify_cc_request(&body(
                msgs,
                Some(("tools", json!([{"name": "bash", "input_schema": {}}])))
            )),
            CcRequestKind::Main,
            "&& not ||: disabling only thinking must not demote a main turn"
        );
    }

    #[test]
    fn malformed_bodies_degrade_to_main() {
        assert_eq!(classify_cc_request(&json!({})), CcRequestKind::Main);
        assert_eq!(
            classify_cc_request(&json!({"messages": "not-an-array"})),
            CcRequestKind::Main
        );
        assert_eq!(
            classify_cc_request(&json!({"messages": [{"role": "user", "content": 42}]})),
            CcRequestKind::Main,
            "non-array content carries no marker → fallback rules apply"
        );
    }

    #[test]
    fn find_index_scans_backwards_over_plain_messages() {
        // No markers anywhere → None.
        let plain = vec![
            user_msg(json!("a")),
            user_msg(json!([{"type":"text","text":"b"}])),
            user_msg(json!("c")),
        ];
        assert_eq!(find_last_cache_control_index(&plain), None);
        // Two markers → the LAST one wins (backwards scan).
        let marked = vec![
            marked_msg(json!([{"type":"text","text":"first"}])),
            user_msg(json!("mid")),
            marked_msg(json!([{"type":"text","text":"last"}])),
        ];
        assert_eq!(find_last_cache_control_index(&marked), Some(2));
        assert_eq!(find_last_cache_control_index(&[]), None);
    }

    #[test]
    fn extract_string_content_returns_itself() {
        assert_eq!(
            extract_last_user_text(&json!("typed input")).as_deref(),
            Some("typed input")
        );
    }

    #[test]
    fn extract_skips_prepended_system_reminder_blocks() {
        // Real CC shape: <system-reminder> metadata blocks first, typed input last.
        let content = json!([
            {"type":"text","text":"<system-reminder>env context</system-reminder>"},
            {"type":"text","text":"<system-reminder>more metadata</system-reminder>"},
            {"type":"text","text":"what is rust?"}
        ]);
        assert_eq!(
            extract_last_user_text(&content).as_deref(),
            Some("what is rust?")
        );
    }

    #[test]
    fn extract_ignores_tool_result_image_and_non_string_text_blocks() {
        // Last usable text wins; trailing tool_result does not hide it…
        let content = json!([
            {"type":"text","text":"earlier typed"},
            {"type":"tool_result","content":[{"type":"text","text":"tool output"}]},
            {"type":"image","source":{}},
            {"type":"text"}
        ]);
        assert_eq!(
            extract_last_user_text(&content).as_deref(),
            Some("earlier typed"),
            "malformed text block skipped, scan continues backwards"
        );

        // …but all-tool_result content has nothing typed.
        let only_tools = json!([
            {"type":"tool_result","content":"out"}
        ]);
        assert_eq!(extract_last_user_text(&only_tools), None);
    }

    #[test]
    fn extract_empty_or_foreign_content_is_none() {
        assert_eq!(extract_last_user_text(&json!([])), None);
        assert_eq!(extract_last_user_text(&json!(42)), None);
        assert_eq!(extract_last_user_text(&Value::Null), None);
    }
}
