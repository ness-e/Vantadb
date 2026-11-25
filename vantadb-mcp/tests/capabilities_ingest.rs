// ponytail: one-test binary — the ingest registry is process-wide; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-72 (MCP surface): `capabilities` must expose the memory ingest runner,
//! the last pass and the queue depth — and keep every pre-existing key intact
//! (additive only). Without it, "no work queued" and "runner is `none`, nothing
//! will ever be extracted" look identical to an agent or an operator.

use std::sync::Arc;

use serde_json::json;
use vanta_memory::ingest::observability::{self, QueueSnapshot, RunnerState};
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

    // Degraded runner with a backlog, sampled twice (two passes): the operator
    // must see it, once.
    observability::note_runner(RunnerState::None);
    let backlog = QueueSnapshot {
        depth: 5,
        claimed: 0,
    };
    observability::note_skipped_pass(backlog);
    observability::note_skipped_pass(backlog);

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
    assert_eq!(ingest["queue"]["depth"], json!(5), "ingest: {ingest}");
    assert_eq!(
        ingest["degraded_warnings"],
        json!(1),
        "one warning per degraded episode: {ingest}"
    );
    let notice = ingest["notice"]
        .as_str()
        .expect("a degraded pipeline carries a legible notice");
    assert!(
        notice.contains("5 queued task(s)"),
        "notice must name the backlog: {notice}"
    );
    assert_eq!(
        ingest["last_pass"]["skipped"],
        json!(true),
        "a skipped pass must not look like a real one: {ingest}"
    );

    // Additive contract: the pre-existing payload survives untouched.
    assert_eq!(caps["persistence"], json!(true));
    assert!(caps["embedding"].is_object(), "embedding block kept");
    assert!(caps["vector_search"].is_boolean());
}
