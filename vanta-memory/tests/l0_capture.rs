//! D19 dedicated L0 capture tests (MEM-09 contract).
//!
//! Uses a real VantaDB embedded instance with the in-memory backend
//! (pattern: `tests/message_thread_test.rs`) — no mocks.

use std::collections::HashSet;

use tempfile::tempdir;
use vanta_memory::core::conversation::{L0Message, L0Recorder, L0Role};
use vanta_memory::core::hooks::{AutoCaptureConfig, AutoCaptureHook, RawMessage};
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::BackendKind;

fn open_db() -> (VantaEmbedded, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).expect("open embedded");
    (db, dir)
}

fn msg(id: &str, role: L0Role, content: &str, ts: u64) -> L0Message {
    L0Message {
        id: Some(id.to_string()),
        role,
        content: content.to_string(),
        timestamp_ms: ts,
    }
}

fn raw(id: &str, role: &str, content: &str, ts: u64) -> RawMessage {
    RawMessage {
        id: Some(id.to_string()),
        role: role.to_string(),
        content: content.to_string(),
        timestamp_ms: Some(ts),
    }
}

// (a) Same turn twice → 1 record (idempotency by stable key via SDK upsert).
#[test]
fn same_turn_twice_is_idempotent() {
    let (db, _dir) = open_db();
    let recorder = L0Recorder::new(db);
    let session = "sess-a";

    let capture = vanta_memory::core::conversation::L0Capture {
        session_id: session.into(),
        messages: vec![msg("m1", L0Role::User, "hello", 1000)],
    };

    let first = recorder.record_turn(&capture, None).expect("first capture");
    let second = recorder
        .record_turn(&capture, None)
        .expect("second capture");

    assert_eq!(first.recorded_count, 1);
    assert_eq!(second.recorded_count, 0, "replay must not duplicate");

    let stored = recorder.read_messages(session).expect("read");
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].content, "hello");
}

// (b) Roles outside capture_roles are excluded from L0.
#[test]
fn filters_out_non_captured_roles() {
    let (db, _dir) = open_db();
    let config = AutoCaptureConfig {
        capture_roles: HashSet::from([L0Role::Assistant]),
        ..Default::default()
    };
    let hook = AutoCaptureHook::new(db, config);

    let result = hook
        .capture(
            "sess-b",
            vec![
                raw("u1", "user", "user text", 2000),
                raw("a1", "assistant", "assistant text", 2001),
            ],
        )
        .expect("capture");

    assert_eq!(result.recorded_count, 1);
    assert_eq!(result.filtered_messages, 1, "user message filtered by role");
}

// (c) Cursor advances and prevents re-recording of old timestamps.
#[test]
fn cursor_advances_and_does_not_rerun() {
    let (db, _dir) = open_db();
    let recorder = L0Recorder::new(db);
    let session = "sess-c";

    let turn1 = vanta_memory::core::conversation::L0Capture {
        session_id: session.into(),
        messages: vec![msg("m1", L0Role::User, "first", 3000)],
    };
    let r1 = recorder.record_turn(&turn1, None).expect("turn1");
    assert_eq!(r1.cursor_ms, 3000);

    // Same message re-sent plus a newer one: only the newer must be recorded.
    let turn2 = vanta_memory::core::conversation::L0Capture {
        session_id: session.into(),
        messages: vec![
            msg("m1", L0Role::User, "first", 3000),
            msg("m2", L0Role::Assistant, "second", 3100),
        ],
    };
    let r2 = recorder.record_turn(&turn2, None).expect("turn2");

    assert_eq!(r2.recorded_count, 1, "old timestamp must be skipped");
    assert_eq!(r2.cursor_ms, 3100);

    let stored = recorder.read_messages(session).expect("read");
    assert_eq!(stored.len(), 2);
}

// (d) Fallback to plugin_start_timestamp_ms when no cursor exists.
#[test]
fn fallback_to_plugin_start_without_cursor() {
    let (db, _dir) = open_db();
    let recorder = L0Recorder::new(db);
    let session = "sess-d";

    // No cursor persisted; floor at 5000 → message at 4000 is filtered.
    let capture = vanta_memory::core::conversation::L0Capture {
        session_id: session.into(),
        messages: vec![msg("old", L0Role::User, "before-start", 4000)],
    };
    let result = recorder
        .record_turn(&capture, Some(5000))
        .expect("capture with floor");

    assert_eq!(result.recorded_count, 0);
    assert_eq!(result.cursor_ms, 5000);

    let stored = recorder.read_messages(session).expect("read");
    assert!(
        stored.is_empty(),
        "pre-plugin-start message must be excluded"
    );
}

// (e) read_messages returns only messages, never the cursor record.
#[test]
fn read_messages_returns_only_messages() {
    let (db, _dir) = open_db();
    let recorder = L0Recorder::new(db);
    let session = "sess-e";

    let capture = vanta_memory::core::conversation::L0Capture {
        session_id: session.into(),
        messages: vec![msg("m1", L0Role::User, "hi", 6000)],
    };
    recorder.record_turn(&capture, None).expect("capture");

    let stored = recorder.read_messages(session).expect("read");
    assert_eq!(stored.len(), 1);
    assert!(
        stored.iter().all(|m| m.id.as_deref() != Some("__cursor")),
        "cursor record must never leak into read_messages"
    );
}
