//! Persistent text-index certification for memory payloads.

use tempfile::tempdir;
use vantadb::{VantaEmbedded, VantaMemoryInput};

fn input(namespace: &str, key: &str, payload: &str) -> VantaMemoryInput {
    VantaMemoryInput::new(namespace, key, payload)
}

fn posting_key(namespace: &str, token: &str, key: &str) -> Vec<u8> {
    let mut index_key = Vec::new();
    index_key.extend_from_slice(namespace.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(token.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(key.as_bytes());
    index_key
}

fn assert_has_posting(keys: &[Vec<u8>], namespace: &str, token: &str, key: &str) {
    let expected = posting_key(namespace, token, key);
    assert!(
        keys.contains(&expected),
        "missing posting key {:?}",
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn text_index_rebuilds_from_canonical_records() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input("agent/main", "a", "Alpha alpha beta"))
        .expect("put");
    db.flush().expect("flush");

    let before = db.operational_metrics();
    db.debug_clear_text_index_for_tests()
        .expect("clear text index");
    assert!(db
        .debug_text_index_posting_keys_for_tests()
        .expect("text keys")
        .is_empty());

    let rebuild = db.rebuild_index().expect("rebuild");
    assert!(rebuild.success);

    let keys = db
        .debug_text_index_posting_keys_for_tests()
        .expect("text keys after rebuild");
    assert_eq!(keys.len(), 2);
    assert_has_posting(&keys, "agent/main", "alpha", "a");
    assert_has_posting(&keys, "agent/main", "beta", "a");

    let after = db.operational_metrics();
    assert!(after.text_postings_written >= before.text_postings_written + 2);
}

#[test]
fn text_index_repairs_on_open_when_postings_missing_or_state_corrupt() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();
    let repairs_before;

    {
        let db = VantaEmbedded::open(&path).expect("open");
        db.put(input("agent/main", "repair", "repair alpha"))
            .expect("put");
        db.flush().expect("flush");

        repairs_before = db.operational_metrics().text_index_repairs;
        db.debug_clear_text_index_for_tests()
            .expect("clear text index");
        db.debug_corrupt_text_index_state_for_tests()
            .expect("corrupt text state");
        db.flush().expect("flush corrupted state");
        db.close().expect("close");
    }

    let reopened = VantaEmbedded::open(&path).expect("reopen");
    let keys = reopened
        .debug_text_index_posting_keys_for_tests()
        .expect("text keys after repair");
    assert_has_posting(&keys, "agent/main", "repair", "repair");
    assert_has_posting(&keys, "agent/main", "alpha", "repair");

    let after = reopened.operational_metrics();
    assert!(after.text_index_repairs >= repairs_before + 1);
}

#[test]
fn text_index_update_delete_remove_stale_postings() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input("agent/main", "item", "alpha beta"))
        .expect("put initial");
    db.put(input("agent/main", "item", "beta gamma beta"))
        .expect("put update");

    let keys = db
        .debug_text_index_posting_keys_for_tests()
        .expect("text keys after update");
    assert!(!keys.contains(&posting_key("agent/main", "alpha", "item")));
    assert_has_posting(&keys, "agent/main", "beta", "item");
    assert_has_posting(&keys, "agent/main", "gamma", "item");

    assert!(db.delete("agent/main", "item").expect("delete"));
    let keys = db
        .debug_text_index_posting_keys_for_tests()
        .expect("text keys after delete");
    assert!(!keys.iter().any(|key| key.starts_with(b"agent/main\0")));
}

#[test]
fn text_index_tokenization_and_key_contract() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input(
        "agent/main",
        "contract",
        "Hello, VantaDB! Agent-42 memory memory.",
    ))
    .expect("put");

    let keys = db
        .debug_text_index_posting_keys_for_tests()
        .expect("text keys");
    let expected = vec![
        posting_key("agent/main", "42", "contract"),
        posting_key("agent/main", "agent", "contract"),
        posting_key("agent/main", "hello", "contract"),
        posting_key("agent/main", "memory", "contract"),
        posting_key("agent/main", "vantadb", "contract"),
    ];
    assert_eq!(keys, expected);
}

#[test]
fn text_index_export_import_round_trip_rebuildable() {
    let source_dir = tempdir().expect("source tempdir");
    let target_dir = tempdir().expect("target tempdir");
    let export_path = source_dir.path().join("memory.jsonl");

    let source = VantaEmbedded::open(source_dir.path()).expect("open source");
    source
        .put(input("agent/main", "portable", "portable alpha alpha"))
        .expect("put source");
    source
        .export_namespace(&export_path, "agent/main")
        .expect("export namespace");

    let target = VantaEmbedded::open(target_dir.path()).expect("open target");
    let imported = target.import_file(&export_path).expect("import file");
    assert_eq!(imported.inserted, 1);
    assert_eq!(imported.errors, 0);

    let imported_keys = target
        .debug_text_index_posting_keys_for_tests()
        .expect("imported text keys");
    assert_has_posting(&imported_keys, "agent/main", "portable", "portable");
    assert_has_posting(&imported_keys, "agent/main", "alpha", "portable");

    target
        .debug_clear_text_index_for_tests()
        .expect("clear imported text index");
    target.rebuild_index().expect("rebuild target");

    let rebuilt_keys = target
        .debug_text_index_posting_keys_for_tests()
        .expect("rebuilt text keys");
    assert_eq!(rebuilt_keys.len(), 2);
    assert_has_posting(&rebuilt_keys, "agent/main", "portable", "portable");
    assert_has_posting(&rebuilt_keys, "agent/main", "alpha", "portable");
}
