// ponytail: real-binary integration test — controlled subprocess env keeps the
// fallback probe race-free; documented per-call.
#![cfg(all(feature = "cli", feature = "embed-local"))]
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! DEF-08 visible fallback (CLI surface): `vanta-cli status --json` must
//! surface the degraded-embeddings notice for BOTH documented triggers —
//! ONNX Runtime unusable and model missing — never silently.
//!
//! The probe reads `ORT_DYLIB_PATH` / `VANTADB_LOCAL_MODEL` and the process
//! CWD, so each scenario spawns the real binary with an isolated environment.

use std::path::Path;
use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vanta-cli"))
}

/// Create a fresh DB so `status` takes the initialized branch (where the
/// embedding probe is reported).
fn init_db(dir: &Path) -> String {
    let db = dir.join("db").to_string_lossy().to_string();
    let out = cli()
        .current_dir(dir)
        .args([
            "--db",
            &db,
            "put",
            "--namespace",
            "def08",
            "--key",
            "k",
            "--payload",
            "hi",
        ])
        .output()
        .expect("spawn vanta-cli put");
    assert!(
        out.status.success(),
        "put failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    db
}

/// Run `status --json` in an isolated env: provider=local, plus the given
/// overrides. `current_dir(dir)` keeps the provider's CWD-relative default
/// model candidates from resolving into the repo checkout.
fn status_json(dir: &Path, db: &str, model_dir: &str, ort_path: &str) -> serde_json::Value {
    let out = cli()
        .current_dir(dir)
        .env("VANTADB_EMBEDDING_PROVIDER", "local")
        .env("VANTADB_LOCAL_MODEL", model_dir)
        .env("ORT_DYLIB_PATH", ort_path)
        .args(["--db", db, "status", "--json"])
        .output()
        .expect("spawn vanta-cli status");
    assert!(
        out.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("status --json must parse")
}

/// The shared contract (DISTRIBUTION §7): flag + consequence + remedy.
fn assert_degraded_notice(embedding: &serde_json::Value) {
    assert_eq!(
        embedding["fallback"],
        serde_json::json!(true),
        "embedding: {embedding}"
    );
    let notice = embedding["notice"]
        .as_str()
        .expect("degraded embedding status must carry a notice");
    for needle in [
        "fallback: true",
        "deterministic dummy embeddings",
        "NOT semantic",
        "setup-embeddings.ps1",
        "embeddings/verify.py --check",
    ] {
        assert!(
            notice.contains(needle),
            "notice missing {needle:?}: {notice}"
        );
    }
}

// Trigger (b): model files missing (silent today) → visible notice.
#[test]
fn def08_status_reports_missing_model_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let db = init_db(dir.path());
    let model = dir.path().join("no-such-model");
    let json = status_json(
        dir.path(),
        &db,
        model.to_str().unwrap(),
        "C:/nonexistent-def08/onnxruntime.dll",
    );
    assert_degraded_notice(&json["embedding"]);
    assert_eq!(json["embedding"]["reason"], serde_json::json!("model"));
    assert_eq!(json["embedding"]["model_present"], serde_json::json!(false));
    assert!(
        json["embedding"]["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("model files missing"),
        "detail: {}",
        json["embedding"]["detail"]
    );
}

// Trigger (a): ORT unusable (log-only today) → visible notice.
#[test]
fn def08_status_reports_unusable_ort_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let db = init_db(dir.path());
    // Model files "present" (the probe checks presence, not loadability) so the
    // remaining trigger is the unusable runtime.
    let model = dir.path().join("model");
    std::fs::create_dir_all(model.join("onnx")).unwrap();
    std::fs::write(model.join("onnx/model.onnx"), b"not-a-real-onnx").unwrap();
    std::fs::write(model.join("tokenizer.json"), b"{}").unwrap();
    let json = status_json(
        dir.path(),
        &db,
        model.to_str().unwrap(),
        "C:/nonexistent-def08/onnxruntime.dll",
    );
    assert_degraded_notice(&json["embedding"]);
    assert_eq!(json["embedding"]["reason"], serde_json::json!("dylib"));
    assert_eq!(json["embedding"]["model_present"], serde_json::json!(true));
    assert!(
        json["embedding"]["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("ONNX Runtime unusable"),
        "detail: {}",
        json["embedding"]["detail"]
    );
}
