// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// MEMG-17: gated on the crate feature (the module imports `vantadb::crypto`,
// itself behind `vantadb/encryption`); default test builds must stay green.
#![cfg(feature = "erasure")]
//! MEMG-17 (piezas b+c) — erasure criptográfica por destrucción de DEK y
//! recibos verificables (contrato VER-02). In-memory store, sin LLM.
//!
//! What could break: que el "erase" no destruya la DEK (o deje residuos en la
//! versión histórica), que el sellado no bindee el scope (clave equivocada
//! abre), que la DEK quede en claro en el store, y que el recibo sea
//! claim-driven (aceptar un JSON editado o una superficie inventada).

use sha2::Digest;
use vanta_memory::utils::erasure::{
    create_scope, erase_scope, open, seal, verify_erasure_receipt, ErasureError, ErasureReceipt,
    ERASURE_DEK_NAMESPACE,
};
use vantadb::config::Config;
use vantadb::crypto::Cipher;
use vantadb::sdk::{Embedded, MemoryInput};

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

/// Fixed raw master key (32B) — deterministic tests, no env dependency.
fn master() -> Cipher {
    Cipher::new(&[0x11u8; 32])
}

#[test]
fn seal_open_roundtrip_binds_scope() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create scope");

    let blob = seal(&db, &master, "agent-1", b"secret payload").expect("seal");
    assert_ne!(
        blob.as_slice(),
        b"secret payload",
        "stored form is ciphertext"
    );

    let plain = open(&db, &master, "agent-1", &blob).expect("open");
    assert_eq!(plain, b"secret payload");

    // The scope participates in the key: another scope's DEK cannot open it.
    create_scope(&db, &master, "agent-2").expect("create scope 2");
    match open(&db, &master, "agent-2", &blob) {
        Err(ErasureError::DecryptFailed(_)) => {}
        other => panic!("expected DecryptFailed across scopes, got {other:?}"),
    }

    // The AEAD tag rejects tampering.
    let mut tampered = blob.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0xFF;
    match open(&db, &master, "agent-1", &tampered) {
        Err(ErasureError::DecryptFailed(_)) => {}
        other => panic!("expected DecryptFailed on tamper, got {other:?}"),
    }
}

#[test]
fn create_scope_twice_is_rejected() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("first create");

    match create_scope(&db, &master, "agent-1") {
        Err(ErasureError::ScopeExists(scope)) => assert_eq!(scope, "agent-1"),
        other => panic!("expected ScopeExists, got {other:?}"),
    }
}

#[test]
fn seal_without_scope_is_not_found() {
    let db = db();
    let master = master();

    match seal(&db, &master, "never-created", b"x") {
        Err(ErasureError::DekNotFound(scope)) => assert_eq!(scope, "never-created"),
        other => panic!("expected DekNotFound, got {other:?}"),
    }
}

#[test]
fn corrupted_dek_payload_is_malformed_not_panic() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create");

    // A corrupted store payload can contain multi-byte chars; decoding it by
    // byte-slicing would split a char and panic. Must surface as a typed
    // error instead (OCR-delegation fix, MEMG-17).
    db.put(MemoryInput {
        namespace: ERASURE_DEK_NAMESPACE.to_string(),
        key: "agent-1".to_string(),
        payload: "ab\u{1F600}".to_string(), // "ab" + 4-byte emoji = 6 bytes
        ..Default::default()
    })
    .expect("overwrite dek record");

    match seal(&db, &master, "agent-1", b"x") {
        Err(ErasureError::DekMalformed(scope)) => assert_eq!(scope, "agent-1"),
        other => panic!("expected DekMalformed, got {other:?}"),
    }
}

#[test]
fn erase_scope_destroys_dek_and_tombstones() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create scope");
    let blob = seal(&db, &master, "agent-1", b"payload").expect("seal");

    let receipt = erase_scope(&db, "agent-1", "gdpr art.17 request").expect("erase");

    assert_eq!(receipt.status, "erased");
    assert_eq!(receipt.scope, "agent-1");
    assert_eq!(receipt.reason, "gdpr art.17 request");
    assert!(
        !receipt.out_of_scope.is_empty(),
        "declared scope never silent"
    );
    assert!(
        receipt
            .out_of_scope
            .iter()
            .any(|s| s.contains("master key")),
        "master-key limit declared: {:?}",
        receipt.out_of_scope
    );
    assert!(
        !receipt.integrity.sha256.is_empty(),
        "receipt is finalized with its integrity hash"
    );

    // The DEK is gone from the live store AND from the retained history
    // (tombstone + version purge — the delete path).
    assert!(db
        .get(ERASURE_DEK_NAMESPACE, "agent-1")
        .expect("get dek")
        .is_none());
    assert!(db
        .versions(ERASURE_DEK_NAMESPACE, "agent-1")
        .expect("versions")
        .is_empty());

    // Everything sealed under the destroyed DEK is unrecoverable.
    match open(&db, &master, "agent-1", &blob) {
        Err(ErasureError::DekNotFound(_)) => {}
        other => panic!("expected DekNotFound after erase, got {other:?}"),
    }
}

