//! AUD-37: Edge case tests for VantaDB.
//! Covers NaN/Inf vectors, empty inputs, zero-dim vectors,
//! concurrent access, metadata special chars, and WAL failure.

use std::sync::Arc;
use std::thread;
use tempfile::tempdir;
use vantadb::{
    InMemoryEngine, UnifiedNode, VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest,
    VantaValue,
};

fn field_string(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

// ── 1. NaN/Inf vectors ─────────────────────────────────────

#[test]
fn nan_vector_rejected() {
    let engine = InMemoryEngine::new();
    let node = UnifiedNode::with_vector(1, vec![f32::NAN, 0.0, 0.0]);
    // cosine_similarity with NaN returns None, so search won't return it
    let id = engine.insert(node).unwrap();
    assert_eq!(id, 1);

    let result = engine.vector_search(&[1.0, 0.0, 0.0], 10, 0.0, None);
    assert!(
        result.nodes.is_empty(),
        "NaN vector should not match in cosine search"
    );
}

#[test]
fn inf_vector_rejected() {
    let engine = InMemoryEngine::new();
    let node = UnifiedNode::with_vector(1, vec![f32::INFINITY, 0.0, 0.0]);
    let id = engine.insert(node).unwrap();
    assert_eq!(id, 1);

    let result = engine.vector_search(&[1.0, 0.0, 0.0], 10, 0.0, None);
    assert!(
        result.nodes.is_empty(),
        "Inf vector should not match in cosine search (denom ~inf, similarity → NaN)"
    );
}

#[test]
fn neg_inf_vector_rejected() {
    let engine = InMemoryEngine::new();
    let node = UnifiedNode::with_vector(1, vec![f32::NEG_INFINITY, 0.0, 0.0]);
    let id = engine.insert(node).unwrap();
    assert_eq!(id, 1);

    let result = engine.vector_search(&[1.0, 0.0, 0.0], 10, 0.0, None);
    assert!(
        result.nodes.is_empty(),
        "Negative Inf vector should not match"
    );
}

// ── 2. Empty key ───────────────────────────────────────────

#[test]
fn empty_key_returns_error() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let input = VantaMemoryInput::new("test", "", "payload");
    let err = db.put(input).expect_err("empty key must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("key must not be empty"),
        "expected key validation error, got: {msg}"
    );
}

// ── 3. Empty batch ─────────────────────────────────────────

#[test]
fn empty_batch_succeeds() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let results = db.put_batch(vec![]).expect("empty batch must succeed");
    assert!(
        results.is_empty(),
        "empty batch should return empty results"
    );
}

// ── 4. Empty namespace search ──────────────────────────────

#[test]
fn empty_namespace_search_returns_error() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let request = VantaMemorySearchRequest {
        namespace: String::new(),
        query_vector: vec![1.0, 0.0, 0.0],
        ..Default::default()
    };
    let err = db.search(request).expect_err("empty namespace must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("namespace must not be empty"),
        "expected namespace validation error, got: {msg}"
    );
}

#[test]
fn empty_namespace_put_returns_error() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let input = VantaMemoryInput::new("", "key", "payload");
    let err = db.put(input).expect_err("empty namespace must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("namespace must not be empty"),
        "expected namespace validation error, got: {msg}"
    );
}

// ── 5. Non-existent ID delete ──────────────────────────────

#[test]
fn delete_non_existent_key_returns_false() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let deleted = db
        .delete("nonexistent", "ghost-key")
        .expect("delete non-existent must not error");
    assert!(!deleted, "deleting non-existent key should return false");
}

#[test]
fn delete_non_existent_engine_id_returns_error() {
    let engine = InMemoryEngine::new();
    let err = engine
        .delete(999_999)
        .expect_err("deleting unknown ID must fail");
    assert!(
        matches!(err, vantadb::VantaError::NodeNotFound(999_999)),
        "expected NodeNotFound, got: {err}"
    );
}

// ── 6. Special characters in metadata ──────────────────────

