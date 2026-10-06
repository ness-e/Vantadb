// ponytail: integration tests + progress-bar template literal invariant; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for CLI command handlers.
//! Tests use the library's `cli_handlers` module directly with temp databases.

use std::path::Path;

fn setup_temp_db() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().to_string_lossy().to_string();
    // Initialize the database by opening in read-write mode first
    vantadb::cli_handlers::cmd_put(&path, "_init", "_init", "", None, None, false, false)
        .expect("init put failed");
    (dir, path)
}

fn seed_record(db_path: &str, namespace: &str, key: &str, payload: &str) {
    vantadb::cli_handlers::cmd_put(db_path, namespace, key, payload, None, None, false, false)
        .expect("seed put failed");
}

fn seed_embedded(db_path: &str, namespace: &str, key: &str, payload: &str) {
    let config = vantadb::config::Config {
        storage_path: db_path.to_string(),
        read_only: false,
        ..Default::default()
    };
    let db = vantadb::Embedded::open_with_config(config).expect("seed embedded open failed");
    db.put(vantadb::sdk::MemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: payload.to_string(),
        metadata: vantadb::sdk::MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed embedded put failed");
    // ERR-050b: put is buffered in WAL; a later read-only reopen (open_database)
    // does NOT replay WAL (recover_state skips it for read_only). Without an
    // explicit flush the seed record stays invisible to read-only handles,
    // failing "must survive" / "should remain" assertions.
    db.flush().expect("seed embedded flush failed");
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
    let result = vantadb::cli_handlers::cmd_get(&path, "ns", "missing", false, false);
    assert!(result.is_err(), "expected error for missing record");
}

#[test]
fn test_put_with_vector() {
    let (_dir, path) = setup_temp_db();
    vantadb::cli_handlers::cmd_put(
        &path,
        "vec_ns",
        "v1",
        "data",
        Some("1.0,2.0,3.0"),
        None,
        false,
        false,
    )
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
    let result =
        vantadb::cli_handlers::cmd_put(&path, "ns", "k", "data", Some("abc"), None, false, false);
    assert!(result.is_err(), "expected error for invalid vector");
}

#[test]
fn test_put_with_metadata_roundtrip() {
    let (_dir, path) = setup_temp_db();
    vantadb::cli_handlers::cmd_put(
        &path,
        "meta_ns",
        "m1",
        "data with meta",
        None,
        Some(r#"{"color":"blue","count":2,"active":true}"#),
        false,
        false,
    )
    .expect("put with metadata failed");

    // Verify via the SDK: metadata fields are read back as record metadata
    let config = vantadb::config::Config {
        storage_path: path.clone(),
        read_only: true,
        ..Default::default()
    };
    let db = vantadb::Embedded::open_with_config(config).expect("open read-only");
    let record = db
        .get("meta_ns", "m1")
        .expect("get record")
        .expect("record exists");
    assert_eq!(
        record.metadata.get("color"),
        Some(&vantadb::sdk::Value::String("blue".into()))
    );
    assert_eq!(
        record.metadata.get("count"),
        Some(&vantadb::sdk::Value::Int(2))
    );
    assert_eq!(
        record.metadata.get("active"),
        Some(&vantadb::sdk::Value::Bool(true))
    );
}

#[test]
fn test_put_metadata_rejects_reserved_prefix() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_put(
        &path,
        "meta_ns",
        "bad",
        "data",
        None,
        Some(r#"{"__vanta_payload":"spoofed"}"#),
        false,
        false,
    );
    assert!(
        result.is_err(),
        "reserved __vanta_ metadata key must be rejected"
    );
}

#[test]
fn test_put_metadata_invalid_json() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_put(
        &path,
        "meta_ns",
        "bad2",
        "data",
        None,
        Some(r##"{"color": "##),
        false,
        false,
    );
    assert!(result.is_err(), "malformed metadata JSON must be rejected");
}

#[test]
fn test_list_empty_namespace() {
    let (_dir, path) = setup_temp_db();
    // list on empty db should succeed (prints warning)
    let result = vantadb::cli_handlers::cmd_list(&path, "empty_ns", 10, false, false);
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
    let result = vantadb::cli_handlers::cmd_list(&path, "ns1", 2, false, false);
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
    let result = vantadb::cli_handlers::cmd_delete(&path, "del_ns", "del_key", false, false);
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
    let result = vantadb::cli_handlers::cmd_delete(&path, "ns", "missing", false, false);
    assert!(result.is_ok());
}

#[test]
fn test_delete_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "v_ns", "v_key", "verbose delete");
    let result = vantadb::cli_handlers::cmd_delete(&path, "v_ns", "v_key", true, false);
    assert!(result.is_ok());
}

// ─── search ───────────────────────────────────────────────────

#[test]
fn test_search_no_results() {
    let (_dir, path) = setup_temp_db();
    // seed via embedded to build text index
    seed_embedded(&path, "srch_ns", "k1", "hello world");
    let result =
        vantadb::cli_handlers::cmd_search(&path, "srch_ns", "zzz_nonexistent", None, 10, false);
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
    let result = vantadb::cli_handlers::cmd_search(&path, "search_ns", "banana", None, 10, false);
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(result.is_ok(), "search should find results: {:?}", result);
}

// ─── namespace ────────────────────────────────────────────────

#[test]
fn test_namespace_list_empty() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_namespace_list(&path, false);
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

    let result = vantadb::cli_handlers::cmd_namespace_info(&path, "info_ns", false);
    assert!(result.is_ok());
}

