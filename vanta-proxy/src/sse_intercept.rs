//! SSE stream interception (MEM-51 / D46): buffer an upstream
//! `text/event-stream` response, accumulate its deltas into ONE complete
//! assistant message, then decide whether any Vanta memory tool was invoked.
//!
//! Deliberately manual byte-level parsing over the buffered payload (no
//! eventsource-stream dep): we only ever parse streams we fully own, so a
//! simple `data:` line scanner is enough. Non-SSE responses never reach this.

use axum::body::Body;
use bytes::Bytes;
use serde_json::Value;

use crate::error::ProxyError;

pub(crate) use axum::http;

/// One reconstructed tool call from the accumulated message. `arguments` is
/// the raw JSON string exactly as the model emitted it (OpenAI deltas carry a
/// JSON string; Anthropic's `input_json_delta` partials concatenate to one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolCallAcc {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Complete assistant message accumulated from an SSE stream.
#[derive(Debug, Default, Clone)]
pub(crate) struct Accumulated {
    /// Concatenated text deltas (`delta.content` / `text_delta`).
    pub text: String,
    /// Tool calls in emission order (OpenAI fragments merged by `index`;
    /// Anthropic `tool_use` blocks by block index).
    pub tool_calls: Vec<ToolCallAcc>,
    /// Anthropic content blocks ready for history reconstruction (empty for
    /// OpenAI, whose assistant message is rebuilt from text + tool_calls).
    pub blocks: Vec<Value>,
}

/// Buffered stream: original chunk boundaries (for verbatim replay) plus the
/// concatenated bytes (for parsing).
#[derive(Debug, Default)]
pub(crate) struct StreamCapture {
    pub chunks: Vec<Bytes>,
    pub full: Vec<u8>,
}

impl StreamCapture {
    fn push(&mut self, chunk: Bytes) {
        self.full.extend_from_slice(&chunk);
        self.chunks.push(chunk);
    }
}

/// Drain a response body into memory, preserving chunk boundaries.
///
/// # Errors
/// [`ProxyError::Forward`] if the underlying stream errors mid-flight (the
/// client gets a typed error response, never a panic).
pub(crate) async fn drain(body: Body) -> Result<StreamCapture, ProxyError> {
    use futures::StreamExt;
    let mut out = StreamCapture::default();
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ProxyError::Forward(format!("stream read: {e}")))?;
        out.push(chunk);
    }
    Ok(out)
}

/// Replay buffered chunks verbatim under the original status/headers.
pub(crate) fn replay(
    parts: http::response::Parts,
    chunks: Vec<Bytes>,
) -> axum::response::Response<Body> {
    use futures::stream;
    let body = Body::from_stream(stream::iter(
        chunks
            .into_iter()
            .map(Ok::<Bytes, std::convert::Infallible>),
    ));
    axum::response::Response::from_parts(parts, body)
}

/// Extract every `data:` JSON payload from a buffered SSE body, in order.
/// Tolerates CRLF, multi-line data fields, `[DONE]` sentinels and garbage
/// lines (skipped silently — a malformed event must never kill the wire).
pub(crate) fn data_events(full: &[u8]) -> Vec<Value> {
    let text = String::from_utf8_lossy(full).replace("\r\n", "\n");
    let mut events = Vec::new();
    for raw_event in text.split("\n\n") {
        let mut data_lines = Vec::new();
        for line in raw_event.split('\n') {
            if let Some(rest) = line.strip_prefix("data:") {
                data_lines.push(rest.strip_prefix(' ').unwrap_or(rest));
            }
        }
        if data_lines.is_empty() {
            continue;
        }
        let payload = data_lines.join("\n");
        let trimmed = payload.trim();
        if trimmed.is_empty() || trimmed == "[DONE]" {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(&payload) {
            events.push(value);
        }
    }
    events
}

/// Reconstruct the complete assistant message from OpenAI Chat Completions
/// SSE chunks (`choices[0].delta` fragments merged by tool-call `index`).
pub(crate) fn openai_message(events: &[Value]) -> Accumulated {
    let mut acc = Accumulated::default();
    for ev in events {
        let Some(choice) = ev
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|c| c.first())
        else {
            continue;
        };
        if let Some(content) = choice.pointer("/delta/content").and_then(Value::as_str) {
            acc.text.push_str(content);
        }
        let Some(frags) = choice
            .pointer("/delta/tool_calls")
            .and_then(Value::as_array)
        else {
            continue;
        };
        for frag in frags {
            let idx = frag.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
            while acc.tool_calls.len() <= idx {
                acc.tool_calls.push(ToolCallAcc {
                    id: String::new(),
                    name: String::new(),
                    arguments: String::new(),
                });
            }
            let slot = &mut acc.tool_calls[idx];
            if let Some(id) = frag.get("id").and_then(Value::as_str) {
                if !id.is_empty() {
                    slot.id = id.to_string();
                }
            }
            if let Some(name) = frag.pointer("/function/name").and_then(Value::as_str) {
                if !name.is_empty() {
                    slot.name = name.to_string();
                }
            }
            if let Some(args) = frag.pointer("/function/arguments").and_then(Value::as_str) {
                slot.arguments.push_str(args);
            }
        }
    }
    acc.tool_calls.retain(|c| !c.name.is_empty());
    acc
}

