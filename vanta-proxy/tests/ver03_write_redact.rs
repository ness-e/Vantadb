// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! VER-03 (RED): redacción-on-write persistida + envelope AEAD por namespace.
//!
//! Contrato (plan Task 39):
//! 1. lo que se persiste en store/índices es la versión redactada (mismos
//!    kinds del `Redactor`; PII sintética → 0 en claro en store+índices+export v2);
//! 2. el original solo sobrevive en un envelope AEAD por namespace
//!    (descifrable únicamente con su key; rotación declarada y testeada);
//! 3. sin key la degradación es explícita (modo configurado; nunca caída
//!    silenciosa a claro).

use vanta_memory::core::record::l1_reader::{l1_namespace, read_session_records};
use vanta_proxy::capture::{list_turns, turn_job, WriteGuard, TURNS_NAMESPACE};
use vanta_proxy::envelope::{Envelope, EnvelopeBlob, EnvelopeConfig, EnvelopeError, EnvelopeMode};
use vanta_proxy::redact::{RedactConfig, RedactMode, Redactor};
use vantadb::crypto::Cipher;
use vantadb::sdk::{export_line_from_record, Embedded, MemoryListOptions};

const EMAIL: &str = "jane.doe@example.com";
const AWS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
/// Non-PII filler that survives masking (keeps the turn realistic).
const MARKER: &str = "quasar7429";
const SESSION: &str = "sess-ver03";

fn memory() -> Embedded {
    let config = vantadb::config::Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Default::default()
    };
    vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
        .map(|engine| Embedded::from_engine(engine.into()))
        .expect("in-memory engine")
}

fn mask_redactor() -> Redactor {
    Redactor::new(&RedactConfig {
        enabled: true,
        mode: RedactMode::Mask,
        ..RedactConfig::default()
    })
    .expect("mask redactor builds")
}

fn test_master() -> Cipher {
    Cipher::new(&[7u8; 32])
}

fn envelope_config(enabled: bool, key_version: u8) -> EnvelopeConfig {
    EnvelopeConfig {
        enabled,
        key_version,
    }
}

fn turn_text() -> String {
    format!("contact {EMAIL} and deploy {AWS_KEY} {MARKER} now")
}

/// Capture one turn through the real write path and return the parsed
/// `proxy-turns` payload.
async fn capture_and_payload(
    db: &Embedded,
    text: &str,
    redactor: &Redactor,
    envelope: &Envelope,
) -> serde_json::Value {
    let guard = WriteGuard { redactor, envelope };
    let job = turn_job(db.clone(), SESSION, "openai", "space", "m", text, &guard);
    job().await.expect("L0 job runs");
    let turns = list_turns(db);
    assert_eq!(turns.len(), 1, "exactly one captured turn");
    serde_json::from_str(&turns[0].payload).expect("proxy-turns payload is JSON")
}

// ── c1: redacción-on-write persistida ────────────────────────────────────────

#[tokio::test]
async fn pii_synthetic_captured_lands_redacted_in_store_indices_and_export() {
    let db = memory();
    let redactor = mask_redactor();
    let envelope = Envelope::with_master(&envelope_config(true, 1), Some(test_master()));
    let text = turn_text();

    let payload = capture_and_payload(&db, &text, &redactor, &envelope).await;

    // store (proxy-turns): masked text + provenance kinds, no values.
    let persisted = payload["text"].as_str().expect("text field");
    assert!(
        persisted.contains("[REDACTED_EMAIL]"),
        "email masked: {persisted}"
    );
    assert!(
        persisted.contains("[REDACTED_AWS_KEY]"),
        "aws key masked: {persisted}"
    );
    assert!(!persisted.contains(EMAIL), "no email in clear");
    assert!(!persisted.contains(AWS_KEY), "no aws key in clear");
    assert_eq!(payload["redacted"], serde_json::json!(["email", "aws_key"]));

    // store (l1): the search-facing record carries the masked text.
    let l1 = read_session_records(&db, SESSION).expect("read l1");
    assert_eq!(l1.len(), 1);
    assert!(l1[0].content.contains("[REDACTED_EMAIL]"));
    assert!(!l1[0].content.contains(EMAIL));
    assert!(!l1[0].content.contains(AWS_KEY));

    // store + índices: full scan of every namespace in the store. Captured
    // turns carry no vectors (no HNSW entry) and this feature set compiles no
    // BM25 text index (`advanced-tokenizer` off) — the search-facing surface
    // IS the L1 record content, so a full-store scan is the strongest scan
    // available. 0 records may carry the synthetic PII in clear.
    let mut scanned = 0usize;
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
            assert!(
                !record.payload.contains(EMAIL),
                "namespace {ns} leaked the email in clear"
            );
            assert!(
                !record.payload.contains(AWS_KEY),
                "namespace {ns} leaked the aws key in clear"
            );
        }
    }
    assert!(
        scanned >= 2,
        "both destinations scanned (proxy-turns + l1), got {scanned}"
    );

    // export v2: interchange lines serialize masked content only.
    let l1_page = db
        .list(
            &l1_namespace(SESSION),
            MemoryListOptions {
                limit: 100,
                ..Default::default()
            },
        )
        .expect("list l1 sdk records");
    assert_eq!(l1_page.records.len(), 1);
    for record in list_turns(&db).into_iter().chain(l1_page.records) {
        let exported =
            serde_json::to_string(&export_line_from_record(record)).expect("export json");
        assert!(!exported.contains(EMAIL), "export v2 must not carry PII");
        assert!(!exported.contains(AWS_KEY), "export v2 must not carry PII");
    }
}