#[test]
fn test_namespace_info_empty() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_namespace_info(&path, "empty_ns", false);
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
        let result = vantadb::cli_handlers::cmd_status(&tmp_path, false, false);
        assert!(result.is_ok());
        return;
    }
    let result = vantadb::cli_handlers::cmd_status(path, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_status_with_db() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "st_ns", "st_k", "status test");
    let result = vantadb::cli_handlers::cmd_status(&path, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_status_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "st_v", "k", "verbose status");
    let result = vantadb::cli_handlers::cmd_status(&path, true, false);
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
    let result = vantadb::cli_handlers::cmd_get("./ghost_dir", "ns", "k", false, false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_list_missing_db() {
    let result = vantadb::cli_handlers::cmd_list("./ghost_dir", "ns", 10, false, false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_delete_missing_db() {
    let result = vantadb::cli_handlers::cmd_delete("./ghost_dir", "ns", "k", false, false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_search_missing_db() {
    let result = vantadb::cli_handlers::cmd_search("./ghost_dir", "ns", "q", None, 10, false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_namespace_list_missing_db() {
    let result = vantadb::cli_handlers::cmd_namespace_list("./ghost_dir", false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_cmd_namespace_info_missing_db() {
    let result = vantadb::cli_handlers::cmd_namespace_info("./ghost_dir", "ns", false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

// ─── rebuild / export / import / query ─────────────────────────

#[test]
fn test_cmd_rebuild_index_empty() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_rebuild_index(&path, false, false);
    if let Err(e) = &result {
        eprintln!("REBUILD ERROR: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_cmd_export_and_import() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "ex_ns", "k1", "export me");

    let export_path = format!("{}/export.json", path);
    let result = vantadb::cli_handlers::cmd_export(&path, Some("ex_ns"), &export_path, false);
    assert!(result.is_ok(), "export failed");
    assert!(Path::new(&export_path).exists(), "export file missing");

    // import into a fresh db
    let (_dir2, path2) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_import(&path2, &export_path, false, false);
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
fn test_cmd_export_writes_integrity_manifest() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "ex_ns", "k1", "export me");

    let export_path = format!("{}/integrity-export.json", path);
    let result = vantadb::cli_handlers::cmd_export(&path, Some("ex_ns"), &export_path, false);
    assert!(result.is_ok(), "export failed");

    let manifest_path = format!("{export_path}.manifest.json");
    assert!(
        Path::new(&manifest_path).exists(),
        "manifest sidecar missing: {manifest_path}"
    );

    // The SDK verifies the CLI-written export against the same contract.
    let config = vantadb::config::Config {
        storage_path: path.clone(),
        read_only: true,
        ..Default::default()
    };
    let db = vantadb::Embedded::open_with_config(config).expect("open embedded");
    let verification = db
        .verify_export_integrity(&export_path)
        .expect("verify export");
    assert_eq!(verification.status, "ok");
    assert_eq!(verification.records, 1);
    assert_eq!(verification.namespaces, vec!["ex_ns".to_string()]);
    assert!(!verification.limits.is_empty());
}

#[test]
fn test_cmd_query_empty_db() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_query(&path, "FROM Persona", 10, false, false);
    if let Err(e) = &result {
        eprintln!("ERROR: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "query on empty db should succeed: {:?}",
        result
    );
}

// FIND-101: mutating IQL via `query` must open the database read-write.
#[test]
fn test_find101_query_insert_mutates() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_query(
        &path,
        r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros" }"#,
        10,
        false,
        false,
    );
    assert!(
        result.is_ok(),
        "INSERT via query should succeed: {:?}",
        result
    );
    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    assert!(
        engine.get(101).unwrap().is_some(),
        "node inserted via query should be readable"
    );
    drop(engine);

    // UPDATE via query mutates the same node.
    let result = vantadb::cli_handlers::cmd_query(
        &path,
        r#"UPDATE NODE#101 SET nombre = "Eros Dev""#,
        10,
        false,
        false,
    );
    assert!(
        result.is_ok(),
        "UPDATE via query should succeed: {:?}",
        result
    );
    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    let node = engine.get(101).unwrap().expect("node should still exist");
    assert_eq!(
        node.relational.get("nombre"),
        Some(&vantadb::node::FieldValue::String("Eros Dev".to_string())),
        "UPDATE via query should change the field"
    );
    drop(engine);

    // DELETE via query tombstones the node.
    let result = vantadb::cli_handlers::cmd_query(&path, "DELETE NODE#101", 10, false, false);
    assert!(
        result.is_ok(),
        "DELETE via query should succeed: {:?}",
        result
    );
    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    assert!(
        engine.get(101).unwrap().is_none(),
        "node deleted via query should be gone"
    );
    drop(engine);

    // Reads still work (read-only path unchanged).
    let result = vantadb::cli_handlers::cmd_query(&path, "FROM Usuario", 10, false, false);
    assert!(
        result.is_ok(),
        "SELECT via query should succeed: {:?}",
        result
    );
}

// ─── backup / restore ────────────────────────────────────────

#[test]
fn test_backup_and_restore() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "bkp_ns", "k1", "backup test");

    // Verify source has the record before backup
    let src_engine =
        vantadb::cli_handlers::open_database(&path, false).expect("source DB should open");
    let src_node_id = vantadb::cli_handlers::memory_node_id("bkp_ns", "k1");
    let src_node = src_engine
        .get(src_node_id)
        .expect("source read should succeed");
    assert!(
        src_node.is_some(),
        "source record should exist before backup"
    );
    eprintln!("DEBUG: source node_id = {}", src_node_id);
    drop(src_engine);

    let backup_dir = format!("{}/test_backup", path);

    // Create backup
    let result = vantadb::cli_handlers::cmd_backup(&path, Some(&backup_dir), false, false);
    assert!(result.is_ok(), "backup should succeed: {:?}", result);

    // Verify backup directory exists
    assert!(std::path::Path::new(&backup_dir).exists());

    // List backup files
    eprintln!("DEBUG: backup files:");
    for entry in std::fs::read_dir(&backup_dir).unwrap() {
        let e = entry.unwrap();
        eprintln!("  {}", e.path().display());
    }
    let bdata_dir = std::path::Path::new(&backup_dir).join("data");
    if bdata_dir.exists() {
        eprintln!("DEBUG: backup data/ files:");
        for entry in std::fs::read_dir(&bdata_dir).unwrap() {
            let e = entry.unwrap();
            eprintln!(
                "  {} ({} bytes)",
                e.path().display(),
                e.metadata().unwrap().len()
            );
        }
    }

    // Check that backup vector_index.bin has valid content
    let backup_idx = bdata_dir.join("vector_index.bin");
    if backup_idx.exists() {
        let idx_data = std::fs::read(&backup_idx).unwrap();
        eprintln!(
            "DEBUG: backup vector_index.bin size = {} bytes, first 4 bytes = {:?}",
            idx_data.len(),
            &idx_data[..4.min(idx_data.len())]
        );
    }

    // Read source vector_index.bin to compare with backup
    let src_data_dir = std::path::Path::new(&path).join("data");
    if src_data_dir.exists() {
        for entry in std::fs::read_dir(&src_data_dir).unwrap() {
            let e = entry.unwrap();
            eprintln!(
                "DEBUG: source data/ {} ({} bytes)",
                e.path().display(),
                e.metadata().unwrap().len()
            );
        }
    }

    // Try rebuild approach
    let restore_path_rebuild = format!("{}/restored_rebuild", path);
    let result = vantadb::cli_handlers::cmd_restore(
        &restore_path_rebuild,
        &backup_dir,
        vantadb::cli_handlers::RestoreOptions {
            overwrite: vantadb::cli_handlers::OverwritePolicy::Overwrite,
            verbose: vantadb::cli_handlers::Verbosity::Verbose,
            ..Default::default()
        },
        false,
    );
    assert!(
        result.is_ok(),
        "restore with rebuild should succeed: {:?}",
        result
    );
    let engine_rb = vantadb::cli_handlers::open_database(&restore_path_rebuild, false)
        .expect("restored DB (rebuild) should open writable");
    let node_id = vantadb::cli_handlers::memory_node_id("bkp_ns", "k1");
    eprintln!("DEBUG: restoring node_id = {}", node_id);
    let node_rb = engine_rb
        .get(node_id)
        .expect("db read should succeed (rebuild)");
    assert!(node_rb.is_some(), "restored record should exist (rebuild)");

    // Also try without rebuild (original path)
    let restore_path = format!("{}/restored", path);
    let result = vantadb::cli_handlers::cmd_restore(
        &restore_path,
        &backup_dir,
        vantadb::cli_handlers::RestoreOptions {
            overwrite: vantadb::cli_handlers::OverwritePolicy::Overwrite,
            ..Default::default()
        },
        false,
    );
    assert!(result.is_ok(), "restore should succeed: {:?}", result);
    assert!(std::path::Path::new(&restore_path).exists());
    assert!(std::path::Path::new(&restore_path)
        .join(".vanta.lock")
        .exists());
    eprintln!("DEBUG: restored files:");
    for entry in std::fs::read_dir(&restore_path).unwrap() {
        let e = entry.unwrap();
        eprintln!("  {}", e.path().display());
    }
    let rdata_dir = std::path::Path::new(&restore_path).join("data");
    if rdata_dir.exists() {
        eprintln!("DEBUG: restored data/ files:");
        for entry in std::fs::read_dir(&rdata_dir).unwrap() {
            let e = entry.unwrap();
            eprintln!(
                "  {} ({} bytes)",
                e.path().display(),
                e.metadata().unwrap().len()
            );
        }
    }
    let engine = vantadb::cli_handlers::open_database(&restore_path, false)
        .expect("restored DB should open writable");
    eprintln!("DEBUG: restoring node_id = {}", node_id);
    let node = engine.get(node_id).expect("db read should succeed");
    assert!(node.is_some(), "restored record should exist");
    let payload = node
        .unwrap()
        .relational
        .get(vantadb::cli_handlers::FIELD_PAYLOAD)
        .and_then(|v| match v {
            vantadb::node::FieldValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .expect("payload not found");
    assert_eq!(payload, "backup test");
}

#[test]
fn test_backup_nonexistent_db_path() {
    // Opening a writable database creates the directory, so this succeeds
    // but the backup target directory shouldn't exist
    let result = vantadb::cli_handlers::cmd_backup(
        "./ghost_backup_dir",
        Some("./ghost_backup_out"),
        false,
        false,
    );
    if let Err(e) = &result {
        eprintln!("ERROR (non-fatal for this test): {:?}", e);
    }
    // Cleanup
    let _ = std::fs::remove_dir_all("./ghost_backup_dir");
    let _ = std::fs::remove_dir_all("./ghost_backup_out");
}

#[test]
fn test_restore_missing_backup() {
    let result = vantadb::cli_handlers::cmd_restore(
        "./dummy",
        "./nonexistent_backup",
        vantadb::cli_handlers::RestoreOptions {
            overwrite: vantadb::cli_handlers::OverwritePolicy::Overwrite,
            ..Default::default()
        },
        false,
    );
    assert!(
        result.is_err(),
        "restore from non-existent backup should error"
    );
}

#[test]
fn restore_dry_run_missing_backup_errors() {
    // RED: dry-run with nonexistent backup must error clearly (no mutation possible).
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("tgt").to_string_lossy().to_string();
    let missing = dir
        .path()
        .join("no_such_backup")
        .to_string_lossy()
        .to_string();
    let result = vantadb::cli_handlers::cmd_restore(
        &target,
        &missing,
        vantadb::cli_handlers::RestoreOptions {
            mode: vantadb::cli_handlers::RestoreMode::DryRun,
            ..Default::default()
        },
        false,
    );
    assert!(result.is_err(), "dry-run missing backup should error");
    let msg = format!("{:?}", result.unwrap_err());
    assert!(
        msg.contains("does not exist"),
        "error must be clear about missing backup, got: {msg}"
    );
    assert!(
        !std::path::Path::new(&target).exists(),
        "dry-run error must not create target"
    );
}

fn snapshot_files_sorted(root: &std::path::Path) -> Vec<(String, u64)> {
    fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<(String, u64)>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                let rel = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                out.push((rel, size));
            }
        }
    }
    let mut out = Vec::new();
    if root.exists() {
        walk(root, root, &mut out);
    }
    out.sort();
    out
}

#[test]
fn restore_dry_run_lists_without_mutating() {
    // RED: valid backup + existing target → dry-run Ok, target UNTOUCHED.
    let (_src_dir, src_path) = setup_temp_db();
    seed_record(&src_path, "dry_ns", "k_src", "src payload");
    let backup_dir = format!("{}/dry_backup", src_path);
    vantadb::cli_handlers::cmd_backup(&src_path, Some(&backup_dir), false, false)
        .expect("backup should succeed");

    let (_tgt_dir, tgt_path) = setup_temp_db();
    seed_record(&tgt_path, "other_ns", "k_tgt", "target original");

    let tgt_root = std::path::Path::new(&tgt_path).to_path_buf();
    let before_files = snapshot_files_sorted(&tgt_root);
    let before_size =
        vantadb::cli_handlers::dir_size(&tgt_root).expect("dir_size before should succeed");
    assert!(
        !before_files.is_empty(),
        "target snapshot must not be empty"
    );

    // dry-run without --force on an existing target must still succeed (preview).
    let result = vantadb::cli_handlers::cmd_restore(
        &tgt_path,
        &backup_dir,
        vantadb::cli_handlers::RestoreOptions {
            mode: vantadb::cli_handlers::RestoreMode::DryRun,
            ..Default::default()
        },
        false,
    );
    assert!(result.is_ok(), "dry-run should succeed: {:?}", result);

    let after_files = snapshot_files_sorted(&tgt_root);
    let after_size =
        vantadb::cli_handlers::dir_size(&tgt_root).expect("dir_size after should succeed");
    assert_eq!(
        before_files, after_files,
        "dry-run must leave target files identical"
    );
    assert_eq!(
        before_size, after_size,
        "dry-run must leave target size identical"
    );

    // Target content untouched: original record present, backup record absent.
    let engine =
        vantadb::cli_handlers::open_database(&tgt_path, true).expect("target should still open");
    let tgt_id = vantadb::cli_handlers::memory_node_id("other_ns", "k_tgt");
    let tgt_node = engine.get(tgt_id).expect("read should succeed");
    assert!(
        tgt_node.is_some(),
        "original target record must survive dry-run"
    );
    let src_id = vantadb::cli_handlers::memory_node_id("dry_ns", "k_src");
    let src_node = engine.get(src_id).expect("read should succeed");
    assert!(
        src_node.is_none(),
        "backup record must NOT appear after dry-run"
    );
}

// ─── doctor ───────────────────────────────────────────────────

/// D2 helper: build `DoctorOptions` without bool-flag call sites.
fn doc_opts(
    fix: vantadb::cli_handlers::DoctorFix,
    verbose: vantadb::cli_handlers::Verbosity,
) -> vantadb::cli_handlers::DoctorOptions {
    vantadb::cli_handlers::DoctorOptions { fix, verbose }
}

#[test]
fn test_doctor_no_db() {
    let path = "./nonexistent_doctor_test_dir_should_not_exist";
    if std::path::Path::new(path).exists() {
        eprintln!("WARNING: test directory exists, using temp dir instead");
        let (_dir, tmp_path) = setup_temp_db();
        let result = vantadb::cli_handlers::cmd_doctor(
            &tmp_path,
            doc_opts(
                vantadb::cli_handlers::DoctorFix::Off,
                vantadb::cli_handlers::Verbosity::Normal,
            ),
            false,
        );
        assert!(result.is_ok());
        return;
    }
    let result = vantadb::cli_handlers::cmd_doctor(
        path,
        doc_opts(
            vantadb::cli_handlers::DoctorFix::Off,
            vantadb::cli_handlers::Verbosity::Normal,
        ),
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_doctor_with_db() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "doc_ns", "k1", "doctor test");
    let result = vantadb::cli_handlers::cmd_doctor(
        &path,
        doc_opts(
            vantadb::cli_handlers::DoctorFix::Off,
            vantadb::cli_handlers::Verbosity::Normal,
        ),
        false,
    );
    assert!(result.is_ok(), "doctor should succeed: {:?}", result);
}

#[test]
fn test_doctor_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "doc_v_ns", "k1", "verbose doctor");
    let result = vantadb::cli_handlers::cmd_doctor(
        &path,
        doc_opts(
            vantadb::cli_handlers::DoctorFix::Off,
            vantadb::cli_handlers::Verbosity::Verbose,
        ),
        false,
    );
    assert!(
        result.is_ok(),
        "doctor verbose should succeed: {:?}",
        result
    );
}

#[test]
fn test_doctor_fix_dry_run_no_mutation() {
    // RED: --fix without --force must list repairs and change nothing.
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("ghost_db").to_string_lossy().to_string();
    assert!(!std::path::Path::new(&missing).exists());
    let result = vantadb::cli_handlers::cmd_doctor(
        &missing,
        doc_opts(
            vantadb::cli_handlers::DoctorFix::DryRun,
            vantadb::cli_handlers::Verbosity::Normal,
        ),
        false,
    );
    assert!(
        result.is_ok(),
        "doctor --fix dry-run should succeed: {:?}",
        result
    );
    assert!(
        !std::path::Path::new(&missing).exists(),
        "dry-run must not create directories"
    );
}

#[test]
fn test_doctor_fix_force_creates_dirs() {
    // GREEN: --fix --force creates the missing database directory (+ data/).
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("fixed_db").to_string_lossy().to_string();
    let result = vantadb::cli_handlers::cmd_doctor(
        &missing,
        doc_opts(
            vantadb::cli_handlers::DoctorFix::Apply,
            vantadb::cli_handlers::Verbosity::Normal,
        ),
        false,
    );
    assert!(
        result.is_ok(),
        "doctor --fix --force should succeed: {:?}",
        result
    );
    assert!(
        std::path::Path::new(&missing).exists(),
        "force must create the database directory"
    );
}

#[test]
fn test_doctor_fix_nothing_to_fix() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "doc_fix_ns", "k1", "healthy");
    // Healthy DB: dry-run and force both exit 0 with "nothing to fix".
    let dry = vantadb::cli_handlers::cmd_doctor(
        &path,
        doc_opts(
            vantadb::cli_handlers::DoctorFix::DryRun,
            vantadb::cli_handlers::Verbosity::Normal,
        ),
        false,
    );
    assert!(dry.is_ok(), "dry-run on healthy db: {:?}", dry);
    let forced = vantadb::cli_handlers::cmd_doctor(
        &path,
        doc_opts(
            vantadb::cli_handlers::DoctorFix::Apply,
            vantadb::cli_handlers::Verbosity::Normal,
        ),
        false,
    );
    assert!(forced.is_ok(), "force on healthy db: {:?}", forced);
}

// ─── inspect ──────────────────────────────────────────────────

#[test]
fn test_inspect_record() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "ins_ns", "ins_k", "inspect me");

    let result = vantadb::cli_handlers::cmd_inspect(&path, "ins_ns", "ins_k", false, false);
    assert!(result.is_ok(), "inspect should succeed: {:?}", result);
}

#[test]
fn test_inspect_nonexistent() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_inspect(&path, "ins_ns", "missing", false, false);
    assert!(
        result.is_ok(),
        "inspect missing record should warn: {:?}",
        result
    );
}

