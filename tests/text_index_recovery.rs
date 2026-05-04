//! Persistent text-index certification for memory payloads.

use tempfile::tempdir;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaMemorySearchRequest,
    VantaOpenOptions, VantaValue,
};

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

fn field_string(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

fn search_keys(
    db: &VantaEmbedded,
    namespace: &str,
    text_query: &str,
    filters: VantaMemoryMetadata,
    top_k: usize,
) -> Vec<String> {
    db.search(VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector: Vec::new(),
        filters,
        text_query: Some(text_query.to_string()),
        top_k,
    })
    .expect("text search")
    .into_iter()
    .map(|hit| hit.record.key)
    .collect()
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
    assert!(
        db.debug_text_index_audit_for_tests()
            .expect("audit after rebuild")
            .passed
    );

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
    assert!(
        reopened
            .debug_text_index_audit_for_tests()
            .expect("audit after repair")
            .passed
    );

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
    let posting = db
        .debug_text_index_posting_for_tests("agent/main", "beta", "item")
        .expect("posting")
        .expect("beta posting");
    assert_eq!(posting.1, 2);
    assert_eq!(
        search_keys(&db, "agent/main", "alpha", Default::default(), 10),
        Vec::<String>::new()
    );
    assert_eq!(
        search_keys(&db, "agent/main", "gamma", Default::default(), 10),
        vec!["item".to_string()]
    );

    assert!(db.delete("agent/main", "item").expect("delete"));
    let keys = db
        .debug_text_index_posting_keys_for_tests()
        .expect("text keys after delete");
    assert!(!keys.iter().any(|key| key.starts_with(b"agent/main\0")));
    assert!(
        db.debug_text_index_audit_for_tests()
            .expect("audit after delete")
            .passed
    );
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
    let posting = db
        .debug_text_index_posting_for_tests("agent/main", "memory", "contract")
        .expect("posting")
        .expect("memory posting");
    assert_eq!(posting.1, 2);
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
    assert_eq!(
        search_keys(&target, "agent/main", "portable", Default::default(), 10),
        vec!["portable".to_string()]
    );
    assert!(
        target
            .debug_text_index_audit_for_tests()
            .expect("audit imported")
            .passed
    );
}

#[test]
fn text_query_bm25_uses_tf_df_and_document_length() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input("agent/main", "tf-low", "alpha signal"))
        .expect("put low tf");
    db.put(input("agent/main", "tf-high", "alpha alpha alpha signal"))
        .expect("put high tf");
    let hits = db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("alpha".to_string()),
            top_k: 2,
        })
        .expect("tf search");
    assert_eq!(hits[0].record.key, "tf-high");
    assert!(hits[0].score > hits[1].score);

    let rare_dir = tempdir().expect("rare tempdir");
    let rare_db = VantaEmbedded::open(rare_dir.path()).expect("open rare");
    rare_db
        .put(input("agent/main", "rare-doc", "common rare"))
        .expect("put rare");
    rare_db
        .put(input("agent/main", "common-doc", "common common"))
        .expect("put common");
    rare_db
        .put(input("agent/main", "common-a", "common filler a"))
        .expect("put common a");
    rare_db
        .put(input("agent/main", "common-b", "common filler b"))
        .expect("put common b");
    assert_eq!(
        search_keys(&rare_db, "agent/main", "common rare", Default::default(), 4)[0],
        "rare-doc"
    );

    let len_dir = tempdir().expect("len tempdir");
    let len_db = VantaEmbedded::open(len_dir.path()).expect("open len");
    len_db
        .put(input("agent/main", "short", "anchor"))
        .expect("put short");
    len_db
        .put(input(
            "agent/main",
            "long",
            "anchor filler filler filler filler filler filler filler filler",
        ))
        .expect("put long");
    let hits = len_db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("anchor".to_string()),
            top_k: 2,
        })
        .expect("length search");
    assert_eq!(hits[0].record.key, "short");
    assert!(hits[0].score > hits[1].score);
}

#[test]
fn text_query_is_namespace_scoped_filtered_and_deterministic() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input("agent/a", "a1", "shared term"))
        .expect("put a");
    db.put(input("agent/b", "b1", "shared term"))
        .expect("put b");
    assert_eq!(
        search_keys(&db, "agent/a", "shared", Default::default(), 10),
        vec!["a1".to_string()]
    );

    let mut task = input("agent/main", "task", "filtered alpha");
    task.metadata
        .insert("category".to_string(), field_string("task"));
    db.put(task).expect("put task");
    let mut note = input("agent/main", "note", "filtered alpha");
    note.metadata
        .insert("category".to_string(), field_string("note"));
    db.put(note).expect("put note");

    let mut filters = VantaMemoryMetadata::new();
    filters.insert("category".to_string(), field_string("task"));
    assert_eq!(
        search_keys(&db, "agent/main", "filtered", filters, 10),
        vec!["task".to_string()]
    );

    db.put(input("agent/main", "a", "tie token"))
        .expect("put tie a");
    db.put(input("agent/main", "b", "tie token"))
        .expect("put tie b");
    let keys = search_keys(&db, "agent/main", "tie", Default::default(), 10);
    assert_eq!(keys[..2], ["a".to_string(), "b".to_string()]);

    let before = db.operational_metrics();
    db.search(VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: Vec::new(),
        filters: Default::default(),
        text_query: Some("tie".to_string()),
        top_k: 2,
    })
    .expect("metrics search");
    let after = db.operational_metrics();
    assert!(after.text_lexical_queries >= before.text_lexical_queries + 1);
    assert!(after.text_candidates_scored >= before.text_candidates_scored + 2);
}

#[test]
fn hybrid_text_vector_remains_deferred_and_read_only_does_not_repair() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    {
        let db = VantaEmbedded::open(&path).expect("open");
        let mut input = input("agent/main", "hybrid", "hybrid alpha");
        input.vector = Some(vec![1.0, 0.0]);
        db.put(input).expect("put");
        let hybrid = db.search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0],
            filters: Default::default(),
            text_query: Some("alpha".to_string()),
            top_k: 10,
        });
        assert!(hybrid.is_err());
        db.debug_corrupt_text_index_state_for_tests()
            .expect("corrupt state");
        db.flush().expect("flush");
        db.close().expect("close");
    }

    let read_only = VantaEmbedded::open_with_options(
        &path,
        VantaOpenOptions {
            memory_limit_bytes: None,
            read_only: true,
        },
    )
    .expect("open read-only");
    let text = read_only.search(VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: Vec::new(),
        filters: Default::default(),
        text_query: Some("alpha".to_string()),
        top_k: 10,
    });
    assert!(text.is_err());
    drop(read_only);

    let reopened = VantaEmbedded::open(&path).expect("open writable");
    assert_eq!(
        search_keys(&reopened, "agent/main", "alpha", Default::default(), 10),
        vec!["hybrid".to_string()]
    );
}
