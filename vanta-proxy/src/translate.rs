//! Anthropic↔OpenAI translation (PRX-11 slice 1 lib, slice 2 fidelity+contract).
//!
//! Pure functions over `serde_json::Value` — no I/O, no pipeline state, no
//! new dependencies. The wire stays verbatim by default; this module only
//! translates when a caller explicitly asks (slice-2 wiring decides policy).
//! Fidelity notes: `thinking` / `redacted_thinking` request blocks are dropped
//! per-variant (OpenAI has no counterpart); response `reasoning_content` /
//! `reasoning` (string or `{content|text}` object) maps back to `thinking`,
//! preserving an opaque `signature` when the upstream carries one.

use serde_json::{Map, Value};

/// Default `max_tokens` when the Anthropic request omits it (Anthropic
/// requires it, OpenAI does not — rejecting would break compat).
pub const DEFAULT_MAX_TOKENS: u64 = 1024;
/// Hard ceiling for the `max_tokens` guard (generous; providers reject above).
pub const MAX_MAX_TOKENS: u64 = 128_000;

/// Clamp an optional `max_tokens` into `[1, MAX_MAX_TOKENS]`, defaulting when
/// absent. Pure and total.
pub fn clamp_max_tokens(v: Option<u64>) -> u64 {
    v.unwrap_or(DEFAULT_MAX_TOKENS).clamp(1, MAX_MAX_TOKENS)
}

/// Opt-in switch for the pipeline hook (contract-first: the type and gate
/// land in slice 2 so the hook consumes them; the `ProxyConfig` field +
/// `server.rs` insertion are DEFERRED until `config.rs` ownership clears —
/// see `docs/tasks/PRX-11.md` slice 2). Default off → verbatim wire.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct TranslateConfig {
    /// When false (default), the hook never fires — byte-identical forward.
    pub enabled: bool,
}

/// Inbound client protocol for the translate gate (mirror of
/// `inject::Protocol` without the dependency — the hook maps it at the call
/// site so this module stays a leaf: zero imports from the pipeline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslateSource {
    Anthropic,
    OpenAI,
}

