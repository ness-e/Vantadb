//! MCP `initialize` request handler.

use serde_json::{json, Value};
use vantadb::metadata;

/// Latest stable MCP protocol version this server advertises.
pub const LATEST_PROTOCOL_VERSION: &str = "2025-06-18";

/// All protocol versions this server understands (latest first).
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2024-11-05"];

/// Recall-first server instructions, surfaced to the agent via the MCP
/// `initialize` result (`instructions` field, spec 2025-06-18 — see
/// https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle/).
/// This is what the agent reads before any tool call: recall before answering,
/// cognitive lifecycle L0->L3, never improvise temporal ranges (see
/// `temporal::parse_temporal_expression`), and curate through threads.
pub const SERVER_INSTRUCTIONS: &str = "VantaDB persistent memory. Cognitive lifecycle L0->L3: \
    L0 Working Context: assemble prompt context via `context_assemble` and capture conversation turns via `thread_send`. \
    L1 Episodic Memory: recall-first policy — before answering questions about past work, call `memory_recall` (scope agent, top_k 5) and only inject hits into context when recalled is non-empty — when nothing is recalled, say so and do not fill the gap; mutate with `memory_put`, `memory_supersede`, and `memory_forget`. \
    L2 Narrative Scenes: navigate structured narrative state via `vanta_scene` (or `scene_*`). \
    L3 Axioms & Dreams: consolidate patterns, insights, and persona principles via `vanta_dream` (or `dream_*` / `read_axioms`). \
    Full catalog: access all 88 tools via `vanta_dispatch` or inspect schema via resource `vanta://tools/catalog`. \
    Operating manual: read `vanta://brain/operating-manual`. \
    Temporal expressions (\"yesterday at 2pm\", \"ayer a las 2pm\") are deterministic [from_ms, to_ms] ranges, never guesses — see the skill recall policy. \
    Capture durable turns with `thread_send`; curate proxy turns into threads through the approval inbox. \
    Namespaces isolate contexts; TTL + supersession govern vigencia; `audit_text_index` verifies the text index.";

/// Handle the `initialize` request, returning protocol version, server info and capabilities.
///
/// Negotiation: if `params.protocolVersion` is a supported version, echo it
/// back; otherwise default to `LATEST_PROTOCOL_VERSION`. This keeps old
/// clients (2024-11-05) working while new clients get 2025-06-18
/// (structured output, annotations). Unknown versions fall forward to latest
/// rather than erroring — forward-compatible per MCP spec guidance.
pub fn handle_initialize(params: Option<&Value>) -> Result<Value, Value> {
    let requested = params
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str());
    let negotiated = match requested {
        Some(v) if SUPPORTED_PROTOCOL_VERSIONS.contains(&v) => v,
        _ => LATEST_PROTOCOL_VERSION,
    };
    Ok(json!({
        "protocolVersion": negotiated,
        "serverInfo": {
            "name": metadata::MCP_SERVER_INFO_NAME,
            "version": metadata::reported_version().into_owned()
        },
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "instructions": SERVER_INSTRUCTIONS
    }))
}
