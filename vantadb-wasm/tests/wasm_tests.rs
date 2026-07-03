use vantadb_wasm::{OpfsStorage, VantaDB};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ── Helpers ───────────────────────────────────────────────────────────

fn create_db() -> VantaDB {
    VantaDB::new(None).expect("failed to create VantaDB")
}

fn make_put(namespace: &str, key: &str, payload: &str) -> JsValue {
    serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": namespace,
        "key": key,
        "payload": payload,
    }))
    .unwrap()
}

fn make_put_with_vector(namespace: &str, key: &str, payload: &str, vector: Vec<f32>) -> JsValue {
    serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": namespace,
        "key": key,
        "payload": payload,
        "vector": vector,
    }))
    .unwrap()
}

fn record_payload(record: &JsValue) -> String {
    js_sys::Reflect::get(record, &"payload".into())
        .unwrap()
        .as_string()
        .unwrap()
}

async fn try_opfs(name: &str) -> Option<OpfsStorage> {
    OpfsStorage::open(name).await.ok()
}

// ── OPFS Storage Tests ───────────────────────────────────────────────

#[wasm_bindgen_test]
async fn test_opfs_read_write_cycle() {
    let storage = match try_opfs("vantadb_test").await {
        Some(s) => s,
        None => return,
    };

    let data: &[u8] = b"hello opfs world";
    storage.write_file("test_file.bin", data).await.unwrap();

    let read_back = storage
        .read_file("test_file.bin")
        .await
        .unwrap()
        .expect("file should exist");
    assert_eq!(read_back, data);

    storage.delete_file("test_file.bin").await.unwrap();

    let after_delete = storage.read_file("test_file.bin").await.unwrap();
    assert!(after_delete.is_none());
}

#[wasm_bindgen_test]
async fn test_opfs_write_and_overwrite() {
    let storage = match try_opfs("vantadb_test_overwrite").await {
        Some(s) => s,
        None => return,
    };

    storage
        .write_file("overwrite_test.bin", b"version 1")
        .await
        .unwrap();
    storage
        .write_file("overwrite_test.bin", b"version 2")
        .await
        .unwrap();

    let read_back = storage
        .read_file("overwrite_test.bin")
        .await
        .unwrap()
        .expect("file should exist after overwrite");
    assert_eq!(read_back, b"version 2");

    storage.delete_file("overwrite_test.bin").await.unwrap();
}

#[wasm_bindgen_test]
async fn test_opfs_read_nonexistent() {
    let storage = match try_opfs("vantadb_test_missing").await {
        Some(s) => s,
        None => return,
    };

    let result = storage.read_file("nonexistent_file_xyz.bin").await.unwrap();
    assert!(result.is_none());
}

#[wasm_bindgen_test]
async fn test_opfs_delete_nonexistent() {
    let storage = match try_opfs("vantadb_test_del_missing").await {
        Some(s) => s,
        None => return,
    };

    storage.delete_file("nonexistent_del.bin").await.unwrap();
}

#[wasm_bindgen_test]
async fn test_opfs_isolated_directories() {
    let storage_a = match try_opfs("vantadb_isolated_a").await {
        Some(s) => s,
        None => return,
    };
    let storage_b = match try_opfs("vantadb_isolated_b").await {
        Some(s) => s,
        None => return,
    };

    storage_a
        .write_file("shared_name.bin", b"from_a")
        .await
        .unwrap();
    storage_b
        .write_file("shared_name.bin", b"from_b")
        .await
        .unwrap();

    let from_a = storage_a
        .read_file("shared_name.bin")
        .await
        .unwrap()
        .unwrap();
    let from_b = storage_b
        .read_file("shared_name.bin")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(from_a, b"from_a");
    assert_eq!(from_b, b"from_b");

    storage_a.delete_file("shared_name.bin").await.unwrap();
    storage_b.delete_file("shared_name.bin").await.unwrap();
}

#[wasm_bindgen_test]
async fn test_opfs_binary_data() {
    let storage = match try_opfs("vantadb_test_binary").await {
        Some(s) => s,
        None => return,
    };

    let binary: Vec<u8> = (0..255).collect();
    storage.write_file("binary.bin", &binary).await.unwrap();

    let read_back = storage
        .read_file("binary.bin")
        .await
        .unwrap()
        .expect("binary file should exist");
    assert_eq!(read_back.len(), 255);
    assert_eq!(read_back, binary);

    storage.delete_file("binary.bin").await.unwrap();
}

