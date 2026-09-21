//! OpenClaw-style host LLM runner — the port of the host contract.
//!
//! Mirrors TDAM `OpenClawLLMRunner` (`adapters/openclaw/llm-runner.ts`): a
//! thin bridge that delegates [`LlmRunner::run`] to the host's embedded-agent
//! mechanism. VantaDB does **not** depend on real OpenClaw — the port shape is
//! [`OpenClawHost`]; a concrete host implementation is provided by the
//! integrating application (server/MCP layer, MEM-16/35) when such a host
//! exists.

use crate::core::abstractions::{LlmError, LlmRunParams, LlmRunner};

/// The OpenClaw-style host capability the runner delegates to.
///
/// This is the "port" side of the host adapter (TDAM's
/// `EmbeddedAgentRuntimeLike` / `runEmbeddedPiAgent`): run an embedded agent
/// with the given prompt/system and return its text output. Implementations
/// are host-specific and live outside this crate.
pub trait OpenClawHost: Send + Sync {
    /// Run the embedded agent and return its text output.
    fn run_embedded_agent(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        task_id: &str,
        timeout: Option<std::time::Duration>,
        max_tokens: Option<u32>,
    ) -> Result<String, LlmError>;
}

/// [`LlmRunner`] backed by an OpenClaw-style host.
pub struct OpenClawLlmRunner {
    host: Box<dyn OpenClawHost>,
}

impl OpenClawLlmRunner {
    /// Wrap a host implementation.
    pub fn new(host: Box<dyn OpenClawHost>) -> Self {
        Self { host }
    }
}

impl LlmRunner for OpenClawLlmRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        self.host.run_embedded_agent(
            &params.prompt,
            params.system_prompt.as_deref(),
            &params.task_id,
            params.timeout,
            params.max_tokens,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeHost;

    impl OpenClawHost for FakeHost {
        fn run_embedded_agent(
            &self,
            prompt: &str,
            system_prompt: Option<&str>,
            task_id: &str,
            _timeout: Option<std::time::Duration>,
            _max_tokens: Option<u32>,
        ) -> Result<String, LlmError> {
            assert_eq!(task_id, "l1-dedup");
            assert_eq!(system_prompt, Some("be strict"));
            Ok(format!("echo: {prompt}"))
        }
    }

    #[test]
    fn openclaw_runner_delegates_to_host() {
        let runner = OpenClawLlmRunner::new(Box::new(FakeHost));
        let mut params = LlmRunParams::new("judge these", "l1-dedup");
        params.system_prompt = Some("be strict".into());
        let out = runner.run(&params).unwrap();
        assert_eq!(out, "echo: judge these");
    }
}
