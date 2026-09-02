// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! D19 integration tests for the sandboxed scene tools (MEM-13, F4).
//!
//! Pattern AAA: arrange → act → assert. Uses an in-memory `VantaEmbedded`
//! (same setup as `tests/scene.rs`).

use vanta_memory::core::scene::scene_tools::{
    edit_scene_tool, execute_scene_tool, read_scene_tool, write_scene_tool, SceneToolCall,
    SceneToolError, SceneToolResult,
};
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
const SCENE: &str = "2024-08-01-22-10";
const SUMMARY: &str = "user researched VantaDB pricing";
const CONTENT: &str = "notes about the pricing page";

#[test]
fn read_missing_returns_none() {
    let db = open_db();
    let scene = read_scene_tool(&db, SESSION, "nope").expect("read missing");
    assert!(scene.is_none());
}

#[test]
fn write_creates_heat_one_and_is_retrievable() {
    let db = open_db();
    let block = write_scene_tool(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("write create");

    assert_eq!(block.scene_name, SCENE);
    assert_eq!(block.meta.heat, 1);
    assert_eq!(block.meta.created, block.meta.updated);

    let read = read_scene_tool(&db, SESSION, SCENE)
        .expect("read")
        .expect("exists");
    assert_eq!(read, block);
}

#[test]
fn write_updates_existing_full_replace() {
    let db = open_db();
    let first = write_scene_tool(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create");

    let second = write_scene_tool(&db, SESSION, SCENE, "updated summary", "updated content")
        .expect("update");

    assert_eq!(second.meta.heat, 2, "UPDATE bumps heat to old+1");
    assert_eq!(second.meta.created, first.meta.created, "created preserved");
    assert!(
        second.meta.updated > first.meta.updated,
        "updated refreshed"
    );
    assert_eq!(second.meta.summary, "updated summary");
    assert_eq!(second.content, "updated content");
}

#[test]
fn edit_content_only_preserves_summary() {
    let db = open_db();
    let original = write_scene_tool(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create");

    let edited =
        edit_scene_tool(&db, SESSION, SCENE, None, Some("rewritten notes")).expect("edit content");

    assert_eq!(edited.meta.summary, SUMMARY, "summary preserved");
    assert_eq!(edited.content, "rewritten notes");
    assert_eq!(edited.meta.heat, 2, "edit bumps heat");
    assert_eq!(
        edited.meta.created, original.meta.created,
        "created preserved"
    );
}

#[test]
fn edit_summary_only_preserves_content() {
    let db = open_db();
    write_scene_tool(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create");

    let edited =
        edit_scene_tool(&db, SESSION, SCENE, Some("refined summary"), None).expect("edit summary");

    assert_eq!(edited.meta.summary, "refined summary");
    assert_eq!(edited.content, CONTENT, "content preserved");
}

#[test]
fn edit_missing_returns_not_found() {
    let db = open_db();
    let err = edit_scene_tool(&db, SESSION, "ghost", Some("s"), None).expect_err("edit missing");
    assert!(matches!(err, SceneToolError::NotFound(_)), "got {err:?}");
}

#[test]
fn edit_without_fields_is_invalid() {
    let db = open_db();
    write_scene_tool(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create");
    let err = edit_scene_tool(&db, SESSION, SCENE, None, None).expect_err("edit no fields");
    assert!(matches!(err, SceneToolError::Invalid(_)), "got {err:?}");
}

#[test]
fn empty_scene_name_is_invalid() {
    let db = open_db();
    let err = write_scene_tool(&db, SESSION, "", "s", "c").expect_err("empty name");
    assert!(matches!(err, SceneToolError::Invalid(_)), "got {err:?}");
}

#[test]
fn nul_in_scene_name_is_invalid() {
    let db = open_db();
    let err = read_scene_tool(&db, SESSION, "scene\0name").expect_err("nul name");
    assert!(matches!(err, SceneToolError::Invalid(_)), "got {err:?}");
}

#[test]
fn oversized_content_is_invalid() {
    let db = open_db();
    let huge = "x".repeat(vanta_memory::core::scene::scene_tools::MAX_CONTENT_BYTES + 1);
    let err = write_scene_tool(&db, SESSION, SCENE, "s", &huge).expect_err("oversized content");
    assert!(matches!(err, SceneToolError::Invalid(_)), "got {err:?}");
}

#[test]
fn empty_or_whitespace_content_is_invalid() {
    let db = open_db();
    // Mirrors the TDAM write-tool validation: empty/whitespace-only content
    // cannot be written, so the LLM cannot "delete" via a blank write.
    for bad in ["", "   ", "\n\t"] {
        let err = write_scene_tool(&db, SESSION, SCENE, "s", bad).expect_err("empty content");
        assert!(matches!(err, SceneToolError::Invalid(_)), "got {err:?}");
    }
}

#[test]
fn wire_roundtrip_call_dispatch_and_result() {
    let db = open_db();

    // LLM tool call arrives as JSON → SceneToolCall.
    let call: SceneToolCall = serde_json::from_str(
        r#"{"tool":"write","scene_name":"2024-08-01-22-10","summary":"s","content":"c"}"#,
    )
    .expect("parse write call");
    let result = execute_scene_tool(&db, SESSION, &call).expect("dispatch write");
    let SceneToolResult::Write { scene } = &result else {
        panic!("expected write result, got {result:?}");
    };
    assert_eq!(scene.meta.heat, 1);

    // Result serializes back to the caller.
    let json = serde_json::to_string(&result).expect("serialize result");
    assert!(
        json.contains("\"result\":\"write\""),
        "tagged result: {json}"
    );

    // Edit wire: fields optional, defaults to None.
    let call: SceneToolCall =
        serde_json::from_str(r#"{"tool":"edit","scene_name":"2024-08-01-22-10","content":"c2"}"#)
            .expect("parse edit call");
    let result = execute_scene_tool(&db, SESSION, &call).expect("dispatch edit");
    let SceneToolResult::Edit { scene } = &result else {
        panic!("expected edit result, got {result:?}");
    };
    assert_eq!(scene.meta.summary, "s", "summary preserved through edit");
    assert_eq!(scene.content, "c2");
}

#[test]
fn tools_are_session_confined() {
    let db = open_db();
    write_scene_tool(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create in sess-1");

    // The same scene name in another session is invisible.
    assert!(read_scene_tool(&db, "sess-2", SCENE)
        .expect("read other session")
        .is_none());
}
