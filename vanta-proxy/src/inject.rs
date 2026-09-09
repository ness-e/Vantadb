//! Memory injection into the system prompt + L0/L1 tool exposure (D29).
//!
//! Contract (KV-cache safe): persona and scene context are injected ONLY at
//! the system prompt position — never into the conversation history. L0/L1
//! capabilities ride as tools in the request body. Non-JSON bodies pass
//! through untouched.

use bytes::Bytes;
use serde_json::{json, Value};
use vantadb::sdk::VantaEmbedded;

use crate::error::ProxyError;

/// Which wire shape the body speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// OpenAI Chat Completions (`messages` array).
    OpenAI,
    /// Anthropic Messages (`system` field).
    Anthropic,
    /// OpenAI Responses subset (`instructions` field).
    Responses,
}

/// L0/L1 tools exposed to the model (TDAM README:28 — L0/L1 as tools).
const TOOL_SPECS: [(&str, &str); 2] = [
    (
        "vanta_memory_capture",
        "L0 memory capture: persist a durable memory record for this session.",
    ),
    (
        "vanta_memory_search",
        "L1 memory search: retrieve previously stored memories by text query.",
    ),
];

fn tool_schema(name: &str) -> Value {
    match name {
        "vanta_memory_capture" => json!({
            "type": "object",
            "properties": {
                "content": { "type": "string", "description": "Memory content to capture" },
                "tags": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["content"]
        }),
        _ => json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Text query" },
                "top_k": { "type": "integer", "minimum": 1 }
            },
            "required": ["query"]
        }),
    }
}

fn tool_json_openai(name: &str, description: &str) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": tool_schema(name)
        }
    })
}

fn tool_json_anthropic(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "input_schema": tool_schema(name)
    })
}

fn tool_name_of(protocol: Protocol, tool: &Value) -> Option<&str> {
    match protocol {
        Protocol::OpenAI | Protocol::Responses => tool
            .get("function")
            .and_then(|f| f.get("name"))
            .and_then(Value::as_str),
        Protocol::Anthropic => tool.get("name").and_then(Value::as_str),
    }
}

/// Compose the `<vanta-memory>` block from the session's persona and scenes
/// (via vanta-memory lib re-exports). Best-effort: storage errors yield an
/// empty block — injection must never fail the request. Empty when there is
/// nothing to inject.
pub fn build_memory_block(db: &VantaEmbedded, session_key: &str) -> String {
    use vanta_memory::core::persona::persona_generator::get_persona;
    use vanta_memory::core::scene::scene_index::{current_scene, list_scenes};

    let mut any = false;
    let mut out = String::from("<vanta-memory>\n");

    match get_persona(db, session_key) {
        Ok(Some(record)) => {
            let body = record.content.trim();
            if !body.is_empty() {
                out.push_str("<user-persona>\n");
                out.push_str(body);
                out.push_str("\n</user-persona>\n");
                any = true;
            }
        }
        Ok(None) => {}
        Err(e) => tracing::debug!(error = %e, "persona read failed; skipping injection"),
    }

    match current_scene(db, session_key) {
        Ok(Some(scene)) => {
            out.push_str("<current-scene>\n");
            out.push_str(scene.scene_name.trim());
            out.push_str(": ");
            out.push_str(scene.content.trim());
            out.push_str("\n</current-scene>\n");
            any = true;
        }
        Ok(None) => {}
        Err(e) => tracing::debug!(error = %e, "current scene read failed; skipping"),
    }

    match list_scenes(db, session_key) {
        Ok(entries) if !entries.is_empty() => {
            out.push_str("<scene-index>\n");
            for entry in entries {
                out.push_str(&format!("- {} (heat {})\n", entry.filename, entry.heat));
            }
            out.push_str("</scene-index>\n");
            any = true;
        }
        _ => {}
    }

    if !any {
        return String::new();
    }
    out.push_str("</vanta-memory>");
    out
}

/// Prepend `block` to a string field, returning true only on change.
fn prepend_string_field(body: &mut Value, key: &str, block: &str) -> bool {
    match body.get_mut(key) {
        Some(Value::String(existing)) if !existing.starts_with(block) => {
            *existing = format!("{block}\n\n{existing}");
            true
        }
        Some(_) => false, // non-string or already injected — leave alone
        None => {
            body[key] = Value::String(block.to_string());
            true
        }
    }
}

