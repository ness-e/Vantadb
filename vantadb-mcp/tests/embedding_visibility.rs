// ponytail: one-test binary — env is mutated in-process on purpose; documented per-call.
#![cfg(feature = "embed-local")]
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! DEF-08 visible fallback (MCP surface): `capabilities` must report
//! `embedding.fallback = true` + the user-facing notice when local embeddings
//! degrade — an agent reading the status tool is never left in the dark
//! (`SPEC.md` L74: "fallback avisado, nunca silencioso").

use std::sync::Arc;

use serde_json::json;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

fn capabilities_payload() -> serde_json::Value {
    let dir = tempfile::tempdir().expect("tempdir");
    let storage = Arc::new(
        StorageEngine::open(dir.path().to_str().expect("utf8 path")).expect("open storage"),
    );
    let executor = Executor::new(&storage);
    let res = handle_tools_call(
        &Some(json!({ "name": "capabilities", "arguments": {} })),
        &executor,
        &storage,
        &McpConfig::default(),
    )
    .expect("capabilities must not error");
    serde_json::from_str(res["content"][0]["text"].as_str().expect("text content"))
        .expect("capabilities payload must be JSON")
}

#[test]
fn def08_capabilities_reports_degraded_embeddings_notice() {
    // Model absent (temp dir + package CWD: the provider's CWD-relative
    // default candidates do not resolve) and ORT absent → deterministic dummy.
    std::env::set_var("VANTADB_EMBEDDING_PROVIDER", "local");
    std::env::set_var(
        "VANTADB_LOCAL_MODEL",
        std::env::temp_dir()
            .join("vantadb-def08-mcp-no-such-model")
            .to_string_lossy()
            .to_string(),
    );
    std::env::set_var("ORT_DYLIB_PATH", "C:/nonexistent-def08/onnxruntime.dll");

    let caps = capabilities_payload();
    let embedding = &caps["embedding"];
    assert_eq!(caps["persistence"], true, "capabilities: {caps}");
    assert_eq!(
        embedding["fallback"],
        json!(true),
        "embedding status: {embedding}"
    );
    assert_eq!(
        embedding["reason"],
        json!("model"),
        "embedding: {embedding}"
    );
    let notice = embedding["notice"]
        .as_str()
        .expect("degraded embedding status must carry a notice");
    for needle in [
        "fallback: true",
        "deterministic dummy embeddings",
        "setup-embeddings.ps1",
        "embeddings/verify.py --check",
    ] {
        assert!(
            notice.contains(needle),
            "notice missing {needle:?}: {notice}"
        );
    }
}
