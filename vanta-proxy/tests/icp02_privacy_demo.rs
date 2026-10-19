// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! ICP-02: privacy chain E2E demo — capture with synthetic PII through the
//! real proxy wire, audit store+indexes+export for cleartext (the versioned
//! `vanta-pii-audit` binary, with a negative control), certified forget on the
//! same store, and the consultable injection audit.
//!
//! `#[ignore]` because the demo needs an ON-DISK store (the certified delete
//! runs later from the `vanta-cli` process) and `vanta-proxy` does not enable
//! `vantadb/fjall` by default. Run it as the demo script does:
//!
//! ```text
//! cargo nextest run -p vanta-proxy --features vantadb/fjall \
//!     --test icp02_privacy_demo --run-ignored ignored-only
//! ```
//!
//! It writes `<ICP02_DEMO_DIR>/handoff.json` (default
//! `<workspace>/target/icp02-demo`) consumed by
//! `scripts/demo-privacy-e2e.ps1` for the cross-crate steps:
//! `vanta-cli delete --attest` + `vanta-cli certificate verify`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vanta_memory::core::record::l1_reader::{l1_namespace, read_session_records};
use vanta_proxy::capture::{list_turns, turn_job, WriteGuard, TURNS_NAMESPACE};
use vanta_proxy::config::{
    AuthConfig, InjectionConfig, MemCommandConfig, ProxyConfig, ServerConfig, UpstreamConfig,
    WritebackConfig,
};
use vanta_proxy::envelope::{Envelope, EnvelopeBlob, EnvelopeConfig, EnvelopeMode};
use vanta_proxy::redact::{RedactConfig, RedactMode, Redactor};
use vanta_proxy::server::{router, AppState};
use vantadb::entity::{EntityStore, EntityWrite};
use vantadb::node::FieldValue;
use vantadb::sdk::{
    export_line_from_record, Embedded, MemoryInput, MemoryListOptions, MemoryMetadata,
};
use vantadb::storage::StorageEngine;

const USER_KEY: &str = "sk-icp02";
const USER_ID: &str = "usr-icp02";
const SESSION: &str = "sess-icp02";
/// Synthetic PII (ver03_write_redact set + `aws_secret`/`token` kinds).
const EMAIL: &str = "jane.doe@example.com";
const AWS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
const AWS_SECRET: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
const TOKEN: &str = "sk-demo0123456789abcdef";
/// Non-PII filler that survives masking (keeps the turn realistic).
const MARKER: &str = "quasar7429";

fn pii_turn_text() -> String {
    format!(
        "contact {EMAIL} and deploy {AWS_KEY} using aws_secret {AWS_SECRET} plus token {TOKEN} {MARKER} now"
    )
}

/// Demo directory: `ICP02_DEMO_DIR` or `<workspace>/target/icp02-demo`.
fn demo_dir() -> PathBuf {
    std::env::var("ICP02_DEMO_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("workspace root")
                .join("target/icp02-demo")
        })
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, router).await.expect("serve") });
    format!("http://{addr}")
}

/// A proxy server that can be shut down deterministically: the file audit runs
/// after the store is closed (on Windows an open engine byte-range-locks its
/// lock files, and "not audited is not clean").
struct ControlledServer {
    url: String,
    stop: Arc<AtomicBool>,
    join: tokio::task::JoinHandle<()>,
}

async fn spawn_controlled(router: Router) -> ControlledServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let join = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                while !stopped.load(Ordering::Relaxed) {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .expect("serve");
    });
    ControlledServer {
        url: format!("http://{addr}"),
        stop,
        join,
    }
}

/// On-disk engine (Fjall) seeded with the demo user the proxy auth resolves.
fn seeded_engine(db_path: &Path) -> Arc<StorageEngine> {
    let config = vantadb::config::Config {
        backend_kind: vantadb::storage::BackendKind::Fjall,
        read_only: false,
        ..vantadb::config::Config::default()
    };
    let engine =
        StorageEngine::open_with_config(db_path.to_str().expect("utf8 path"), Some(config))
            .expect("open on-disk engine");
    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("user_key".into(), FieldValue::String(USER_KEY.to_string()));
    EntityStore::new(&engine)
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: USER_ID,
            fields,
        })
        .expect("seed user");
    Arc::new(engine)
}

