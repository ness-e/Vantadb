//! D19 dedicated L1 extractor tests (MEM-10 contract).
//!
//! Uses a local `CapturingRunner` (deterministic fake implementing the
//! [`LlmRunner`] trait) so the split, the single-call contract, degradation,
//! and tolerant parsing are verified without a real LLM or the `mock` feature.

use std::sync::Mutex;

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::conversation::{L0Message, L0Role};
use vanta_memory::core::record::{extract_l1_memories, L1ExtractorConfig};

/// Deterministic fake runner: records every call (system + user prompt) and
/// replays a pre-loaded script of responses/errors.
struct CapturingRunner {
    calls: Mutex<Vec<(String, String)>>,
    script: Mutex<Vec<Result<String, LlmError>>>,
}

impl CapturingRunner {
    fn new(script: Vec<Result<String, LlmError>>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            script: Mutex::new(script),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    fn last_prompt(&self) -> String {
        self.calls.lock().unwrap().last().unwrap().1.clone()
    }
}

impl LlmRunner for CapturingRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        self.calls.lock().unwrap().push((
            params.system_prompt.clone().unwrap_or_default(),
            params.prompt.clone(),
        ));
        let mut script = self.script.lock().unwrap();
        if script.is_empty() {
            return Err(LlmError::Other("unexpected extra call".into()));
        }
        script.remove(0)
    }
}

fn msg(id: &str, role: L0Role, content: &str, ts: u64) -> L0Message {
    L0Message {
        id: Some(id.to_string()),
        role,
        content: content.to_string(),
        timestamp_ms: ts,
    }
}

fn config() -> L1ExtractorConfig {
    L1ExtractorConfig::default()
}

// (a) End-to-end: valid LLM JSON → typed memories, scene names, single call.
#[test]
fn extracts_memories_end_to_end() {
    let runner = CapturingRunner::new(vec![Ok(
        r#"[
          {"scene_name": "Setting up the project", "message_ids": ["m2", "m3"], "memories": [
            {"content": "User prefers dark mode", "type": "preference", "priority": 80, "source_message_ids": ["m3"], "metadata": {}},
            {"content": "Decided to use cargo nextest", "type": "episode", "priority": 75, "source_message_ids": ["m2"]}
          ]}
        ]"#
        .to_string(),
    )]);
    let messages = vec![
        msg("m1", L0Role::User, "hey", 1000),
        msg("m2", L0Role::User, "let's use cargo nextest", 1100),
        msg("m3", L0Role::User, "I prefer dark mode", 1200),
    ];

    let result = extract_l1_memories(&runner, &messages, None, &config());

    assert!(result.success);
    assert_eq!(result.extracted_count, 2);
    assert_eq!(result.scene_names, vec!["Setting up the project"]);
    assert_eq!(
        result.last_scene_name.as_deref(),
        Some("Setting up the project")
    );
    // Writing + dedup land in MEM-11; extraction only reports.
    assert!(result.records.is_empty());
    assert_eq!(result.stored_count, 0);
    assert_eq!(runner.call_count(), 1, "exactly one LLM call");
}

// (b) LLM failure degrades to success:false — L0 data is never touched.
#[test]
fn runner_failure_degrades_gracefully() {
    let runner = CapturingRunner::new(vec![Err(LlmError::Timeout)]);
    let messages = vec![msg("m1", L0Role::User, "hello world", 1000)];

    let result = extract_l1_memories(&runner, &messages, None, &config());

    assert!(!result.success);
    assert_eq!(result.extracted_count, 0);
    assert!(result.scene_names.is_empty());
    assert!(result.last_scene_name.is_none());
}

// (c) Split: new messages are the extraction target; background is present in
// its own context-only section. Uses a small window so the split is real
// (defaults: max_new=10, max_bg=5).
#[test]
fn split_keeps_new_and_background_in_prompt() {
    let runner = CapturingRunner::new(vec![Ok("[]".to_string())]);
    let messages = vec![
        msg("old1", L0Role::User, "oldest — outside bg window", 100),
        msg("old2", L0Role::User, "older background", 200),
        msg("old3", L0Role::User, "old background", 300),
        msg("n1", L0Role::User, "recent one", 400),
        msg("n2", L0Role::User, "recent two", 500),
    ];
    let mut config = config();
    config.max_new_messages = 2;
    config.max_background_messages = 2;

    let result = extract_l1_memories(&runner, &messages, Some("prev"), &config);

    assert!(result.success);
    let prompt = runner.last_prompt();
    assert!(prompt.contains("PREVIOUS SCENE: prev"));
    assert!(prompt.contains("BACKGROUND CONVERSATION"));

    let sep = prompt.find("NEW MESSAGES TO EXTRACT FROM").unwrap();
    let bg_part = &prompt[..sep];
    let new_part = &prompt[sep..];
    // Background messages appear only in the background section.
    assert!(bg_part.contains("[old2]") && bg_part.contains("[old3]"));
    assert!(!new_part.contains("[old2]"));
    // New messages appear only in the new section.
    assert!(new_part.contains("[n1]") && new_part.contains("[n2]"));
    assert!(!bg_part.contains("[n1]"));
    // oldest1 is beyond both windows — excluded entirely.
    assert!(!bg_part.contains("[old1]") && !new_part.contains("[old1]"));
}

