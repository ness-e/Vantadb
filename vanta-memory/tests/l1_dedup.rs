//! D19 dedicated L1 dedup tests (MEM-11 contract).
//!
//! Two-phase dedup over a real VantaDB in-memory instance (no mocks for the
//! store): recall → LLM judgment → write_memory applying store/update/merge/
//! skip. LLM behavior is scripted via a fake `LlmRunner`; persistence is real.

use std::sync::{Arc, Mutex};

use tempfile::tempdir;
use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, LlmError, LlmRunParams, LlmRunner, MemoryRecord,
    MemoryType,
};
use vanta_memory::core::prompts::{format_batch_conflict_prompt, CandidateMatch};
use vanta_memory::core::record::{
    apply_dedup_batch, batch_dedup, generate_memory_id, l1_namespace, parse_batch_result,
    prepare_pending, read_session_records, run_l1_dedup, write_memory, EmbedFn, L1DedupConfig,
};
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};
use vantadb::storage::BackendKind;

fn open_db() -> (VantaEmbedded, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).expect("open embedded");
    (db, dir)
}

fn memory(content: &str) -> ExtractedMemory {
    ExtractedMemory {
        content: content.into(),
        memory_type: MemoryType::Episodic,
        priority: 70,
        source_message_ids: vec![],
        scene_name: "s".into(),
        metadata: serde_json::Value::Null,
    }
}

fn record(id: &str, content: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "s".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec!["2026-08-20T10:00:00.000Z".into()],
        created_at: "2026-08-20T10:00:00.000Z".into(),
        updated_at: "2026-08-20T10:00:00.000Z".into(),
        version: 1,
        session_key: "sk".into(),
        session_id: "".into(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
    }
}

fn decision(record_id: &str, action: DedupAction) -> DedupDecision {
    DedupDecision {
        record_id: record_id.into(),
        action,
        target_ids: vec![],
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    }
}

fn merge_decision(record_id: &str, targets: &[&str], merged_content: &str) -> DedupDecision {
    DedupDecision {
        record_id: record_id.into(),
        action: DedupAction::Merge,
        target_ids: targets.iter().map(|s| s.to_string()).collect(),
        merged_content: Some(merged_content.into()),
        merged_type: Some(MemoryType::WorkMethod),
        merged_priority: Some(90),
        merged_timestamps: Some(vec!["2026-08-20T10:00:00.000Z".into()]),
    }
}

/// Scripted LLM runner: returns queued responses (or fails on `Err`).
struct ScriptedLlm {
    responses: Mutex<Vec<Result<String, LlmError>>>,
    calls: Mutex<Vec<String>>,
    _marker: Arc<()>,
}

