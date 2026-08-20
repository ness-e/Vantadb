//! Host-neutral LLM runner abstraction (D1: sync base + optional async).
//!
//! Mirrors the TDAM `LLMRunner` contract (`MemoryCore/src/core/types.ts:170-191`)
//! reimplemented in Rust: the pipeline (L1 extract, L1 dedup, L2 scene, L3
//! persona) depends on this trait — never on a concrete host. The crate ships
//! no LLM runtime; with `llm-driver` off (default) every LLM-dependent path
//! degrades to an LLM-free equivalent (local compression, store-all, heuristic
//! dedup) and nothing blocks.

use serde::de::DeserializeOwned;
use thiserror::Error;

/// Parameters for a single LLM execution.
///
/// Source: TDAM `LLMRunParams` (`core/types.ts:64-135`) — reduced to the
/// host-neutral core the pipeline actually consumes. Tool-call loops (L2/L3
/// sandboxed read/write/edit) are added by MEM-13/14 on top of this struct.
#[derive(Debug, Clone)]
pub struct LlmRunParams {
    /// User-facing prompt (or combined prompt when no system prompt).
    pub prompt: String,
    /// Optional system prompt. When present, `prompt` is the user message.
    pub system_prompt: Option<String>,
    /// Unique task identifier for logging and metrics.
    pub task_id: String,
    /// Execution timeout.
    pub timeout: Option<std::time::Duration>,
    /// Max output tokens (optional — defaults to model/config value).
    pub max_tokens: Option<u32>,
    /// Working directory for tool-enabled runs (unused for pure-text tasks).
    pub workspace_dir: Option<String>,
    /// Plugin instance ID for metric reporting (optional).
    pub instance_id: Option<String>,
}

impl LlmRunParams {
    /// Build a minimal pure-text task (the common L1/L2/L3 case).
    pub fn new(prompt: impl Into<String>, task_id: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            system_prompt: None,
            task_id: task_id.into(),
            timeout: None,
            max_tokens: None,
            workspace_dir: None,
            instance_id: None,
        }
    }
}

/// Errors surfaced by [`LlmRunner::run`]. Callers degrade per layer
/// (store-all, heuristic dedup) — an LLM failure never loses data.
///
/// `Clone` is needed by the scripted [`crate::adapters::MockLlmRunner`]
/// (`mock` feature) so tests can pre-load errors.
#[derive(Debug, Clone, Error)]
pub enum LlmError {
    /// The runner is not configured for LLM calls (LLM-free mode). Callers
    /// must degrade, never block.
    #[error("LLM runner not configured (LLM-free mode)")]
    NotConfigured,
    /// The request timed out.
    #[error("LLM call timed out")]
    Timeout,
    /// Transport/network failure.
    #[error("LLM transport error: {0}")]
    Transport(String),
    /// The provider returned an error status.
    #[error("LLM HTTP error {status}: {message}")]
    Http { status: u16, message: String },
    /// The model output was not valid JSON for the requested shape.
    #[error("LLM output is not valid JSON for {task_id}: {message}")]
    InvalidJson { task_id: String, message: String },
    /// Any other unrecoverable failure.
    #[error("LLM call failed: {0}")]
    Other(String),
}

/// Host-neutral, **synchronous** LLM execution contract (D1).
///
/// Implementations:
/// - [`crate::adapters::standalone::llm_runner::StandaloneLlmRunner`] — direct
///   OpenAI-compatible HTTP (host-less), wired under the `llm-driver` feature.
/// - [`crate::adapters::openclaw::llm_runner::OpenClawLlmRunner`] — delegates
///   to an OpenClaw-style host via [`crate::adapters::openclaw::llm_runner::OpenClawHost`].
/// - [`crate::adapters::mock::MockLlmRunner`] — deterministic fake for tests
///   (`mock` feature).
///
/// Returns the model's text output. Errors are reported as [`LlmError`]; the
/// pipeline degrades gracefully instead of blocking.
pub trait LlmRunner {
    /// Execute a prompt and return the LLM's text output.
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError>;

