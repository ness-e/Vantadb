//! Integration tests for CLI command handlers.
//! Tests use the library's `cli_handlers` module directly with temp databases.

use std::path::Path;

fn setup_temp_db() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().to_string_lossy().to_string();
    // Initialize the database by opening in read-write mode first
    vantadb::cli_handlers::cmd_put(&path, "_init", "_init", "", None, false)
        .expect("init put failed");
    (dir, path)
}

fn seed_record(db_path: &str, namespace: &str, key: &str, payload: &str) {
    vantadb::cli_handlers::cmd_put(db_path, namespace, key, payload, None, false)
        .expect("seed put failed");
}

fn seed_embedded(db_path: &str, namespace: &str, key: &str, payload: &str) {
    let config = vantadb::config::VantaConfig {
        storage_path: db_path.to_string(),
        read_only: false,
        ..Default::default()
    };
    let db = vantadb::VantaEmbedded::open_with_config(config).expect("seed embedded open failed");
    db.put(vantadb::sdk::VantaMemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: payload.to_string(),
        metadata: vantadb::sdk::VantaMemoryMetadata::new(),
        vector: None,
    })
    .expect("seed embedded put failed");
}

// ─── put / get / list ─────────────────────────────────────────

#[test]
fn test_put_and_get() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "test_ns", "key1", "hello world");

    // get the record back
    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    let node_id = vantadb::cli_handlers::memory_node_id("test_ns", "key1");
    let node = engine.get(node_id).unwrap().expect("record not found");
    let payload = node
        .relational
        .get(vantadb::cli_handlers::FIELD_PAYLOAD)
        .and_then(|v| match v {
            vantadb::node::FieldValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .expect("payload not found");
    assert_eq!(payload, "hello world");
}

#[test]
fn test_get_nonexistent() {
    let (_dir, path) = setup_temp_db();

    // get on empty db should error
    let result = vantadb::cli_handlers::cmd_get(&path, "ns", "missing", false);
    assert!(result.is_err(), "expected error for missing record");
}

#[test]
fn test_put_with_vector() {
    let (_dir, path) = setup_temp_db();
    vantadb::cli_handlers::cmd_put(&path, "vec_ns", "v1", "data", Some("1.0,2.0,3.0"), false)
        .expect("put with vector failed");

    // verify vector was stored
    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    let node_id = vantadb::cli_handlers::memory_node_id("vec_ns", "v1");
    let node = engine.get(node_id).unwrap().expect("record not found");
    assert!(node.flags.is_set(vantadb::node::NodeFlags::HAS_VECTOR));
}

#[test]
fn test_put_invalid_vector() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_put(&path, "ns", "k", "data", Some("abc"), false);
    assert!(result.is_err(), "expected error for invalid vector");
}

#[test]
fn test_list_empty_namespace() {
    let (_dir, path) = setup_temp_db();
    // list on empty db should succeed (prints warning)
    let result = vantadb::cli_handlers::cmd_list(&path, "empty_ns", 10, false);
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "list on empty namespace should succeed: {:?}",
        result
    );
}

#[test]
fn test_list_with_records() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "ns1", "a", "payload a");
    seed_record(&path, "ns1", "b", "payload b");
    seed_record(&path, "ns1", "c", "payload c");

    // list with limit
    let result = vantadb::cli_handlers::cmd_list(&path, "ns1", 2, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_limit() {
    let (_dir, path) = setup_temp_db();
    for i in 0..10 {
        seed_record(&path, "lim_ns", &format!("k{}", i), "data");
    }

    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    let nodes = engine.scan_nodes().unwrap();
    let count = nodes
        .iter()
        .filter(|n| {
            n.relational
                .get(vantadb::cli_handlers::FIELD_NAMESPACE)
                .map(|v| matches!(v, vantadb::node::FieldValue::String(s) if s == "lim_ns"))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(count, 10, "expected 10 records in namespace");
}

// ─── delete ───────────────────────────────────────────────────

#[test]
fn test_delete_record() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "del_ns", "del_key", "to delete");

    // delete existing
    let result = vantadb::cli_handlers::cmd_delete(&path, "del_ns", "del_key", false);
    assert!(result.is_ok());

    // verify gone
    let node_id = vantadb::cli_handlers::memory_node_id("del_ns", "del_key");
    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    assert!(engine.get(node_id).unwrap().is_none());
}

#[test]
fn test_delete_nonexistent() {
    let (_dir, path) = setup_temp_db();
    // delete missing should succeed (prints warning)
    let result = vantadb::cli_handlers::cmd_delete(&path, "ns", "missing", false);
    assert!(result.is_ok());
}

#[test]
fn test_delete_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "v_ns", "v_key", "verbose delete");
    let result = vantadb::cli_handlers::cmd_delete(&path, "v_ns", "v_key", true);
    assert!(result.is_ok());
}

// ─── search ───────────────────────────────────────────────────

#[test]
fn test_search_no_results() {
    let (_dir, path) = setup_temp_db();
    // seed via embedded to build text index
    seed_embedded(&path, "srch_ns", "k1", "hello world");
    let result = vantadb::cli_handlers::cmd_search(&path, "srch_ns", "zzz_nonexistent", 10);
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "search with no match should succeed: {:?}",
        result
    );
}

#[test]
fn test_search_with_results() {
    let (_dir, path) = setup_temp_db();
    // seed via embedded to build text indexes
    seed_embedded(&path, "search_ns", "r1", "apple banana");
    seed_embedded(&path, "search_ns", "r2", "banana cherry");

    // search for "banana" - should find at least 1 result
    let result = vantadb::cli_handlers::cmd_search(&path, "search_ns", "banana", 10);
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(result.is_ok(), "search should find results: {:?}", result);
}

