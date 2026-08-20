//! Deterministic fake LLM runner for tests (feature `mock`).
//!
//! Reused by the dedicated tests of MEM-09..21 (D19): returns configured
//! canned responses so pipeline tests never hit a real LLM.

use std::sync::Mutex;

use crate::core::abstractions::{LlmError, LlmRunParams, LlmRunner};

/// Scripted responses for the mock runner.
#[derive(Debug, Clone)]
pub struct MockScript {
    /// Responses returned in order; the last one repeats indefinitely.
    pub responses: Vec<String>,
    /// Error to return (overrides `responses` when set).
    pub error: Option<LlmError>,
}

/// [`LlmRunner`] that replays fixed responses.
pub struct MockLlmRunner {
    script: MockScript,
    call_count: Mutex<usize>,
}

impl MockLlmRunner {
    /// Create a runner that always returns `response`.
    pub fn fixed(response: impl Into<String>) -> Self {
        Self::script(MockScript {
            responses: vec![response.into()],
            error: None,
        })
    }

    /// Create a runner from a script (ordered responses, error override).
    pub fn script(script: MockScript) -> Self {
        Self {
            script,
            call_count: Mutex::new(0),
        }
    }

    /// Number of `run` calls so far (assert call counts in tests).
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

impl LlmRunner for MockLlmRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        if let Some(err) = &self.script.error {
            return Err(err.clone());
        }
        let mut count = self.call_count.lock().unwrap();
        let idx = (*count).min(self.script.responses.len().saturating_sub(1));
        *count += 1;
        Ok(self.script.responses[idx].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_replays_fixed_json() {
        let runner = MockLlmRunner::fixed(r#"{"record_id":"m1","action":"store","target_ids":[]}"#);
        let decision: crate::core::abstractions::DedupDecision = runner
            .complete_json(&LlmRunParams::new("p", "l1-dedup"))
            .unwrap();
        assert_eq!(decision.record_id, "m1");
        assert_eq!(runner.call_count(), 1);
    }

    #[test]
    fn mock_script_repeats_last_response() {
        let runner = MockLlmRunner::script(MockScript {
            responses: vec!["first".into(), "second".into()],
            error: None,
        });
        let params = LlmRunParams::new("p", "t");
        assert_eq!(runner.run(&params).unwrap(), "first");
        assert_eq!(runner.run(&params).unwrap(), "second");
        assert_eq!(runner.run(&params).unwrap(), "second");
        assert_eq!(runner.call_count(), 3);
    }

    #[test]
    fn mock_propagates_error() {
        let runner = MockLlmRunner::script(MockScript {
            responses: vec![],
            error: Some(LlmError::NotConfigured),
        });
        let result = runner.run(&LlmRunParams::new("p", "t"));
        assert!(matches!(result, Err(LlmError::NotConfigured)));
    }
}