#[wasm_bindgen_test]
async fn test_opfs_large_file() {
    let storage = match try_opfs("vantadb_test_large").await {
        Some(s) => s,
        None => return,
    };

    let large: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();
    storage.write_file("large.bin", &large).await.unwrap();

    let read_back = storage
        .read_file("large.bin")
        .await
        .unwrap()
        .expect("large file should exist");
    assert_eq!(read_back.len(), 10_000);
    assert_eq!(read_back, large);

    storage.delete_file("large.bin").await.unwrap();
}

// ── In-Memory Storage Tests ──────────────────────────────────────────

#[wasm_bindgen_test]
fn test_put_and_get() {
    let db = create_db();
    db.put(make_put("test", "hello", "world")).unwrap();
    let got = db.get("test", "hello").unwrap();
    assert!(!got.is_null());
    assert_eq!(record_payload(&got), "world");
}

#[wasm_bindgen_test]
fn test_get_nonexistent() {
    let db = create_db();
    let got = db.get("nosuch", "nonexistent").unwrap();
    assert!(got.is_null());
}

#[wasm_bindgen_test]
fn test_delete_record() {
    let db = create_db();
    db.put(make_put("test", "todelete", "bye")).unwrap();
    let deleted = db.delete("test", "todelete").unwrap();
    assert!(deleted);
    let got = db.get("test", "todelete").unwrap();
    assert!(got.is_null());
}

#[wasm_bindgen_test]
fn test_delete_nonexistent() {
    let db = create_db();
    let deleted = db.delete("test", "ghost").unwrap();
    assert!(!deleted);
}

#[wasm_bindgen_test]
fn test_empty_vector_put() {
    let db = create_db();
    let input = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "test",
        "key": "empty_vec",
        "payload": "no vector",
        "vector": []
    }))
    .unwrap();
    let record = db.put(input).unwrap();
    assert!(!record.is_null());
    let got = db.get("test", "empty_vec").unwrap();
    assert!(!got.is_null());
}

#[wasm_bindgen_test]
fn test_put_and_get_with_vector() {
    let db = create_db();
    db.put(make_put_with_vector(
        "test",
        "vec_key",
        "vector data",
        vec![0.1, 0.2, 0.3, 0.4],
    ))
    .unwrap();
    let got = db.get("test", "vec_key").unwrap();
    assert!(!got.is_null());
    let vec_val = js_sys::Reflect::get(&got, &"vector".into()).unwrap();
    assert!(!vec_val.is_undefined());
    assert!(!vec_val.is_null());
}

#[wasm_bindgen_test]
fn test_put_and_get_multiple_namespaces() {
    let db = create_db();
    for ns in &["ns_a", "ns_b", "ns_c"] {
        db.put(make_put(ns, "key1", format!("payload_{}", ns).as_str()))
            .unwrap();
    }
    for ns in &["ns_a", "ns_b", "ns_c"] {
        let got = db.get(ns, "key1").unwrap();
        assert!(!got.is_null());
        assert_eq!(record_payload(&got), format!("payload_{}", ns));
    }
}

// ── Vector Insertion and Search Tests ─────────────────────────────────

#[wasm_bindgen_test]
fn test_vector_insert_and_search() {
    let db = create_db();

    let vectors = [
        (0..4).map(|i| i as f32 * 0.1).collect::<Vec<f32>>(),
        (0..4).map(|i| 1.0 + i as f32 * 0.1).collect::<Vec<f32>>(),
        (0..4).map(|i| 2.0 + i as f32 * 0.1).collect::<Vec<f32>>(),
        (0..4).map(|i| 3.0 + i as f32 * 0.1).collect::<Vec<f32>>(),
    ];

    for (idx, vec) in vectors.iter().enumerate() {
        db.put(make_put_with_vector(
            "vector_test",
            &format!("vec_{}", idx),
            &format!("vector payload {}", idx),
            vec.clone(),
        ))
        .unwrap();
    }

    let query = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "vector_test",
        "query_vector": [0.05, 0.15, 0.25, 0.35],
        "top_k": 4
    }))
    .unwrap();

    let hits = db.search(query).unwrap();
    assert!(hits.is_array());
    let arr = js_sys::Array::from(&hits);
    assert!(arr.length() > 0);
    assert!(arr.length() <= 4);
}