#[test]
fn metadata_unicode_and_emoji_keys() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "special-meta", "payload");
    input
        .metadata
        .insert("café".to_string(), field_string("value1"));
    input
        .metadata
        .insert("emoji_🔥".to_string(), field_string("fire"));
    input
        .metadata
        .insert("日本語".to_string(), field_string("nihongo"));
    input
        .metadata
        .insert("key with spaces".to_string(), field_string("spaced"));
    input
        .metadata
        .insert("newline\nkey".to_string(), field_string("nl"));

    let record = db.put(input).expect("put with special metadata");
    assert_eq!(record.metadata.get("café"), Some(&field_string("value1")));
    assert_eq!(record.metadata.get("emoji_🔥"), Some(&field_string("fire")));
    assert_eq!(
        record.metadata.get("日本語"),
        Some(&field_string("nihongo"))
    );
    assert_eq!(
        record.metadata.get("key with spaces"),
        Some(&field_string("spaced"))
    );
    assert_eq!(
        record.metadata.get("newline\nkey"),
        Some(&field_string("nl"))
    );

    let fetched = db
        .get("test", "special-meta")
        .expect("get")
        .expect("record");
    assert_eq!(fetched.metadata, record.metadata);
}

// ── 7. Zero-dimension vector ───────────────────────────────

#[test]
fn zero_dim_vector_stored_without_vector() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "zero-vec", "payload");
    input.vector = Some(vec![]);

    let record = db.put(input).expect("zero-dim vector should succeed");
    assert!(
        record.vector.is_none(),
        "zero-dim vector should be stored as None"
    );
}

#[test]
fn zero_dim_vector_search_empty() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "zero-vec", "payload");
    input.vector = Some(vec![]);
    db.put(input).expect("put zero-dim vector");

    let request = VantaMemorySearchRequest {
        namespace: "test".to_string(),
        query_vector: vec![1.0],
        top_k: 10,
        ..Default::default()
    };
    let results = db.search(request).expect("search with zero-dim vector");
    assert!(
        results.is_empty(),
        "zero-dim vector should not be searchable"
    );
}

// ── 8. All-zeros vector ────────────────────────────────────

#[test]
fn all_zeros_vector_insert_and_search() {
    let engine = InMemoryEngine::new();
    engine
        .insert(UnifiedNode::with_vector(1, vec![0.0, 0.0, 0.0]))
        .unwrap();
    engine
        .insert(UnifiedNode::with_vector(2, vec![1.0, 0.0, 0.0]))
        .unwrap();

    let result = engine.vector_search(&[1.0, 0.0, 0.0], 10, 0.0, None);
    assert_eq!(
        result.nodes.len(),
        1,
        "zero vector should not match (cosine of zero vector with query is undefined/invalid)"
    );
    assert_eq!(result.nodes[0].id, 2, "only non-zero vector should match");
}

#[test]
fn all_zeros_vector_put_and_list() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "all-zeros", "payload");
    input.vector = Some(vec![0.0, 0.0, 0.0]);
    let record = db.put(input).expect("put all-zeros vector");
    assert!(record.vector.is_some(), "all-zeros vector should be stored");
    assert_eq!(
        record.vector.as_ref().unwrap().len(),
        3,
        "vector dimension should be preserved"
    );

    let fetched = db.get("test", "all-zeros").expect("get").expect("record");
    assert_eq!(fetched.vector, record.vector);
}

// ── 10. Concurrent connections / rapid requests ────────────

#[test]
fn concurrent_rapid_inserts_no_crash() {
    let dir = tempdir().expect("tempdir");
    let db = Arc::new(VantaEmbedded::open(dir.path()).expect("open"));

    let mut handles = Vec::new();
    for i in 0..20 {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for j in 0..5 {
                let input = VantaMemoryInput::new(
                    "concurrent",
                    format!("key-{i}-{j}"),
                    format!("payload-{i}-{j}"),
                );
                db.put(input).ok();
            }
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    // Verify all 100 records were inserted
    let list = db
        .list("concurrent", Default::default())
        .expect("list concurrent");
    assert_eq!(
        list.records.len(),
        100,
        "all 100 concurrent inserts should be present"
    );
}

#[test]
fn concurrent_rapid_searches_no_crash() {
    let dir = tempdir().expect("tempdir");
    let db = Arc::new(VantaEmbedded::open(dir.path()).expect("open"));

    for i in 0..50 {
        let mut input =
            VantaMemoryInput::new("search-test", format!("key-{i}"), format!("payload-{i}"));
        input.vector = Some(vec![i as f32 * 0.1, 0.0, 0.0]);
        db.put(input).expect("seed search data");
    }

    let mut handles = Vec::new();
    for _ in 0..20 {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for _ in 0..5 {
                let request = VantaMemorySearchRequest {
                    namespace: "search-test".to_string(),
                    query_vector: vec![1.0, 0.0, 0.0],
                    top_k: 5,
                    ..Default::default()
                };
                db.search(request).ok();
            }
        }));
    }

    for h in handles {
        h.join().expect("search thread panicked");
    }
    // If we reach here without panic, no crash occurred
}