/// Reconstruct the complete assistant message from Anthropic Messages SSE
/// events (`content_block_start` / `content_block_delta`).
pub(crate) fn anthropic_message(events: &[Value]) -> Accumulated {
    #[derive(Default)]
    struct BlockAcc {
        tool_use: bool,
        /// Text deltas, or the raw JSON argument string for tool_use blocks.
        text: String,
        id: String,
        name: String,
    }
    let mut blocks: Vec<BlockAcc> = Vec::new();
    for ev in events {
        match ev.get("type").and_then(Value::as_str) {
            Some("content_block_start") => {
                let idx = ev.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                while blocks.len() <= idx {
                    blocks.push(BlockAcc::default());
                }
                let start = ev.get("content_block").cloned().unwrap_or(Value::Null);
                let slot = &mut blocks[idx];
                slot.tool_use = start.get("type").and_then(Value::as_str) == Some("tool_use");
                if let Some(id) = start.get("id").and_then(Value::as_str) {
                    slot.id = id.to_string();
                }
                if let Some(name) = start.get("name").and_then(Value::as_str) {
                    slot.name = name.to_string();
                }
                if let Some(text) = start.get("text").and_then(Value::as_str) {
                    slot.text.push_str(text);
                }
            }
            Some("content_block_delta") => {
                let idx = ev.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                if idx >= blocks.len() {
                    continue;
                }
                let delta = ev.get("delta").cloned().unwrap_or(Value::Null);
                let slot = &mut blocks[idx];
                match delta.get("type").and_then(Value::as_str) {
                    Some("text_delta") => {
                        if let Some(t) = delta.get("text").and_then(Value::as_str) {
                            slot.text.push_str(t);
                        }
                    }
                    Some("input_json_delta") => {
                        if let Some(p) = delta.get("partial_json").and_then(Value::as_str) {
                            slot.text.push_str(p);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let mut acc = Accumulated::default();
    for block in blocks {
        if block.tool_use {
            acc.blocks.push(serde_json::json!({
                "type": "tool_use",
                "id": block.id,
                "name": block.name,
                "input": serde_json::from_str::<Value>(&block.text)
                    .unwrap_or_else(|_| serde_json::json!({})),
            }));
            acc.tool_calls.push(ToolCallAcc {
                id: block.id,
                name: block.name,
                arguments: block.text,
            });
        } else {
            acc.blocks
                .push(serde_json::json!({ "type": "text", "text": block.text }));
            acc.text.push_str(&block.text);
        }
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events_of(body: &str) -> Vec<Value> {
        data_events(body.as_bytes())
    }

    #[test]
    fn data_events_parses_skips_sentinels_and_garbage() {
        let body = "data: {\"a\":1}\n\ndata: [DONE]\n\n: comment\n\ndata: not json\n\n";
        let events = events_of(body);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["a"], 1);
    }

    #[test]
    fn data_events_handles_crlf_and_multiline_data() {
        // CRLF endings + a payload split across two data lines (joined by \n
        // it is no longer valid JSON → skipped; a joined string field works).
        let body = "data: {\"text\": \"line1\\nline2\"}\r\n\r\ndata: {\"ok\":true}\r\n\r\n";
        let events = events_of(body);
        assert_eq!(events.len(), 2);
        assert_eq!(events[1]["ok"], true);
    }

    #[test]
    fn openai_message_accumulates_content_and_fragmented_arguments() {
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":\"Hel\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_9\",",
            "\"type\":\"function\",\"function\":{\"name\":\"vanta_memory_capture\",\"arguments\":\"\"}}]}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,",
            "\"function\":{\"arguments\":\"{\\\"text\\\":\\\"hi\\\"}\"}}]}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n",
            "data: [DONE]\n\n",
        );
        let acc = openai_message(&events_of(body));
        assert_eq!(acc.text, "Hello");
        assert_eq!(acc.tool_calls.len(), 1);
        assert_eq!(acc.tool_calls[0].id, "call_9");
        assert_eq!(acc.tool_calls[0].name, "vanta_memory_capture");
        assert_eq!(acc.tool_calls[0].arguments, r#"{"text":"hi"}"#);
    }

    #[test]
    fn anthropic_message_accumulates_text_and_tool_use_blocks() {
        let body = concat!(
            "data: {\"type\":\"message_start\"}\n\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"thinking\"}}\n\n",
            "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"vanta_memory_search\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"query\\\":\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"prefs\\\"}\"}}\n\n",
            "data: {\"type\":\"message_stop\"}\n\n",
        );
        let acc = anthropic_message(&events_of(body));
        assert_eq!(acc.text, "thinking");
        assert_eq!(acc.blocks.len(), 2);
        assert_eq!(acc.tool_calls.len(), 1);
        assert_eq!(acc.tool_calls[0].id, "toolu_1");
        assert_eq!(acc.tool_calls[0].name, "vanta_memory_search");
        assert_eq!(acc.tool_calls[0].arguments, r#"{"query":"prefs"}"#);
        assert_eq!(acc.blocks[1]["input"]["query"], "prefs");
    }

    #[test]
    fn drain_replays_chunks_verbatim() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        rt.block_on(async move {
            let captured = drain(Body::from_stream(futures::stream::iter([
                Ok::<_, std::convert::Infallible>(Bytes::from("a")),
                Ok(Bytes::from("b")),
            ])))
            .await
            .expect("drain");
            assert_eq!(&captured.full, b"ab");
            assert_eq!(captured.chunks.len(), 2);
        });
    }
}