impl ScriptedLlm {
    fn new(responses: Vec<Result<String, LlmError>>) -> Self {
        Self {
            responses: Mutex::new(responses),
            calls: Mutex::new(Vec::new()),
            _marker: Arc::new(()),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl LlmRunner for ScriptedLlm {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        self.calls.lock().unwrap().push(params.prompt.clone());
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            return Ok("[]".to_string());
        }
        responses.remove(0)
    }
}

// (a) e2e: no candidates → all memories stored with generated ids.
#[test]
fn run_l1_dedup_empty_store_stores_all() {
    let (db, _dir) = open_db();
    let runner = ScriptedLlm::new(vec![]);
    let memories = vec![
        memory("user prefers dark mode"),
        memory("team uses postgres"),
    ];

    let written = run_l1_dedup(
        &db,
        &runner,
        "session-a",
        "si",
        &memories,
        &L1DedupConfig::default(),
    )
    .expect("pipeline");

    assert_eq!(written.len(), 2);
    assert_eq!(written[0].version, 1);
    assert!(written[0].id.starts_with("m_"));
    // No LLM call happened (no candidates → store-all, Principio 4).
    assert!(runner.calls().is_empty());

    let ns = l1_namespace("session-a");
    let records = read_session_records(&db, "session-a").expect("read back");
    assert_eq!(records.len(), 2);
    assert_eq!(ns, "l1/session-a");
}

// (b) e2e: runner failure → store-all, data never lost.
#[test]
fn run_l1_dedup_llm_failure_stores_all() {
    let (db, _dir) = open_db();
    let runner = ScriptedLlm::new(vec![Err(LlmError::Other("down".into()))]);
    let memories = vec![memory("user prefers dark mode")];

    // Prime the store so recall has candidates and the LLM path is reached.
    let existing = vec![record("m_old", "user prefers dark mode")];
    put_records(&db, "session-b", &existing);

    let written = run_l1_dedup(
        &db,
        &runner,
        "session-b",
        "si",
        &memories,
        &L1DedupConfig::default(),
    )
    .expect("pipeline");

    assert_eq!(written.len(), 1);
    assert_eq!(written[0].version, 1); // stored fresh, not merged
    assert_eq!(read_session_records(&db, "session-b").unwrap().len(), 2); // old + new
}

// (c) e2e: merge removes targets and bumps version.
#[test]
fn write_memory_merge_deletes_targets_and_bumps_version() {
    let (db, _dir) = open_db();
    let existing = vec![
        record("m_old_1", "user prefers dark mode"),
        record("m_old_2", "vim is great"),
    ];
    put_records(&db, "session-c", &existing);

    let decision = merge_decision(
        "m_new",
        &["m_old_1", "m_old_2"],
        "user prefers dark mode + vim",
    );
    let written = write_memory(
        &db,
        "session-c",
        "si",
        &memory("dark vim"),
        &decision,
        1_700_000_000_000,
        0,
        None,
    )
    .expect("write");

    let records = read_session_records(&db, "session-c").expect("read back");
    assert_eq!(records.len(), 1); // both targets gone
    let merged = written.expect("merged record");
    assert_eq!(merged.id, "m_new");
    assert_eq!(merged.version, 2); // max(targets=1) + 1
    assert_eq!(merged.content, "user prefers dark mode + vim");
    assert_eq!(merged.memory_type, MemoryType::WorkMethod);
    assert_eq!(merged.priority, 90);
    // Decision timestamps win; union is sorted+deduped.
    assert_eq!(merged.timestamps, vec!["2026-08-20T10:00:00.000Z"]);
}

// (d) e2e: update with no merged_* falls back to new memory fields.
#[test]
fn write_memory_update_falls_back_to_memory_fields() {
    let (db, _dir) = open_db();
    put_records(&db, "session-d", &[record("m_old", "old content")]);

    let decision = DedupDecision {
        record_id: "m_new".into(),
        action: DedupAction::Update,
        target_ids: vec!["m_old".into()],
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    };
    let written = write_memory(
        &db,
        "session-d",
        "si",
        &memory("fresh content"),
        &decision,
        5,
        0,
        None,
    )
    .expect("write");

    let records = read_session_records(&db, "session-d").expect("read back");
    assert_eq!(records.len(), 1);
    let updated = written.expect("record");
    assert_eq!(updated.content, "fresh content");
    assert_eq!(updated.memory_type, MemoryType::Episodic);
    assert_eq!(updated.version, 2);
    // Fallback timestamps = target timestamps + now (deduped).
    assert!(updated
        .timestamps
        .contains(&"2026-08-20T10:00:00.000Z".to_string()));
    assert!(updated
        .timestamps
        .contains(&"1970-01-01T00:00:00.005Z".to_string()));
}

// (e) skip is a no-op.
#[test]
fn write_memory_skip_returns_none() {
    let (db, _dir) = open_db();
    let result = write_memory(
        &db,
        "session-e",
        "si",
        &memory("dup"),
        &decision("m_x", DedupAction::Skip),
        5,
        0,
        None,
    )
    .expect("write");
    assert!(result.is_none());
    assert!(read_session_records(&db, "session-e").unwrap().is_empty());
}

// (f) apply_dedup_batch maps decisions 1:1 to memories (store + skip).
#[test]
fn apply_dedup_batch_honors_skip() {
    let (db, _dir) = open_db();
    let memories = vec![memory("keep me"), memory("drop me")];
    let decisions = vec![
        decision("m_a", DedupAction::Store),
        decision("m_b", DedupAction::Skip),
    ];

    let written =
        apply_dedup_batch(&db, "session-f", "si", &memories, &decisions, 9, None).expect("batch");
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].content, "keep me");
    assert_eq!(read_session_records(&db, "session-f").unwrap().len(), 1);
}

// (g) batch_dedup prompt is one call with unified pool (LLM path exercised).
#[test]
fn batch_dedup_makes_single_call_with_pool() {
    let (db, _dir) = open_db();
    put_records(
        &db,
        "session-g",
        &[record("m_old", "user prefers dark mode")],
    );
    let runner = ScriptedLlm::new(vec![Ok(
        r#"[{"record_id": "m_1700000000000_0", "action": "merge", "target_ids": ["m_old"]}]"#
            .to_string(),
    )]);
    let pending = prepare_pending(&[memory("user prefers dark mode")], 1_700_000_000_000);
    let existing = read_session_records(&db, "session-g").unwrap();

    let decisions = batch_dedup(&runner, &pending, &existing, &L1DedupConfig::default());
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].action, DedupAction::Merge);
    assert_eq!(decisions[0].target_ids, vec!["m_old".to_string()]);

    // Exactly one LLM call with the unified pool.
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains("CANDIDATE POOL"));
    assert!(calls[0].contains("\"record_id\": \"m_old\""));
    assert!(calls[0].contains("NEW MEMORIES TO JUDGE"));
}

