//! MEM-17 D19 — skill extraction + conversation-add pipeline + idempotent
//! sink. All storage runs against an in-memory VantaDB; all LLM behaviour
//! against deterministic fake runners (no network, no sleeps).

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::skill::conversation_add::{
    archive_namespace, prepare_archive_payload, tasks_namespace, trigger_archive, ArchiveStore,
    SkillMessage, SkillWorkerOutcome, COMPRESS_DEFAULTS, OVERSIZE_DEFAULTS,
};
use vanta_memory::core::skill::skill_extractor::{
    extract_skills_with_llm, ExtractMessage, SkillExtractorConfig, SkillSummary,
};
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;

/// Emits a fixed response regardless of prompt.
struct FixedRunner(&'static str);

impl LlmRunner for FixedRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        Ok(self.0.to_string())
    }
}

/// Always fails (Principio 4 degradation path).
struct FailingRunner;

impl LlmRunner for FailingRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        Err(LlmError::Transport("boom".into()))
    }
}

fn db() -> VantaEmbedded {
    VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..VantaConfig::default()
    })
    .expect("open in-memory db")
}

fn transcript() -> Vec<SkillMessage> {
    vec![
        SkillMessage::new("user", "deploys keep failing on crashloopbackoff"),
        SkillMessage::new(
            "assistant",
            "check kubectl describe pod, look at last state, fix the liveness probe",
        ),
    ]
}

