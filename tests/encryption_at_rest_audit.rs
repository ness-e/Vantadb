// ponytail: blanket allow — test assertions and fixtures with documented invariants.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for Encryption-at-Rest (`TEC-05`, `FIND-615..622`)
//! and Physical Vacuum of HNSW (`EGO-14`, `EGO-16`).

#[cfg(feature = "encryption")]
use std::path::Path;
#[cfg(feature = "encryption")]
use vantadb::node::DiskNodeHeader;
use vantadb::node::DistanceMetric;
#[cfg(feature = "encryption")]
use vantadb::storage::vfile::File;
use vantadb::{BackendKind, Config, Embedded, MemoryInput, MemorySearchRequest};

// ─── Helpers ────────────────────────────────────────────────────────

#[cfg(feature = "encryption")]
fn key_hex(seed: u8) -> String {
    format!("{seed:02x}").repeat(32)
}

#[cfg(feature = "encryption")]
fn scan_files_for_bytes(dir: &Path, needle: &[u8], hits: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_files_for_bytes(&path, needle, hits);
        } else {
            // Ignore lock files which contain no user data
            if path.file_name().and_then(|n| n.to_str()) == Some(".vanta.lock") {
                continue;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                if bytes.windows(needle.len()).any(|w| w == needle) {
                    hits.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
}

// ─── Tarea 1 & Tarea 3.1: TEC-05 Cifrado en Reposo Completo ──────────

#[cfg(feature = "encryption")]
#[test]
fn test_tec05_no_plaintext_on_disk_and_full_recovery() {
    const MARKER: &str = "CLASSIFIED_PAYLOAD_MARKER_998877";
    let dir = tempfile::tempdir().expect("tempdir");
    let key = key_hex(0x7a);

    let config = Config {
        storage_path: dir.path().to_str().unwrap().to_string(),
        backend_kind: BackendKind::Fjall,
        encryption_key: Some(key.clone()),
        ..Config::default()
    };

    // 1. Insert records with classified marker and distinctive vectors
    {
        let db = Embedded::open_with_config(config.clone()).expect("open encrypted db");
        for i in 0..10 {
            let mut input = MemoryInput::new(
                "classified_vault",
                format!("secret_{i}"),
                format!("{MARKER}_RECORD_DATA_{i}"),
            );
            input.vector = Some(vec![i as f32 * 1.5, 2.5, 3.5, 4.5]);
            db.put(input).expect("put encrypted record");
        }
        db.flush().expect("flush to disk");
        db.close().expect("close cleanly");
    }

    // 2. Scan byte-by-byte all files on disk; ensure classified marker is ABSENT
    let mut hits = Vec::new();
    scan_files_for_bytes(dir.path(), MARKER.as_bytes(), &mut hits);
    assert!(
        hits.is_empty(),
        "TEC-05 VIOLATION: Plaintext payload marker found in on-disk files: {:?}",
        hits
    );

    // 3. Reopen with the correct key and verify get and vector search recover all data
    {
        let db = Embedded::open_with_config(config).expect("reopen encrypted db with correct key");
        for i in 0..10 {
            let record = db
                .get("classified_vault", &format!("secret_{i}"))
                .expect("get record")
                .unwrap_or_else(|| panic!("Record secret_{i} must exist after reopen"));
            assert_eq!(
                record.payload,
                format!("{MARKER}_RECORD_DATA_{i}"),
                "Payload must match exactly"
            );
        }

        // Test vector search
        let query_vec = vec![0.0, 2.5, 3.5, 4.5];
        let req = MemorySearchRequest {
            namespace: "classified_vault".into(),
            query_vector: query_vec,
            top_k: 3,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        let results = db.search(req).expect("search");
        assert!(!results.is_empty(), "Search must return results");
        assert_eq!(
            results[0].record.payload,
            format!("{MARKER}_RECORD_DATA_0"),
            "Nearest vector must be secret_0"
        );
    }
}

// ─── Tarea 1 & Tarea 3.2: Guardas FIND-616, FIND-620 y FIND-621 ─────

#[cfg(feature = "encryption")]
#[test]
fn test_find_616_620_621_guards() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_bytes = [0x55u8; 32];
    let vfile_path = dir.path().join("guard_test.vanta");

    // Guard FIND-621: write_header on encrypted segment must return Err
    {
        let mut file = File::open_encrypted(vfile_path.clone(), 64 * 1024, &key_bytes)
            .expect("open encrypted segment");
        assert!(file.is_encrypted(), "Segment must be encrypted v3");

        let header = DiskNodeHeader::new(42);
        let res = file.write_header(64, &header);
        assert!(
            res.is_err(),
            "FIND-621: write_header directly on encrypted v3 segment must return Err"
        );
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains("write_header cannot be called directly on an encrypted v3 segment"),
            "Error detail must state write_header cannot be called directly: got {err_msg}"
        );
    }

    // Guard FIND-616: DecryptCache budget bounding
    {
        // When memory_limit = 32 MiB, budget per segment = 32 MiB / 16 = 2 MiB (<= 2 MiB)
        vantadb::storage::vfile::set_decrypt_cache_memory_limit(32 * 1024 * 1024);
        let budget = vantadb::storage::vfile::decrypt_cache_budget();
        assert!(
            budget <= 2 * 1024 * 1024,
            "FIND-616: budget per segment must be <= 2 MiB for 32 MiB memory_limit, got {budget}"
        );
        assert_eq!(budget, 2 * 1024 * 1024);

        // Reset to unconfigured (0)
        vantadb::storage::vfile::set_decrypt_cache_memory_limit(0);
    }

    // Guard FIND-620: checked_add overflow safety in read_record_bytes
    {
        let file = File::open_encrypted(vfile_path, 64 * 1024, &key_bytes)
            .expect("open encrypted segment");
        // An offset near u64::MAX must return None safely and not panic/overflow
        assert!(
            file.read_header(u64::MAX - 64).is_none(),
            "FIND-620: out-of-bounds offset must return None without overflow"
        );
    }
}

// ─── Tarea 2 & Tarea 3.3: Vacuum Físico EGO-14 / EGO-16 ──────────────

#[test]
fn test_ego14_ego16_physical_vacuum_and_recall() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = Config {
        storage_path: dir.path().to_str().unwrap().to_string(),
        backend_kind: BackendKind::Fjall,
        ..Config::default()
    };

    let db = Embedded::open_with_config(config).expect("open db");

    // 1. Insert 50 nodes with distinctive vectors
    for i in 1..=50 {
        let mut input =
            MemoryInput::new("vacuum_test", format!("node_{i}"), format!("payload_{i}"));
        // Distinctive normalized 4D directional vectors
        let angle = i as f32 * 0.1;
        input.vector = Some(vec![
            angle.cos(),
            angle.sin(),
            (angle * 2.0).cos(),
            (angle * 2.0).sin(),
        ]);
        db.put(input).expect("insert node");
    }
    db.flush().expect("flush before delete");

    // 2. Delete 20 nodes (1..=20)
    for i in 1..=20 {
        assert!(
            db.delete("vacuum_test", &format!("node_{i}"))
                .expect("delete node"),
            "Delete must return true for node_{i}"
        );
    }

    // 3. Run vacuum
    let vacuum_report = db.vacuum().expect("vacuum must succeed");
    assert!(vacuum_report.success, "Vacuum report must declare success");
    assert!(
        vacuum_report.scanned_nodes >= 30,
        "Vacuum must have scanned at least the remaining nodes, scanned: {}",
        vacuum_report.scanned_nodes
    );

    // 4. Verify all 20 deleted nodes are purged
    for i in 1..=20 {
        let rec = db
            .get("vacuum_test", &format!("node_{i}"))
            .expect("get deleted node");
        assert!(
            rec.is_none(),
            "Deleted node_{i} must not be returned by get"
        );
    }

    // 5. Verify the remaining 30 nodes (21..=50) maintain recall == 1.0
    let mut hits_correct = 0;
    for i in 21..=50 {
        let angle = i as f32 * 0.1;
        let query_vec = vec![
            angle.cos(),
            angle.sin(),
            (angle * 2.0).cos(),
            (angle * 2.0).sin(),
        ];
        let req = MemorySearchRequest {
            namespace: "vacuum_test".into(),
            query_vector: query_vec,
            top_k: 1,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        let hits = db.search(req).expect("search");
        assert!(!hits.is_empty(), "Search for node_{i} must yield results");
        if hits[0].record.payload == format!("payload_{i}") {
            hits_correct += 1;
        }
    }

    let recall = hits_correct as f64 / 30.0;
    assert_eq!(
        recall, 1.0,
        "EGO-14/16: Recall across remaining 30 nodes must be 1.0, got {recall}"
    );
}