    /// Convenience: run and parse the output as structured JSON (`T`).
    ///
    /// Strips markdown code fences, extracts the first JSON array/object, and
    /// deserializes — the operation L1 extraction, L1 dedup, L2 scene and L3
    /// persona all need. Full output-repair heuristics live in MEM-10
    /// (`offload/local_llm/parsers/json_utils.rs`); this is the base contract.
    fn complete_json<T: DeserializeOwned>(&self, params: &LlmRunParams) -> Result<T, LlmError> {
        let raw = self.run(params)?;
        extract_json(&raw)
            .ok_or_else(|| LlmError::InvalidJson {
                task_id: params.task_id.clone(),
                message: "no JSON array/object found in output".into(),
            })
            .and_then(|slice| {
                serde_json::from_str(slice).map_err(|e| LlmError::InvalidJson {
                    task_id: params.task_id.clone(),
                    message: e.to_string(),
                })
            })
    }
}

/// Extract the first JSON array or object from a model response, stripping
/// markdown code fences and surrounding prose.
fn extract_json(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let body = trimmed
        .strip_prefix("```")
        .map(|s| s.strip_prefix("json").unwrap_or(s))
        .map(str::trim)
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(trimmed);
    let body = body.trim();
    if body.starts_with('[') || body.starts_with('{') {
        return Some(body);
    }
    // Fall back to scanning for the first array/object bracket.
    for (i, ch) in body.char_indices() {
        if ch == '[' || ch == '{' {
            let open = ch;
            let close = if open == '[' { ']' } else { '}' };
            if let Some(end) = find_matching(body, i, open, close) {
                return Some(&body[i..=end]);
            }
        }
    }
    None
}

/// Find the index of the bracket matching `open` at position `start`
/// (naive depth scan — good enough for LLM outputs; the full repair
/// heuristics live in MEM-10).
fn find_matching(s: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in s.char_indices().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Optional **asynchronous** LLM execution contract (D1).
///
/// The server layer (MEM-16 orchestration, MEM-35 data plane) adapts a sync
/// [`LlmRunner`] to this trait (e.g. `tokio::task::spawn_blocking`) — the
/// crate deliberately ships no executor. Gated on `llm-driver` because the
/// async paths only exist when LLM-driven features are enabled.
#[cfg(feature = "llm-driver")]
pub trait AsyncLlmRunner {
    /// Execute a prompt asynchronously and return the LLM's text output.
    ///
    /// Returned futures must be `Send` so the server can drive them on a
    /// worker (e.g. `tokio::spawn`); implementors write a plain
    /// `async fn run(...)`.
    fn run(
        &self,
        params: &LlmRunParams,
    ) -> impl std::future::Future<Output = Result<String, LlmError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRunner(&'static str);

    impl LlmRunner for FixedRunner {
        fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
            Ok(self.0.to_string())
        }
    }

    #[test]
    fn complete_json_parses_fenced_array() {
        let runner = FixedRunner(
            r#"```json
            [{"record_id":"m1","action":"store","target_ids":[]}]
            ```"#,
        );
        let decisions: Vec<crate::core::abstractions::types::DedupDecision> = runner
            .complete_json(&LlmRunParams::new("prompt", "l1-conflict-detection"))
            .unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].record_id, "m1");
    }

    #[test]
    fn complete_json_extracts_object_from_prose() {
        let runner = FixedRunner(
            "Here is the scene:\n{\"created\":\"2026-08-20T10:00:00Z\",\"updated\":\"2026-08-20T10:00:00Z\",\"summary\":\"s\",\"heat\":3}\nDone.",
        );
        let meta: crate::core::abstractions::types::SceneMeta = runner
            .complete_json(&LlmRunParams::new("prompt", "l2-scene"))
            .unwrap();
        assert_eq!(meta.heat, 3);
    }

    #[test]
    fn complete_json_errors_on_no_json() {
        let runner = FixedRunner("I have nothing to report.");
        let result: Result<crate::core::abstractions::types::DedupDecision, LlmError> =
            runner.complete_json(&LlmRunParams::new("prompt", "l1-dedup"));
        assert!(matches!(result, Err(LlmError::InvalidJson { .. })));
    }

    #[test]
    fn extract_json_handles_strings_with_brackets() {
        let raw = r#"{"content":"use [brackets] literally","type":"persona"}"#;
        let slice = extract_json(raw).unwrap();
        let v: serde_json::Value = serde_json::from_str(slice).unwrap();
        assert_eq!(v["content"], "use [brackets] literally");
    }
}