/// Seed a persona + one scene for `session_key` (ver04_governance pattern) so
/// the request injects a `<vanta-memory>` block and leaves audit rows.
fn seed_memory(engine: &Arc<StorageEngine>, session_key: &str) {
    use vanta_memory::core::abstractions::PersonaMode;
    use vanta_memory::core::persona::persona_generator::{
        persona_namespace, PersonaRecord, PERSONA_KEY,
    };
    use vanta_memory::core::scene::scene_index::upsert_scene;

    let db = Embedded::from_engine(engine.clone());
    let record = PersonaRecord {
        content: format!("PERSONA-MARKER concise answers. {}", " p ".repeat(200)),
        mode: PersonaMode::First,
        generated_at_ms: 0,
        generated_at: "2026-09-29T00:00:00+00:00".into(),
    };
    db.put(MemoryInput {
        namespace: persona_namespace(session_key),
        key: PERSONA_KEY.into(),
        payload: serde_json::to_string(&record).expect("persona json"),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed persona");
    upsert_scene(
        &db,
        session_key,
        "runbook",
        "deploys",
        "how to deploy the service",
    )
    .expect("seed scene");
}

/// Whether any regular file under `dir` contains `needle` in its raw bytes.
fn dir_bytes_contain(dir: &Path, needle: &[u8]) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    stack.push(entry.path());
                }
            }
        } else if meta.is_file() {
            if let Ok(bytes) = fs::read(&path) {
                if bytes.windows(needle.len()).any(|w| w == needle) {
                    return true;
                }
            }
        }
    }
    false
}