const CANDIDATES_JSON: &str = r#"```json
[{"action":"create","name":"k8s-crashloop-triage","description":"triage crashlooping pods","content":"When to use: pod stuck in CrashLoopBackOff."}]
```"#;

#[test]
fn trigger_writes_archive_then_task_and_both_resolve() {
    let db = db();
    let result = trigger_archive(&db, "sess-1", &transcript(), 1_000).expect("trigger");
    assert_eq!(result.task_id, "skill-extract-task-1000");

    let store = ArchiveStore::new(&db);
    let entry = store
        .read_task("sess-1", &result.task_id)
        .expect("read task")
        .expect("task exists");
    assert_eq!(entry.status, "pending");
    let messages = store
        .read_archive("sess-1", &entry.archive_key)
        .expect("read archive")
        .expect("archive exists");
    assert_eq!(messages.len(), 2);
    // Namespaces are sanitized and deterministic.
    assert_eq!(archive_namespace("sess-1"), "skill_archive/sess-1");
    assert_eq!(tasks_namespace("sess-1"), "skill_tasks/sess-1");
}

#[test]
fn prepare_payload_compresses_and_reports_flags() {
    let long_tool = SkillMessage::new("tool_result", "x".repeat(4_096));
    let prepared = prepare_archive_payload(
        &[],
        &[long_tool],
        true,
        &COMPRESS_DEFAULTS,
        &OVERSIZE_DEFAULTS,
    );
    assert!(prepared.used_compress);
    assert!(!prepared.used_oversize);

    // Non-compress path never triggers oversize (TDAM parity).
    let plain = prepare_archive_payload(
        &[],
        &[SkillMessage::new("user", "y".repeat(9_999))],
        false,
        &COMPRESS_DEFAULTS,
        &OVERSIZE_DEFAULTS,
    );
    assert!(!plain.used_compress);
    assert!(!plain.used_oversize);
}

#[test]
fn extractor_parses_candidates_from_fenced_json() {
    let runner = FixedRunner(CANDIDATES_JSON);
    let msgs: Vec<ExtractMessage> = transcript()
        .into_iter()
        .map(|m| ExtractMessage::new(m.role, m.content))
        .collect();
    let result = extract_skills_with_llm(&runner, &msgs, &[], &SkillExtractorConfig::default());
    assert!(result.success);
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].name, "k8s-crashloop-triage");
}

#[test]
fn extractor_sentinel_means_empty_success() {
    let runner = FixedRunner("Nothing to save.");
    let msgs = vec![ExtractMessage::new("user", "hello")];
    let result = extract_skills_with_llm(&runner, &msgs, &[], &Default::default());
    assert!(result.success);
    assert!(result.candidates.is_empty());
}

#[test]
fn extractor_failure_is_degraded_not_fatal() {
    let runner = FailingRunner;
    let msgs = vec![ExtractMessage::new("user", "hello")];
    let result = extract_skills_with_llm(&runner, &msgs, &[], &Default::default());
    assert!(!result.success);
    assert!(result.candidates.is_empty());
    assert!(result.error.as_deref().unwrap_or("").contains("boom"));
}

#[test]
fn worker_end_to_end_applies_and_marks_done() {
    let db = db();
    let triggered = trigger_archive(&db, "agent-7", &transcript(), 5_000).expect("trigger");
    let runner = FixedRunner(CANDIDATES_JSON);

    let outcome = vanta_memory::core::skill::conversation_add::run_skill_extract_once(
        &db,
        &runner,
        "agent-7",
        &triggered.task_id,
        &[],
    )
    .expect("run once");
    let SkillWorkerOutcome::Applied {
        counts,
        candidate_count,
    } = outcome
    else {
        panic!("expected Applied, got {outcome:?}");
    };
    assert_eq!(candidate_count, 1);
    let counts = counts.expect("first apply returns counts");
    assert_eq!(counts.created, 1);

    // Skill persisted in the scope namespace.
    let sink = vanta_memory::core::skill::conversation_add::SkillCoreSink::new(&db);
    let stored = sink
        .read_skill("agent-7", "k8s-crashloop-triage")
        .expect("read")
        .expect("stored");
    assert!(stored.content.contains("CrashLoopBackOff"));

    // Task marked done.
    let entry = ArchiveStore::new(&db)
        .read_task("agent-7", &triggered.task_id)
        .expect("read")
        .expect("exists");
    assert_eq!(entry.status, "done");
}

#[test]
fn sink_idempotency_reprocessing_same_task_never_duplicates() {
    let db = db();
    let triggered = trigger_archive(&db, "agent-7", &transcript(), 6_000).expect("trigger");
    let runner = FixedRunner(CANDIDATES_JSON);

    let first = vanta_memory::core::skill::conversation_add::run_skill_extract_once(
        &db,
        &runner,
        "agent-7",
        &triggered.task_id,
        &[],
    )
    .expect("first run");
    assert!(matches!(first, SkillWorkerOutcome::Applied { .. }));

    // Client retry / worker re-consumption of the SAME task:
    let retry = vanta_memory::core::skill::conversation_add::run_skill_extract_once(
        &db,
        &runner,
        "agent-7",
        &triggered.task_id,
        &[],
    )
    .expect("retry run");
    assert_eq!(
        retry,
        SkillWorkerOutcome::AlreadyDone,
        "done task short-circuits"
    );

    // Even bypassing the worker status, the sink cursor makes re-application a
    // no-op (the double idempotency layer).
    let sink = vanta_memory::core::skill::conversation_add::SkillCoreSink::new(&db);
    let candidates = vec![
        vanta_memory::core::skill::skill_extractor::ExtractedSkillCandidate {
            action: "create".into(),
            name: "k8s-crashloop-triage".into(),
            description: "tampered".into(),
            content: "DIFFERENT CONTENT".into(),
        },
    ];
    let reapplied = sink
        .apply_candidates("agent-7", &triggered.task_id, &candidates, 9_999)
        .expect("reapply");
    assert_eq!(reapplied, None, "cursor hit → no-op");
    let stored = sink
        .read_skill("agent-7", "k8s-crashloop-triage")
        .expect("read")
        .expect("stored");
    assert!(
        stored.content.contains("CrashLoopBackOff"),
        "store untouched by retry"
    );
}

#[test]
fn llm_failure_leaves_task_pending_for_retry() {
    let db = db();
    let triggered = trigger_archive(&db, "agent-7", &transcript(), 7_000).expect("trigger");

    let failed = vanta_memory::core::skill::conversation_add::run_skill_extract_once(
        &db,
        &FailingRunner,
        "agent-7",
        &triggered.task_id,
        &[],
    )
    .expect("run");
    let SkillWorkerOutcome::ExtractionFailed { error } = failed else {
        panic!("expected ExtractionFailed, got {failed:?}");
    };
    assert!(error.contains("boom"));

    // Task still pending → a later healthy runner can consume it.
    let recovered = vanta_memory::core::skill::conversation_add::run_skill_extract_once(
        &db,
        &FixedRunner(CANDIDATES_JSON),
        "agent-7",
        &triggered.task_id,
        &[],
    )
    .expect("retry run");
    assert!(matches!(recovered, SkillWorkerOutcome::Applied { .. }));
}

#[test]
fn ghost_task_without_archive_is_dropped() {
    let db = db();
    let store = ArchiveStore::new(&db);
    store
        .register_task(
            &vanta_memory::core::skill::conversation_add::SkillTaskEntry {
                task_id: "ghost-task".into(),
                session_id: "agent-7".into(),
                archive_key: "missing".into(),
                archived_at_ms: 8_000,
                status: "pending".into(),
            },
        )
        .expect("register");

    let outcome = vanta_memory::core::skill::conversation_add::run_skill_extract_once(
        &db,
        &FixedRunner(CANDIDATES_JSON),
        "agent-7",
        "ghost-task",
        &[],
    )
    .expect("run");
    assert_eq!(outcome, SkillWorkerOutcome::GhostDropped);

    let entry = store
        .read_task("agent-7", "ghost-task")
        .expect("read")
        .expect("exists");
    assert_eq!(entry.status, "dropped");
}

#[test]
fn prefix_block_injected_when_existing_skills_present() {
    // The prompt must contain the existing-skill block ahead of the transcript.
    struct CaptureRunner(std::sync::Arc<std::sync::Mutex<Option<String>>>);
    impl LlmRunner for CaptureRunner {
        fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
            *self.0.lock().expect("lock") = Some(params.prompt.clone());
            Ok("Nothing to save.".to_string())
        }
    }
    let captured = std::sync::Arc::new(std::sync::Mutex::new(None));
    let runner = CaptureRunner(captured.clone());
    let existing = vec![SkillSummary {
        name: "existing-skill".into(),
        description: "already covers this".into(),
    }];
    let msgs = vec![ExtractMessage::new("user", "do the thing")];
    let config = SkillExtractorConfig {
        prefix_skills_limit: 5,
        ..Default::default()
    };
    let result = extract_skills_with_llm(&runner, &msgs, &existing, &config);
    assert!(result.success);
    let prompt = captured.lock().expect("lock").clone().expect("captured");
    assert!(prompt.contains("existing-skill"));
    assert!(prompt.contains("<<end-of-transcript>>"));
}