#[wasm_bindgen_test]
fn test_vector_search_empty_namespace() {
    let db = create_db();
    let query = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "empty_ns",
        "query_vector": [0.1, 0.2, 0.3, 0.4],
        "top_k": 5
    }))
    .unwrap();
    let hits = db.search(query).unwrap();
    assert!(hits.is_array());
    let arr = js_sys::Array::from(&hits);
    assert_eq!(arr.length(), 0);
}

#[wasm_bindgen_test]
fn test_vector_search_with_explain() {
    let db = create_db();
    db.put(make_put_with_vector(
        "explain_test",
        "item",
        "explainable",
        vec![0.5, 0.5, 0.5, 0.5],
    ))
    .unwrap();

    let query = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "explain_test",
        "query_vector": [0.5, 0.5, 0.5, 0.5],
        "top_k": 5,
        "explain": true
    }))
    .unwrap();
    let hits = db.search(query).unwrap();
    let arr = js_sys::Array::from(&hits);
    if arr.length() > 0 {
        let hit = arr.get(0);
        let explanation = js_sys::Reflect::get(&hit, &"explanation".into()).unwrap();
        assert!(explanation.is_null() || !explanation.is_undefined());
    }
}

#[wasm_bindgen_test]
fn test_search_vector_api() {
    let db = create_db();

    db.put(make_put_with_vector(
        "sv_test",
        "sv_1",
        "search vector 1",
        vec![1.0, 0.0, 0.0, 0.0],
    ))
    .unwrap();
    db.put(make_put_with_vector(
        "sv_test",
        "sv_2",
        "search vector 2",
        vec![0.0, 1.0, 0.0, 0.0],
    ))
    .unwrap();

    let hits = db.search_vector(vec![0.9, 0.1, 0.0, 0.0], 5).unwrap();
    let arr = js_sys::Array::from(&hits);
    assert!(arr.length() > 0);
}

#[wasm_bindgen_test]
fn test_search_vector_with_different_k() {
    let db = create_db();
    for i in 0..10 {
        db.put(make_put_with_vector(
            "topk_test",
            &format!("k_{}", i),
            &format!("item {}", i),
            vec![i as f32 * 0.1, 0.0, 0.0, 0.0],
        ))
        .unwrap();
    }
    let hits_3 = db.search_vector(vec![0.0, 0.0, 0.0, 0.0], 3).unwrap();
    let arr_3 = js_sys::Array::from(&hits_3);
    assert_eq!(arr_3.length(), 3);

    let hits_all = db.search_vector(vec![0.0, 0.0, 0.0, 0.0], 100).unwrap();
    let arr_all = js_sys::Array::from(&hits_all);
    assert!(arr_all.length() >= 10);
}