// ─── namespace ────────────────────────────────────────────────

#[test]
fn test_namespace_list_empty() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_namespace_list(&path);
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(result.is_ok(), "namespace list on empty db: {:?}", result);
}

#[test]
fn test_namespace_list_with_data() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "ns_a", "k1", "data");
    seed_record(&path, "ns_b", "k2", "data");

    // open embedded to verify namespaces
    let db = vantadb::cli_handlers::open_embedded(&path, true).unwrap();
    let namespaces = db.list_namespaces().unwrap();
    assert!(namespaces.contains(&"ns_a".to_string()));
    assert!(namespaces.contains(&"ns_b".to_string()));
}

#[test]
fn test_namespace_info() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "info_ns", "k1", "hello world");

    let result = vantadb::cli_handlers::cmd_namespace_info(&path, "info_ns");
    assert!(result.is_ok());
}

#[test]
fn test_namespace_info_empty() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_namespace_info(&path, "empty_ns");
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(result.is_ok(), "namespace info on empty ns: {:?}", result);
}

// ─── status ───────────────────────────────────────────────────

#[test]
fn test_status_no_db() {
    let path = "./nonexistent_test_dir_should_not_exist";
    if Path::new(path).exists() {
        eprintln!("WARNING: test directory exists, using temp dir instead");
        let (_dir, tmp_path) = setup_temp_db();
        let result = vantadb::cli_handlers::cmd_status(&tmp_path, false);
        assert!(result.is_ok());
        return;
    }
    let result = vantadb::cli_handlers::cmd_status(path, false);
    assert!(result.is_ok());
}

#[test]
fn test_status_with_db() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "st_ns", "st_k", "status test");
    let result = vantadb::cli_handlers::cmd_status(&path, false);
    assert!(result.is_ok());
}

#[test]
fn test_status_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "st_v", "k", "verbose status");
    let result = vantadb::cli_handlers::cmd_status(&path, true);
    assert!(result.is_ok());
}

// ─── memory_node_id ───────────────────────────────────────────

#[test]
fn test_memory_node_id_deterministic() {
    let id1 = vantadb::cli_handlers::memory_node_id("ns", "key");
    let id2 = vantadb::cli_handlers::memory_node_id("ns", "key");
    assert_eq!(id1, id2, "node ID must be deterministic");
}

#[test]
fn test_memory_node_id_different_keys() {
    let id1 = vantadb::cli_handlers::memory_node_id("ns", "a");
    let id2 = vantadb::cli_handlers::memory_node_id("ns", "b");
    assert_ne!(id1, id2, "different keys must produce different IDs");
}

#[test]
fn test_memory_node_id_different_namespaces() {
    let id1 = vantadb::cli_handlers::memory_node_id("ns1", "key");
    let id2 = vantadb::cli_handlers::memory_node_id("ns2", "key");
    assert_ne!(id1, id2, "different namespaces must produce different IDs");
}

// ─── missing db path ──────────────────────────────────────────

#[test]
fn test_cmd_get_missing_db() {
    let result = vantadb::cli_handlers::cmd_get("./ghost_dir", "ns", "k", false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_list_missing_db() {
    let result = vantadb::cli_handlers::cmd_list("./ghost_dir", "ns", 10, false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_delete_missing_db() {
    let result = vantadb::cli_handlers::cmd_delete("./ghost_dir", "ns", "k", false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_search_missing_db() {
    let result = vantadb::cli_handlers::cmd_search("./ghost_dir", "ns", "q", 10);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_namespace_list_missing_db() {
    let result = vantadb::cli_handlers::cmd_namespace_list("./ghost_dir");
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_namespace_info_missing_db() {
    let result = vantadb::cli_handlers::cmd_namespace_info("./ghost_dir", "ns");
    assert!(result.is_ok(), "missing db should warn, not error");
}

// ─── rebuild / export / import / query ─────────────────────────

#[test]
fn test_cmd_rebuild_index_empty() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_rebuild_index(&path, false);
    assert!(result.is_ok());
}

#[test]
fn test_cmd_export_and_import() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "ex_ns", "k1", "export me");

    let export_path = format!("{}/export.json", &path);
    let result = vantadb::cli_handlers::cmd_export(&path, Some("ex_ns"), &export_path);
    assert!(result.is_ok(), "export failed");
    assert!(Path::new(&export_path).exists(), "export file missing");

    // import into a fresh db
    let (_dir2, path2) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_import(&path2, &export_path, false);
    assert!(result.is_ok(), "import failed");

    // verify imported
    let node_id = vantadb::cli_handlers::memory_node_id("ex_ns", "k1");
    let engine = vantadb::cli_handlers::open_database(&path2, true).unwrap();
    assert!(
        engine.get(node_id).unwrap().is_some(),
        "imported record not found"
    );
}

#[test]
fn test_cmd_query_empty_db() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_query(&path, "FROM Persona", 10, false);
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "query on empty db should succeed: {:?}",
        result
    );
}

// ─── verbose mode ─────────────────────────────────────────────

#[test]
fn test_put_verbose() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_put(&path, "v_ns", "v_key", "verbose", None, true);
    assert!(result.is_ok());
}

#[test]
fn test_list_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "lv_ns", "k", "data");
    let result = vantadb::cli_handlers::cmd_list(&path, "lv_ns", 10, true);
    assert!(result.is_ok());
}
