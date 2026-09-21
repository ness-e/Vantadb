//! Standalone LLM runner — no host required.
//!
//! Mirrors TDAM `StandaloneLLMRunner` (`adapters/standalone/llm-runner.ts`):
//! direct OpenAI-compatible HTTP calls (`/chat/completions`) for the
//! host-less scenario. Pure-text tasks only (L1 extract, L1 dedup, L2 scene,
//! L3 persona) — the tool-call loop (L2/L3 sandboxed read/write/edit) is
//! added by MEM-13/14.
//!
//! Feature gating: the real HTTP transport compiles only with `llm-driver`.
//! Without it (default, LLM-free) the runner exists but [`run`] returns
//! [`LlmError::NotConfigured`] — callers degrade (store-all, heuristic dedup)
//! and never block.

use std::time::Duration;

use crate::core::abstractions::{LlmError, LlmRunParams, LlmRunner};

/// Configuration for an OpenAI-compatible endpoint.
///
/// Source: TDAM `StandaloneLLMConfig` (`adapters/standalone/llm-runner.ts:80-103`)
/// reduced to the host-neutral core.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// OpenAI-compatible API base URL (e.g. `https://api.openai.com/v1`).
    pub base_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// Default model name (e.g. `gpt-4o`).
    pub model: String,
    /// Default max output tokens.
    pub max_tokens: Option<u32>,
    /// Request timeout.
    pub timeout: Option<Duration>,
}

/// Host-less runner that calls an external OpenAI-compatible endpoint.
#[derive(Debug, Clone)]
pub struct StandaloneLlmRunner {
    // Only consumed by the `llm-driver` HTTP transport; kept for the
    // default (LLM-free) build so the runner still carries its endpoint
    // config and can be constructed identically.
    #[cfg_attr(not(feature = "llm-driver"), allow(dead_code))]
    config: LlmConfig,
    model: String,
}

impl StandaloneLlmRunner {
    /// Create a runner for the given endpoint config.
    pub fn new(config: LlmConfig) -> Self {
        let model = config.model.clone();
        Self { config, model }
    }

    /// Override the model for this runner instance.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// The effective model used for calls.
    pub fn model(&self) -> &str {
        &self.model
    }
}

#[cfg(feature = "llm-driver")]
mod http {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    pub(super) struct ChatCompletionRequest<'a> {
        pub(super) model: &'a str,
        pub(super) messages: Vec<ChatMessage<'a>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(super) max_tokens: Option<u32>,
    }

    #[derive(Serialize)]
    pub(super) struct ChatMessage<'a> {
        pub(super) role: &'a str,
        pub(super) content: &'a str,
    }

    #[derive(Deserialize)]
    pub(super) struct ChatCompletionResponse {
        pub(super) choices: Vec<ChatChoice>,
    }

    #[derive(Deserialize)]
    pub(super) struct ChatChoice {
        pub(super) message: ChatResponseMessage,
    }

    #[derive(Deserialize)]
    pub(super) struct ChatResponseMessage {
        #[serde(default)]
        pub(super) content: String,
    }
}

#[cfg(feature = "llm-driver")]
use http::*;

impl LlmRunner for StandaloneLlmRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        #[cfg(not(feature = "llm-driver"))]
        {
            // LLM-free mode: the runner is a placeholder; the pipeline must
            // degrade, never block.
            let _ = params;
            Err(LlmError::NotConfigured)
        }
        #[cfg(feature = "llm-driver")]
        {
            self.run_http(params)
        }
    }
}

#[cfg(feature = "llm-driver")]
impl StandaloneLlmRunner {
    fn run_http(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        let timeout = params
            .timeout
            .or(self.config.timeout)
            .unwrap_or(Duration::from_secs(120));
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| LlmError::Transport(e.to_string()))?;

        let mut messages = Vec::with_capacity(2);
        if let Some(system) = &params.system_prompt {
            messages.push(ChatMessage {
                role: "system",
                content: system,
            });
        }
        messages.push(ChatMessage {
            role: "user",
            content: &params.prompt,
        });

        let body = ChatCompletionRequest {
            model: &self.model,
            messages,
            max_tokens: params.max_tokens.or(self.config.max_tokens),
        };

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let resp = client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout
                } else {
                    LlmError::Transport(e.to_string())
                }
            })?;

        let status = resp.status();
        if !status.is_success() {
            let message = resp.text().unwrap_or_default();
            return Err(LlmError::Http {
                status: status.as_u16(),
                message,
            });
        }

        let parsed: ChatCompletionResponse = resp.json().map_err(|e| LlmError::InvalidJson {
            task_id: params.task_id.clone(),
            message: e.to_string(),
        })?;

        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| LlmError::Other("empty choices in chat completion response".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_holds_model_override() {
        let runner = StandaloneLlmRunner::new(LlmConfig {
            base_url: "http://localhost:11434/v1".into(),
            api_key: "unused".into(),
            model: "qwen2.5".into(),
            max_tokens: None,
            timeout: None,
        })
        .with_model("deepseek-v3");
        assert_eq!(runner.model(), "deepseek-v3");
    }

    #[test]
    fn llm_free_mode_reports_not_configured() {
        // Default features (no `llm-driver`): the runner must degrade.
        let runner = StandaloneLlmRunner::new(LlmConfig {
            base_url: "http://localhost:11434/v1".into(),
            api_key: "unused".into(),
            model: "qwen2.5".into(),
            max_tokens: None,
            timeout: None,
        });
        let result = runner.run(&LlmRunParams::new("hello", "l1-extraction"));
        assert!(matches!(result, Err(LlmError::NotConfigured)));
    }
}