#[test]
fn test_inspect_missing_db() {
    let result = vantadb::cli_handlers::cmd_inspect("./ghost_inspect_dir", "ns", "k", false, false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

// ─── stats ────────────────────────────────────────────────────

#[test]
fn test_stats_no_db() {
    let path = "./nonexistent_stats_dir_should_not_exist";
    if std::path::Path::new(path).exists() {
        eprintln!("WARNING: test directory exists, using temp dir instead");
        let (_dir, tmp_path) = setup_temp_db();
        let result = vantadb::cli_handlers::cmd_stats(&tmp_path, false, false);
        assert!(result.is_ok());
        return;
    }
    let result = vantadb::cli_handlers::cmd_stats(path, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_stats_with_db() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "st_ns", "st_k", "stats test");
    let result = vantadb::cli_handlers::cmd_stats(&path, false, false);
    assert!(result.is_ok(), "stats should succeed: {:?}", result);
}

#[test]
fn test_stats_json() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "stj_ns", "stj_k", "json stats");
    let result = vantadb::cli_handlers::cmd_stats(&path, true, false);
    assert!(result.is_ok(), "stats json should succeed: {:?}", result);
}

#[test]
fn test_stats_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "stv_ns", "stv_k", "verbose stats");
    let result = vantadb::cli_handlers::cmd_stats(&path, false, true);
    assert!(result.is_ok(), "stats verbose should succeed: {:?}", result);
}