// (c2) Windows: with >10 messages, the 6 oldest are excluded; with 5 bg slots,
// the 6th-oldest is dropped entirely (not even background).
#[test]
fn window_excludes_oldest_beyond_background() {
    let runner = CapturingRunner::new(vec![Ok("[]".to_string())]);
    let messages: Vec<L0Message> = (0..16)
        .map(|i| msg(&format!("m{i}"), L0Role::User, &format!("message {i}"), i))
        .collect();

    let result = extract_l1_memories(&runner, &messages, None, &config());

    assert!(result.success);
    let prompt = runner.last_prompt();
    assert!(prompt.contains("[m15]"), "newest new message present");
    assert!(prompt.contains("[m5]"), "last background message present");
    assert!(
        !prompt.contains("[m0]") && !prompt.contains("message 0"),
        "oldest message must be outside both windows"
    );
}

// (d) Quality gate: pure noise never reaches the LLM.
#[test]
fn noise_only_input_skips_llm_call() {
    let runner = CapturingRunner::new(vec![]);
    let messages = vec![
        msg("x1", L0Role::User, "???", 100),
        msg("x2", L0Role::User, "/clear", 200),
        msg("x3", L0Role::User, "!!!", 300),
    ];

    let result = extract_l1_memories(&runner, &messages, None, &config());

    assert!(result.success);
    assert_eq!(result.extracted_count, 0);
    assert_eq!(runner.call_count(), 0, "no LLM call for pure noise");
}

// (e) Tolerant parse: code fence + prose + trailing commas all repaired.
#[test]
fn repairs_trailing_commas_and_fences() {
    let runner = CapturingRunner::new(vec![Ok(
        "Sure!\n```json\n[{\"scene_name\":\"s\",\"message_ids\":[\"m1\",],\"memories\":[{\"content\":\"c\",\"type\":\"episodic\",\"priority\":70,}]}]\n```\nDone."
            .to_string(),
    )]);
    let messages = vec![msg("m1", L0Role::User, "usable content", 0)];

    let result = extract_l1_memories(&runner, &messages, None, &config());

    assert!(result.success);
    assert_eq!(result.extracted_count, 1);
}

// (f) Invalid/unknown memory types are dropped, valid ones survive.
#[test]
fn invalid_types_dropped_in_e2e() {
    let runner = CapturingRunner::new(vec![Ok(
        r#"[{"scene_name":"s","message_ids":[],"memories":[
            {"content":"good one","type":"preference","priority":60},
            {"content":"bad one","type":"quantum_state","priority":60}
        ]}]"#
            .to_string(),
    )]);
    let messages = vec![msg("m1", L0Role::User, "content", 0)];

    let result = extract_l1_memories(&runner, &messages, None, &config());

    assert!(result.success);
    assert_eq!(result.extracted_count, 1);
}

// (g) max_memories_per_session truncates the extracted batch.
#[test]
fn truncates_to_max_memories() {
    let memories: Vec<String> = (0..5)
        .map(|i| format!(r#"{{"content":"mem {i}","type":"episodic","priority":60}}"#))
        .collect();
    let raw = format!(
        r#"[{{"scene_name":"s","message_ids":[],"memories":[{memories}]}}]"#,
        memories = memories.join(",")
    );
    let runner = CapturingRunner::new(vec![Ok(raw)]);
    let messages = vec![msg("m1", L0Role::User, "content", 0)];

    let mut config = config();
    config.max_memories_per_session = 2;

    let result = extract_l1_memories(&runner, &messages, None, &config);

    assert!(result.success);
    assert_eq!(result.extracted_count, 2);
}

// (h) Empty scene memories still report the scene segmentation.
#[test]
fn scenes_without_memories_are_reported() {
    let runner = CapturingRunner::new(vec![Ok(
        r#"[{"scene_name":"scene a","message_ids":["m1"],"memories":[]},
            {"scene_name":"scene b","message_ids":["m2"],"memories":[]}]"#
            .to_string(),
    )]);
    let messages = vec![msg("m1", L0Role::User, "content", 0)];

    let result = extract_l1_memories(&runner, &messages, None, &config());

    assert!(result.success);
    assert_eq!(result.extracted_count, 0);
    assert_eq!(result.scene_names, vec!["scene a", "scene b"]);
    assert_eq!(result.last_scene_name.as_deref(), Some("scene b"));
}