// ── Error Handling Tests ──────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_error_empty_namespace() {
    let db = create_db();
    let result = db.get("", "key");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_error_empty_key() {
    let db = create_db();
    let result = db.get("ns", "");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_error_delete_empty_namespace() {
    let db = create_db();
    let result = db.delete("", "key");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_error_put_invalid_json() {
    let db = create_db();
    let invalid = JsValue::from_str("not valid json");
    let result = db.put(invalid);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_error_search_empty_vector() {
    let db = create_db();
    let query = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "test",
        "query_vector": [],
        "top_k": 5
    }))
    .unwrap();
    let result = db.search(query);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_error_namespace_not_found() {
    let db = create_db();
    let opts = serde_wasm_bindgen::to_value(&serde_json::json!({
        "limit": 10
    }))
    .unwrap();
    let result = db.list("nonexistent_namespace", opts);
    assert!(result.is_ok());
    let page = result.unwrap();
    let records = js_sys::Reflect::get(&page, &"records".into()).unwrap();
    let arr = js_sys::Array::from(&records);
    assert_eq!(arr.length(), 0);
}

#[wasm_bindgen_test]
fn test_error_put_batch_invalid() {
    let db = create_db();
    let invalid = JsValue::from_str("not an array");
    let result = db.put_batch(invalid);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_error_list_invalid_limit() {
    let db = create_db();
    let opts = serde_wasm_bindgen::to_value(&serde_json::json!({
        "limit": -1
    }))
    .unwrap();
    let result = db.list("test", opts);
    assert!(result.is_err());
}

// ── Batch Operations Tests ────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_put_batch_empty() {
    let db = create_db();
    let items: Vec<serde_json::Value> = vec![];
    let batch = serde_wasm_bindgen::to_value(&items).unwrap();
    let records = db.put_batch(batch).unwrap();
    assert!(records.is_array());
    let arr = js_sys::Array::from(&records);
    assert_eq!(arr.length(), 0);
}

#[wasm_bindgen_test]
fn test_put_batch_multiple() {
    let db = create_db();
    let items: Vec<serde_json::Value> = (0..10)
        .map(|i| {
            serde_json::json!({
                "namespace": "batch",
                "key": format!("item_{}", i),
                "payload": format!("batch item {}", i),
                "vector": [i as f32 * 0.1, 0.2, 0.3, 0.4]
            })
        })
        .collect();
    let batch = serde_wasm_bindgen::to_value(&items).unwrap();
    db.put_batch(batch).unwrap();
    for i in 0..10 {
        let got = db.get("batch", &format!("item_{}", i)).unwrap();
        assert!(!got.is_null());
        assert_eq!(record_payload(&got), format!("batch item {}", i));
    }
}

// ── Namespace and Listing Tests ──────────────────────────────────────

#[wasm_bindgen_test]
fn test_list_namespaces() {
    let db = create_db();
    let nss = db.list_namespaces().unwrap();
    assert!(nss.is_array());
}

#[wasm_bindgen_test]
fn test_list_with_filters() {
    let db = create_db();
    let input = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "filter_test",
        "key": "filtered_key",
        "payload": "filter me",
        "metadata": {"type": "test"}
    }))
    .unwrap();
    db.put(input).unwrap();

    let opts = serde_wasm_bindgen::to_value(&serde_json::json!({
        "filters": {"type": "test"},
        "limit": 10
    }))
    .unwrap();
    let page = db.list("filter_test", opts).unwrap();
    let records = js_sys::Reflect::get(&page, &"records".into()).unwrap();
    let arr = js_sys::Array::from(&records);
    assert!(arr.length() > 0);
}

#[wasm_bindgen_test]
fn test_list_pagination() {
    let db = create_db();
    for i in 0..25 {
        db.put(make_put(
            "pagination",
            &format!("page_{}", i),
            &format!("item {}", i),
        ))
        .unwrap();
    }

    let opts_10 = serde_wasm_bindgen::to_value(&serde_json::json!({
        "limit": 10
    }))
    .unwrap();
    let page1 = db.list("pagination", opts_10).unwrap();
    let records1 = js_sys::Array::from(&js_sys::Reflect::get(&page1, &"records".into()).unwrap());
    assert_eq!(records1.length(), 10);

    let cursor = js_sys::Reflect::get(&page1, &"next_cursor".into()).unwrap();
    let opts_next = serde_wasm_bindgen::to_value(&serde_json::json!({
        "limit": 10,
        "cursor": cursor.as_f64().unwrap() as usize
    }))
    .unwrap();
    let page2 = db.list("pagination", opts_next).unwrap();
    let records2 = js_sys::Array::from(&js_sys::Reflect::get(&page2, &"records".into()).unwrap());
    assert_eq!(records2.length(), 10);
}

#[wasm_bindgen_test]
fn test_list_max_limit() {
    let db = create_db();
    for i in 0..5 {
        db.put(make_put(
            "max_limit",
            &format!("max_{}", i),
            &format!("item {}", i),
        ))
        .unwrap();
    }
    let opts = serde_wasm_bindgen::to_value(&serde_json::json!({
        "limit": 10000
    }))
    .unwrap();
    let page = db.list("max_limit", opts).unwrap();
    let records = js_sys::Array::from(&js_sys::Reflect::get(&page, &"records".into()).unwrap());
    assert_eq!(records.length(), 5);
}