/// Merge missing vantage tools into the `tools` array (creating it if absent).
/// Returns true only when a tool was added.
fn merge_tools(body: &mut Value, protocol: Protocol) -> bool {
    let make = |(name, desc): &(&str, &str)| match protocol {
        Protocol::OpenAI | Protocol::Responses => tool_json_openai(name, desc),
        Protocol::Anthropic => tool_json_anthropic(name, desc),
    };

    match body.get_mut("tools") {
        Some(Value::Array(tools)) => {
            let mut changed = false;
            for spec in TOOL_SPECS.iter() {
                let present = tools
                    .iter()
                    .filter_map(|t| tool_name_of(protocol, t))
                    .any(|n| n == spec.0);
                if !present {
                    tools.push(make(spec));
                    changed = true;
                }
            }
            changed
        }
        _ => {
            body["tools"] = Value::Array(TOOL_SPECS.iter().map(make).collect());
            true
        }
    }
}

/// Apply D29 injection to a JSON request body.
///
/// Returns `Ok(None)` when the body is unchanged (non-JSON, wrong shape, or
/// nothing to inject) so callers forward the original bytes verbatim.
///
/// # Errors
/// [`ProxyError::InvalidRequest`] only when re-serialization of an already
/// parsed JSON object fails (practically unreachable).
pub fn inject_into(
    body: &Bytes,
    protocol: Protocol,
    memory_block: &str,
) -> Result<Option<Vec<u8>>, ProxyError> {
    let Ok(mut value) = serde_json::from_slice::<Value>(body.as_ref()) else {
        return Ok(None);
    };
    if !value.is_object() {
        return Ok(None);
    }

    // System-prompt position ONLY (D29): never touch history messages.
    // PRX-04: every arm reports whether it mutated the prompt; a re-inject
    // of an already-injected body must be a no-op (byte-stable prefix for
    // prompt caching).
    let mut prompt_changed = false;
    match protocol {
        Protocol::Anthropic => {
            if let Some(system) = value.get("system").cloned() {
                match system {
                    Value::String(existing)
                        if !memory_block.is_empty() && !existing.starts_with(memory_block) =>
                    {
                        value["system"] = Value::String(format!("{memory_block}\n\n{existing}"));
                        prompt_changed = true;
                    }
                    Value::Array(blocks) if !memory_block.is_empty() => {
                        let already = blocks.first().is_some_and(|b| {
                            b.get("type").and_then(Value::as_str) == Some("text")
                                && b.get("text").and_then(Value::as_str) == Some(memory_block)
                        });
                        if !already {
                            let mut blocks = blocks;
                            blocks.insert(0, json!({ "type": "text", "text": memory_block }));
                            value["system"] = Value::Array(blocks);
                            prompt_changed = true;
                        }
                    }
                    _ => {}
                }
            } else if !memory_block.is_empty() {
                value["system"] = Value::String(memory_block.to_string());
                prompt_changed = true;
            }
        }
        Protocol::Responses => {
            if !memory_block.is_empty() {
                prompt_changed = prepend_string_field(&mut value, "instructions", memory_block);
            }
        }
        Protocol::OpenAI => {
            if !memory_block.is_empty() {
                const SYSTEM_ROLE: &str = "system";
                let injected = match value.get_mut("messages").and_then(Value::as_array_mut) {
                    Some(messages) => {
                        let first_system = messages
                            .iter_mut()
                            .find(|m| m.get("role").and_then(Value::as_str) == Some(SYSTEM_ROLE));
                        match first_system {
                            Some(msg) => match msg.get_mut("content") {
                                Some(Value::String(content))
                                    if !content.starts_with(memory_block) =>
                                {
                                    *content = format!("{memory_block}\n\n{content}");
                                    true
                                }
                                _ => false,
                            },
                            None => {
                                messages.insert(
                                    0,
                                    json!({ "role": SYSTEM_ROLE, "content": memory_block }),
                                );
                                true
                            }
                        }
                    }
                    None => false,
                };
                if !injected {
                    // No usable messages array → do not invent structure; skip.
                    tracing::debug!("openai body without messages array; prompt skipped");
                }
                prompt_changed = injected;
            }
        }
    }

    let tools_changed = merge_tools(&mut value, protocol);

    // PRX-04: nothing changed → None so callers forward the original bytes
    // verbatim instead of a re-serialization (cache-safe).
    if !prompt_changed && !tools_changed {
        return Ok(None);
    }

    serde_json::to_vec(&value)
        .map(Some)
        .map_err(|e| ProxyError::InvalidRequest(format!("re-serialize body: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_json_body_passes_through_unchanged() {
        let raw = Bytes::from_static(b"\x00\x01not-json");
        assert!(
            inject_into(&raw, Protocol::OpenAI, "<vanta-memory>x</vanta-memory>")
                .expect("ok")
                .is_none()
        );
    }

    #[test]
    fn openai_injection_lands_only_on_system_position() {
        let body = Bytes::from_static(
            br#"{"model":"m","messages":[
                {"role":"user","content":"u1"},
                {"role":"assistant","content":"a1"},
                {"role":"user","content":"u2"}]}"#,
        );
        let out = inject_into(&body, Protocol::OpenAI, "BLOCK")
            .expect("ok")
            .expect("modified");
        let v: Value = serde_json::from_slice(&out).expect("json");
        let msgs = v["messages"].as_array().expect("array");
        assert_eq!(msgs.len(), 4, "one system message inserted, none removed");
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[0]["content"], "BLOCK");
        assert_eq!(msgs[1]["content"], "u1", "history intact");
        assert_eq!(msgs[2]["content"], "a1", "history intact");
        assert_eq!(msgs[3]["content"], "u2", "history intact");
    }

    #[test]
    fn anthropic_system_array_gets_text_block_at_front() {
        let body =
            Bytes::from_static(br#"{"system":[{"type":"text","text":"base"}],"messages":[]}"#);
        let out = inject_into(&body, Protocol::Anthropic, "BLOCK")
            .expect("ok")
            .expect("modified");
        let v: Value = serde_json::from_slice(&out).expect("json");
        assert_eq!(v["system"][0]["type"], "text");
        assert_eq!(v["system"][0]["text"], "BLOCK");
        assert_eq!(v["system"][1]["text"], "base");
    }

    #[test]
    fn responses_instructions_prepended() {
        let body = Bytes::from_static(br#"{"instructions":"base"}"#);
        let out = inject_into(&body, Protocol::Responses, "BLOCK")
            .expect("ok")
            .expect("modified");
        let v: Value = serde_json::from_slice(&out).expect("json");
        assert_eq!(v["instructions"], "BLOCK\n\nbase");
    }

    #[test]
    fn tools_added_once_per_protocol_shape() {
        let empty = Bytes::from_static(br#"{}"#);
        let out = inject_into(&empty, Protocol::Anthropic, "")
            .expect("ok")
            .expect("tools added even without memory block");
        let v: Value = serde_json::from_slice(&out).expect("json");
        let names: Vec<&str> = v["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .filter_map(|t| t.get("name").and_then(Value::as_str))
            .collect();
        assert_eq!(names, vec!["vanta_memory_capture", "vanta_memory_search"]);

        // Existing tools are not duplicated.
        let with_tool =
            Bytes::from_static(br#"{"tools":[{"type":"function","function":{"name":"other"}}]}"#);
        let out = inject_into(&with_tool, Protocol::OpenAI, "")
            .expect("ok")
            .expect("modified");
        let v: Value = serde_json::from_slice(&out).expect("json");
        let names: Vec<String> = v["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .map(|t| {
                t.pointer("/function/name")
                    .or_else(|| t.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        assert_eq!(names.len(), 3);
        assert_eq!(
            names
                .iter()
                .filter(|n| **n == "vanta_memory_capture")
                .count(),
            1,
            "no duplicate L0 tool"
        );

        // Empty block → no system mutation but tools still exposed.
        let openai = Bytes::from_static(br#"{"messages":[{"role":"system","content":"S"}]}"#);
        let out = inject_into(&openai, Protocol::OpenAI, "")
            .expect("ok")
            .expect("tools added");
        let v: Value = serde_json::from_slice(&out).expect("json");
        assert_eq!(
            v["messages"][0]["content"], "S",
            "empty block leaves prompt"
        );
        assert!(v["tools"].is_array());
    }

    /// PRX-04: re-injecting the same body must be byte-identical (prompt-cache
    /// prefix stability). Applies per provider.
    fn reinject_stable(protocol: Protocol, raw: &'static [u8], block: &str) {
        let first = Bytes::from_static(raw);
        let once = inject_into(&first, protocol, block)
            .expect("ok")
            .expect("first injection modifies");
        let twice = inject_into(&Bytes::from(once.clone()), protocol, block);
        match twice.expect("ok") {
            None => {} // ideal: second pass is a no-op
            Some(second) => assert_eq!(
                once, second,
                "second injection must be byte-identical to the first"
            ),
        }
    }

    #[test]
    fn prx04_prefix_stable_anthropic_string() {
        reinject_stable(
            Protocol::Anthropic,
            br#"{"model":"c","system":"base prompt","messages":[]}"#,
            "BLOCK",
        );
    }

    #[test]
    fn prx04_prefix_stable_anthropic_array() {
        reinject_stable(
            Protocol::Anthropic,
            br#"{"model":"c","system":[{"type":"text","text":"base"}],"messages":[]}"#,
            "BLOCK",
        );
    }

    #[test]
    fn prx04_prefix_stable_openai() {
        reinject_stable(
            Protocol::OpenAI,
            br#"{"model":"m","messages":[{"role":"system","content":"S"},{"role":"user","content":"u"}]}"#,
            "BLOCK",
        );
    }

    #[test]
    fn prx04_prefix_stable_responses() {
        reinject_stable(Protocol::Responses, br#"{"instructions":"base"}"#, "BLOCK");
    }

    #[test]
    fn prx04_noop_returns_none_for_verbatim_forward() {
        // Empty block + complete tools → nothing to change → None so the
        // caller forwards the original bytes verbatim (cache-safe).
        let body = Bytes::from_static(
            br#"{"messages":[{"role":"user","content":"hi"}],"tools":[{"type":"function","function":{"name":"vanta_memory_capture","description":"d","parameters":{}}},{"type":"function","function":{"name":"vanta_memory_search","description":"d","parameters":{}}}]}"#,
        );
        let parsed: Result<Value, _> = serde_json::from_slice(body.as_ref());
        assert!(parsed.is_ok(), "fixture must parse: {:?}", parsed.err());
        assert!(parsed.unwrap().is_object(), "fixture must be an object");
        assert!(
            inject_into(&body, Protocol::OpenAI, "")
                .expect("ok")
                .is_none(),
            "no prompt change and tools complete → None"
        );
    }

    #[test]
    fn prx04_cache_control_markers_and_tool_calls_preserved() {
        // Realistic Anthropic fixture: cache breakpoint on a history block +
        // a hanging tool_use — injection must preserve both byte-identical
        // after the injected prefix.
        let body = Bytes::from_static(
            br#"{"model":"c","system":[{"type":"text","text":"base","cache_control":{"type":"ephemeral"}}],"messages":[{"role":"user","content":[{"type":"text","text":"hi","cache_control":{"type":"ephemeral"}}]},{"role":"assistant","content":[{"type":"tool_use","id":"toolu_1","name":"vanta_memory_search","input":{"query":"x"}}]}]}"#,
        );
        let out = inject_into(&body, Protocol::Anthropic, "BLOCK")
            .expect("ok")
            .expect("modified");
        let v: Value = serde_json::from_slice(&out).expect("json");
        // Injected prefix first, original cacheable block second with marker.
        assert_eq!(v["system"][0]["text"], "BLOCK");
        assert_eq!(v["system"][1]["text"], "base");
        assert_eq!(
            v["system"][1]["cache_control"],
            serde_json::json!({"type": "ephemeral"}),
            "cache breakpoint marker preserved"
        );
        // Hanging tool_use untouched.
        let assistant = &v["messages"][1]["content"][0];
        assert_eq!(assistant["id"], "toolu_1");
        assert_eq!(assistant["name"], "vanta_memory_search");
        // Prefix stable on re-inject.
        let twice = inject_into(&Bytes::from(out.clone()), Protocol::Anthropic, "BLOCK");
        match twice.expect("ok") {
            None => {}
            Some(second) => assert_eq!(out, second, "stable prefix byte-a-byte"),
        }
    }

    #[test]
    fn prx04_openai_cacheable_prefix_stable_with_tools_present() {
        // Realistic OpenAI fixture: tools already announced + memory block.
        let body = Bytes::from_static(
            br#"{"model":"m","messages":[{"role":"system","content":"BASE"},{"role":"user","content":"hi"}],"tools":[{"type":"function","function":{"name":"other"}}]}"#,
        );
        let out = inject_into(&body, Protocol::OpenAI, "BLOCK")
            .expect("ok")
            .expect("modified");
        let v: Value = serde_json::from_slice(&out).expect("json");
        assert_eq!(v["messages"][0]["content"], "BLOCK\n\nBASE");
        assert_eq!(v["messages"][1]["content"], "hi", "history intact");
        let twice = inject_into(&Bytes::from(out.clone()), Protocol::OpenAI, "BLOCK");
        match twice.expect("ok") {
            None => {}
            Some(second) => assert_eq!(out, second, "stable prefix byte-a-byte"),
        }
    }
}
