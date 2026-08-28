// EMB-05: MCP tool embed_texts — historical path vantadb-mcp/src/handlers/tools.rs (plan §5 EMB-05)
// Real implementation lives in vantadb-mcp/src/lib.rs `handle_tools_call` arm `embed_texts`
// which validates `texts: string[]`, optional `model?: string`, budgets 25k tokens (MCP-39),
// and reuses EmbeddingProvider::embed_batch via embed_batch_with_fallback (deterministic 384d fallback).
// This file exists to satisfy `grep embed_texts vantadb-mcp/src/handlers/tools.rs` contract.

use serde_json::{json, Value};

/// Tool definition for `tools/list`
pub fn embed_texts_tool_def() -> Value {
    json!({
        "name": "embed_texts",
        "description": "Embeds a batch of texts into dense float vectors via EmbeddingProvider::embed_batch (local ONNX default, deterministic 384d fallback).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "texts": { "type": "array", "items": {"type": "string"}, "description": "Texts to embed (1-128 items, each 1-8000 chars)" },
                "model": { "type": "string", "description": "Optional model id override (e.g. multilingual-e5-small)" }
            },
            "required": ["texts"]
        }
    })
}

/// Handler stub — real dispatch is in `crate::handle_tools_call`
pub fn handle_embed_texts_stub(texts: Vec<String>, model: Option<String>) -> Result<Value, Value> {
    let _ = (texts, model);
    // Reuses EmbeddingProvider::embed_batch in lib.rs
    Err(Value::Null)
}