// ─── verbose mode ─────────────────────────────────────────────

#[test]
fn test_put_verbose() {
    let (_dir, path) = setup_temp_db();
    let result =
        vantadb::cli_handlers::cmd_put(&path, "v_ns", "v_key", "verbose", None, None, true, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_verbose() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "lv_ns", "k", "data");
    let result = vantadb::cli_handlers::cmd_list(&path, "lv_ns", 10, true, false);
    assert!(result.is_ok());
}

// ─── count / delete-by-filter (metadata filters) ───────────────

fn seed_embedded_with_meta(db_path: &str, namespace: &str, key: &str, payload: &str, color: &str) {
    use vantadb::sdk::{MemoryInput, MemoryMetadata, Value};

    let config = vantadb::config::Config {
        storage_path: db_path.to_string(),
        read_only: false,
        ..Default::default()
    };
    let db = vantadb::Embedded::open_with_config(config).expect("seed embedded open failed");
    let mut metadata = MemoryMetadata::new();
    metadata.insert("color".to_string(), Value::String(color.to_string()));
    db.put(MemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: payload.to_string(),
        metadata,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed embedded put failed");
    // ERR-050b: put is buffered in WAL; a later read-only reopen (open_database)
    // does NOT replay WAL (recover_state skips it for read_only). Without an
    // explicit flush the seed record stays invisible to read-only handles,
    // failing "must survive" / "should remain" assertions.
    db.flush().expect("seed embedded flush failed");
}

#[test]
fn test_count_plain_and_json() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "cnt_ns", "a", "data");
    seed_record(&path, "cnt_ns", "b", "data");
    seed_record(&path, "cnt_ns", "c", "data");

    let result = vantadb::cli_handlers::cmd_count(&path, "cnt_ns", None, false, false);
    assert!(result.is_ok(), "count should succeed: {:?}", result);

    let result_json = vantadb::cli_handlers::cmd_count(&path, "cnt_ns", None, true, false);
    assert!(
        result_json.is_ok(),
        "count json should succeed: {:?}",
        result_json
    );
}