#[tokio::test]
#[ignore = "on-disk demo: needs --features vantadb/fjall; run via scripts/demo-privacy-e2e.ps1 (--run-ignored ignored-only)"]
async fn privacy_chain_capture_audit_forget_injection_end_to_end() {
    // ── 0. demo dir + encryption key (the server resolves it at build time) ─
    let demo = demo_dir();
    let _ = fs::remove_dir_all(&demo);
    fs::create_dir_all(&demo).expect("demo dir");
    let db_path = demo.join("db");
    let export_path = demo.join("export-v2.jsonl");
    let audit_path = demo.join("injection-audit.jsonl");
    let handoff_path = demo.join("handoff.json");

    let key_hex = format!("{:02x}", 0x2au8).repeat(32); // 64 hex chars = 32 bytes
    std::env::set_var("VANTADB_ENCRYPTION_KEY", &key_hex);

    // ── 1. on-disk store + auth user + persona/scene ────────────────────────
    let engine = seeded_engine(&db_path);
    let db = Embedded::from_engine(engine.clone());
    seed_memory(&engine, SESSION);

    let redact_cfg = RedactConfig {
        enabled: true,
        mode: RedactMode::Mask,
        ..RedactConfig::default()
    };
    let envelope_cfg = EnvelopeConfig {
        enabled: true,
        key_version: 1,
    };
    let redactor = Redactor::new(&redact_cfg).expect("redactor");

    // Mock upstream: one JSON response is enough for the forward to complete.
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(|| async { Json(json!({ "id": "chatcmpl-icp02" })) }),
    );
    let upstream_url = spawn(upstream).await;

    let cfg = ProxyConfig {
        report: Default::default(),
        cost: Default::default(),
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream_url,
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: WritebackConfig {
            persist_path: String::new(),
        },
        cache: Default::default(),
        routing: Default::default(),
        redact: redact_cfg.clone(),
        context: Default::default(),
        guardrails: Default::default(),
        translate: Default::default(),
        injection: InjectionConfig {
            max_tokens: 10_000,
            namespace_allow_prefixes: Vec::new(),
            tainted_namespaces: Vec::new(),
            include_tainted: false,
            audit_log_path: audit_path.to_string_lossy().to_string(),
        },
        envelope: envelope_cfg.clone(),
    };
    let state = AppState::from_engine(cfg, engine.clone()).expect("proxy state");
    let writeback = state.writeback.clone();
    let server = spawn_controlled(router(state)).await;
    let proxy_url = server.url.clone();

    // ── 2. capture with synthetic PII through the real proxy wire ──────────
    let text = pii_turn_text();
    let resp = reqwest::Client::new()
        .post(format!("{proxy_url}/agent/space/v1/chat/completions"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", SESSION)
        .json(&json!({ "model": "m", "messages": [{ "role": "user", "content": text }] }))
        .send()
        .await
        .expect("post chat");
    assert_eq!(resp.status(), 200);

    writeback.flush(Duration::from_secs(10)).await;
    let mut l1_records = Vec::new();
    for _ in 0..100 {
        l1_records = read_session_records(&db, SESSION).expect("read l1");
        if l1_records.iter().any(|r| r.content.contains(MARKER)) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        l1_records.iter().any(|r| r.content.contains(MARKER)),
        "captured turn must land in l1/{SESSION}"
    );

    // ── 3. redaction-on-write persisted (store + l1) ────────────────────────
    let turns = list_turns(&db);
    let turn = turns
        .iter()
        .find(|r| r.payload.contains(MARKER))
        .expect("pii turn captured in proxy-turns");
    let payload: Value = serde_json::from_str(&turn.payload).expect("payload json");
    let persisted = payload["text"].as_str().expect("text field");
    for placeholder in [
        "[REDACTED_EMAIL]",
        "[REDACTED_AWS_KEY]",
        "[REDACTED_SECRET]",
        "[REDACTED_TOKEN]",
    ] {
        assert!(
            persisted.contains(placeholder),
            "missing {placeholder}: {persisted}"
        );
    }
    for clear in [EMAIL, AWS_KEY, AWS_SECRET, TOKEN] {
        assert!(!persisted.contains(clear), "clear value persisted: {clear}");
    }
    let kinds = payload["redacted"].as_array().expect("kinds array");
    for kind in ["email", "aws_key", "aws_secret", "token"] {
        assert!(
            kinds.iter().any(|k| k.as_str() == Some(kind)),
            "kind {kind} missing: {kinds:?}"
        );
    }

    // Envelope (VER-03): the original survives — and only — inside the AEAD blob.
    let active = Envelope::from_config(&envelope_cfg);
    assert_eq!(
        active.mode(),
        EnvelopeMode::Active,
        "a valid 32-byte key must activate the envelope"
    );
    let blob: EnvelopeBlob = serde_json::from_value(payload["original_envelope"].clone())
        .expect("original_envelope parses");
    assert_eq!(blob.namespace, TURNS_NAMESPACE);
    assert_eq!(active.open(&blob).expect("open with its key"), text);

    let l1 = l1_records
        .iter()
        .find(|r| r.content.contains(MARKER))
        .expect("l1 record");
    assert!(l1.content.contains("[REDACTED_EMAIL]"));
    for clear in [EMAIL, AWS_KEY, AWS_SECRET, TOKEN] {
        assert!(!l1.content.contains(clear), "l1 leaked {clear}");
    }
    assert_eq!(l1.id, turn.key, "dual-write keys must match");

    // ── 4. injection audit consultable (op=injection), metadata only ───────
    let audit_raw = fs::read_to_string(&audit_path).expect("injection audit exists");
    let rows: Vec<Value> = audit_raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("jsonl line"))
        .collect();
    let injected: Vec<&Value> = rows.iter().filter(|r| r["op"] == "injection").collect();
    assert!(
        !injected.is_empty(),
        "op=injection rows must be consultable: {rows:?}"
    );
    let persona_row = injected
        .iter()
        .find(|r| r["namespace"] == format!("persona/{SESSION}"))
        .expect("persona injection audited");
    assert_eq!(persona_row["outcome"], "ok");
    assert!(
        persona_row["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("surface=proxy"),
        "reason: {persona_row:?}"
    );
    assert!(
        !audit_raw.contains("PERSONA-MARKER"),
        "audit leaked payload content"
    );

    // ── 5. Disarmed degradation (no key → explicit, never cleartext) ───────
    let disarmed = Envelope::with_master(&envelope_cfg, None);
    assert_eq!(disarmed.mode(), EnvelopeMode::Disarmed);
    assert!(
        disarmed.disarm_reason().is_some(),
        "degradation must be explicit (mode + reason)"
    );
    let guard = WriteGuard {
        redactor: &redactor,
        envelope: &disarmed,
    };
    turn_job(
        db.clone(),
        "sess-icp02-disarmed",
        "openai",
        "space",
        "m",
        &text,
        &guard,
    )()
    .await
    .expect("disarmed turn job");
    let disarmed_turn = list_turns(&db)
        .into_iter()
        .find(|r| r.payload.contains("sess-icp02-disarmed"))
        .expect("disarmed turn persisted");
    let dpayload: Value = serde_json::from_str(&disarmed_turn.payload).expect("json");
    assert!(
        dpayload.get("original_envelope").is_none(),
        "disarmed must not persist an original"
    );
    let dtext = dpayload["text"].as_str().expect("text");
    for clear in [EMAIL, AWS_KEY, AWS_SECRET, TOKEN] {
        assert!(!dtext.contains(clear), "disarmed leaked {clear}");
    }

    // ── 6. PII audit: full-store + export scan, 0 cleartext ────────────────
    let mut scanned = 0usize;
    let mut export_lines: Vec<String> = Vec::new();
    for ns in db.list_namespaces().expect("list namespaces") {
        let page = db
            .list(
                &ns,
                MemoryListOptions {
                    limit: 1000,
                    ..Default::default()
                },
            )
            .expect("list namespace");
        for record in page.records {
            scanned += 1;
            for clear in [EMAIL, AWS_KEY, AWS_SECRET, TOKEN] {
                assert!(
                    !record.payload.contains(clear),
                    "namespace {ns} leaked {clear} in clear"
                );
            }
            export_lines.push(
                serde_json::to_string(&export_line_from_record(record)).expect("export line"),
            );
        }
    }
    assert!(
        scanned >= 4,
        "store scan must cover the PII turn + disarmed turn + both l1 records (got {scanned})"
    );
    let export_text = export_lines.join("\n");
    for clear in [EMAIL, AWS_KEY, AWS_SECRET, TOKEN] {
        assert!(!export_text.contains(clear), "export v2 leaked {clear}");
    }
    fs::write(&export_path, format!("{export_text}\n")).expect("write export");

    db.flush().expect("flush store to disk");
    assert!(
        dir_bytes_contain(&db_path, b"[REDACTED_EMAIL]"),
        "masked text must be persisted — proves the file audit scans real content"
    );

    // Close the store before the file audit: on Windows an open engine
    // byte-range-locks its lock files, and `unscanned` fails the audit —
    // correctly: not audited is not clean.
    server.stop.store(true, Ordering::Relaxed);
    let _ = server.join.await;
    db.close().expect("close db handle");
    drop(db);
    drop(engine);
    drop(writeback);

    // The versioned audit (the same binary users run) on store + export.
    let bin = env!("CARGO_BIN_EXE_vanta-pii-audit");
    let out = Command::new(bin)
        .arg("--json")
        .arg(&db_path)
        .arg(&export_path)
        .output()
        .expect("run vanta-pii-audit");
    let stdout = String::from_utf8(out.stdout).expect("audit stdout utf8");
    assert!(
        out.status.success(),
        "audit must pass on the demo store; stdout: {stdout}; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: Value = serde_json::from_str(&stdout).expect("audit json");
    assert_eq!(report["ok"], true, "report: {stdout}");
    assert_eq!(
        report["findings"].as_array().map(Vec::len),
        Some(0),
        "report: {stdout}"
    );
    assert!(
        report["audited_files"].as_u64().unwrap_or(0) >= 1,
        "report: {stdout}"
    );
    assert!(
        report["non_utf8_files"].as_u64().unwrap_or(0) >= 1,
        "the real store's non-UTF-8 files must be scanned byte-wise, not skipped \
         (ICP-02 review Critical): {stdout}"
    );

    // Negative control (binary, ICP-02 review): a planted cleartext leak must
    // fail the audit — including inside a non-UTF-8 file, where the old
    // fail-open path reported `ok:true`.
    let neg_dir = demo.join("negative-control");
    fs::create_dir_all(&neg_dir).expect("neg dir");
    let mut leak = format!("leaked {EMAIL} ").into_bytes();
    leak.extend_from_slice(&[0xff, 0x00, 0xfe, 0x80]);
    fs::write(neg_dir.join("leak.bin"), leak).expect("neg file");
    let neg = Command::new(bin)
        .arg("--json")
        .arg(&neg_dir)
        .output()
        .expect("run negative audit");
    assert_eq!(
        neg.status.code(),
        Some(1),
        "audit must fail on a planted leak"
    );
    let neg_report: Value =
        serde_json::from_str(&String::from_utf8_lossy(&neg.stdout)).expect("neg json");
    assert_eq!(neg_report["findings"][0]["kind"], "email");
    fs::remove_dir_all(&neg_dir).expect("cleanup negative control");

    // ── 7. handoff for the cross-crate script (CLI forget) ─────────────────
    let handoff = json!({
        "generated_by": "icp02_privacy_demo",
        "db_dir": db_path.display().to_string(),
        "export": export_path.display().to_string(),
        "injection_audit": audit_path.display().to_string(),
        "records": [
            { "namespace": TURNS_NAMESPACE, "key": turn.key },
            { "namespace": l1_namespace(SESSION), "key": turn.key },
        ],
    });
    fs::write(
        &handoff_path,
        serde_json::to_string_pretty(&handoff).expect("handoff json"),
    )
    .expect("write handoff");
    println!(
        "ICP-02 privacy demo OK — handoff: {} (next: vanta-cli delete --attest + certificate verify)",
        handoff_path.display()
    );
}
