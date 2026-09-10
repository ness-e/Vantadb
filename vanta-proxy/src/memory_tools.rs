//! Server-side executor for the `vanta_memory_*` tools (MEM-51, D46/D47/D48).
//!
//! When the model invokes one of OUR tools inside a proxied turn, the proxy —
//! not the client — executes it: capture persists through the single D47
//! write path ([`crate::writeback::WriteBack::track`], fire-and-forget) and
//! search runs synchronous recall over the embedded store. A standard-shaped
//! tool result is then synthesized so the upstream can continue the turn.

use serde_json::{json, Value};
use vantadb::sdk::VantaEmbedded;

use crate::capture;
use crate::inject::Protocol;
use crate::sse_intercept::{Accumulated, ToolCallAcc};
use crate::writeback::WriteBack;

pub(crate) const TOOL_CAPTURE: &str = "vanta_memory_capture";
pub(crate) const TOOL_SEARCH: &str = "vanta_memory_search";

const MEMORY_TOOLS: [&str; 2] = [TOOL_CAPTURE, TOOL_SEARCH];

/// True when the request body's `tools` array announces any Vanta memory tool
/// (either wire shape). This is the interceptor gate: bodies without our
/// tools never pay for parsing.
pub(crate) fn announces(body: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<Value>(body) else {
        return false;
    };
    let Some(tools) = value.get("tools").and_then(Value::as_array) else {
        return false;
    };
    tools.iter().any(|tool| {
        tool.get("function")
            .and_then(|f| f.get("name"))
            .and_then(Value::as_str)
            .or_else(|| tool.get("name").and_then(Value::as_str))
            .is_some_and(|name| MEMORY_TOOLS.contains(&name))
    })
}

/// One server-executable memory tool invocation.
#[derive(Debug)]
pub(crate) struct MemoryCall {
    pub id: String,
    pub name: String,
    pub args: Value,
}

/// Extract memory-tool invocations from a complete assistant message.
///
/// Non-memory tool calls are ignored here (they belong to the client's own
/// tool loop); only ours get executed server-side. Malformed arguments
/// degrade to an empty object rather than failing the turn.
pub(crate) fn extract(message: &Accumulated) -> Vec<MemoryCall> {
    message
        .tool_calls
        .iter()
        .filter(|call| MEMORY_TOOLS.contains(&call.name.as_str()))
        .map(|call| MemoryCall {
            id: call.id.clone(),
            name: call.name.clone(),
            args: serde_json::from_str(&call.arguments).unwrap_or_else(|_| json!({})),
        })
        .collect()
}

/// Execute one memory tool and return the model-facing result text.
///
/// Capture is fire-and-forget through [`WriteBack::track`] (D47): success is
/// reported as soon as the job is queued — a slow write can never delay the
/// wire. Search runs synchronously (the model needs its answer to continue).
/// Neither path can fail the request: storage errors degrade into descriptive
/// result text the model can react to.
pub(crate) fn execute(
    memory: &VantaEmbedded,
    writeback: &WriteBack,
    session_key: &str,
    protocol_label: &str,
    space_id: &str,
    model: &str,
    call: &MemoryCall,
) -> String {
    match call.name.as_str() {
        TOOL_CAPTURE => {
            let Some(text) = call
                .args
                .get("text")
                .and_then(Value::as_str)
                .or_else(|| call.args.get("content").and_then(Value::as_str))
                .filter(|t| !t.trim().is_empty())
            else {
                return "Capture rejected: no non-empty `text` provided.".to_string();
            };
            // D47: same L0 write path as automatic turn capture.
            let job = capture::turn_job(
                memory.clone(),
                session_key,
                protocol_label,
                space_id,
                model,
                text,
            );
            writeback.track(format!("tool:{session_key}:{}", call.id), job);
            "Memory captured.".to_string()
        }
        TOOL_SEARCH => search(memory, session_key, call),
        other => format!("Unknown memory tool `{other}`."),
    }
}

/// Synchronous recall (D46): run one auto-recall pass scoped to the session
/// and format the hits as the standard `<relevant-memories>` block.
fn search(memory: &VantaEmbedded, session_key: &str, call: &MemoryCall) -> String {
    use vanta_memory::core::hooks::{perform_auto_recall, AutoRecallParams, RecallConfig};

    let query = call
        .args
        .get("query")
        .and_then(Value::as_str)
        .or_else(|| call.args.get("text").and_then(Value::as_str))
        .unwrap_or("");
    if query.trim().is_empty() {
        return "Search rejected: no `query` provided.".to_string();
    }
    let params = AutoRecallParams {
        user_text: query,
        session_key,
        isolation: None,
        config: RecallConfig::default(),
    };
    match perform_auto_recall(memory, params, None) {
        Ok(Some(result)) => result
            .prepend_context
            .unwrap_or_else(|| NO_MEMORIES.to_string()),
        Ok(None) => NO_MEMORIES.to_string(),
        Err(e) => format!("Memory search failed: {e}"),
    }
}