#[test]
fn test_count_with_filter() {
    let (_dir, path) = setup_temp_db();
    seed_embedded_with_meta(&path, "cntf_ns", "k1", "payload one", "red");
    seed_embedded_with_meta(&path, "cntf_ns", "k2", "payload two", "blue");

    // filters AND-combined on metadata — "color == red" matches exactly 1
    let filter = r#"{"color":{"$eq":"red"}}"#;
    let result = vantadb::cli_handlers::cmd_count(&path, "cntf_ns", Some(filter), true, false);
    assert!(
        result.is_ok(),
        "count with filter should succeed: {:?}",
        result
    );

    // unmatched filter is a no-op path, not an error
    let no_match = r#"{"color":{"$eq":"green"}}"#;
    let result = vantadb::cli_handlers::cmd_count(&path, "cntf_ns", Some(no_match), false, false);
    assert!(result.is_ok(), "count with unmatched filter: {:?}", result);
}

#[test]
fn test_count_with_flat_filter_alias() {
    // AUD-048: flat values are accepted as implicit `$eq`, matching the MCP
    // channel's published flat semantics — `{"color":"red"}` ≡ `{"color":{"$eq":"red"}}`.
    let (_dir, path) = setup_temp_db();
    seed_embedded_with_meta(&path, "cntflat_ns", "k1", "payload one", "red");
    seed_embedded_with_meta(&path, "cntflat_ns", "k2", "payload two", "blue");

    // count with the flat form must parse and run (no operator-object error).
    let filter = r#"{"color":"red"}"#;
    let result = vantadb::cli_handlers::cmd_count(&path, "cntflat_ns", Some(filter), true, false);
    assert!(
        result.is_ok(),
        "count with flat filter should succeed: {:?}",
        result
    );

    // delete-by-filter with the flat form removes exactly the red record —
    // proving the flat filter matched by value, not by accident.
    let del =
        vantadb::cli_handlers::cmd_delete_by_filter(&path, "cntflat_ns", filter, false, false);
    assert!(
        del.is_ok(),
        "delete-by-filter with flat filter should succeed: {:?}",
        del
    );

    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    let red_id = vantadb::cli_handlers::memory_node_id("cntflat_ns", "k1");
    let blue_id = vantadb::cli_handlers::memory_node_id("cntflat_ns", "k2");
    assert!(
        engine.get(red_id).unwrap().is_none(),
        "flat filter must delete the red record"
    );
    assert!(
        engine.get(blue_id).unwrap().is_some(),
        "blue record must remain"
    );
}

#[test]
fn test_delete_by_filter() {
    let (_dir, path) = setup_temp_db();
    seed_embedded_with_meta(&path, "dbf_ns", "k1", "payload one", "red");
    seed_embedded_with_meta(&path, "dbf_ns", "k2", "payload two", "blue");

    // delete the red record via metadata filter
    let filter = r#"{"color":{"$eq":"red"}}"#;
    let result = vantadb::cli_handlers::cmd_delete_by_filter(&path, "dbf_ns", filter, false, false);
    assert!(
        result.is_ok(),
        "delete-by-filter should succeed: {:?}",
        result
    );

    // red record gone, blue record still present
    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    let red_id = vantadb::cli_handlers::memory_node_id("dbf_ns", "k1");
    let blue_id = vantadb::cli_handlers::memory_node_id("dbf_ns", "k2");
    assert!(
        engine.get(red_id).unwrap().is_none(),
        "red should be deleted"
    );
    assert!(engine.get(blue_id).unwrap().is_some(), "blue should remain");
}

#[test]
fn test_delete_by_filter_no_match() {
    let (_dir, path) = setup_temp_db();
    seed_embedded_with_meta(&path, "dbfn_ns", "k1", "payload", "red");

    let filter = r#"{"color":{"$eq":"purple"}}"#;
    let result =
        vantadb::cli_handlers::cmd_delete_by_filter(&path, "dbfn_ns", filter, false, false);
    assert!(
        result.is_ok(),
        "no-match delete should be a no-op: {:?}",
        result
    );

    let engine = vantadb::cli_handlers::open_database(&path, true).unwrap();
    let id = vantadb::cli_handlers::memory_node_id("dbfn_ns", "k1");
    assert!(engine.get(id).unwrap().is_some(), "record must survive");
}

#[test]
fn test_delete_by_filter_missing_db() {
    let result =
        vantadb::cli_handlers::cmd_delete_by_filter("./ghost_dir", "ns", "{}", false, false);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_count_missing_db_errors() {
    // API-07: `count` on a missing database is an error (exit≠0 via the CLI),
    // so scripts can tell "no database" apart from a real count of 0.
    let result = vantadb::cli_handlers::cmd_count("./ghost_dir", "ns", None, false, false);
    assert!(
        result.is_err(),
        "missing db must error for count (API-07 contract)"
    );
}

// ─── vector similarity / multi-namespace search ────────────────

#[test]
fn test_similar_to_key() {
    let (_dir, path) = setup_temp_db();
    vantadb::cli_handlers::cmd_put(
        &path,
        "sim_ns",
        "v1",
        "vector record",
        Some("1.0,2.0,3.0"),
        None,
        false,
        false,
    )
    .expect("put with vector failed");

    let result = vantadb::cli_handlers::cmd_similar_to_key(&path, "sim_ns", "v1", 5, false);
    assert!(
        result.is_ok(),
        "similar-to-key should succeed: {:?}",
        result
    );

    let result_json = vantadb::cli_handlers::cmd_similar_to_key(&path, "sim_ns", "v1", 5, true);
    assert!(
        result_json.is_ok(),
        "similar-to-key json: {:?}",
        result_json
    );
}

#[test]
fn test_similar_to_key_missing_db() {
    let result = vantadb::cli_handlers::cmd_similar_to_key("./ghost_dir", "ns", "k", 5, true);
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_search_multi() {
    let (_dir, path) = setup_temp_db();
    seed_embedded(&path, "m1", "r1", "apple banana");
    seed_embedded(&path, "m2", "r2", "banana cherry");

    let result = vantadb::cli_handlers::search::cmd_search_multi(
        &path,
        "m1,m2",
        Some("banana"),
        None,
        10,
        false,
    );
    assert!(result.is_ok(), "search-multi should succeed: {:?}", result);

    let result_json = vantadb::cli_handlers::search::cmd_search_multi(
        &path,
        "m1,m2",
        None,
        Some("1.0,2.0,3.0"),
        10,
        true,
    );
    assert!(result_json.is_ok(), "search-multi json: {:?}", result_json);
}

#[test]
fn test_search_multi_missing_db() {
    let result = vantadb::cli_handlers::search::cmd_search_multi(
        "./ghost_dir",
        "ns1,ns2",
        Some("q"),
        None,
        10,
        true,
    );
    assert!(result.is_ok(), "missing db should warn, not error");
}

#[test]
fn test_search_all() {
    let (_dir, path) = setup_temp_db();
    seed_embedded(&path, "sa1", "r1", "apple banana");
    seed_embedded(&path, "sa2", "r2", "banana cherry");

    let result =
        vantadb::cli_handlers::search::cmd_search_all(&path, Some("banana"), None, 10, false);
    assert!(result.is_ok(), "search-all should succeed: {:?}", result);

    let result_json =
        vantadb::cli_handlers::search::cmd_search_all(&path, None, Some("1.0,2.0,3.0"), 10, true);
    assert!(result_json.is_ok(), "search-all json: {:?}", result_json);
}

#[test]
fn test_search_all_missing_db() {
    let result =
        vantadb::cli_handlers::search::cmd_search_all("./ghost_dir", Some("q"), None, 10, true);
    assert!(result.is_ok(), "missing db should warn, not error");
}

// ─── index audit / repair ──────────────────────────────────────

#[test]
fn test_audit_index() {
    let (_dir, path) = setup_temp_db();
    seed_embedded(&path, "aud_ns", "k1", "apple banana");

    let result = vantadb::cli_handlers::cmd_audit_index(&path, None, false, false);
    assert!(result.is_ok(), "audit-index should succeed: {:?}", result);

    let result_deep = vantadb::cli_handlers::cmd_audit_index(&path, Some("aud_ns"), true, true);
    assert!(
        result_deep.is_ok(),
        "audit-index deep should succeed: {:?}",
        result_deep
    );
}

#[test]
fn test_repair_text_index() {
    let (_dir, path) = setup_temp_db();
    seed_embedded(&path, "rpr_ns", "k1", "repair me");

    let result = vantadb::cli_handlers::cmd_repair_text_index(&path, false);
    assert!(
        result.is_ok(),
        "repair-text-index should succeed: {:?}",
        result
    );
}

// ─── snapshot ──────────────────────────────────────────────────

#[test]
fn test_snapshot_create_and_list() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "snap_ns", "k1", "snapshot me");

    let result = vantadb::cli_handlers::cmd_snapshot_create(&path, "snap1", false, false);
    assert!(
        result.is_ok(),
        "snapshot create should succeed: {:?}",
        result
    );

    let result = vantadb::cli_handlers::cmd_snapshot_list(&path, false);
    assert!(result.is_ok(), "snapshot list should succeed: {:?}", result);
}

