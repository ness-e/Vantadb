// ponytail: one-test binary — the L1 queue and the ingest status are
// process-wide; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MCP-44 (MCP surface): `thread_send` feeds the memory pipeline instead of
//! only writing storage, and a runner-less pipeline with queued turns is
//! reported as broken (MEM-72) instead of idle.
//!
//! External half of the contract, driven through the public tool API only. The
//! full L0→pass→`memory_recall` round-trip lives next to the queue
//! (`src/threads.rs`, `memory_feed_tests`): running the pass needs the
//! process-local queue handle, which is crate-private on purpose.

use serde_json::{json, Value};
use std::sync::Arc;
use vanta_memory::ingest::observability::{self, RunnerState};
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{handle_tools_call, McpConfig, McpProfile};

fn full_config() -> McpConfig {
    McpConfig {
        profile: McpProfile::Full,
        ..Default::default()
    }
}

fn call(name: &str, args: Value, storage: &Arc<StorageEngine>) -> Value {
    let executor = Executor::new(storage);
    let res = handle_tools_call(
        &Some(json!({ "name": name, "arguments": args })),
        &executor,
        storage,
        &full_config(),
    )
    .expect("tool call must not error");
    serde_json::from_str(res["content"][0]["text"].as_str().expect("text content"))
        .expect("JSON payload")
}

#[test]
fn thread_send_queues_ingest_once_per_turn_and_degraded_pipeline_reads_broken() {
    // The host knows there is no ingest runner (MEM-72's single mapping).
    observability::note_runner(RunnerState::None);

    let dir = tempfile::tempdir().expect("tempdir");
    let storage = Arc::new(
        StorageEngine::open(dir.path().to_str().expect("utf8 path")).expect("open storage"),
    );

    let thread_id = call("thread_create", json!({ "title": "mcp chat" }), &storage)["thread_id"]
        .as_str()
        .expect("thread id")
        .to_string();

    // One turn → one queued L1 task (not two: double enqueue duplicates memory).
    let sent = call(
        "thread_send",
        json!({
            "thread_id": thread_id,
            "role": "user",
            "content": "I prefer the fjall storage backend for this project",
        }),
        &storage,
    );
    assert_eq!(sent["ok"], json!(true));
    assert_eq!(
        sent["memory"],
        json!({ "captured": true, "queued": true }),
        "thread_send must feed the pipeline: {sent}"
    );

    let caps = call("capabilities", json!({}), &storage);
    assert_eq!(caps["ingest"]["runner"], json!("none"), "caps: {caps}");
    assert_eq!(
        caps["ingest"]["queue"]["depth"],
        json!(1),
        "one task: {caps}"
    );
    // `IngestStatus::is_broken()`: degraded AND queued work — the contract's
    // honest half, read from the same surface `capabilities` embeds.
    assert!(
        observability::status().is_broken(),
        "a queued turn with runner `none` is a broken pipeline, not an idle one: {caps}"
    );

    // Second turn → depth 2 (one per turn, never two per turn).
    call(
        "thread_send",
        json!({
            "thread_id": thread_id,
            "role": "assistant",
            "content": "Understood - fjall keeps the storage local and fast",
        }),
        &storage,
    );
    let caps = call("capabilities", json!({}), &storage);
    assert_eq!(
        caps["ingest"]["queue"]["depth"],
        json!(2),
        "one task per turn: {caps}"
    );
    assert_eq!(
        observability::degraded_warnings(),
        1,
        "one warning per degraded episode, not per turn: {caps}"
    );
    assert!(
        caps["ingest"]["notice"]
            .as_str()
            .unwrap_or_default()
            .contains("2 queued task(s)"),
        "the backlog must be legible to an operator: {caps}"
    );
}