const NO_MEMORIES: &str = "No relevant memories found.";

/// Responses history append (PRX-06): executed calls echo as
/// `function_call` items plus their `function_call_output` results in the
/// `input` item array (Responses bodies have no `messages`).
fn append_responses_exchange(
    request: &mut Value,
    owned: Vec<&ToolCallAcc>,
    results: &[(String, String)],
) {
    let Some(input) = request.get_mut("input").and_then(Value::as_array_mut) else {
        // ponytail: string-form input can't carry tool history — replay
        // without append (documented ceiling, loop still caps).
        tracing::warn!("responses tool turn with non-array input: replaying");
        return;
    };
    for call in owned {
        input.push(json!({
            "type": "function_call",
            "call_id": call.id,
            "name": call.name,
            "arguments": call.arguments,
        }));
    }
    for (id, text) in results {
        input.push(json!({
            "type": "function_call_output",
            "call_id": id,
            "output": text,
        }));
    }
}

/// Append the assistant message (with its EXECUTED tool calls) plus the
/// synthesized results to the request's history, in the standard shape of
/// `protocol`.
///
/// Only calls we executed server-side are echoed: echoing foreign
/// (client-side) calls without results leaves them dangling upstream-side
/// (OpenAI 400s on result-less `tool_calls`), so mixed turns serve our
/// tools this round and drop foreign ones with a warn (PRX-08 S6 —
/// `ponytail:` ceiling: serving both needs protocol change, out of scope).
pub(crate) fn append_exchange(
    protocol: Protocol,
    request: &mut Value,
    message: &Accumulated,
    results: &[(String, String)],
) {
    let executed: Vec<&str> = results.iter().map(|(id, _)| id.as_str()).collect();
    let owned: Vec<_> = message
        .tool_calls
        .iter()
        .filter(|call| executed.contains(&call.id.as_str()))
        .collect();
    for dropped in message
        .tool_calls
        .iter()
        .filter(|call| !executed.contains(&call.id.as_str()))
    {
        tracing::warn!(
            tool = %dropped.name,
            id = %dropped.id,
            "mixed tool turn: dropping unexecuted client tool call from re-request"
        );
    }
    // Responses bodies carry history in `input`, not `messages`.
    if protocol == Protocol::Responses {
        append_responses_exchange(request, owned, results);
        return;
    }
    let Some(messages) = request.get_mut("messages").and_then(Value::as_array_mut) else {
        return;
    };
    match protocol {
        Protocol::OpenAI => {
            let calls: Vec<Value> = owned
                .iter()
                .map(|call| {
                    json!({
                        "id": call.id,
                        "type": "function",
                        "function": { "name": call.name, "arguments": call.arguments },
                    })
                })
                .collect();
            let mut assistant = json!({ "role": "assistant" });
            assistant["content"] = if message.text.is_empty() {
                Value::Null
            } else {
                Value::String(message.text.clone())
            };
            assistant["tool_calls"] = Value::Array(calls);
            messages.push(assistant);
            for (id, text) in results {
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": id,
                    "content": text,
                }));
            }
        }
        Protocol::Responses => {
            // Unreachable: Responses returns early above (history in `input`).
        }
        Protocol::Anthropic => {
            // Same filter for wire blocks: drop foreign `tool_use` blocks so
            // every echoed use has a matching result below.
            let owned_blocks: Vec<Value> = message
                .blocks
                .iter()
                .filter(|b| {
                    b.get("id").and_then(Value::as_str).is_none_or(|id| {
                        // Non-tool blocks (text/thinking) always echo; tool_use
                        // blocks echo only when executed.
                        b.get("type").and_then(Value::as_str) != Some("tool_use")
                            || executed.contains(&id)
                    })
                })
                .cloned()
                .collect();
            messages.push(json!({ "role": "assistant", "content": owned_blocks }));
            let blocks: Vec<Value> = results
                .iter()
                .map(|(id, text)| {
                    json!({
                        "type": "tool_result",
                        "tool_use_id": id,
                        "content": text,
                    })
                })
                .collect();
            messages.push(json!({ "role": "user", "content": blocks }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sse_intercept::{anthropic_message, data_events, openai_message};

    #[test]
    fn detects_our_tools_in_both_wire_shapes_only() {
        let openai =
            br#"{"tools":[{"type":"function","function":{"name":"vanta_memory_capture"}}]}"#;
        let anthropic = br#"{"tools":[{"name":"vanta_memory_search"}]}"#;
        let foreign = br#"{"tools":[{"function":{"name":"web_search"}}]}"#;
        assert!(announces(openai));
        assert!(announces(anthropic));
        assert!(!announces(foreign));
        assert!(!announces(b"not json"));
        assert!(!announces(br#"{"messages":[]}"#));
    }

    #[test]
    fn extract_filters_foreign_calls_and_defaults_bad_args() {
        let call = |name: &str, id: &str, args: &str| crate::sse_intercept::ToolCallAcc {
            id: id.to_string(),
            name: name.to_string(),
            arguments: args.to_string(),
        };
        let acc = Accumulated {
            tool_calls: vec![
                call(
                    "vanta_memory_capture",
                    "call_a",
                    r#"{"text":"remember me"}"#,
                ),
                call("run_shell", "call_b", "{}"),
                call("vanta_memory_search", "call_c", "not json"),
            ],
            ..Default::default()
        };
        let calls = extract(&acc);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].id, "call_a");
        assert_eq!(calls[0].args["text"], "remember me");
        assert_eq!(calls[1].args, json!({}));
    }

    #[test]
    fn append_exchange_openai_standard_shape() {
        let body = r#"{"messages":[{"role":"user","content":"hi"}]}"#;
        let mut request: Value = serde_json::from_str(body).unwrap();
        let events = data_events(
            concat!(
                r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"c1","function":{"name":"vanta_memory_capture","arguments":"{}"}}]}}]}"#,
                "\n\n",
            )
            .as_bytes(),
        );
        let message = openai_message(&events);
        append_exchange(
            Protocol::OpenAI,
            &mut request,
            &message,
            &[("c1".into(), "Memory captured.".into())],
        );
        let messages = request["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["tool_calls"][0]["id"], "c1");
        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "c1");
        assert_eq!(messages[2]["content"], "Memory captured.");
    }

    #[test]
    fn append_exchange_anthropic_standard_shape() {
        let body = r#"{"messages":[{"role":"user","content":"hi"}]}"#;
        let mut request: Value = serde_json::from_str(body).unwrap();
        let events = data_events(
            concat!(
                r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"tu1","name":"vanta_memory_search"}}"#,
                "\n\n",
                r#"data: {"type":"message_stop"}"#,
                "\n\n",
            )
            .as_bytes(),
        );
        let message = anthropic_message(&events);
        append_exchange(
            Protocol::Anthropic,
            &mut request,
            &message,
            &[("tu1".into(), NO_MEMORIES.into())],
        );
        let messages = request["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1]["content"][0]["type"], "tool_use");
        assert_eq!(messages[2]["role"], "user");
        assert_eq!(messages[2]["content"][0]["type"], "tool_result");
        assert_eq!(messages[2]["content"][0]["tool_use_id"], "tu1");
    }

    #[test]
    fn append_exchange_drops_unanswered_foreign_calls() {
        // PRX-08 S6 RED: mixed turn — the re-request assistant message must
        // carry ONLY executed calls, so upstream never sees dangling ones.
        let call = |name: &str, id: &str| crate::sse_intercept::ToolCallAcc {
            id: id.to_string(),
            name: name.to_string(),
            arguments: "{}".to_string(),
        };
        let message = Accumulated {
            tool_calls: vec![
                call("vanta_memory_capture", "ours-1"),
                call("client_shell", "theirs-1"),
            ],
            ..Default::default()
        };
        let mut request: Value =
            serde_json::from_str(r#"{"messages":[{"role":"user","content":"hi"}]}"#).unwrap();
        append_exchange(
            Protocol::OpenAI,
            &mut request,
            &message,
            &[("ours-1".into(), "Memory captured.".into())],
        );
        let messages = request["messages"].as_array().unwrap();
        let echoed: Vec<&str> = messages[1]["tool_calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap())
            .collect();
        assert_eq!(echoed, vec!["ours-1"], "foreign call must not echo");
        assert_eq!(messages[2]["tool_call_id"], "ours-1");
    }

    #[test]
    fn capture_rejects_empty_text_without_touching_writeback() {
        let db = memory();
        let wb = WriteBack::new(None);
        let result = execute(
            &db,
            &wb,
            "sess",
            "openai",
            "space",
            "m",
            &MemoryCall {
                id: "x".into(),
                name: TOOL_CAPTURE.into(),
                args: json!({"text": "   "}),
            },
        );
        assert!(result.contains("rejected"));
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(crate::capture::list_turns(&db).is_empty());
    }

    #[test]
    fn search_empty_db_reports_no_memories_and_unknown_tool_degrades() {
        let db = memory();
        let wb = WriteBack::new(None);
        let result = execute(
            &db,
            &wb,
            "sess",
            "openai",
            "space",
            "m",
            &MemoryCall {
                id: "y".into(),
                name: TOOL_SEARCH.into(),
                args: json!({"query": "anything"}),
            },
        );
        assert_eq!(result, NO_MEMORIES);

        let unknown = execute(
            &db,
            &wb,
            "sess",
            "openai",
            "space",
            "m",
            &MemoryCall {
                id: "z".into(),
                name: "vanta_memory_delete_everything".into(),
                args: json!({}),
            },
        );
        assert!(unknown.contains("Unknown memory tool"));
    }

    fn memory() -> VantaEmbedded {
        let config = vantadb::config::VantaConfig {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            ..Default::default()
        };
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
            .map(|engine| VantaEmbedded::from_engine(engine.into()))
            .expect("in-memory engine")
    }
}