/// Pure gate: only Anthropic→OpenAI when explicitly enabled (the verbatim gap
/// per Spec #1). OpenAI-client bodies stay verbatim — symmetric translation
/// is DEFERRED with the hook. Pure and total.
pub fn should_translate(cfg: &TranslateConfig, src: TranslateSource) -> bool {
    cfg.enabled && matches!(src, TranslateSource::Anthropic)
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn compact_text(parts: &[String]) -> Value {
    Value::String(parts.join("\n\n"))
}

/// Translate an Anthropic `/v1/messages` request body to OpenAI
/// `/v1/chat/completions` shape. Unknown fields pass through only when they
/// are shared (`temperature`, `top_p`, `stream`); Anthropic-only envelopes
/// (`system` array extras, `thinking` config) are folded or dropped per Spec.
pub fn anthropic_to_openai(req: &Value) -> Value {
    let mut out = Map::new();
    if let Some(m) = str_field(req, "model") {
        out.insert("model".to_string(), Value::String(m));
    }
    let mut messages: Vec<Value> = Vec::new();

    // `system` (string or content-block array) → leading system message.
    match req.get("system") {
        Some(Value::String(s)) if !s.is_empty() => {
            messages.push(serde_json::json!({"role": "system", "content": s}));
        }
        Some(Value::Array(blocks)) => {
            let texts: Vec<String> = blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .map(str::to_string)
                .collect();
            if !texts.is_empty() {
                messages.push(serde_json::json!({"role": "system", "content": texts.join("\n\n")}));
            }
        }
        _ => {}
    }

    // Conversation messages.
    if let Some(arr) = req.get("messages").and_then(Value::as_array) {
        for msg in arr {
            let role = msg.get("role").and_then(Value::as_str).unwrap_or("user");
            match msg.get("content") {
                Some(Value::String(s)) => {
                    messages.push(serde_json::json!({"role": role, "content": s}));
                }
                Some(Value::Array(blocks)) => {
                    let mut texts: Vec<String> = Vec::new();
                    let mut tool_calls: Vec<Value> = Vec::new();
                    let mut tool_results: Vec<(String, String)> = Vec::new();
                    for b in blocks {
                        match b.get("type").and_then(Value::as_str) {
                            Some("text") => {
                                if let Some(t) = b.get("text").and_then(Value::as_str) {
                                    texts.push(t.to_string());
                                }
                            }
                            Some("image") => {
                                if let Some(url) = image_to_openai_url(b) {
                                    texts.push(format!("![image]({url})"));
                                }
                            }
                            Some("tool_use") => {
                                let id = b.get("id").and_then(Value::as_str).unwrap_or_default();
                                let name =
                                    b.get("name").and_then(Value::as_str).unwrap_or_default();
                                let input = b.get("input").unwrap_or(&Value::Null);
                                let args = serde_json::to_string(input)
                                    .unwrap_or_else(|_| "{}".to_string());
                                tool_calls.push(serde_json::json!({
                                    "id": id, "type": "function",
                                    "function": {"name": name, "arguments": args}
                                }));
                            }
                            Some("tool_result") => {
                                let id = b
                                    .get("tool_use_id")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default()
                                    .to_string();
                                let content = match b.get("content") {
                                    Some(Value::String(s)) => s.clone(),
                                    Some(other) => serde_json::to_string(other)
                                        .unwrap_or_else(|_| String::new()),
                                    None => String::new(),
                                };
                                tool_results.push((id, content));
                            }
                            // Extended-thinking content blocks have no OpenAI
                            // counterpart on the request path — dropped per
                            // variant (Spec #4, verified slice 2). NOTE
                            // (PRX-07): thinking may carry PII; the deferred
                            // hook runs post-redact, so dropped content never
                            // leaves toward the upstream.
                            Some("thinking") | Some("redacted_thinking") => {}
                            _ => {}
                        }
                    }
                    if !tool_results.is_empty() {
                        if !texts.is_empty() {
                            messages.push(
                                serde_json::json!({"role": role, "content": compact_text(&texts)}),
                            );
                        }
                        for (id, content) in tool_results {
                            messages.push(serde_json::json!({
                                "role": "tool", "tool_call_id": id, "content": content
                            }));
                        }
                    } else {
                        let mut m = Map::new();
                        m.insert("role".to_string(), Value::String(role.to_string()));
                        if texts.len() == 1 && tool_calls.is_empty() {
                            m.insert("content".to_string(), Value::String(texts.remove(0)));
                        } else if !texts.is_empty() {
                            m.insert("content".to_string(), compact_text(&texts));
                        } else {
                            m.insert("content".to_string(), Value::String(String::new()));
                        }
                        if !tool_calls.is_empty() {
                            m.insert("tool_calls".to_string(), Value::Array(tool_calls));
                        }
                        messages.push(Value::Object(m));
                    }
                }
                _ => {
                    messages.push(serde_json::json!({"role": role, "content": ""}));
                }
            }
        }
    }
    out.insert("messages".to_string(), Value::Array(messages));

    // Tools: `input_schema` → `parameters`.
    if let Some(tools) = req.get("tools").and_then(Value::as_array) {
        let mapped: Vec<Value> = tools
            .iter()
            .map(|t| {
                let name = t.get("name").and_then(Value::as_str).unwrap_or_default();
                let desc = t
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let params = t
                    .get("input_schema")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({"type": "object"}));
                serde_json::json!({
                    "type": "function",
                    "function": {"name": name, "description": desc, "parameters": params}
                })
            })
            .collect();
        out.insert("tools".to_string(), Value::Array(mapped));
    }
    // `tool_choice`: auto/any/tool → OpenAI vocabulary.
    match req.get("tool_choice") {
        Some(Value::String(s)) => {
            out.insert("tool_choice".to_string(), Value::String(s.clone()));
        }
        Some(obj) if obj.is_object() => {
            let kind = obj.get("type").and_then(Value::as_str).unwrap_or("auto");
            match kind {
                "any" => {
                    out.insert(
                        "tool_choice".to_string(),
                        Value::String("required".to_string()),
                    );
                }
                "tool" => {
                    let name = obj.get("name").and_then(Value::as_str).unwrap_or_default();
                    out.insert(
                        "tool_choice".to_string(),
                        serde_json::json!({"type": "function", "function": {"name": name}}),
                    );
                }
                _ => {
                    out.insert("tool_choice".to_string(), Value::String("auto".to_string()));
                }
            }
        }
        _ => {}
    }

    // Shared sampling / lattice fields.
    for key in ["temperature", "top_p", "stream", "stop_sequences"] {
        if let Some(v) = req.get(key) {
            if key == "stop_sequences" {
                out.insert("stop".to_string(), v.clone());
            } else {
                out.insert(key.to_string(), v.clone());
            }
        }
    }
    let max = req.get("max_tokens").and_then(Value::as_u64);
    out.insert("max_tokens".to_string(), Value::from(clamp_max_tokens(max)));
    sanitize_request(&Value::Object(out))
}

