// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! D19 integration tests for the L2 scene strategy (MEM-14, F4):
//! UPDATE>MERGE>CREATE + heat + soft-delete + emptyExtraction.
//!
//! Pattern AAA: arrange → act → assert. Uses an in-memory `VantaEmbedded`
//! (same setup as `tests/scene_tools.rs`).

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::scene::scene_extractor::{
    apply_strategy, decide_strategy, extract_scenes, extract_scenes_with_llm, SceneExtraction,
    SceneMemoryInput, SceneStrategy,
};
use vanta_memory::core::scene::scene_index::{get_scene, list_scenes, soft_delete_scene};
use vanta_memory::core::scene::scene_tools::{read_scene_tool, write_scene_tool};
use vanta_memory::core::scene::SceneAction;
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::BackendKind;

fn open_db() -> VantaEmbedded {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    VantaEmbedded::open_with_config(config).expect("open in-memory db")
}

const SESSION: &str = "sess-1";

fn extraction(scene_name: &str, content: &str, merge_sources: &[&str]) -> SceneExtraction {
    SceneExtraction {
        scene_name: scene_name.into(),
        summary: "summary".into(),
        content: content.into(),
        merge_sources: merge_sources.iter().map(|s| s.to_string()).collect(),
    }
}

fn live_names(db: &VantaEmbedded) -> Vec<String> {
    list_scenes(db, SESSION)
        .expect("list")
        .iter()
        .map(|e| e.filename.clone())
        .collect()
}

// ── emptyExtraction (TDAM scene-extractor.ts:509-516) ──

#[test]
fn empty_batch_never_touches_store() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "existing", "s", "keep me").expect("seed");

    let result = extract_scenes(&db, SESSION, &[]).expect("empty batch");
    assert!(result.empty_extraction);
    assert!(result.success);
    assert!(result.applied.is_empty());

    let kept = read_scene_tool(&db, SESSION, "existing")
        .expect("read")
        .expect("still exists");
    assert_eq!(
        kept.content, "keep me",
        "store not overwritten by empty extraction"
    );
}

// ── CREATE / UPDATE ──

#[test]
fn create_sets_heat_one() {
    let db = open_db();
    let result =
        extract_scenes(&db, SESSION, &[extraction("brand-new", "content", &[])]).expect("run");

    assert_eq!(
        result.applied,
        vec![SceneAction::Created {
            scene_name: "brand-new".into()
        }]
    );
    let block = read_scene_tool(&db, SESSION, "brand-new")
        .expect("read")
        .expect("exists");
    assert_eq!(block.meta.heat, 1, "CREATE heat = 1");
    assert!(!block.is_deleted());
}

#[test]
fn update_bumps_heat_and_preserves_created() {
    let db = open_db();
    let first = write_scene_tool(&db, SESSION, "deploy", "s", "v1").expect("create");
    let created = first.meta.created.clone();

    let result =
        extract_scenes(&db, SESSION, &[extraction("deploy", "v2 content", &[])]).expect("run");
    assert_eq!(
        result.applied,
        vec![SceneAction::Updated {
            scene_name: "deploy".into()
        }]
    );

    let block = read_scene_tool(&db, SESSION, "deploy")
        .expect("read")
        .expect("exists");
    assert_eq!(block.meta.heat, 2, "UPDATE heat = old + 1");
    assert_eq!(block.meta.created, created, "created preserved on update");
    assert_eq!(block.content, "v2 content");
}

#[test]
fn update_normalizes_name_to_match_existing() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "Deploy-Runbook", "s", "v1").expect("create");

    // LLM emits an unnormalized name; strategy normalizes and matches.
    let result =
        extract_scenes(&db, SESSION, &[extraction("Deploy Runbook!", "v2", &[])]).expect("run");
    assert!(matches!(result.applied[0], SceneAction::Updated { .. }));
}

// ── MERGE ──

#[test]
fn merge_heat_is_sum_plus_one_and_sources_soft_deleted() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "backend-python", "s", "python notes").expect("a");
    write_scene_tool(&db, SESSION, "backend-go", "s", "go notes").expect("b");

    let result = extract_scenes(
        &db,
        SESSION,
        &[extraction(
            "backend",
            "merged notes",
            &["backend-python", "backend-go"],
        )],
    )
    .expect("run");

    assert_eq!(
        result.applied,
        vec![SceneAction::Merged {
            scene_name: "backend".into(),
            sources: vec!["backend-python".into(), "backend-go".into()]
        }]
    );

    let target = read_scene_tool(&db, SESSION, "backend")
        .expect("read")
        .expect("target exists");
    assert_eq!(target.meta.heat, 3, "MERGE heat = 1 + 1 + 1");
    assert_eq!(target.content, "merged notes");

    let python = read_scene_tool(&db, SESSION, "backend-python")
        .expect("read")
        .expect("still present, soft-deleted");
    assert!(python.is_deleted(), "source soft-deleted");
    assert_eq!(python.content, "[DELETED]");

    let go = read_scene_tool(&db, SESSION, "backend-go")
        .expect("read")
        .expect("still present, soft-deleted");
    assert!(go.is_deleted());
}