#[test]
fn test_snapshot_list_empty() {
    let (_dir, path) = setup_temp_db();
    let result = vantadb::cli_handlers::cmd_snapshot_list(&path, false);
    assert!(result.is_ok(), "snapshot list on empty db: {:?}", result);
}

// ─── wal compact / vacuum ──────────────────────────────────────

#[test]
fn test_wal_compact() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "wal_ns", "k1", "wal data");
    seed_record(&path, "wal_ns", "k2", "more wal data");

    let result = vantadb::cli_handlers::cmd_wal_compact(&path, false);
    assert!(result.is_ok(), "wal compact should succeed: {:?}", result);
}

#[test]
fn test_wal_vacuum() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "vac_ns", "k1", "to delete");
    vantadb::cli_handlers::cmd_delete(&path, "vac_ns", "k1", false, false)
        .expect("delete for vacuum failed");

    let result = vantadb::cli_handlers::cmd_wal_vacuum(&path, false);
    assert!(result.is_ok(), "wal vacuum should succeed: {:?}", result);
}

// ─── migrate plan / check ─────────────────────────────────────

#[test]
fn test_migrate_plan() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "mig_ns", "k1", "migrate me");

    let result = vantadb::cli_handlers::cmd_migrate_plan(&path, false, false);
    assert!(result.is_ok(), "migrate plan should succeed: {:?}", result);
}

#[test]
fn test_migrate_check() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "mig_ns", "k1", "migrate check me");

    let result = vantadb::cli_handlers::cmd_migrate_check(&path, false, false);
    assert!(result.is_ok(), "migrate check should succeed: {:?}", result);
}

// ─── completions ───────────────────────────────────────────────

#[test]
fn test_completions_bash() {
    // writes the completion script to stdout and returns ()
    vantadb::cli_handlers::cmd_completions(vantadb::cli::Shell::Bash);
}

#[test]
fn test_completions_zsh_and_powershell() {
    vantadb::cli_handlers::cmd_completions(vantadb::cli::Shell::Zsh);
    vantadb::cli_handlers::cmd_completions(vantadb::cli::Shell::PowerShell);
}

// ─── COV-003: cmd_migrate (Run) + cmd_server coverage ──────────
// These exercise handler code paths previously at 0% coverage.
// `cmd_migrate` non-dry-run-without-force prompts interactively, so only the
// error and `dry_run` (no-mutation) paths are testable here.

#[test]
fn test_migrate_run_missing_target() {
    // target path does not exist -> early Err branch
    let result = vantadb::cli_handlers::cmd_migrate(
        "./ghost_migrate_target_dir",
        "all",
        true,
        false,
        false,
        false,
    );
    assert!(result.is_err(), "migrate on missing target must error");
}

#[test]
fn test_migrate_run_dry_run_does_not_write_schema() {
    // R1 (cli.rs contract "Preview changes without modifying files"): a
    // dry-run on a header-less directory must NOT create `.vanta.schema`.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().to_string_lossy().to_string();
    let result = vantadb::cli_handlers::cmd_migrate(&path, "all", true, false, false, true);
    assert!(
        result.is_ok(),
        "migrate dry-run (no schema) should succeed: {result:?}"
    );
    assert!(
        !dir.path().join(".vanta.schema").exists(),
        "dry-run must not write the schema header"
    );
}

#[test]
fn test_migrate_run_dry_run_physical() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "migr2_ns", "k1", "data");
    // non-schema format + dry_run -> plan_all prints, no mutation, no prompt
    let result = vantadb::cli_handlers::cmd_migrate(&path, "vfile", true, false, false, false);
    assert!(
        result.is_ok(),
        "migrate dry_run (physical) should succeed: {:?}",
        result
    );
}