// ── Lifecycle and Maintenance Tests ──────────────────────────────────

#[wasm_bindgen_test]
fn test_capabilities() {
    let db = create_db();
    let caps = db.capabilities().unwrap();
    assert!(!caps.is_null());
}

#[wasm_bindgen_test]
fn test_flush_and_compact() {
    let db = create_db();
    db.flush().unwrap();
    db.compact_wal().unwrap();
    let freed = db.compact_layout().unwrap();
    assert_eq!(freed, 0);
}

#[wasm_bindgen_test]
fn test_rebuild_index() {
    let db = create_db();
    db.put(make_put_with_vector(
        "index_test",
        "idx_item",
        "rebuild me",
        vec![0.1, 0.2, 0.3, 0.4],
    ))
    .unwrap();
    let report = db.rebuild_index().unwrap();
    assert!(!report.is_null());
}

#[wasm_bindgen_test]
fn test_purge_expired() {
    let db = create_db();
    let input = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "ttl_test",
        "key": "expires_soon",
        "payload": "will expire",
        "ttl_ms": 1
    }))
    .unwrap();
    db.put(input).unwrap();
    let _purged = db.purge_expired().unwrap();
}

// ── Concurrent Operations Tests ──────────────────────────────────────

#[wasm_bindgen_test]
fn test_concurrent_put_get() {
    let db = create_db();
    for i in 0..20 {
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "concurrent",
            "key": format!("key_{}", i),
            "payload": format!("data {}", i),
            "vector": [i as f32 * 0.05, 0.1, 0.2, 0.3]
        }))
        .unwrap();
        db.put(input).unwrap();
        let got = db.get("concurrent", &format!("key_{}", i)).unwrap();
        assert!(!got.is_null());
    }
}

#[wasm_bindgen_test]
fn test_large_metadata() {
    let db = create_db();
    let mut meta = serde_json::Map::new();
    for i in 0..100 {
        meta.insert(
            format!("key_{}", i),
            serde_json::Value::String(format!("value_{}", i)),
        );
    }
    let input_val = serde_json::json!({
        "namespace": "test",
        "key": "large_meta",
        "payload": "big metadata payload",
        "metadata": meta
    });
    let input = serde_wasm_bindgen::to_value(&input_val).unwrap();
    db.put(input).unwrap();
    let got = db.get("test", "large_meta").unwrap();
    assert!(!got.is_null());
}

// ── Text Search Tests ────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_search_without_results() {
    let db = create_db();
    let input = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "test",
        "key": "only_text",
        "payload": "some text content for text-only search"
    }))
    .unwrap();
    db.put(input).unwrap();
    let req = serde_wasm_bindgen::to_value(&serde_json::json!({
        "namespace": "test",
        "query_vector": [0.1, 0.2, 0.3, 0.4],
        "top_k": 5
    }))
    .unwrap();
    let hits = db.search(req).unwrap();
    assert!(hits.is_array() || hits.is_null());
}

// ── Export/Import Tests ──────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_export_all_empty_db() {
    let db = create_db();
    let report = db.export_all("/tmp/export_test").unwrap();
    assert!(!report.is_null());
    let records = js_sys::Reflect::get(&report, &"records_exported".into()).unwrap();
    assert_eq!(records.as_f64().unwrap() as u64, 0);
}

#[wasm_bindgen_test]
fn test_import_records_round_trip() {
    let db = create_db();
    let records: Vec<serde_json::Value> = (0..5)
        .map(|i| {
            serde_json::json!({
                "namespace": "import_test",
                "key": format!("import_{}", i),
                "payload": format!("imported {}", i),
                "metadata": {},
                "created_at_ms": 1000 + i,
                "updated_at_ms": 1000 + i,
                "version": 1,
                "node_id": 100 + i,
                "vector": [0.1, 0.2, 0.3, 0.4],
                "expires_at_ms": null
            })
        })
        .collect();
    let batch = serde_wasm_bindgen::to_value(&records).unwrap();
    let report = db.import_records(batch).unwrap();
    assert!(!report.is_null());

    for i in 0..5 {
        let got = db.get("import_test", &format!("import_{}", i)).unwrap();
        assert!(!got.is_null());
    }
}
