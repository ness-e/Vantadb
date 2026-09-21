//! Auxiliary endpoints (PRX-05): model discovery + token counting.
//!
//! - `GET /v1/models`: OpenAI List-Models shape derived from
//!   `[upstream] models` config (never hardcoded — pre-mortem PRX-05).
//! - `POST /v1/messages/count_tokens`: local estimate, no upstream call.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use bytes::Bytes;
use serde_json::{json, Value};

use crate::config::UpstreamConfig;
use crate::error::ProxyError;
use crate::server::AppState;

/// OpenAI List-Models shape (`platform.openai.com/docs/api-reference/models/list`)
/// built from `[upstream] models` config.
pub fn models_response(cfg: &UpstreamConfig) -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let data: Vec<Value> = cfg
        .models
        .iter()
        .map(|m| json!({"id": m, "object": "model", "created": now, "owned_by": "vanta-proxy"}))
        .collect();
    json!({"object": "list", "data": data})
}

/// Local token estimate (`docs.anthropic.com` Count Tokens shape
/// `{"input_tokens": N}`): ~4 chars per token over every string in the body.
// ponytail: heuristic ceil(chars/4), no tokenizer dep; min 1 so `{}` never yields 0.
pub fn estimate_tokens(body: &Value) -> u64 {
    fn chars(value: &Value) -> usize {
        match value {
            Value::String(s) => s.chars().count(),
            Value::Array(items) => items.iter().map(chars).sum(),
            Value::Object(map) => map.values().map(chars).sum(),
            _ => 0,
        }
    }
    let total = chars(body);
    ((total + 3) / 4).max(1) as u64
}

/// `GET /v1/models` (also `/{agent}/{spaceId}/v1/models`) — auth required (D34).
pub async fn models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ProxyError> {
    state.auth.authenticate(&headers)?;
    Ok(Json(models_response(&state.config.upstream)))
}

/// `POST /v1/messages/count_tokens` (also `/{agent}/{spaceId}/...`) —
/// auth required (D34); local estimate, never forwarded upstream.
pub async fn count_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ProxyError> {
    state.auth.authenticate(&headers)?;
    let value: Value = serde_json::from_slice(body.as_ref())
        .map_err(|e| ProxyError::InvalidRequest(format!("invalid JSON body: {e}")))?;
    Ok(Json(json!({"input_tokens": estimate_tokens(&value)})))
}

#[cfg(test)]
mod tests {
    use crate::config::UpstreamConfig;

    #[test]
    fn models_derive_from_config_not_hardcoded() {
        let cfg = UpstreamConfig {
            models: vec!["claude-test-a".to_string(), "gpt-test-b".to_string()],
            ..UpstreamConfig::default()
        };
        let value = super::models_response(&cfg);
        assert_eq!(value["object"], "list");
        let ids: Vec<&str> = value["data"]
            .as_array()
            .expect("data array")
            .iter()
            .filter_map(|m| m.get("id").and_then(|v| v.as_str()))
            .collect();
        assert_eq!(ids, vec!["claude-test-a", "gpt-test-b"]);
        for m in value["data"].as_array().expect("array") {
            assert_eq!(m["object"], "model");
        }
    }

    #[test]
    fn models_empty_config_returns_empty_list() {
        let value = super::models_response(&UpstreamConfig::default());
        assert_eq!(value["object"], "list");
        assert_eq!(value["data"].as_array().expect("array").len(), 0);
    }

    #[test]
    fn count_tokens_scales_with_input_text() {
        let short =
            serde_json::json!({"model": "c", "messages": [{"role": "user", "content": "hi"}]});
        let long = serde_json::json!({"model": "c", "messages": [{"role": "user", "content": "hi".repeat(400)}]});
        let n_short = super::estimate_tokens(&short);
        let n_long = super::estimate_tokens(&long);
        assert!(n_short >= 1, "minimum one token");
        assert!(n_long > n_short, "longer input → more tokens");
    }

    #[test]
    fn count_tokens_empty_body_counts_one() {
        let value = serde_json::json!({});
        assert!(super::estimate_tokens(&value) >= 1);
    }
}