#[test]
fn erase_without_dek_is_not_found_receipt() {
    let db = db();

    let receipt = erase_scope(&db, "never-created", "cleanup").expect("receipt emitted");

    assert_eq!(receipt.status, "not_found");
    assert_eq!(receipt.scope, "never-created");
    assert!(
        !receipt.out_of_scope.is_empty(),
        "a not_found receipt still declares its limits"
    );
}

// ---------------------------------------------------------------------------
// Step 4 — verifiable receipts (contract VER-02, not claim-driven).
// ---------------------------------------------------------------------------

/// Recompute the receipt self-hash exactly as the producer does (canonical
/// body with `integrity.sha256` emptied) — the "attacker also fixed the hash"
/// scenario for the schema tests.
fn rehash(receipt: &mut ErasureReceipt) {
    receipt.integrity.sha256.clear();
    let digest = sha2::Sha256::digest(serde_json::to_vec(receipt).unwrap());
    receipt.integrity.sha256 = digest.iter().map(|b| format!("{b:02x}")).collect();
}

#[test]
fn receipt_verifies_against_live_state() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create");
    let receipt = erase_scope(&db, "agent-1", "gdpr art.17").expect("erase");
    let json = serde_json::to_string(&receipt).expect("serialize");

    let verdict = verify_erasure_receipt(&db, &json).expect("verify");

    assert_eq!(verdict.integrity, "ok");
    assert_eq!(verdict.schema_version, 1);
    assert_eq!(verdict.scope, "agent-1");
    assert_eq!(verdict.status, "erased");
    assert_eq!(verdict.residues_now, 0);
    assert!(verdict
        .rechecked_surfaces
        .contains(&"dek_record".to_string()));
    assert!(verdict
        .rechecked_surfaces
        .contains(&"version_history".to_string()));
}

#[test]
fn receipt_integrity_detects_edits() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create");
    let receipt = erase_scope(&db, "agent-1", "gdpr art.17").expect("erase");

    // Flip a semantic field without touching the hash.
    let json = serde_json::to_string(&receipt)
        .unwrap()
        .replace("gdpr art.17", "never happened");

    match verify_erasure_receipt(&db, &json) {
        Err(ErasureError::ReceiptInvalid(msg)) => {
            assert!(msg.contains("integrity"), "integrity rejection: {msg}")
        }
        other => panic!("expected integrity rejection, got {other:?}"),
    }
}

#[test]
fn receipt_rejects_partial_surfaces_even_with_recomputed_hash() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create");
    let mut receipt = erase_scope(&db, "agent-1", "gdpr art.17").expect("erase");

    // The attacker drops the surface that could hold residues and recomputes
    // the self-hash: the schema check must still reject (not claim-driven).
    receipt.surfaces.retain(|s| s.surface != "dek_record");
    rehash(&mut receipt);
    let json = serde_json::to_string(&receipt).unwrap();

    match verify_erasure_receipt(&db, &json) {
        Err(ErasureError::ReceiptInvalid(msg)) => {
            assert!(msg.contains("dek_record"), "schema rejection: {msg}")
        }
        other => panic!("expected schema rejection, got {other:?}"),
    }
}

#[test]
fn receipt_rejects_unknown_status_even_with_recomputed_hash() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create");
    let mut receipt = erase_scope(&db, "agent-1", "gdpr art.17").expect("erase");

    receipt.status = "trust me".into();
    rehash(&mut receipt);
    let json = serde_json::to_string(&receipt).unwrap();

    match verify_erasure_receipt(&db, &json) {
        Err(ErasureError::ReceiptInvalid(msg)) => {
            assert!(msg.contains("unknown status"), "schema rejection: {msg}")
        }
        other => panic!("expected schema rejection, got {other:?}"),
    }
}

#[test]
fn receipt_detects_recreated_scope() {
    let db = db();
    let master = master();
    create_scope(&db, &master, "agent-1").expect("create");
    let receipt = erase_scope(&db, "agent-1", "gdpr art.17").expect("erase");
    let json = serde_json::to_string(&receipt).unwrap();
    verify_erasure_receipt(&db, &json).expect("clean right after the erasure");

    // A new DEK for the same scope appears: the old receipt no longer attests
    // the live state (its clean claim broke).
    create_scope(&db, &master, "agent-1").expect("re-create after erase");

    match verify_erasure_receipt(&db, &json) {
        Err(ErasureError::ReceiptInvalid(msg)) => {
            assert!(msg.contains("reappeared"), "residues verdict: {msg}")
        }
        other => panic!("expected reappeared-residues rejection, got {other:?}"),
    }
}
