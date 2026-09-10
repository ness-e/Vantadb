// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! PRX-11 slice 1 (RED): Anthropic↔OpenAI translation contract.
//! Fails with E0432 until `vanta_proxy::translate` exists (TDD RED).

use serde_json::{json, Value};
use vanta_proxy::translate::{
    anthropic_event_to_openai, anthropic_to_openai, clamp_max_tokens, openai_chunk_to_anthropic,
    openai_to_anthropic, pass_through_beta_headers, sanitize_request,
};

const CLAUDE_REQ: &str = include_str!("fixtures/claude_code_messages_request.json");
const OPENCODE_REQ: &str = include_str!("fixtures/opencode_chat_request.json");

#[test]
fn claude_request_maps_to_openai_shape() {
    let req: Value = serde_json::from_str(CLAUDE_REQ).expect("fixture parses");
    let out = anthropic_to_openai(&req);
    assert_eq!(out["model"], "claude-opus-4-6");
    let msgs = out["messages"].as_array().expect("messages array");
    assert_eq!(msgs[0]["role"], "system");
    assert!(
        msgs[0]["content"]
            .as_str()
            .unwrap_or_default()
            .contains("Claude Code"),
        "system string lands on system message"
    );
    assert_eq!(msgs[1]["role"], "user");
    assert_eq!(out["max_tokens"], 1024, "max_tokens preserved");
    let tools = out["tools"].as_array().expect("tools array");
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "Read");
    assert!(
        tools[0]["function"]["parameters"]["properties"].is_object(),
        "input_schema mapped to parameters"
    );
    assert_eq!(out["tool_choice"], "auto");
}

#[test]
fn missing_max_tokens_defaults_and_clamps() {
    assert_eq!(clamp_max_tokens(None), 1024);
    assert_eq!(clamp_max_tokens(Some(0)), 1);
    assert_eq!(clamp_max_tokens(Some(9_999_999)), 128_000);
    let out = anthropic_to_openai(&json!({"model": "m", "messages": []}));
    assert_eq!(out["max_tokens"], 1024);
}

#[test]
fn tool_use_blocks_become_tool_calls() {
    let req = json!({
        "model": "m", "max_tokens": 8,
        "messages": [{"role": "assistant", "content": [
            {"type": "text", "text": "reading"},
            {"type": "tool_use", "id": "toolu_1", "name": "Read", "input": {"path": "src/main.rs"}}
        ]}]
    });
    let out = anthropic_to_openai(&req);
    let calls = out["messages"][0]["tool_calls"]
        .as_array()
        .expect("tool_calls");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["id"], "toolu_1");
    assert_eq!(calls[0]["function"]["name"], "Read");
    assert_eq!(
        calls[0]["function"]["arguments"],
        r#"{"path":"src/main.rs"}"#
    );
}

#[test]
fn tool_result_maps_to_tool_role() {
    let req = json!({
        "model": "m", "max_tokens": 8,
        "messages": [{"role": "user", "content": [
            {"type": "tool_result", "tool_use_id": "toolu_1", "content": "file bytes"}
        ]}]
    });
    let out = anthropic_to_openai(&req);
    assert_eq!(out["messages"][0]["role"], "tool");
    assert_eq!(out["messages"][0]["tool_call_id"], "toolu_1");
}

#[test]
fn thinking_blocks_dropped_on_request() {
    let req = json!({
        "model": "m", "max_tokens": 8,
        "messages": [{"role": "assistant", "content": [
            {"type": "thinking", "thinking": "hmm"},
            {"type": "text", "text": "done"}
        ]}]
    });
    let out = anthropic_to_openai(&req);
    let content = out["messages"][0]["content"].as_str().unwrap_or_default();
    assert_eq!(content, "done");
}

#[test]
fn openai_response_maps_to_anthropic() {
    let resp = json!({
        "id": "chatcmpl_1", "model": "m",
        "choices": [{"message": {"role": "assistant", "content": "hi",
            "tool_calls": [{"id": "call_1", "function": {"name": "Read", "arguments": "{\"path\":\"x\"}"}}]},
            "finish_reason": "tool_calls"}],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    });
    let out = openai_to_anthropic(&resp);
    assert_eq!(out["type"], "message");
    assert_eq!(out["stop_reason"], "tool_use");
    let blocks = out["content"].as_array().expect("blocks");
    assert_eq!(blocks[0]["type"], "text");
    assert_eq!(blocks[1]["type"], "tool_use");
    assert_eq!(blocks[1]["id"], "call_1");
    assert_eq!(out["usage"]["input_tokens"], 10);
    assert_eq!(out["usage"]["output_tokens"], 5);
}

#[test]
fn sse_chunks_map_per_event() {
    let anth_ev = json!({"type": "content_block_delta", "index": 0,
        "delta": {"type": "text_delta", "text": "hel"}});
    let oai = anthropic_event_to_openai(&anth_ev).expect("mapped");
    assert_eq!(oai["choices"][0]["delta"]["content"], "hel");
    let oai_chunk = json!({"choices": [{"delta": {"content": "lo"}}]});
    let evs = openai_chunk_to_anthropic(&oai_chunk);
    assert!(!evs.is_empty());
    assert_eq!(evs[0]["delta"]["text"], "lo");
    // [DONE] / message_stop never kill the wire: unmapped → None / empty.
    assert!(anthropic_event_to_openai(&json!({"type": "message_stop"})).is_none());
    assert!(openai_chunk_to_anthropic(&json!({"choices": []})).is_empty());
}

#[test]
fn sanitize_strips_nulls_and_beta_allowlist() {
    let clean = sanitize_request(&json!({"a": null, "b": 1, "c": {"d": null, "e": "x"}}));
    assert_eq!(clean, json!({"b": 1, "c": {"e": "x"}}));
    let pass = pass_through_beta_headers(&[
        ("anthropic-beta", "tools-2024-01-01"),
        ("authorization", "Bearer SECRET"),
        ("x-custom", "1"),
    ]);
    assert_eq!(pass.len(), 1);
    assert_eq!(pass[0].0, "anthropic-beta");
}

#[test]
fn opencode_fixture_survives_sanitize_untouched() {
    let req: Value = serde_json::from_str(OPENCODE_REQ).expect("fixture parses");
    let clean = sanitize_request(&req);
    assert_eq!(clean["model"], "gpt-5-mini-test");
}