// ── c2: envelope per-namespace (roundtrip + binding) ─────────────────────────

#[tokio::test]
async fn envelope_recovers_original_and_binds_namespace_version_and_ciphertext() {
    let db = memory();
    let redactor = mask_redactor();
    let envelope = Envelope::with_master(&envelope_config(true, 1), Some(test_master()));
    let text = turn_text();

    let payload = capture_and_payload(&db, &text, &redactor, &envelope).await;
    let blob: EnvelopeBlob = serde_json::from_value(payload["original_envelope"].clone())
        .expect("original_envelope parses as EnvelopeBlob");

    assert_eq!(blob.version, 1);
    assert_eq!(blob.key_version, 1);
    assert_eq!(blob.namespace, TURNS_NAMESPACE);
    assert_eq!(
        envelope.open(&blob).expect("open with its key"),
        text,
        "the original (pre-redaction) is recoverable with the key"
    );

    // Retyped namespace does not authenticate (the namespace is part of the key).
    let mut forged_ns = blob.clone();
    forged_ns.namespace = "other-namespace".into();
    assert!(
        matches!(
            envelope.open(&forged_ns),
            Err(EnvelopeError::DecryptionFailed)
        ),
        "a retyped namespace must not authenticate"
    );

    // Retyped key version does not authenticate.
    let mut forged_k = blob.clone();
    forged_k.key_version = 2;
    assert!(envelope.open(&forged_k).is_err());

    // Tampered ciphertext does not authenticate.
    let mut forged_ct = blob.clone();
    let mut chars: Vec<char> = blob.ciphertext.chars().collect();
    let last = chars.len() - 1;
    chars[last] = if chars[last] == '0' { '1' } else { '0' };
    forged_ct.ciphertext = chars.into_iter().collect();
    assert!(
        envelope.open(&forged_ct).is_err(),
        "AEAD tag must reject tampering"
    );

    // A different master cannot decrypt.
    let stranger = Envelope::with_master(&envelope_config(true, 1), Some(Cipher::new(&[9u8; 32])));
    assert!(stranger.open(&blob).is_err());
}

/// F-01 + Recomendación 3: malformed envelopes degrade to `Format` — never a
/// panic (multibyte hex at an even offset, unknown framing version, invalid
/// hex).
#[test]
fn open_rejects_malformed_envelopes_instead_of_panicking() {
    let envelope = Envelope::with_master(&envelope_config(true, 1), Some(test_master()));
    let valid = EnvelopeBlob {
        version: 1,
        key_version: 1,
        namespace: TURNS_NAMESPACE.into(),
        ciphertext: "00".into(),
    };

    let mut unknown_version = valid.clone();
    unknown_version.version = 9;
    assert!(matches!(
        envelope.open(&unknown_version),
        Err(EnvelopeError::Format(_))
    ));

    let mut invalid_hex = valid.clone();
    invalid_hex.ciphertext = "zz".into();
    assert!(matches!(
        envelope.open(&invalid_hex),
        Err(EnvelopeError::Format(_))
    ));

    // F-01 PoC: a multibyte char at an even byte offset ("aa€x") must be
    // `Err(Format)`, not a char-boundary panic.
    let mut multibyte = valid.clone();
    multibyte.ciphertext = "aa€x".into();
    assert!(matches!(
        envelope.open(&multibyte),
        Err(EnvelopeError::Format(_))
    ));
}