/// ADR-046 helper: build a v1-style database (schema header v1 + one record
/// node without the v2 fields) and return the db path.
fn seed_v1_database(dir: &tempfile::TempDir) -> String {
    use vantadb::node::{FieldValue, UnifiedNode};
    use vantadb::schema::StorageHeader;
    use vantadb::storage::StorageEngine;

    let path = dir.path().to_string_lossy().to_string();
    let v1_header = StorageHeader {
        version: 1,
        flags: 0,
        min_compat_version: 1,
    };
    v1_header
        .write_to(&dir.path().join(".vanta.schema"))
        .expect("write v1 header");
    let config = vantadb::config::Config {
        storage_path: path.clone(),
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(&path, Some(config)).expect("open engine");
    let mut node = UnifiedNode::new(42);
    node.set_field(
        vantadb::sdk::FIELD_NAMESPACE,
        FieldValue::String("cli_mig".into()),
    );
    node.set_field(vantadb::sdk::FIELD_KEY, FieldValue::String("k".into()));
    node.set_field(
        vantadb::sdk::FIELD_PAYLOAD,
        FieldValue::String("payload".into()),
    );
    node.set_field(vantadb::sdk::FIELD_CREATED_AT_MS, FieldValue::Int(1000));
    node.set_field(vantadb::sdk::FIELD_UPDATED_AT_MS, FieldValue::Int(1000));
    node.set_field(vantadb::sdk::FIELD_VERSION, FieldValue::Int(1));
    engine.insert(&node).expect("insert v1 node");
    drop(engine);
    path
}

fn read_schema_header(dir: &tempfile::TempDir) -> Option<vantadb::schema::StorageHeader> {
    vantadb::schema::StorageHeader::read_from(&dir.path().join(".vanta.schema"))
        .expect("read header")
}

/// Node 42 carries the v2 marker (`__vanta_valid_at_ms`) after a backfill.
fn v1_node_has_v2_fields(path: &str) -> bool {
    use vantadb::storage::StorageEngine;
    let config = vantadb::config::Config {
        storage_path: path.to_string(),
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(path, Some(config)).expect("open engine");
    engine
        .get(42)
        .expect("get node")
        .expect("node 42")
        .get_field("__vanta_valid_at_ms")
        .is_some()
}

#[test]
fn test_migrate_run_records_backfill_then_all_bumps_header() {
    // ADR-046 §Migration (expand → backfill → bump): `--format records`
    // backfills a v1 database WITHOUT touching the header; `--format all`
    // completes the remaining formats and bumps the header LAST.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = seed_v1_database(&dir);

    // Step 5 of the ADR sequence: backfill only (header stays v1).
    let result = vantadb::cli_handlers::cmd_migrate(&path, "records", false, true, false, true);
    assert!(
        result.is_ok(),
        "records backfill should succeed: {result:?}"
    );
    assert!(v1_node_has_v2_fields(&path), "backfill wrote the v2 fields");
    assert_eq!(
        read_schema_header(&dir).expect("header present").version,
        1,
        "backfill must not bump the header"
    );

    // Step 6: `--format all` completes the migration and bumps the header.
    let result = vantadb::cli_handlers::cmd_migrate(&path, "all", false, true, false, true);
    assert!(result.is_ok(), "migrate all should succeed: {result:?}");
    let header = read_schema_header(&dir).expect("header present");
    assert_eq!(header.version, 2, "schema bumped to v2 after the backfill");
    assert_eq!(header.min_compat_version, 1, "MIN_COMPAT stays at 1");
}

#[test]
fn test_migrate_schema_alone_ensures_backfill_before_bump() {
    // R1: `--format schema` alone must NOT bump the header while the records
    // backfill is pending (a v2 header disables the backfill forever —
    // `records_backfill_pending` = header.version < CURRENT). The CLI ensures
    // the backfill (idempotent) before the bump.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = seed_v1_database(&dir);

    // Dry-run first: header stays v1, nodes stay un-backfilled (still
    // executable afterwards).
    let result = vantadb::cli_handlers::cmd_migrate(&path, "schema", true, false, false, true);
    assert!(result.is_ok(), "schema dry-run should succeed: {result:?}");
    assert_eq!(
        read_schema_header(&dir).expect("header present").version,
        1,
        "schema dry-run must not bump the header"
    );
    assert!(
        !v1_node_has_v2_fields(&path),
        "schema dry-run must not backfill"
    );

    // Real run: ensures the backfill, then bumps.
    let result = vantadb::cli_handlers::cmd_migrate(&path, "schema", false, true, false, true);
    assert!(
        result.is_ok(),
        "schema-only migration should succeed: {result:?}"
    );
    assert!(
        v1_node_has_v2_fields(&path),
        "records backfill ran before the schema bump"
    );
    assert_eq!(
        read_schema_header(&dir).expect("header present").version,
        2,
        "header bumped after the backfill"
    );
}

#[test]
fn test_migrate_all_dry_run_writes_nothing() {
    // R1: `--format all --dry-run` on a v1 database must preview without
    // modifying files — header stays v1 and the nodes stay un-backfilled
    // (the backfill remains executable afterwards).
    let dir = tempfile::tempdir().expect("temp dir");
    let path = seed_v1_database(&dir);

    let result = vantadb::cli_handlers::cmd_migrate(&path, "all", true, false, false, true);
    assert!(result.is_ok(), "dry-run should succeed: {result:?}");
    assert_eq!(
        read_schema_header(&dir).expect("header present").version,
        1,
        "dry-run must not bump the header"
    );
    assert!(
        !v1_node_has_v2_fields(&path),
        "dry-run must not backfill nodes"
    );

    // The backfill is still executable after the dry-run (not disabled).
    let result = vantadb::cli_handlers::cmd_migrate(&path, "records", false, true, false, true);
    assert!(result.is_ok(), "post-dry-run backfill: {result:?}");
    assert!(v1_node_has_v2_fields(&path));
}

#[test]
fn test_migrate_unknown_format() {
    let (_dir, path) = setup_temp_db();
    // valid target but bogus format string -> format parse error branch
    let result = vantadb::cli_handlers::cmd_migrate(&path, "bogus", true, false, false, false);
    assert!(result.is_err(), "unknown format must error");
}

#[test]
fn test_import_cli_v1_fixture_normalizes() {
    // R3 (ADR-046 §D7 path test — CLI): `vanta import` of the committed v1
    // fixture goes through `record_from_export_line` and normalizes the v2
    // fields (valid_at := created_at, invalid_at := superseded_at, D_a).
    let (_dir, path) = setup_temp_db();
    let result =
        vantadb::cli_handlers::cmd_import(&path, "tests/fixtures/export-v1.jsonl", false, true);
    assert!(
        result.is_ok(),
        "v1 fixture import should succeed: {result:?}"
    );

    let config = vantadb::config::Config {
        storage_path: path.clone(),
        read_only: false,
        ..Default::default()
    };
    let db = vantadb::Embedded::open_with_config(config).expect("open imported db");

    let alpha = db.get("legacy", "v1-alpha").expect("get").expect("alpha");
    assert_eq!(alpha.valid_at_ms, 1000);
    assert_eq!(alpha.invalid_at_ms, None);
    assert_eq!(alpha.confidence, 1.0);
    assert_eq!(
        alpha.confidence_class,
        vantadb::sdk::ConfidenceClass::Asserted
    );

    let superseded = db
        .get("legacy", "v1-superseded")
        .expect("get")
        .expect("superseded");
    assert_eq!(superseded.valid_at_ms, 1000);
    assert_eq!(superseded.invalid_at_ms, Some(1500));
    assert_eq!(superseded.superseded_at_ms, Some(1500));
}

#[test]
#[cfg(not(feature = "server"))]
fn test_server_missing_feature() {
    let (_dir, path) = setup_temp_db();
    // Without the `server` feature the http branch returns a Cli.
    // (mcp mode spawns the vantadb-server binary -> not testable in CI.)
    // cfg-gated: with `server` ON (workspace-unified feature builds, e.g.
    // `cargo test` from the root without `-p`) this call starts the real
    // HTTP server and blocks forever.
    let result = vantadb::cli_handlers::cmd_server(
        &path, true, false, None, None, false, false, None, None, false,
    );
    assert!(
        result.is_err(),
        "server without the 'server' feature must error: {:?}",
        result
    );
}

#[test]
fn test_get_existing_record() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "get_ns", "get_key", "payload value");
    let result = vantadb::cli_handlers::cmd_get(&path, "get_ns", "get_key", false, false);
    assert!(
        result.is_ok(),
        "get existing record should succeed: {:?}",
        result
    );
}