fn image_to_openai_url(b: &Value) -> Option<String> {
    let src = b.get("source")?;
    match src.get("type").and_then(Value::as_str) {
        Some("url") => src.get("url").and_then(Value::as_str).map(str::to_string),
        Some("base64") => {
            let media = src
                .get("media_type")
                .and_then(Value::as_str)
                .unwrap_or("image/png");
            let data = src.get("data").and_then(Value::as_str).unwrap_or_default();
            Some(format!("data:{media};base64,{data}"))
        }
        _ => None,
    }
}

/// Extract `(text, signature)` from the OpenAI-compat reasoning variants:
/// `reasoning_content: string` (+ optional `reasoning_signature`), or
/// `reasoning: string | {content|text: string}` (+ optional `signature`
/// alongside). Unknown shapes → `(None, None)` — fail-open, never invented.
fn extract_reasoning(message: &Value) -> (Option<String>, Option<String>) {
    if let Some(s) = message
        .get("reasoning_content")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        let sig = message
            .get("reasoning_signature")
            .and_then(Value::as_str)
            .map(str::to_string);
        return (Some(s.to_string()), sig);
    }
    match message.get("reasoning") {
        Some(Value::String(s)) if !s.is_empty() => {
            let sig = message
                .get("signature")
                .and_then(Value::as_str)
                .map(str::to_string);
            (Some(s.clone()), sig)
        }
        Some(obj) if obj.is_object() => {
            let text = obj
                .get("content")
                .or_else(|| obj.get("text"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let sig = obj
                .get("signature")
                .and_then(Value::as_str)
                .map(str::to_string);
            (text, sig)
        }
        _ => (None, None),
    }
}

/// Map the extracted reasoning to one Anthropic `thinking` block, preserving
/// the opaque `signature` (multi-turn block preservation) when present.
fn reasoning_to_thinking(message: &Value) -> Option<Value> {
    let (text, signature) = extract_reasoning(message);
    let text = text?;
    let mut block = serde_json::json!({"type": "thinking", "thinking": text});
    if let Some(sig) = signature {
        block["signature"] = Value::String(sig);
    }
    Some(block)
}

/// Translate an OpenAI Chat Completions response to Anthropic Messages shape
/// (`type: message`, content blocks, `stop_reason`, `usage`).
pub fn openai_to_anthropic(resp: &Value) -> Value {
    let mut out = Map::new();
    out.insert("type".to_string(), Value::String("message".to_string()));
    if let Some(id) = str_field(resp, "id") {
        out.insert("id".to_string(), Value::String(id));
    }
    if let Some(m) = str_field(resp, "model") {
        out.insert("model".to_string(), Value::String(m));
    }
    out.insert("role".to_string(), Value::String("assistant".to_string()));

    let choice = resp
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .cloned()
        .unwrap_or(Value::Null);
    let message = choice.get("message").cloned().unwrap_or(Value::Null);

    let mut blocks: Vec<Value> = Vec::new();
    if let Some(thinking) = reasoning_to_thinking(&message) {
        blocks.push(thinking);
    }
    match message.get("content") {
        Some(Value::String(s)) if !s.is_empty() => {
            blocks.push(serde_json::json!({"type": "text", "text": s}));
        }
        Some(Value::Array(parts)) => {
            for p in parts {
                if let Some(t) = p.get("text").and_then(Value::as_str) {
                    blocks.push(serde_json::json!({"type": "text", "text": t}));
                }
            }
        }
        _ => {}
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for c in calls {
            let id = c.get("id").and_then(Value::as_str).unwrap_or_default();
            let name = c
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let args_str = c
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(Value::as_str)
                .unwrap_or("{}");
            let input: Value = serde_json::from_str(args_str).unwrap_or(Value::Null);
            blocks.push(serde_json::json!({
                "type": "tool_use", "id": id, "name": name, "input": input
            }));
        }
    }
    out.insert("content".to_string(), Value::Array(blocks));

    let finish = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .unwrap_or("stop");
    let stop_reason = match finish {
        "tool_calls" => "tool_use",
        "length" => "max_tokens",
        "content_filter" => "refusal",
        _ => "end_turn",
    };
    // Tool blocks always imply `tool_use` regardless of the wire label.
    let has_tool = out["content"].as_array().is_some_and(|b| {
        b.iter()
            .any(|x| x.get("type").and_then(Value::as_str) == Some("tool_use"))
    });
    out.insert(
        "stop_reason".to_string(),
        Value::String(if has_tool {
            "tool_use".to_string()
        } else {
            stop_reason.to_string()
        }),
    );

    let usage = resp.get("usage").cloned().unwrap_or(Value::Null);
    out.insert(
        "usage".to_string(),
        serde_json::json!({
            "input_tokens": usage.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0),
            "output_tokens": usage.get("completion_tokens").and_then(Value::as_u64).unwrap_or(0)
        }),
    );
    Value::Object(out)
}