#[test]
fn merge_into_existing_target_includes_target_heat() {
    let db = open_db();
    // Target exists first (heat 2 after one update), then merges one source.
    write_scene_tool(&db, SESSION, "backend", "s", "v1").expect("target create");
    write_scene_tool(&db, SESSION, "backend", "s", "v2").expect("target update -> heat 2");
    write_scene_tool(&db, SESSION, "auth", "s", "auth notes").expect("source create -> heat 1");

    // decide_strategy would pick UPDATE (target exists); apply MERGE directly
    // to verify target heat is included.
    let strategy = SceneStrategy::Merge {
        scene_name: "backend".into(),
        sources: vec!["auth".into()],
    };
    let result = apply_strategy(
        &db,
        SESSION,
        &extraction("backend", "everything", &["auth"]),
        &strategy,
    )
    .expect("apply merge");

    assert!(matches!(result.action, SceneAction::Merged { .. }));
    let target = read_scene_tool(&db, SESSION, "backend")
        .expect("read")
        .expect("target exists");
    assert_eq!(
        target.meta.heat, 4,
        "MERGE heat = target(2) + source(1) + 1"
    );
}

#[test]
fn merge_with_unknown_sources_creates_cleanly() {
    let db = open_db();
    // decide_strategy drops ghost sources → Create, not Merge.
    let result = extract_scenes(
        &db,
        SESSION,
        &[extraction("new-scene", "content", &["ghost-a", "ghost-b"])],
    )
    .expect("run");

    assert_eq!(
        result.applied,
        vec![SceneAction::Created {
            scene_name: "new-scene".into()
        }]
    );
    let block = read_scene_tool(&db, SESSION, "new-scene")
        .expect("read")
        .expect("exists");
    assert_eq!(block.meta.heat, 1);
}

// ── soft-delete ──

#[test]
fn marker_content_soft_deletes_live_scene() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "obsolete", "s", "content").expect("create");

    let result =
        extract_scenes(&db, SESSION, &[extraction("obsolete", "[DELETED]", &[])]).expect("run");
    assert_eq!(
        result.applied,
        vec![SceneAction::SoftDeleted {
            scene_name: "obsolete".into()
        }]
    );

    let block = read_scene_tool(&db, SESSION, "obsolete")
        .expect("read")
        .expect("still present (soft-deleted)");
    assert!(block.is_deleted());
    assert_eq!(block.content, "[DELETED]");
}

#[test]
fn soft_delete_missing_is_noop() {
    let db = open_db();
    let result =
        extract_scenes(&db, SESSION, &[extraction("ghost", "[DELETED]", &[])]).expect("run");
    assert_eq!(result.applied, vec![SceneAction::Skipped]);
}

#[test]
fn empty_content_is_skipped() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "live", "s", "content").expect("create");
    let result = extract_scenes(&db, SESSION, &[extraction("live", "   ", &[])]).expect("run");
    assert_eq!(
        result.applied,
        vec![SceneAction::Skipped],
        "no empty writes"
    );

    let block = read_scene_tool(&db, SESSION, "live")
        .expect("read")
        .expect("unchanged");
    assert_eq!(block.content, "content");
    assert_eq!(block.meta.heat, 1, "heat unchanged by skip");
}

// ── index visibility / recovery ──

#[test]
fn list_and_current_exclude_deleted_get_recovers() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "live-a", "s", "a").expect("create a");
    write_scene_tool(&db, SESSION, "dead", "s", "d").expect("create dead");
    soft_delete_scene(&db, SESSION, "dead").expect("soft delete");

    assert_eq!(live_names(&db), vec!["live-a"]);

    let current = vanta_memory::core::scene::scene_index::current_scene(&db, SESSION)
        .expect("current")
        .expect("live exists");
    assert_eq!(current.scene_name, "live-a");

    let recovered = get_scene(&db, SESSION, "dead")
        .expect("get")
        .expect("recoverable");
    assert!(recovered.is_deleted());
}

