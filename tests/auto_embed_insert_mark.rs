// ponytail: real-binary integration test — a controlled subprocess env keeps the
// provider probe race-free (same pattern as tests/embedding_fallback_visibility.rs).
#![cfg(feature = "cli")]
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! FIND-485: an `INSERT` whose auto-embedding degrades must leave a mark the
//! user can consult — not only a `tracing::warn!` in the logs.
//!
//! The write half is `INSERT NODE#… { text: … }` with a provider that cannot
//! produce a vector; the read half is `FROM … --json`, which prints every
//! relational field (`src/cli_handlers/data.rs`), so the mark is visible with
//! no CLI change at all.

use std::path::Path;
use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vanta-cli"))
}

/// Run one IQL statement with an isolated, guaranteed-broken embedding provider.
/// `no_model` doubles as the model dir and the ORT dylib path: neither exists,
/// so the provider cannot be constructed in any feature configuration.
fn query(dir: &Path, db: &str, iql: &str, no_model: &Path) -> serde_json::Value {
    let no_model = no_model.to_str().expect("temp dir path is utf-8");
    let out = cli()
        .current_dir(dir)
        .env("VANTADB_EMBEDDING_PROVIDER", "bogus-find485")
        .env("VANTADB_LOCAL_MODEL", no_model)
        .env("ORT_DYLIB_PATH", no_model)
        .env("VANTADB_STRICT_EMBEDDINGS", "1")
        .args(["--db", db, "query", iql, "--json"])
        .output()
        .expect("spawn vanta-cli query");
    assert!(
        out.status.success(),
        "query {iql:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("query --json must parse")
}

#[test]
fn auto_embed_insert_mark_visible_via_cli() {
    // Arrange: fresh DB, provider that cannot embed.
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db").to_string_lossy().to_string();
    let no_model = dir.path().join("no-such-model");

    // Act 1: INSERT with `text` and no `VECTOR` → auto-embedding is attempted
    // and fails. The INSERT itself must still succeed (graceful degradation).
    let write = query(
        dir.path(),
        &db,
        r#"INSERT NODE#1 TYPE Note { text: "hola mundo" }"#,
        &no_model,
    );
    assert_eq!(write["mode"], serde_json::json!("write"), "{write}");
    assert_eq!(write["node_id"], serde_json::json!("1"), "{write}");

    // Act 2: read it back through the CLI's JSON surface.
    let read = query(dir.path(), &db, "FROM Note", &no_model);
    let records = read["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1, "{read}");

    // Assert: the record says why it has no vector. `failed` under
    // `remote-inference` (provider errored), `unavailable` when the feature is
    // not compiled in — both are the same promise: never silent.
    let mark = records[0]["fields"]["__vanta_embed_status"]
        .as_str()
        .unwrap_or_else(|| panic!("degraded INSERT must carry a mark: {read}"));
    // Feature-agnostic smoke: which degradation is exercised depends on the
    // build (`failed` with `remote-inference`, `unavailable` without it). The
    // exact value is pinned by the unit test
    // `executor::tests::auto_embed_insert_marks_node_when_provider_fails`.
    assert!(
        ["failed", "synthetic", "unavailable"].contains(&mark),
        "unexpected embed status {mark:?}: {read}"
    );
}
