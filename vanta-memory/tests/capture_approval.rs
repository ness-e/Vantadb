// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-68: gate opcional de aprobación de capturas (patrón Cursor).
//!
//! Pattern AAA. Default off = passthrough byte-idéntico (guardia de
//! regresión, pre-mortem #1); con gate on las memorias esperan en cola
//! hasta approve (persisten) o reject (se descartan).

use vanta_memory::core::abstractions::{ExtractedMemory, MemoryType};
use vanta_memory::core::record::approval::{
    should_gate, CaptureApprovalConfig, CaptureApprovalQueue,
};
use vanta_memory::core::record::{apply_dedup_batch, read_session_records};
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

fn memory(content: &str) -> ExtractedMemory {
    ExtractedMemory {
        content: content.into(),
        memory_type: MemoryType::Episodic,
        priority: 70,
        source_message_ids: vec![],
        scene_name: "s".into(),
        metadata: serde_json::Value::Null,
    }
}

const SESSION_KEY: &str = "sess-68";
const SESSION_ID: &str = "thread-68";

#[test]
fn default_off_passthrough_writes_immediately() {
    // Arrange: gate deshabilitado (default).
    let config = CaptureApprovalConfig::default();
    assert!(!config.enabled);
    assert!(!should_gate(&config));

    // Act: path existente sin gate — escribe directo.
    let db = open_db();
    let written = apply_dedup_batch(
        &db,
        SESSION_KEY,
        SESSION_ID,
        &[memory("user prefers dark mode")],
        &[],
        1_700_000_000_000,
        None,
    )
    .expect("write without gate");

    // Assert: persistió sin pasar por cola.
    assert_eq!(written.len(), 1);
    assert_eq!(
        read_session_records(&db, SESSION_KEY).expect("read").len(),
        1
    );
}

#[test]
fn pending_to_approve_persists() {
    // Arrange: gate on, una captura en cola.
    let config = CaptureApprovalConfig { enabled: true };
    assert!(should_gate(&config));
    let queue = CaptureApprovalQueue::new();
    let db = open_db();
    let id = queue.submit(
        SESSION_KEY,
        SESSION_ID,
        vec![memory("user prefers dark mode")],
        1_700_000_000_000,
    );

    // Act: nada persiste antes del approve; approve persiste.
    assert_eq!(queue.pending_count(), 1);
    assert_eq!(queue.list_pending().len(), 1);
    assert_eq!(
        read_session_records(&db, SESSION_KEY).expect("read").len(),
        0
    );
    let written = queue
        .approve(&db, &id, 1_700_000_000_001, None)
        .expect("approve persists");

    // Assert.
    assert_eq!(written.len(), 1);
    assert_eq!(queue.pending_count(), 0);
    assert_eq!(
        read_session_records(&db, SESSION_KEY).expect("read").len(),
        1
    );
}

#[test]
fn pending_to_reject_drops() {
    // Arrange.
    let queue = CaptureApprovalQueue::new();
    let db = open_db();
    let id = queue.submit(
        SESSION_KEY,
        SESSION_ID,
        vec![memory("ephemeral note")],
        1_700_000_000_000,
    );

    // Act: reject descarta sin persistir.
    assert!(queue.reject(&id));
    assert!(!queue.reject(&id)); // idempotente: segunda vez false

    // Assert.
    assert_eq!(queue.pending_count(), 0);
    assert_eq!(
        read_session_records(&db, SESSION_KEY).expect("read").len(),
        0
    );
}

#[test]
fn approve_unknown_id_fails() {
    // Arrange.
    let queue = CaptureApprovalQueue::new();
    let db = open_db();

    // Act + assert: approve de id inexistente es error, no panic.
    assert!(queue.approve(&db, "p_0_999", 0, None).is_err());
}
