// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! D19 — dedicated tests for the `LlmRunner` trait contract (MEM-08b).
//!
//! Uses a local fixed runner (no features required) to prove the call
//! contract: `run()` returns text, `complete_json()` parses structured JSON
//! (the operation L1 extract/dedup, L2 scene, L3 persona need), and errors
//! propagate as `LlmError`.

use vanta_memory::core::abstractions::{
    DedupDecision, LlmError, LlmRunParams, LlmRunner, SceneMeta,
};

/// Test-local deterministic runner: returns a canned string and records the
/// last params it saw.
struct FixedRunner {
    output: String,
    last_params: std::sync::Mutex<Option<LlmRunParams>>,
}

impl FixedRunner {
    fn new(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            last_params: std::sync::Mutex::new(None),
        }
    }
}

impl LlmRunner for FixedRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        *self.last_params.lock().unwrap() = Some(params.clone());
        Ok(self.output.clone())
    }
}

#[test]
fn run_returns_text_and_sees_params() {
    let runner = FixedRunner::new("plain text");
    let mut params = LlmRunParams::new("extract memories", "l1-extraction");
    params.system_prompt = Some("You are a memory extractor.".into());
    params.max_tokens = Some(1024);

    let out = runner.run(&params).unwrap();
    assert_eq!(out, "plain text");

    let seen = runner.last_params.lock().unwrap().clone().unwrap();
    assert_eq!(seen.task_id, "l1-extraction");
    assert_eq!(
        seen.system_prompt.as_deref(),
        Some("You are a memory extractor.")
    );
    assert_eq!(seen.max_tokens, Some(1024));
}

#[test]
fn complete_json_parses_dedup_decisions() {
    let wire = r#"```json
    [{"record_id":"m1","action":"update","target_ids":["m_old"],"merged_priority":90}]
    ```"#;
    let runner = FixedRunner::new(wire);
    let decisions: Vec<DedupDecision> = runner
        .complete_json(&LlmRunParams::new(
            "judge conflicts",
            "l1-conflict-detection",
        ))
        .unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].record_id, "m1");
    assert_eq!(decisions[0].merged_priority, Some(90));
}

#[test]
fn complete_json_parses_scene_meta_from_prose() {
    let wire = r#"The scene:
{"created":"2026-08-20T10:00:00Z","updated":"2026-08-20T10:00:00Z","summary":"deploy","heat":5}
Done."#;
    let runner = FixedRunner::new(wire);
    let meta: SceneMeta = runner
        .complete_json(&LlmRunParams::new("extract scene", "l2-scene"))
        .unwrap();
    assert_eq!(meta.heat, 5);
}

#[test]
fn complete_json_propagates_invalid_json() {
    let runner = FixedRunner::new("I could not parse that request.");
    let result: Result<DedupDecision, LlmError> =
        runner.complete_json(&LlmRunParams::new("judge", "l1-dedup"));
    assert!(matches!(result, Err(LlmError::InvalidJson { task_id, .. }) if task_id == "l1-dedup"));
}

#[test]
fn complete_json_propagates_runner_error() {
    struct FailingRunner;
    impl LlmRunner for FailingRunner {
        fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
            Err(LlmError::Timeout)
        }
    }
    let result: Result<SceneMeta, LlmError> =
        FailingRunner.complete_json(&LlmRunParams::new("x", "l2-scene"));
    assert!(matches!(result, Err(LlmError::Timeout)));
}

#[cfg(feature = "mock")]
mod mock_feature {
    use super::*;
    use vanta_memory::adapters::MockLlmRunner;

    #[test]
    fn reusable_mock_runner_serves_dedup() {
        // The shared `mock`-feature runner: what MEM-09..21 pipeline tests
        // will reuse (D19).
        let runner = MockLlmRunner::fixed(r#"{"record_id":"m9","action":"store","target_ids":[]}"#);
        let decision: DedupDecision = runner
            .complete_json(&LlmRunParams::new("p", "l1-conflict-detection"))
            .unwrap();
        assert_eq!(decision.record_id, "m9");
        assert_eq!(runner.call_count(), 1);
    }
}