#[test]
fn rotation_bump_keeps_old_envelopes_readable_and_new_writes_use_the_new_key() {
    let master = test_master();
    let v1 = Envelope::with_master(&envelope_config(true, 1), Some(master.clone()));
    let v2 = Envelope::with_master(&envelope_config(true, 2), Some(master));

    let old = v1
        .seal(TURNS_NAMESPACE, "hello rotation")
        .expect("sealed v1");
    assert_eq!(old.key_version, 1, "old write carries its version");
    assert_eq!(
        v2.open(&old).expect("old envelope still opens"),
        "hello rotation",
        "rotation v1: previous envelopes stay readable"
    );

    let fresh = v2
        .seal(TURNS_NAMESPACE, "hello rotation")
        .expect("sealed v2");
    assert_eq!(fresh.key_version, 2, "new writes carry the rotated version");
    assert_eq!(
        v2.open(&fresh).expect("new envelope opens"),
        "hello rotation"
    );
}

// ── c3: degradación explícita (nunca claro) ──────────────────────────────────

#[tokio::test]
async fn missing_key_disarms_explicitly_and_never_persists_clear_originals() {
    let db = memory();
    let redactor = mask_redactor();
    let disarmed = Envelope::with_master(&envelope_config(true, 1), None);
    assert_eq!(disarmed.mode(), EnvelopeMode::Disarmed);
    assert!(
        disarmed.disarm_reason().is_some(),
        "degradation must be explicit (mode + reason for the startup warning)"
    );

    let payload = capture_and_payload(&db, &turn_text(), &redactor, &disarmed).await;
    assert!(
        payload.get("original_envelope").is_none(),
        "no envelope without a key"
    );
    let persisted = payload["text"].as_str().expect("text field");
    assert!(!persisted.contains(EMAIL));
    assert!(
        !serde_json::to_string(&payload).unwrap().contains(EMAIL),
        "no clear original anywhere in the payload"
    );
    assert!(
        disarmed.seal(TURNS_NAMESPACE, "x").is_none(),
        "disarmed seal is a no-op"
    );

    // A passphrase master cannot derive namespace keys → explicit disarm too.
    let passphrase = Envelope::with_master(
        &envelope_config(true, 1),
        Some(Cipher::new(b"short passphrase")),
    );
    assert_eq!(passphrase.mode(), EnvelopeMode::Disarmed);
    assert!(passphrase.disarm_reason().is_some());
}

#[tokio::test]
async fn envelope_disabled_is_declared_redacted_only() {
    let db = memory();
    let redactor = mask_redactor();
    let off = Envelope::with_master(&envelope_config(false, 1), None);
    assert_eq!(off.mode(), EnvelopeMode::RedactedOnly);
    assert!(
        off.disarm_reason().is_none(),
        "off is a declared mode, not a degradation"
    );

    let payload = capture_and_payload(&db, &turn_text(), &redactor, &off).await;
    assert!(payload.get("original_envelope").is_none());
    assert!(off.seal(TURNS_NAMESPACE, "x").is_none());

    let blob = EnvelopeBlob {
        version: 1,
        key_version: 1,
        namespace: TURNS_NAMESPACE.into(),
        ciphertext: "00".into(),
    };
    assert!(matches!(off.open(&blob), Err(EnvelopeError::Inactive)));
}

/// Opt-in invariant: with both features off the payload is byte-shape
/// compatible with the pre-VER-03 write path (5 keys, original text).
#[tokio::test]
async fn redact_and_envelope_off_keeps_legacy_payload_shape() {
    let db = memory();
    let disabled = Redactor::new(&RedactConfig::default()).expect("disabled redactor builds");
    let off = Envelope::with_master(&envelope_config(false, 1), None);
    let text = turn_text();

    let payload = capture_and_payload(&db, &text, &disabled, &off).await;
    let obj = payload.as_object().expect("payload object");
    assert_eq!(
        obj.len(),
        5,
        "legacy shape: session/protocol/space/model/text — got {obj:?}"
    );
    assert_eq!(payload["text"].as_str(), Some(text.as_str()));
    assert!(payload.get("redacted").is_none());
    assert!(payload.get("original_envelope").is_none());
}