// ── 11. Search with unusual parameters ─────────────────────

#[test]
fn search_top_k_zero_returns_empty() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "key1", "payload");
    input.vector = Some(vec![1.0, 0.0, 0.0]);
    db.put(input).expect("put");

    let request = VantaMemorySearchRequest {
        namespace: "test".to_string(),
        query_vector: vec![1.0, 0.0, 0.0],
        top_k: 0,
        ..Default::default()
    };
    let results = db.search(request).expect("top_k=0 search must not error");
    assert!(results.is_empty(), "top_k=0 should return empty results");
}

#[test]
fn search_empty_query_vector_returns_no_vector_hits() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "key1", "payload");
    input.vector = Some(vec![1.0, 0.0, 0.0]);
    db.put(input).expect("put");

    let request = VantaMemorySearchRequest {
        namespace: "test".to_string(),
        query_vector: vec![],
        top_k: 10,
        ..Default::default()
    };
    let results = db
        .search(request)
        .expect("empty query vector search must not error");
    assert!(
        results.is_empty(),
        "empty query vector should return no hits"
    );
}

// ── 12. Vector dimension mismatch at low level ──────────────

#[test]
fn vector_dimension_mismatch_search() {
    let engine = InMemoryEngine::new();
    engine
        .insert(UnifiedNode::with_vector(1, vec![1.0, 0.0, 0.0]))
        .unwrap();

    // cosine_similarity returns None when dimensions differ
    let result = engine.vector_search(&[1.0, 0.0, 0.0, 0.0], 10, 0.0, None);
    assert!(
        result.nodes.is_empty(),
        "dimension-mismatched query should match nothing"
    );
}

// ── 13. Extremely large metadata ───────────────────────────

#[test]
fn large_metadata_value() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "large-meta", "payload");
    let large_value = "x".repeat(10_000);
    input.metadata.insert(
        "large_field".to_string(),
        VantaValue::String(large_value.clone()),
    );

    let record = db.put(input).expect("put with large metadata");
    assert_eq!(
        record.metadata.get("large_field"),
        Some(&VantaValue::String(large_value))
    );
}

// ── 14. Delete after expired TTL ─────────────────────────

#[test]
fn delete_expired_ttl_record() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("test", "ttl-record", "will-expire");
    input.ttl_ms = Some(1);
    db.put(input).expect("put with 1ms TTL");

    // Wait for TTL to expire
    thread::sleep(std::time::Duration::from_millis(5));

    let deleted = db
        .delete("test", "ttl-record")
        .expect("delete after TTL expiry");
    // TTL is enforced lazily — expired records are treated as non-existent,
    // so delete returns false (nothing to delete from the SDK's perspective)
    assert!(!deleted, "TTL-expired record should return false on delete");
}

// ── 15. Same key in multiple namespaces ─────────────────

#[test]
fn same_key_different_namespaces_are_independent() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(VantaMemoryInput::new("ns1", "shared-key", "payload-a"))
        .expect("put ns1");
    db.put(VantaMemoryInput::new("ns2", "shared-key", "payload-b"))
        .expect("put ns2");

    let a = db
        .get("ns1", "shared-key")
        .expect("get ns1")
        .expect("record");
    let b = db
        .get("ns2", "shared-key")
        .expect("get ns2")
        .expect("record");

    assert_eq!(a.payload, "payload-a");
    assert_eq!(b.payload, "payload-b");
    assert_ne!(a.node_id, b.node_id);
}

// ── 16. Insert with explicit ID collision handling ─────────

#[test]
fn duplicate_engine_id_returns_error() {
    let engine = InMemoryEngine::new();
    engine.insert(UnifiedNode::new(42)).expect("first insert");
    let err = engine
        .insert(UnifiedNode::new(42))
        .expect_err("duplicate insert must fail");
    assert!(
        matches!(err, vantadb::VantaError::DuplicateNode(42)),
        "expected DuplicateNode(42), got: {err}"
    );
}

// ── 17. Update non-existent node ─────────────────────────

#[test]
fn update_non_existent_node_returns_error() {
    let engine = InMemoryEngine::new();
    let err = engine
        .update(999, UnifiedNode::new(999))
        .expect_err("update non-existent must fail");
    assert!(
        matches!(err, vantadb::VantaError::NodeNotFound(999)),
        "expected NodeNotFound(999), got: {err}"
    );
}