// (h) full pipeline e2e: recall → merge decision → write; version bump applied.
#[test]
fn run_l1_dedup_full_merge_flow() {
    let (db, _dir) = open_db();
    put_records(
        &db,
        "session-h",
        &[record("m_old", "user prefers dark mode")],
    );

    // Dynamic runner: echoes a merge decision using the REAL record_id that
    // the pipeline assigned (run_l1_dedup stamps ids with the wall clock).
    struct MergeEcho;
    impl LlmRunner for MergeEcho {
        fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
            // The pool appears before NEW MEMORIES; take the id from the
            // judgment block only (the first record_id after the marker).
            let new_block = params
                .prompt
                .split("NEW MEMORIES TO JUDGE")
                .nth(1)
                .unwrap_or(&params.prompt);
            let record_id = new_block
                .split("\"record_id\": \"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .unwrap_or("m_0")
                .to_string();
            Ok(format!(
                r#"[{{"record_id": "{record_id}", "action": "merge", "target_ids": ["m_old"], "merged_content": "user prefers dark mode and vim", "merged_type": "persona", "merged_priority": 85}}]"#
            ))
        }
    }

    let written = run_l1_dedup(
        &db,
        &MergeEcho,
        "session-h",
        "si",
        &[memory("user prefers dark mode and vim")],
        &L1DedupConfig::default(),
    )
    .expect("pipeline");

    assert_eq!(written.len(), 1);
    assert_eq!(written[0].content, "user prefers dark mode and vim");
    assert_eq!(written[0].version, 2);
    let records = read_session_records(&db, "session-h").unwrap();
    assert_eq!(records.len(), 1); // target removed, merged stored
}

fn put_records(db: &VantaEmbedded, session: &str, records: &[MemoryRecord]) {
    let ns = l1_namespace(session);
    for r in records {
        db.put(VantaMemoryInput {
            namespace: ns.clone(),
            key: r.id.clone(),
            payload: serde_json::to_string(r).expect("serialize"),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .expect("put");
    }
}

// Sanity: prompt helper is reachable from the public surface (used above via
// format_batch_conflict_prompt in tests of l1_dedup module; keep a public
// smoke here for D19 contract coverage).
#[test]
fn conflict_prompt_is_public_surface() {
    let matches = vec![CandidateMatch {
        record_id: "m_0".into(),
        memory: memory("dark mode"),
        candidates: vec![record("m_old", "dark mode")],
    }];
    let prompt = format_batch_conflict_prompt(&matches);
    assert!(prompt.contains("CANDIDATE POOL"));
    assert!(prompt.contains("NEW MEMORIES TO JUDGE"));
}

// Sanity: generated ids are stable across runs for the same batch position.
#[test]
fn generate_memory_id_is_stable() {
    assert_eq!(generate_memory_id(12345, 7), "m_12345_7");
}

// Sanity: tolerant parse is public (D19 contract — deuda MEM-10 resuelta).
#[test]
fn parse_batch_result_is_public_and_tolerant() {
    let matches = vec![CandidateMatch {
        record_id: "m_0".into(),
        memory: memory("a"),
        candidates: vec![],
    }];
    let decisions = parse_batch_result("garbage not json", &matches);
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].action, DedupAction::Store);
}

// ── MEM-46: embeddings para records L1 (best-effort, P4) ────────────────

const FAKE_DIM: usize = 8;
const FAKE_VALUE: f32 = 0.25;

/// Deterministic fake provider: fixed-dimension vector, no network.
struct FixedEmbedding;

impl vantadb::llm::EmbeddingProvider for FixedEmbedding {
    fn embed(&self, _text: &str) -> Result<Vec<f32>, vantadb::error::VantaError> {
        Ok(vec![FAKE_VALUE; FAKE_DIM])
    }
}

/// Provider that always fails (simulates provider down).
struct FailingEmbedding;

impl vantadb::llm::EmbeddingProvider for FailingEmbedding {
    fn embed(&self, _text: &str) -> Result<Vec<f32>, vantadb::error::VantaError> {
        Err(vantadb::error::VantaError::backend_error("provider down"))
    }
}

fn hook<P: vantadb::llm::EmbeddingProvider + 'static>(p: P) -> EmbedFn {
    Arc::new(move |text: &str| p.embed(text).ok())
}