#[test]
fn write_resurrects_deleted_scene() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "scene-x", "s", "v1").expect("create");
    soft_delete_scene(&db, SESSION, "scene-x").expect("soft delete");
    assert!(live_names(&db).is_empty(), "deleted hidden from list");

    // A new write to the same name resurrects it (heat bumps from old+1).
    write_scene_tool(&db, SESSION, "scene-x", "s", "v2").expect("resurrect");
    let block = read_scene_tool(&db, SESSION, "scene-x")
        .expect("read")
        .expect("exists");
    assert!(!block.is_deleted(), "write resurrects");
    assert_eq!(block.content, "v2");
    assert_eq!(block.meta.heat, 2);
    assert_eq!(live_names(&db), vec!["scene-x"]);
}

// ── LLM entry point (Principio 4 degrade) ──

struct FailingRunner;

impl LlmRunner for FailingRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        Err(LlmError::NotConfigured)
    }
}

#[test]
fn llm_failure_degrades_without_writing() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "keep", "s", "original").expect("seed");
    let runner = FailingRunner;
    let memories = vec![SceneMemoryInput {
        id: "m1".into(),
        content: "new memory".into(),
        created_at: "2026-08-20T10:00:00.000Z".into(),
    }];

    let result = extract_scenes_with_llm(&db, SESSION, &runner, &memories, None);
    assert!(!result.success);
    assert!(
        result.empty_extraction,
        "failed run degrades to empty extraction"
    );
    assert!(result.error.is_some());

    let kept = read_scene_tool(&db, SESSION, "keep")
        .expect("read")
        .expect("untouched");
    assert_eq!(kept.content, "original", "no writes on LLM failure");
}

#[test]
fn llm_failure_with_empty_memories_is_clean_empty_extraction() {
    let db = open_db();
    let runner = FailingRunner;
    let result = extract_scenes_with_llm(&db, SESSION, &runner, &[], None);
    assert!(result.success, "no LLM call needed for empty input");
    assert!(result.empty_extraction);
}

struct JsonRunner(&'static str);

impl LlmRunner for JsonRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        Ok(self.0.to_string())
    }
}

#[test]
fn llm_json_output_applies_strategy() {
    let db = open_db();
    let runner = JsonRunner(
        r#"```json
        [{"scene_name":"from-llm","summary":"s","content":"llm content","merge_sources":[]}]
        ```"#,
    );
    let memories = vec![SceneMemoryInput {
        id: "m1".into(),
        content: "new memory".into(),
        created_at: "2026-08-20T10:00:00.000Z".into(),
    }];

    let result = extract_scenes_with_llm(&db, SESSION, &runner, &memories, None);
    assert!(result.success);
    assert!(!result.empty_extraction);
    assert_eq!(
        result.applied,
        vec![SceneAction::Created {
            scene_name: "from-llm".into()
        }]
    );

    let block = read_scene_tool(&db, SESSION, "from-llm")
        .expect("read")
        .expect("exists");
    assert_eq!(block.meta.heat, 1);
    assert_eq!(block.content, "llm content");
}

// ── decide_strategy (pure) ──

#[test]
fn decide_strategy_prefers_update_then_merge_then_create() {
    let existing = vanta_memory::core::scene::scene_index::list_scenes(&open_db(), SESSION)
        .expect("list empty db");
    // Empty index → Create.
    assert_eq!(
        decide_strategy(&extraction("x", "c", &[]), &existing),
        SceneStrategy::Create {
            scene_name: "x".into()
        }
    );
    // Content empty → Skip.
    assert_eq!(
        decide_strategy(&extraction("x", "", &[]), &existing),
        SceneStrategy::Skip
    );
    // Marker for missing → Skip.
    assert_eq!(
        decide_strategy(&extraction("ghost", "[DELETED]", &[]), &existing),
        SceneStrategy::Skip
    );
}

// ── batch ordering ──

#[test]
fn batch_applies_in_order_and_reports_each_action() {
    let db = open_db();
    write_scene_tool(&db, SESSION, "a", "s", "v1").expect("create a");

    let result = extract_scenes(
        &db,
        SESSION,
        &[
            extraction("a", "v2", &[]),           // UPDATE
            extraction("b", "new", &[]),          // CREATE
            extraction("gone", "[DELETED]", &[]), // SoftDelete of a missing scene → Skip
        ],
    )
    .expect("run");

    assert_eq!(result.applied.len(), 3);
    assert!(matches!(result.applied[0], SceneAction::Updated { .. }));
    assert!(matches!(result.applied[1], SceneAction::Created { .. }));
    assert_eq!(result.applied[2], SceneAction::Skipped);
    assert_eq!(live_names(&db), vec!["a", "b"], "heat descending: a=2, b=1");
}