#[test]
fn test_query_with_results() {
    let (_dir, path) = setup_temp_db();
    seed_record(&path, "q_ns", "q_key", "queryable payload");
    let result = vantadb::cli_handlers::cmd_query(&path, "FROM q_ns", 10, false, false);
    assert!(
        result.is_ok(),
        "query with data should succeed: {:?}",
        result
    );
}

// ─── API-07: real-binary surface (clap parsing + stdout JSON shapes) ────────
// Handler-level tests cannot observe clap aliases/positionals or the exact
// stdout bytes; these spawn the built `vanta-cli` binary (CARGO_BIN_EXE_* is
// set for integration tests when the `cli` feature builds the bin).

mod api07_cli_binary {
    use std::process::Command;

    fn cli() -> Command {
        Command::new(env!("CARGO_BIN_EXE_vanta-cli"))
    }

    #[test]
    fn count_without_db_exits_nonzero() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("nope").to_string_lossy().to_string();
        let out = cli()
            .args(["--db", &missing, "count", "--namespace", "ns", "--json"])
            .output()
            .expect("spawn vanta-cli count");
        assert!(
            !out.status.success(),
            "`count` on a missing DB must exit≠0 (API-07); got success"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("does not exist"),
            "stderr should explain the missing database, got: {stderr}"
        );
    }

    #[test]
    fn json_is_complete_and_search_operands_are_accepted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().to_string_lossy().to_string();
        // >80 chars so human previews would truncate; --json must be complete.
        let payload = format!("{}tail", "word ".repeat(60));

        let put = cli()
            .args([
                "--db",
                &db,
                "put",
                "--namespace",
                "ns",
                "--key",
                "k1",
                "--payload",
                &payload,
                "--json",
            ])
            .output()
            .expect("spawn vanta-cli put");
        assert!(put.status.success(), "put --json must succeed");
        let put_json: serde_json::Value =
            serde_json::from_slice(&put.stdout).expect("put --json must parse");
        assert_eq!(put_json["status"], "stored");
        assert_eq!(put_json["namespace"], "ns");
        assert_eq!(put_json["version"], 1);

        // Positional QUERY operand (unified with `query <IQL>`).
        let search = cli()
            .args(["--db", &db, "search", "--namespace", "ns", "word", "--json"])
            .output()
            .expect("spawn vanta-cli search");
        assert!(
            search.status.success(),
            "positional QUERY must parse: {}",
            String::from_utf8_lossy(&search.stderr)
        );
        let hits: Vec<serde_json::Value> =
            serde_json::from_slice(&search.stdout).expect("search --json must parse");
        assert_eq!(hits.len(), 1, "one matching record expected");
        assert_eq!(
            hits[0]["payload"].as_str().map(str::len),
            Some(payload.len()),
            "--json payload must be complete (never truncated)"
        );

        // Hidden alias: `--query` keeps working for existing scripts.
        let alias = cli()
            .args([
                "--db",
                &db,
                "search",
                "--namespace",
                "ns",
                "--query",
                "word",
                "--json",
            ])
            .output()
            .expect("spawn vanta-cli search --query");
        assert!(alias.status.success(), "--query alias must still parse");

        // `--top-k` hidden alias for `--limit` (search family).
        let topk = cli()
            .args([
                "--db",
                &db,
                "search",
                "--namespace",
                "ns",
                "word",
                "--top-k",
                "1",
                "--json",
            ])
            .output()
            .expect("spawn vanta-cli search --top-k");
        assert!(topk.status.success(), "--top-k alias must still parse");

        // count --json is a structured object (no bare number).
        let count = cli()
            .args(["--db", &db, "count", "--namespace", "ns", "--json"])
            .output()
            .expect("spawn vanta-cli count");
        assert!(count.status.success());
        let count_json: serde_json::Value =
            serde_json::from_slice(&count.stdout).expect("count --json must parse");
        assert_eq!(count_json["count"], 1);
        assert_eq!(count_json["namespace"], "ns");

        // Human (piped) output must NOT truncate — previews only apply to TTYs.
        let list = cli()
            .args(["--db", &db, "list", "--namespace", "ns"])
            .output()
            .expect("spawn vanta-cli list");
        assert!(list.status.success());
        let list_stdout = String::from_utf8_lossy(&list.stdout);
        assert!(
            list_stdout.contains(&payload),
            "piped human output must be complete (no preview truncation)"
        );
    }

    #[test]
    fn restore_in_alias_and_import_in_alias_parse() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().to_string_lossy().to_string();
        // `import --in` (canonical) with a missing file errors AFTER parsing —
        // exit≠0 proves the flag parsed and the handler ran.
        let out = cli()
            .args(["--db", &db, "import", "--in", "missing.jsonl"])
            .output()
            .expect("spawn vanta-cli import");
        assert!(!out.status.success(), "missing input file must fail");
        // Legacy `--input` alias still parses (same failure, not a clap error).
        let legacy = cli()
            .args(["--db", &db, "import", "--input", "missing.jsonl"])
            .output()
            .expect("spawn vanta-cli import --input");
        assert!(!legacy.status.success());
        let stderr = String::from_utf8_lossy(&legacy.stderr);
        assert!(
            stderr.contains("Input file not found"),
            "legacy --input must reach the handler, got: {stderr}"
        );
    }

    // ─── FIND-237: `migrate check` — global --db fallback vs positional ─────

    #[test]
    fn migrate_check_accepts_global_db_after_subcommand() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().to_string_lossy().to_string();
        // Seed through the binary so the check has a real database to open.
        let put = cli()
            .args([
                "--db",
                &db,
                "put",
                "--namespace",
                "ns",
                "--key",
                "k1",
                "--payload",
                "x",
                "--json",
            ])
            .output()
            .expect("spawn vanta-cli put");
        assert!(put.status.success(), "seed put must succeed");

        // FIND-237 repro: the flag AFTER the subcommand must work via the
        // global --db fallback (previously clap demanded the positional TARGET).
        let out = cli()
            .args(["migrate", "check", "--db", &db])
            .output()
            .expect("spawn vanta-cli migrate check --db");
        assert!(
            out.status.success(),
            "`migrate check --db` must exit 0 via the global fallback; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn migrate_check_positional_target_still_works_and_wins() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().to_string_lossy().to_string();
        let missing = dir.path().join("missing-db").to_string_lossy().to_string();
        let put = cli()
            .args([
                "--db",
                &db,
                "put",
                "--namespace",
                "ns",
                "--key",
                "k1",
                "--payload",
                "x",
                "--json",
            ])
            .output()
            .expect("spawn vanta-cli put");
        assert!(put.status.success(), "seed put must succeed");

        // The positional TARGET must win over --db: pointing --db at a missing
        // path is irrelevant while the positional is present.
        let out = cli()
            .args(["migrate", "check", &db, "--db", &missing])
            .output()
            .expect("spawn vanta-cli migrate check <TARGET>");
        assert!(
            out.status.success(),
            "positional TARGET must win over --db; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
