// ponytail: one-test binary — the ingest registry is process-wide; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-72 + ADR-0065 (MCP surface): `capabilities` must expose the memory
//! ingest runner, the last pass, the **live** queue and the derived health —
//! and keep every pre-existing key intact (additive only).
//!
//! Since ADR-0065 the queue block is a live read, so a test cannot inject a
//! backlog through `observability` (that would be the photograph this ADR
//! deleted). A queued backlog is proven through the public tool API in
//! `thread_memory_pipeline.rs`; this file keeps the two things that are unique
//! to it: the additive contract, and "degraded with an empty queue is idle" —
//! the orthogonality D3 pins.

use std::sync::Arc;

use serde_json::json;
use vanta_memory::ingest::observability::{self, RunnerState};
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

#[test]
fn capabilities_reports_a_degraded_ingest_runner_and_stays_additive() {
    let dir = tempfile::tempdir().expect("tempdir");
    let storage = Arc::new(
        StorageEngine::open(dir.path().to_str().expect("utf8 path")).expect("open storage"),
    );
    let executor = Executor::new(&storage);

    // Degraded runner, nothing queued: a legitimate idle state, not a broken
    // pipeline (ADR-0065 D3 keeps `degraded` orthogonal to `health`).
    observability::note_runner(RunnerState::None);

    let res = handle_tools_call(
        &Some(json!({ "name": "capabilities", "arguments": {} })),
        &executor,
        &storage,
        &McpConfig::default(),
    )
    .expect("capabilities must not error");
    let caps: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().expect("text content"))
            .expect("capabilities payload must be JSON");

    let ingest = &caps["ingest"];
    assert_eq!(ingest["runner"], json!("none"), "capabilities: {caps}");
    assert_eq!(ingest["degraded"], json!(true), "ingest: {ingest}");
    // ADR-0065 D1: `queue` is this process's live read — never null here (the
    // process hosts the queue), never a photograph either.
    assert_eq!(
        ingest["queue"],
        json!({ "depth": 0, "claimed": 0 }),
        "a live read of an empty queue: {ingest}"
    );
    assert_eq!(ingest["health"], json!("idle"), "ingest: {ingest}");
    assert_eq!(ingest["drain_path"], json!("none"), "ingest: {ingest}");
    assert_eq!(
        ingest["last_pass"],
        json!(null),
        "no pass ran in this process: {ingest}"
    );
    assert_eq!(
        ingest["backfill"],
        json!(null),
        "no startup sweep ran in this process (FIND-745): {ingest}"
    );
    assert_eq!(
        ingest["degraded_warnings"],
        json!(0),
        "nothing queued, nothing to warn about: {ingest}"
    );
    let notice = ingest["notice"]
        .as_str()
        .expect("a degraded pipeline carries a legible notice");
    assert!(
        notice.contains("runner is none"),
        "notice must name the missing runner: {notice}"
    );

    // Additive contract: the pre-existing payload survives untouched.
    assert_eq!(caps["persistence"], json!(true));
    assert!(caps["embedding"].is_object(), "embedding block kept");
    assert!(caps["vector_search"].is_boolean());
}