/// Map ONE Anthropic SSE event to an OpenAI chunk (`None` = terminal/control
/// event with no delta — the wire replays nothing for it).
pub fn anthropic_event_to_openai(event: &Value) -> Option<Value> {
    let kind = event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match kind {
        "content_block_delta" => {
            let delta = event.get("delta").cloned().unwrap_or(Value::Null);
            let dtype = delta
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            match dtype {
                "text_delta" => {
                    let text = delta.get("text").cloned().unwrap_or(Value::Null);
                    Some(serde_json::json!({"choices": [{"delta": {"content": text}}]}))
                }
                "input_json_delta" => {
                    let partial = delta.get("partial_json").cloned().unwrap_or(Value::Null);
                    let index = event.get("index").and_then(Value::as_u64).unwrap_or(0);
                    Some(serde_json::json!({"choices": [{"delta": {
                        "tool_calls": [{"index": index,
                            "function": {"arguments": partial}}]}}]}))
                }
                _ => None,
            }
        }
        "message_start" => {
            let role = event
                .get("message")
                .and_then(|m| m.get("role"))
                .and_then(Value::as_str)
                .unwrap_or("assistant");
            Some(serde_json::json!({"choices": [{"delta": {"role": role}}]}))
        }
        _ => None,
    }
}

/// Map ONE OpenAI SSE chunk to zero or more Anthropic SSE events.
pub fn openai_chunk_to_anthropic(chunk: &Value) -> Vec<Value> {
    let Some(choice) = chunk
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
    else {
        return Vec::new();
    };
    let delta = choice.get("delta").cloned().unwrap_or(Value::Null);
    let mut out = Vec::new();
    if let Some(text) = delta.get("content").and_then(Value::as_str) {
        if !text.is_empty() {
            out.push(serde_json::json!({"type": "content_block_delta",
                "delta": {"type": "text_delta", "text": text}}));
        }
    }
    if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
        for c in calls {
            let partial = c
                .get("function")
                .and_then(|f| f.get("arguments"))
                .cloned()
                .unwrap_or(Value::Null);
            let index = c.get("index").and_then(Value::as_u64).unwrap_or(0);
            out.push(
                serde_json::json!({"type": "content_block_delta", "index": index,
                "delta": {"type": "input_json_delta", "partial_json": partial}}),
            );
        }
    }
    out
}

/// Recursively strip `null` object members (sanitización — never echoes
/// secrets, only drops empty slots). Arrays keep their length.
pub fn sanitize_request(req: &Value) -> Value {
    match req {
        Value::Object(map) => {
            let mut clean = Map::new();
            for (k, v) in map {
                if !v.is_null() {
                    clean.insert(k.clone(), sanitize_request(v));
                }
            }
            Value::Object(clean)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sanitize_request).collect()),
        other => other.clone(),
    }
}

/// Allowlist passthrough for Anthropic beta headers (case-insensitive).
/// Auth secrets are NEVER forwarded by the translator (the forwarder owns
/// credentials; see `forward.rs` hop-by-hop + `x-vanta-user-key` strip).
pub fn pass_through_beta_headers(headers: &[(&str, &str)]) -> Vec<(String, String)> {
    const ALLOW: &[&str] = &["anthropic-beta", "anthropic-version"];
    headers
        .iter()
        .filter(|(k, v)| !v.is_empty() && ALLOW.iter().any(|a| k.eq_ignore_ascii_case(a)))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_bounds() {
        assert_eq!(clamp_max_tokens(None), DEFAULT_MAX_TOKENS);
        assert_eq!(clamp_max_tokens(Some(0)), 1);
        assert_eq!(clamp_max_tokens(Some(MAX_MAX_TOKENS + 1)), MAX_MAX_TOKENS);
    }

    #[test]
    fn system_array_concats_text_only() {
        let req = serde_json::json!({
            "model": "m",
            "system": [{"type": "text", "text": "a"}, {"type": "image", "source": {}}],
            "messages": []
        });
        let out = anthropic_to_openai(&req);
        assert_eq!(out["messages"][0]["content"], "a");
    }
}