/// Read a stored record straight from the SDK to inspect its vector.
fn get_raw(
    db: &VantaEmbedded,
    session_key: &str,
    id: &str,
) -> Option<vantadb::sdk::VantaMemoryRecord> {
    db.get(&l1_namespace(session_key), id).expect("get")
}

/// The SDK represents "no usable vector" as `Some([])` on read-back
/// (`usable_vector` filters empty/zero vectors before indexing), so absence
/// is asserted as None-or-empty.
fn assert_no_usable_vector(v: &Option<Vec<f32>>) {
    assert!(
        v.as_ref().is_none_or(|vec| vec.is_empty()),
        "expected no vector, got {v:?}"
    );
}

// (a) embedding enabled → record persisted WITH the vector, consultable.
#[test]
fn write_memory_with_hook_stores_vector() {
    let (db, _dir) = open_db();
    let h = hook(FixedEmbedding);
    let written = write_memory(
        &db,
        "session-emb",
        "si",
        &memory("user prefers dark mode"),
        &decision("m-emb-1", DedupAction::Store),
        5,
        0,
        Some(&h),
    )
    .expect("write")
    .expect("stored");

    let raw = get_raw(&db, "session-emb", &written.id).expect("raw record");
    assert_eq!(raw.vector, Some(vec![FAKE_VALUE; FAKE_DIM]));
}

// (b) embedding failure → record still written WITHOUT vector, no panic.
#[test]
fn embedding_failure_stores_without_vector() {
    let (db, _dir) = open_db();
    let h = hook(FailingEmbedding);
    let written = write_memory(
        &db,
        "session-emb-fail",
        "si",
        &memory("user prefers light mode"),
        &decision("m-emb-2", DedupAction::Store),
        5,
        0,
        Some(&h),
    )
    .expect("write must not fail")
    .expect("stored");

    assert_eq!(written.content, "user prefers light mode");
    let raw = get_raw(&db, "session-emb-fail", &written.id).expect("raw record");
    assert_no_usable_vector(&raw.vector);
}

// (c) disabled (default None) → identical behavior: no vector.
#[test]
fn embedding_disabled_keeps_records_vector_free() {
    let (db, _dir) = open_db();
    let written = write_memory(
        &db,
        "session-emb-off",
        "si",
        &memory("team uses postgres"),
        &decision("m-emb-3", DedupAction::Store),
        5,
        0,
        None,
    )
    .expect("write")
    .expect("stored");

    let records = read_session_records(&db, "session-emb-off").expect("read back");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].content, written.content);
    let raw = get_raw(&db, "session-emb-off", &written.id).expect("raw record");
    assert_no_usable_vector(&raw.vector);
}

// (d) dimension consistency: every record from one hook shares the dimension.
#[test]
fn embedding_dimension_is_consistent_across_records() {
    let (db, _dir) = open_db();
    let h = hook(FixedEmbedding);
    let memories = vec![memory("alpha fact"), memory("beta fact")];
    let decisions = vec![
        decision("m-emb-a", DedupAction::Store),
        decision("m-emb-b", DedupAction::Store),
    ];
    let written = apply_dedup_batch(
        &db,
        "session-emb-dim",
        "si",
        &memories,
        &decisions,
        7,
        Some(&h),
    )
    .expect("batch");
    assert_eq!(written.len(), 2);
    for w in &written {
        let raw = get_raw(&db, "session-emb-dim", &w.id).expect("raw record");
        let v = raw.vector.expect("vector present");
        assert_eq!(v.len(), FAKE_DIM);
        assert!(v.iter().all(|&x| x == FAKE_VALUE));
    }
}

// e2e: run_l1_dedup threads config.embed → pipeline writes carry vectors.
#[test]
fn run_l1_dedup_with_embed_config_stores_vectors() {
    let (db, _dir) = open_db();
    let runner = ScriptedLlm::new(vec![]);
    let memories = vec![memory("user prefers dark mode")];
    let config = L1DedupConfig {
        embed: Some(hook(FixedEmbedding)),
        ..L1DedupConfig::default()
    };

    let written =
        run_l1_dedup(&db, &runner, "session-emb-e2e", "si", &memories, &config).expect("pipeline");
    assert_eq!(written.len(), 1);

    // Ids are generated (`m_{now}_{idx}`): alphanumerics + underscores pass
    // through key sanitization unchanged.
    let ns = l1_namespace("session-emb-e2e");
    let listed = db.list(&ns, Default::default()).expect("list");
    assert_eq!(listed.records.len(), 1);
    assert!(listed.records[0].vector.is_some());
}
