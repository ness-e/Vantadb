#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Snapshot certification suite for VantaDB.
//!
//! Tests three critical stability properties:
//! - HNSW recall matches deterministic expected baseline
//! - Export/import format versioning and round-trip fidelity
//! - WAL format integrity via CRC32C and record-level serialization
//!
//! Run with: cargo test --test snapshot_certification -- --nocapture

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs;
use tempfile::tempdir;

use std::time::Instant;
use vantadb::index::{CPIndex, HnswConfig, VectorRepresentations};
use vantadb::node::{
    DiskNodeHeader, DistanceMetric, FieldValue, FilterBitset, NodeFlags, UnifiedNode,
};
use vantadb::sdk::*;
use vantadb::storage::{BackendKind, StorageEngine};
use vantadb::wal::{WalReader, WalRecord, WalWriter};
use vantadb::Embedded;
use zerocopy::FromBytes;

// ═══════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════

fn generate_vectors_seeded(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut vec = Vec::with_capacity(dims);
        for _ in 0..dims {
            vec.push(rng.random_range(-1.0..1.0));
        }
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vectors.push(vec);
    }
    vectors
}

fn brute_force_search(query: &[f32], all_vectors: &[(u128, Vec<f32>)], top_k: usize) -> Vec<u128> {
    let mut distances = Vec::with_capacity(all_vectors.len());
    let query_vector = VectorRepresentations::Full(query.to_vec());
    for (id, vec) in all_vectors {
        let node_vec = VectorRepresentations::Full(vec.clone());
        let sim = query_vector.cosine_similarity(&node_vec).unwrap_or(0.0);
        distances.push((*id, sim));
    }
    distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    distances.truncate(top_k);
    distances.into_iter().map(|(id, _)| id).collect()
}

fn str_value(value: &str) -> Value {
    Value::String(value.to_string())
}

fn record(namespace: &str, key: &str, payload: &str, category: &str) -> MemoryInput {
    let mut input = MemoryInput::new(namespace, key, payload);
    input
        .metadata
        .insert("category".to_string(), str_value(category));
    input.vector = Some(vec![1.0, 0.0, 0.0]);
    input
}

// ═══════════════════════════════════════════════════════════════════
// SECTION 1: HNSW RECALL SNAPSHOT
// ═══════════════════════════════════════════════════════════════════

#[test]
fn hnsw_recall_snapshot_baseline() {
    TerminalReporter::suite_banner("HNSW RECALL SNAPSHOT CERTIFICATION", 1);
    let mut harness = VantaHarness::new("HNSW RECALL SNAPSHOT");

    harness.execute("Deterministic Recall Baseline (5K, D=64, K=10)", || {
        let node_count = 5000;
        let query_count = 100;
        let dims = 64;
        let top_k = 10;
        let seed = 42u64;

        let raw_vectors = generate_vectors_seeded(node_count, dims, seed);
        let query_vectors = generate_vectors_seeded(query_count, dims, seed + 1000);
        let dataset: Vec<(u128, Vec<f32>)> = raw_vectors
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u128, v))
            .collect();

        // Use deterministic config with high recall guarantee
        let config = HnswConfig {
            m: 24,
            m_max0: 48,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (24_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: vantadb::index::IndexType::Hnsw,
            auto_tune: false,
        };
        let index = CPIndex::new_with_config(config);

        for (id, vec) in &dataset {
            index
                .add(
                    *id,
                    FilterBitset::all_set(),
                    VectorRepresentations::Full(vec.clone()),
                    0,
                )
                .expect("test insert");
        }

        // First run: compute baseline recall
        let mut total_recall_run1 = 0.0;
        for query in &query_vectors {
            let true_neighbors = brute_force_search(query, &dataset, top_k);
            let hnsw_results =
                index.search_nearest(query, None, None, &vantadb::node::ALL_BITSET, top_k, None);
            let hnsw_ids: Vec<u128> = hnsw_results.into_iter().map(|(id, _)| id).collect();
            let intersection = true_neighbors
                .iter()
                .filter(|&id| hnsw_ids.contains(id))
                .count();
            total_recall_run1 += intersection as f64 / top_k as f64;
        }
        let mean_recall_run1 = total_recall_run1 / query_count as f64;

        // Second run: must be bitwise identical (determinism)
        let mut total_recall_run2 = 0.0;
        for query in &query_vectors {
            let true_neighbors = brute_force_search(query, &dataset, top_k);
            let hnsw_results =
                index.search_nearest(query, None, None, &vantadb::node::ALL_BITSET, top_k, None);
            let hnsw_ids: Vec<u128> = hnsw_results.into_iter().map(|(id, _)| id).collect();
            let intersection = true_neighbors
                .iter()
                .filter(|&id| hnsw_ids.contains(id))
                .count();
            total_recall_run2 += intersection as f64 / top_k as f64;
        }
        let mean_recall_run2 = total_recall_run2 / query_count as f64;

        TerminalReporter::info(&format!("Recall@10 Run 1: {:.4}", mean_recall_run1));
        TerminalReporter::info(&format!("Recall@10 Run 2: {:.4}", mean_recall_run2));

        // Determinism: both runs must produce identical recall
        assert!(
            (mean_recall_run1 - mean_recall_run2).abs() < 1e-12,
            "HNSW recall snapshot is not deterministic: run1={} run2={}",
            mean_recall_run1,
            mean_recall_run2
        );

        // Quality: recall must exceed the certified threshold
        assert!(
            mean_recall_run1 >= 0.90,
            "HNSW recall snapshot below threshold: {:.4} < 0.90",
            mean_recall_run1
        );

        TerminalReporter::success(&format!(
            "Snapshot recall baseline certified at {:.4} (threshold >= 0.90)",
            mean_recall_run1
        ));
    });
}

#[test]
fn hnsw_recall_snapshot_deterministic_across_scales() {
    TerminalReporter::suite_banner("HNSW DETERMINISTIC SNAPSHOT ACROSS SCALES", 1);
    let mut harness = VantaHarness::new("HNSW DETERMINISTIC SNAPSHOT");

    harness.execute("Scale 1K: Snapshot identity across 3 builds", || {
        let n = 1000;
        let dims = 64;
        let k = 10;
        let n_queries = 50;
        let seed = 42u64;

        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u128, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u128, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 1000);

        let mut recalls = Vec::new();
        let mut run_lines = Vec::new();
        for run in 0..3 {
            let index = CPIndex::new_with_config(HnswConfig::default());
            for (id, vec) in &dataset {
                index
                    .add(
                        *id,
                        FilterBitset::all_set(),
                        VectorRepresentations::Full(vec.clone()),
                        0,
                    )
                    .expect("test insert");
            }
            let mut total = 0.0;
            for query in &queries {
                let truth = brute_force_search(query, &dataset, k);
                let hnsw_ids: Vec<u128> = index
                    .search_nearest(query, None, None, &vantadb::node::ALL_BITSET, k, None)
                    .into_iter()
                    .map(|(id, _)| id)
                    .collect();
                let hits = truth.iter().filter(|id| hnsw_ids.contains(id)).count();
                total += hits as f64 / k as f64;
            }
            let recall = total / n_queries as f64;
            // Banners ordenados: el reporting se agrupa tras la medición,
            // no interleado con el Act bulk.
            run_lines.push(format!("  Run {}: Recall@10 = {:.4}", run + 1, recall));
            recalls.push(recall);
        }

        for line in &run_lines {
            TerminalReporter::info(line);
        }
        for i in 1..recalls.len() {
            assert!(
                (recalls[0] - recalls[i]).abs() < 1e-12,
                "Deterministic snapshot broken across builds: run1={} run{}={}",
                recalls[0],
                i + 1,
                recalls[i]
            );
        }
        TerminalReporter::success("Deterministic recall snapshot confirmed across 3 builds.");
    });
}

// ═══════════════════════════════════════════════════════════════════
// SECTION 2: EXPORT/IMPORT FORMAT VERSIONING
// ═══════════════════════════════════════════════════════════════════

#[test]
fn export_format_schema_version_roundtrip() {
    TerminalReporter::suite_banner("EXPORT FORMAT SCHEMA VERSION SNAPSHOT", 1);
    let mut harness = VantaHarness::new("EXPORT FORMAT SNAPSHOT");

    harness.execute("Export produces schema_version=2", || {
        let source_dir = tempdir().expect("source tempdir");
        let export_path = source_dir.path().join("export.jsonl");

        let source = Embedded::open(source_dir.path()).expect("open source");
        source
            .put(record("ns/snap", "a", "snapshot payload", "test"))
            .expect("put");
        source.flush().expect("flush");

        source
            .export_namespace(&export_path, "ns/snap", None)
            .expect("export");

        // Read the raw JSONL and verify schema_version field
        let content = fs::read_to_string(&export_path).expect("read export");
        let line = content.trim();
        let parsed: serde_json::Value = serde_json::from_str(line).expect("parse export line");
        assert_eq!(
            parsed["schema_version"].as_u64(),
            Some(2),
            "Export schema_version must be 2 (ADR-046 §D7), got: {:?}",
            parsed["schema_version"]
        );

        TerminalReporter::success("Export format schema_version=2 snapshot confirmed.");
    });

    harness.execute(
        "Round-trip: export → import → re-export produces identical JSONL",
        || {
            let source_dir = tempdir().expect("source tempdir");
            let target_dir = tempdir().expect("target tempdir");
            let export1 = source_dir.path().join("round1.jsonl");
            let export2 = target_dir.path().join("round2.jsonl");

            {
                let source = Embedded::open(source_dir.path()).expect("open source");
                source
                    .put(record("ns/rnd", "k1", "round trip alpha", "cat-a"))
                    .expect("put k1");
                source
                    .put(record("ns/rnd", "k2", "round trip beta", "cat-b"))
                    .expect("put k2");
                source.flush().expect("flush");
                source.export_all(&export1).expect("export all");
            }

            {
                let target = Embedded::open(target_dir.path()).expect("open target");
                let import = target.import_file(&export1, false).expect("import");
                assert_eq!(import.inserted, 2);
                assert_eq!(import.errors, 0);

                target.export_all(&export2).expect("re-export");
            }

            let content1 = fs::read_to_string(&export1).expect("read export1");
            let content2 = fs::read_to_string(&export2).expect("read export2");

            let mut lines1: Vec<&str> = content1.lines().collect();
            let mut lines2: Vec<&str> = content2.lines().collect();
            lines1.sort_unstable();
            lines2.sort_unstable();

            assert_eq!(
                lines1, lines2,
                "Round-trip export/import must produce identical JSONL content"
            );

            TerminalReporter::success("Export/import round-trip produces byte-identical JSONL.");
        },
    );
}

#[test]
fn export_format_updates_existing_records_version_tracking() {
    TerminalReporter::suite_banner("EXPORT FORMAT VERSION TRACKING", 1);
    let mut harness = VantaHarness::new("EXPORT VERSION TRACKING");

    harness.execute("Import updates existing records and bumps version", || {
        let source_dir = tempdir().expect("source tempdir");
        let target_dir = tempdir().expect("target tempdir");
        let export_path = source_dir.path().join("update.jsonl");

        let source = Embedded::open(source_dir.path()).expect("open source");
        source
            .put(record("ns/upd", "a", "original payload", "test"))
            .expect("put original");
        source.flush().expect("flush");
        source.export_all(&export_path).expect("export");

        let target = Embedded::open(target_dir.path()).expect("open target");
        target
            .put(record("ns/upd", "a", "stale payload", "test"))
            .expect("seed stale");

        let import = target.import_file(&export_path, false).expect("import");
        assert_eq!(import.updated, 1);
        assert_eq!(import.inserted, 0);
        assert_eq!(import.errors, 0);

        let fetched = target.get("ns/upd", "a").expect("get").expect("record");
        assert_eq!(
            fetched.payload, "original payload",
            "Import must overwrite stale payload"
        );
        assert_eq!(fetched.version, 1, "Version must be 1 after import");

        TerminalReporter::success("Export format correctly updates existing records.");
    });
}

#[test]
fn export_format_preserves_all_record_fields() {
    TerminalReporter::suite_banner("EXPORT FORMAT FIELD FIDELITY", 1);
    let mut harness = VantaHarness::new("EXPORT FIELD FIDELITY");

    harness.execute("All MemoryExportLine fields survive round-trip", || {
        let dir = tempdir().expect("tempdir");
        let export_path = dir.path().join("fidelity.jsonl");

        let db = Embedded::open(dir.path()).expect("open");
        let mut input = MemoryInput::new("ns/fidelity", "key-f", "fidelity payload");
        input
            .metadata
            .insert("int_field".to_string(), Value::Int(42));
        input
            .metadata
            .insert("float_field".to_string(), Value::Float(2.72));
        input
            .metadata
            .insert("bool_field".to_string(), Value::Bool(true));
        input.vector = Some(vec![0.5, 0.5, 0.5]);
        input.ttl_ms = Some(86400000);
        db.put(input).expect("put");
        db.flush().expect("flush");

        db.export_all(&export_path).expect("export");

        let restore_dir = tempdir().expect("restore dir");
        let restored = Embedded::open(restore_dir.path()).expect("open restored");
        restored.import_file(&export_path, false).expect("import");

        let record = restored
            .get("ns/fidelity", "key-f")
            .expect("get")
            .expect("record");
        assert_eq!(record.payload, "fidelity payload");
        assert_eq!(record.metadata.get("int_field"), Some(&Value::Int(42)));
        assert_eq!(
            record.metadata.get("float_field"),
            Some(&Value::Float(2.72))
        );
        assert_eq!(record.metadata.get("bool_field"), Some(&Value::Bool(true)));
        assert_eq!(record.vector, Some(vec![0.5, 0.5, 0.5]));

        TerminalReporter::success("All export line fields survive round-trip.");
    });
}

#[test]
fn export_format_empty_lines_skipped() {
    TerminalReporter::suite_banner("EXPORT FORMAT EMPTY LINE HANDLING", 1);
    let mut harness = VantaHarness::new("EXPORT EMPTY LINE HANDLING");

    harness.execute("Empty lines in JSONL are silently skipped", || {
        let dir = tempdir().expect("tempdir");
        let export_path = dir.path().join("empty_lines.jsonl");
        let target_dir = tempdir().expect("target dir");

        // Write valid JSONL with empty lines interspersed
        // Each line must have a unique key to count as 3 separate inserts
        let line_a = r#"{"schema_version":1,"namespace":"ns/empty","key":"a","payload":"data","metadata":{},"vector":[1.0,0.0,0.0],"created_at_ms":1000,"updated_at_ms":1000,"version":0,"expires_at_ms":null}"#;
        let line_b = r#"{"schema_version":1,"namespace":"ns/empty","key":"b","payload":"data","metadata":{},"vector":[1.0,0.0,0.0],"created_at_ms":1000,"updated_at_ms":1000,"version":0,"expires_at_ms":null}"#;
        let line_c = r#"{"schema_version":1,"namespace":"ns/empty","key":"c","payload":"data","metadata":{},"vector":[1.0,0.0,0.0],"created_at_ms":1000,"updated_at_ms":1000,"version":0,"expires_at_ms":null}"#;
        let content = format!("{line_a}\n\n\n{line_b}\n\n{line_c}\n");
        fs::write(&export_path, &content).expect("write mixed jsonl");

        let target = Embedded::open(target_dir.path()).expect("open target");
        let import = target.import_file(&export_path, false).expect("import");
        assert_eq!(import.inserted, 3);
        // str::lines() yields 3 blank lines (two between a–b, one between
        // b–c); the trailing newline after line_c is not a blank line.
        assert_eq!(import.skipped, 3);
        assert_eq!(import.errors, 0);

        TerminalReporter::success("Empty lines correctly skipped in import.");
    });
}

// ═══════════════════════════════════════════════════════════════════
// SECTION 3: WAL FORMAT INTEGRITY
// ═══════════════════════════════════════════════════════════════════

#[test]
fn wal_crc32c_deterministic_and_detects_corruption() {
    TerminalReporter::suite_banner("WAL CRC32C INTEGRITY SNAPSHOT", 1);
    let mut harness = VantaHarness::new("WAL CRC32C INTEGRITY");

    harness.execute("CRC32C is deterministic and detects bit flips", || {
        let data = b"vanta wal snapshot test payload";
        let crc1 = vantadb::wal::compute_crc32c(data);
        let crc2 = vantadb::wal::compute_crc32c(data);
        assert_eq!(crc1, crc2, "CRC32C must be deterministic");

        // Single byte change must produce different CRC
        let mut corrupted = data.to_vec();
        corrupted[5] ^= 0xFF;
        let crc3 = vantadb::wal::compute_crc32c(&corrupted);
        assert_ne!(crc1, crc3, "CRC32C must detect single-byte corruption");

        TerminalReporter::success("CRC32C deterministic and corruption-sensitive.");
    });

    harness.execute("WalHeader CRC validates on deserialize", || {
        let header = vantadb::wal::WalHeader::new(1);
        let serialized = header.serialize();

        // Deserialize succeeds with valid CRC
        let deserialized = vantadb::wal::WalHeader::deserialize(&serialized).expect("valid header");
        assert_eq!(deserialized.base.format_version, 1);

        // Corrupt the CRC bytes in the serialized output
        let mut corrupted = serialized;
        corrupted[16] ^= 0xFF;
        let result = vantadb::wal::WalHeader::deserialize(&corrupted);
        assert!(
            result.is_err(),
            "Header with corrupted CRC must fail deserialize: {:?}",
            result
        );

        TerminalReporter::success("WalHeader CRC validation confirmed.");
    });
}

#[test]
fn wal_record_all_variants_roundtrip() {
    TerminalReporter::suite_banner("WAL RECORD VARIANT ROUNDTRIP", 1);
    let mut harness = VantaHarness::new("WAL RECORD ROUNDTRIP");

    harness.execute(
        "All WalRecord variants serialize/deserialize correctly",
        || {
            let dir = tempdir().expect("tempdir");
            let wal_path = dir.path().join("variants.wal");

            let records = vec![
                WalRecord::Insert(vantadb::node::UnifiedNode::new(1)),
                WalRecord::Update {
                    id: 1,
                    node: vantadb::node::UnifiedNode::new(2),
                },
                WalRecord::Delete { id: 3 },
                WalRecord::Checkpoint {
                    node_count: 100,
                    index_checksum: Some(0xDEADBEEF),
                    timestamp: Some(1234567890),
                },
                WalRecord::Checkpoint {
                    node_count: 0,
                    index_checksum: None,
                    timestamp: None,
                },
            ];

            // Write all variants
            {
                let mut w = WalWriter::open(&wal_path, vantadb::config::SyncMode::Periodic)
                    .expect("open writer");
                for record in &records {
                    w.append(record).expect("append record");
                }
                w.sync().expect("sync");
                assert_eq!(w.record_count(), records.len() as u64);
            }

            // Read back and verify
            {
                let mut r = WalReader::open(&wal_path).expect("open reader");
                let mut read_back = Vec::new();
                r.replay_all(|rec| {
                    read_back.push(rec);
                    Ok(())
                })
                .expect("replay");

                assert_eq!(read_back.len(), records.len());

                // Check Insert
                match &read_back[0] {
                    WalRecord::Insert(node) => assert_eq!(node.id, 1),
                    other => panic!("Expected Insert, got {:?}", other),
                }
                // Check Update
                match &read_back[1] {
                    WalRecord::Update { id, node } => {
                        assert_eq!(*id, 1);
                        assert_eq!(node.id, 2);
                    }
                    other => panic!("Expected Update, got {:?}", other),
                }
                // Check Delete
                match &read_back[2] {
                    WalRecord::Delete { id } => assert_eq!(*id, 3),
                    other => panic!("Expected Delete, got {:?}", other),
                }
                // Check Checkpoint with checksum
                match &read_back[3] {
                    WalRecord::Checkpoint {
                        node_count,
                        index_checksum,
                        timestamp,
                    } => {
                        assert_eq!(*node_count, 100);
                        assert_eq!(*index_checksum, Some(0xDEADBEEF));
                        assert_eq!(*timestamp, Some(1234567890));
                    }
                    other => panic!("Expected Checkpoint, got {:?}", other),
                }
                // Check Checkpoint without checksum
                match &read_back[4] {
                    WalRecord::Checkpoint {
                        node_count,
                        index_checksum,
                        timestamp,
                    } => {
                        assert_eq!(*node_count, 0);
                        assert!(index_checksum.is_none());
                        assert!(timestamp.is_none());
                    }
                    other => panic!("Expected Checkpoint, got {:?}", other),
                }
            }

            TerminalReporter::success("All 5 WalRecord variants round-tripped correctly.");
        },
    );
}

#[test]
fn wal_replay_recovers_from_known_wal() {
    TerminalReporter::suite_banner("WAL REPLAY RECOVERY SNAPSHOT", 1);
    let mut harness = VantaHarness::new("WAL REPLAY RECOVERY");

    harness.execute(
        "Open database, insert 3 nodes, close, open — WAL replay recovers all",
        || {
            let dir = tempdir().expect("tempdir");
            let db_path = dir.path().to_str().unwrap();

            let config = vantadb::config::Config {
                backend_kind: vantadb::storage::BackendKind::Fjall,
                ..Default::default()
            };

            // Phase 1: Seed 3 nodes
            {
                let storage = vantadb::storage::StorageEngine::open_with_config(
                    db_path,
                    Some(config.clone()),
                )
                .expect("open engine");
                storage
                    .insert(&vantadb::node::UnifiedNode::new(101))
                    .unwrap();
                storage
                    .insert(&vantadb::node::UnifiedNode::new(102))
                    .unwrap();
                storage
                    .insert(&vantadb::node::UnifiedNode::new(103))
                    .unwrap();
                storage.flush().expect("flush");
            }

            // Phase 2: Re-open and verify all 3 nodes survive WAL replay
            {
                let storage = vantadb::storage::StorageEngine::open_with_config(
                    db_path,
                    Some(config.clone()),
                )
                .expect("open engine again");
                let hnsw = storage.hnsw.load();
                assert!(hnsw.contains_node(101), "WAL replay must recover node 101");
                assert!(hnsw.contains_node(102), "WAL replay must recover node 102");
                assert!(hnsw.contains_node(103), "WAL replay must recover node 103");
            }

            TerminalReporter::success("WAL replay recovers all 3 seeded nodes.");
        },
    );

    harness.execute(
        "Checkpoint WAL: records after checkpoint survive replay",
        || {
            let dir = tempdir().expect("tempdir");
            let wal_path = dir.path().join("data").join("vanta.wal");
            fs::create_dir_all(wal_path.parent().unwrap()).expect("create wal dir");

            // Write: Insert(1), Insert(2), Checkpoint, Insert(3)
            {
                let mut w =
                    WalWriter::open(&wal_path, vantadb::config::SyncMode::Periodic).expect("open");
                w.append(&WalRecord::Insert(vantadb::node::UnifiedNode::new(1)))
                    .expect("append 1");
                w.append(&WalRecord::Insert(vantadb::node::UnifiedNode::new(2)))
                    .expect("append 2");
                w.append(&WalRecord::create_checkpoint(2, None))
                    .expect("append checkpoint");
                w.append(&WalRecord::Insert(vantadb::node::UnifiedNode::new(3)))
                    .expect("append 3");
                w.sync().expect("sync");
                assert_eq!(w.record_count(), 4);
            }

            // Read all records back
            {
                let mut r = WalReader::open(&wal_path).expect("open reader");
                let mut recovered = Vec::new();
                r.replay_all(|rec| {
                    let id = match &rec {
                        WalRecord::Insert(n) => n.id,
                        WalRecord::Update { id, .. } => *id,
                        WalRecord::Delete { id } => *id,
                        WalRecord::Checkpoint { node_count, .. } => *node_count as u128,
                        WalRecord::Begin(_) | WalRecord::Commit(_) | WalRecord::Abort(_) => 0,
                        // WAL v2 (RES-01): Prepare is a phase-1 marker, no payload id.
                        WalRecord::Prepare { .. } => 0,
                    };
                    recovered.push((rec, id));
                    Ok(())
                })
                .expect("replay");

                assert_eq!(recovered.len(), 4, "All 4 records must be recovered");
                // Record 3 is the Checkpoint with node_count=2
                match &recovered[2].0 {
                    WalRecord::Checkpoint { node_count, .. } => assert_eq!(*node_count, 2),
                    _ => panic!("Expected Checkpoint at index 2"),
                }
                // Record 4 is Insert(3)
                match &recovered[3].0 {
                    WalRecord::Insert(n) => assert_eq!(n.id, 3),
                    _ => panic!("Expected Insert(3) at index 3"),
                }
            }

            TerminalReporter::success(
                "Checkpoint WAL replayed correctly with records after checkpoint.",
            );
        },
    );
}

#[test]
fn wal_postcard_serialization_deterministic() {
    TerminalReporter::suite_banner("WAL POSTCARD SERIALIZATION SNAPSHOT", 1);
    let mut harness = VantaHarness::new("WAL POSTCARD SERIALIZATION");

    harness.execute("postcard-serialized WalRecord is deterministic", || {
        let record = WalRecord::Insert(vantadb::node::UnifiedNode::new(42));
        let bytes1 = postcard::to_allocvec(&record).expect("serialize");
        let bytes2 = postcard::to_allocvec(&record).expect("serialize again");

        assert_eq!(
            bytes1, bytes2,
            "Postcard serialization must be deterministic"
        );

        let deserialized: WalRecord = postcard::from_bytes(&bytes1).expect("deserialize");
        match deserialized {
            WalRecord::Insert(node) => assert_eq!(node.id, 42),
            _ => panic!("Expected Insert(42)"),
        }

        TerminalReporter::success("Postcard serialization deterministic.");
    });

    harness.execute(
        "Checkpoint record validates index checksum on replay",
        || {
            let index_state = b"deterministic_index_state_snapshot";
            let checkpoint = WalRecord::create_checkpoint(500, Some(index_state));

            // Validation passes with correct state
            assert!(
                checkpoint.validate_checkpoint(index_state).is_ok(),
                "Valid checkpoint must pass validation"
            );

            // Validation fails with corrupted state
            let corrupted = b"corrupted_index_state_snapshot";
            assert!(
                checkpoint.validate_checkpoint(corrupted).is_err(),
                "Corrupted checkpoint must fail validation"
            );

            // Checkpoint without checksum always passes
            let no_crc = WalRecord::Checkpoint {
                node_count: 500,
                index_checksum: None,
                timestamp: None,
            };
            assert!(
                no_crc.validate_checkpoint(b"any_state").is_ok(),
                "Checkpoint without checksum must always pass"
            );

            TerminalReporter::success("Checkpoint index checksum validation certified.");
        },
    );
}

// ═══════════════════════════════════════════════════════════════════
// SECTION 4: VANTAFILE SNAPSHOT TESTS
// ═══════════════════════════════════════════════════════════════════

/// Helper: open a StorageEngine with Fjall backend in the given temporary directory.
fn open_vf_engine(path: &std::path::Path) -> StorageEngine {
    let config = vantadb::config::Config {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(path.to_str().unwrap(), Some(config))
        .expect("open StorageEngine")
}

/// Path to the File inside a storage engine's data directory.
///
/// The LSM layout names the primary segment `vstore_L0.vanta` (the legacy
/// `vector_store.vanta` is migrated on open); small engines write every node
/// to L0, so this is the file that holds the raw node headers.
fn vf_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("data").join("vstore_L0.vanta")
}

/// Deterministic 4-dimensional vector from a seed integer.
fn tiny_vec(seed: u64) -> Vec<f32> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..4).map(|_| rng.random_range(-1.0..1.0)).collect()
}

#[test]
fn vantafile_serialization_determinism() {
    TerminalReporter::suite_banner("VANTAFILE SERIALIZATION DETERMINISM", 1);
    let mut harness = VantaHarness::new("VANTAFILE DETERMINISM");

    harness.execute("File bytes are deterministic for identical input", || {
        let dir1 = tempdir().expect("tempdir1");
        let dir2 = tempdir().expect("tempdir2");

        let nodes: Vec<UnifiedNode> = (0..8)
            .map(|i| UnifiedNode::with_vector(i as u128, tiny_vec(i)))
            .collect();

        // Write to dir1
        {
            let engine = open_vf_engine(dir1.path());
            for node in &nodes {
                engine.insert(node).expect("insert");
            }
            engine.flush().expect("flush");
        }

        // Write to dir2
        {
            let engine = open_vf_engine(dir2.path());
            for node in &nodes {
                engine.insert(node).expect("insert");
            }
            engine.flush().expect("flush");
        }

        let bytes1 = std::fs::read(vf_path(dir1.path())).expect("read vfile1");
        let bytes2 = std::fs::read(vf_path(dir2.path())).expect("read vfile2");

        assert_eq!(
            bytes1.len(),
            bytes2.len(),
            "File sizes must match for identical inputs"
        );

        // Skip the first 16 bytes (Header: magic + version + timestamp).
        // The write-cursor at bytes 16..24 and all subsequent node data
        // must be byte-identical.
        assert_eq!(
            bytes1[16..],
            bytes2[16..],
            "File data after header must be deterministic"
        );

        // Verify both engines can read back the same data
        {
            let engine = open_vf_engine(dir1.path());
            for i in 0..8 {
                let got = engine
                    .get(i as u128)
                    .expect("get")
                    .unwrap_or_else(|| panic!("node {i} must be readable"));
                assert_eq!(got.vector.to_f32().unwrap(), tiny_vec(i as u64));
            }
        }

        TerminalReporter::success("File serialization is deterministic.");
    });
}

#[test]
fn vantafile_mixed_representations_roundtrip() {
    TerminalReporter::suite_banner("VANTAFILE MIXED REPRESENTATIONS ROUNDTRIP", 1);
    let mut harness = VantaHarness::new("VANTAFILE MIXED REPS");

    harness.execute(
        "Full(f32) and SQ8 and None-vector nodes survive round-trip",
        || {
            let dir = tempdir().expect("tempdir");
            let engine = open_vf_engine(dir.path());

            // Node 0: Full(f32) vector — persisted in File
            let full_data = vec![0.25, 0.50, 0.75, 1.00];
            let mut node_full = UnifiedNode::with_vector(100, full_data.clone());
            node_full
                .relational
                .insert("type".into(), FieldValue::String("full".into()));
            engine.insert(&node_full).expect("insert full");

            // Node 1: SQ8(i8[], scale) vector — NOT persisted in File (vec_len=0)
            let mut node_sq8 = UnifiedNode::new(200);
            node_sq8.vector = VectorRepresentations::SQ8(Box::new([10i8, -20i8, 30i8, -40i8]), 2.0);
            node_sq8
                .relational
                .insert("type".into(), FieldValue::String("sq8".into()));
            engine.insert(&node_sq8).expect("insert sq8");

            // Node 2: None vector — no vector data stored
            let mut node_none = UnifiedNode::new(300);
            node_none
                .relational
                .insert("type".into(), FieldValue::String("none".into()));
            engine.insert(&node_none).expect("insert none");

            // Read back Full node
            let got_full = engine.get(100).expect("get full").expect("found full");
            assert_eq!(
                got_full.vector.to_f32().unwrap(),
                full_data,
                "Full vector data must be preserved"
            );
            assert_eq!(
                got_full.get_field("type").and_then(|v| v.as_str()),
                Some("full")
            );

            // Read back SQ8 node — metadata survives even though
            // File stores vec_len=0 for non-Full vectors.
            let got_sq8 = engine.get(200).expect("get sq8").expect("found sq8");
            assert_eq!(
                got_sq8.get_field("type").and_then(|v| v.as_str()),
                Some("sq8"),
                "SQ8 node metadata must survive round-trip"
            );

            // Read back None-vector node
            let got_none = engine.get(300).expect("get none").expect("found none");
            assert_eq!(
                got_none.get_field("type").and_then(|v| v.as_str()),
                Some("none"),
                "None-vector node metadata must survive round-trip"
            );

            // Verify all 3 nodes survive close/reopen
            drop(engine);
            let engine = open_vf_engine(dir.path());
            assert!(engine.get(100).expect("reopen full").is_some());
            assert!(engine.get(200).expect("reopen sq8").is_some());
            assert!(engine.get(300).expect("reopen none").is_some());

            TerminalReporter::success("Mixed representation round-trip certified.");
        },
    );
}

#[test]
fn vantafile_multi_write_rebuild() {
    TerminalReporter::suite_banner("VANTAFILE MULTI-WRITE REBUILD", 1);
    let mut harness = VantaHarness::new("VANTAFILE MULTI-WRITE");

    harness.execute(
        "150 vectors written survive close/reopen and full scan",
        || {
            let dir = tempdir().expect("tempdir");

            let count = 150u64;
            let dims = 16;

            // Phase 1: write
            {
                let engine = open_vf_engine(dir.path());
                for i in 0..count {
                    let vec: Vec<f32> = (0..dims)
                        .map(|d| ((i * dims as u64 + d as u64) as f32) / 1000.0)
                        .collect();
                    engine
                        .insert(&UnifiedNode::with_vector(i as u128, vec))
                        .expect("insert");
                }
                engine.flush().expect("flush");
            }

            // Phase 2: verify all readable via scan
            {
                let engine = open_vf_engine(dir.path());
                let nodes = engine.scan_nodes().expect("scan");
                assert_eq!(
                    nodes.len() as u64,
                    count,
                    "All {count} vectors must be recovered"
                );
                for i in 0..count {
                    let got = engine
                        .get(i as u128)
                        .expect("get")
                        .unwrap_or_else(|| panic!("node {i} must exist"));
                    let expected: Vec<f32> = (0..dims)
                        .map(|d| ((i * dims as u64 + d as u64) as f32) / 1000.0)
                        .collect();
                    assert_eq!(
                        got.vector.to_f32().unwrap(),
                        expected,
                        "Vector {i} must survive rebuild"
                    );
                }
            }

            TerminalReporter::success("All 150 vectors survive close/reopen.");
        },
    );
}

#[test]
fn vantafile_tombstone_persistence() {
    TerminalReporter::suite_banner("VANTAFILE TOMBSTONE PERSISTENCE", 1);
    let mut harness = VantaHarness::new("VANTAFILE TOMBSTONE");

    harness.execute(
        "Tombstoned nodes are skipped and tombstone flag survives reopen",
        || {
            let dir = tempdir().expect("tempdir");

            // Phase 1: insert 3 nodes, delete middle one
            {
                let engine = open_vf_engine(dir.path());
                for i in 0..3 {
                    engine
                        .insert(&UnifiedNode::with_vector(
                            i as u128,
                            vec![i as f32 + 1.0; 4],
                        ))
                        .expect("insert");
                }
                // All 3 visible
                assert!(engine.get(0).expect("get 0").is_some());
                assert!(engine.get(1).expect("get 1").is_some());
                assert!(engine.get(2).expect("get 2").is_some());

                engine.delete(1, "snapshot test").expect("delete");

                // After delete: node 1 gone, 0 and 2 remain
                assert!(engine.get(0).expect("get 0 after").is_some());
                assert!(engine.get(1).expect("get 1 after").is_none());
                assert!(engine.get(2).expect("get 2 after").is_some());

                // Verify raw File has tombstone flag set for node 1
                let bytes = std::fs::read(vf_path(dir.path())).expect("read vfile");
                // Node layout: header(64) + pad-aligned vector per node
                // Node 0: offset=64, vec_len=4f32=16B → cursor = (64+64+16+63)&!63 = 192
                // Node 1: offset=192, flags at offset 192+56=248
                let flags_offset = 64 + 128; // = 192 (node1 header start)
                let hdr = DiskNodeHeader::read_from_bytes(&bytes[flags_offset..flags_offset + 64])
                    .expect("parse node1 header");
                assert_ne!(
                    hdr.flags & NodeFlags::TOMBSTONE,
                    0,
                    "Node 1 DiskNodeHeader must have TOMBSTONE flag"
                );
                // Node 0 flags must NOT be tombstone
                let hdr0 =
                    DiskNodeHeader::read_from_bytes(&bytes[64..128]).expect("parse node0 header");
                assert_eq!(
                    hdr0.flags & NodeFlags::TOMBSTONE,
                    0,
                    "Node 0 must NOT have TOMBSTONE flag"
                );
            }

            // Phase 2: reopen — tombstone persisted in File
            {
                let engine = open_vf_engine(dir.path());
                assert!(engine.get(0).expect("get 0 reopen").is_some());
                assert!(
                    engine.get(1).expect("get 1 reopen").is_none(),
                    "Tombstoned node 1 must not reappear after reopen"
                );
                assert!(engine.get(2).expect("get 2 reopen").is_some());
            }

            TerminalReporter::success("Tombstone persistence certified.");
        },
    );
}

#[test]
fn vantafile_export_golden_file() {
    TerminalReporter::suite_banner("VANTAFILE EXPORT GOLDEN FILE", 1);
    let mut harness = VantaHarness::new("VANTAFILE EXPORT");

    harness.execute(
        "JSONL export has deterministic structure and round-trips",
        || {
            let dir = tempdir().expect("tempdir");
            let export_path = dir.path().join("golden.jsonl");

            let db = Embedded::open(dir.path()).expect("open");
            let mut input = MemoryInput::new("ns/golden", "golden-key", "golden payload");
            input
                .metadata
                .insert("score".to_string(), Value::Float(99.5));
            input
                .metadata
                .insert("active".to_string(), Value::Bool(true));
            input.vector = Some(vec![0.1, 0.2, 0.3, 0.4]);
            db.put(input).expect("put");
            db.flush().expect("flush");

            db.export_all(&export_path).expect("export");

            // Read and parse JSONL
            let content = fs::read_to_string(&export_path).expect("read export");
            let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            assert_eq!(lines.len(), 1, "Expected exactly one export line");

            let parsed: serde_json::Value =
                serde_json::from_str(lines[0]).expect("parse JSONL line");

            // Validate golden structure
            assert_eq!(parsed["schema_version"].as_u64(), Some(2));
            assert_eq!(parsed["namespace"].as_str(), Some("ns/golden"));
            assert_eq!(parsed["key"].as_str(), Some("golden-key"));
            assert_eq!(parsed["payload"].as_str(), Some("golden payload"));
            assert!(parsed["created_at_ms"].as_u64().unwrap_or(0) > 0);
            assert!(parsed["updated_at_ms"].as_u64().unwrap_or(0) > 0);
            // putBatch versioning (c9188639) starts new records at version 1.
            assert_eq!(parsed["version"].as_u64(), Some(1));
            assert!(parsed["expires_at_ms"].is_null());
            assert_eq!(
                parsed["vector"].as_array().map(|v| v.len()),
                Some(4),
                "Vector must have 4 elements"
            );
            assert!(
                (parsed["vector"][0].as_f64().unwrap() - 0.1).abs() < 1e-6,
                "Vector[0] must be 0.1"
            );

            // metadata sub-object — Value serializes with its variant tag
            // (serde enum), e.g. {"Float":99.5} / {"Bool":true}.
            let meta = &parsed["metadata"];
            assert_eq!(meta["score"]["Float"].as_f64(), Some(99.5));
            assert_eq!(meta["active"]["Bool"].as_bool(), Some(true));

            // Round-trip: import into fresh DB
            let restore_dir = tempdir().expect("restore dir");
            let restored = Embedded::open(restore_dir.path()).expect("open restored");
            let import = restored.import_file(&export_path, false).expect("import");
            assert_eq!(import.inserted, 1);
            assert_eq!(import.errors, 0);

            let record = restored
                .get("ns/golden", "golden-key")
                .expect("get")
                .expect("record");
            assert_eq!(record.payload, "golden payload");
            assert_eq!(record.metadata.get("score"), Some(&Value::Float(99.5)));
            assert_eq!(record.metadata.get("active"), Some(&Value::Bool(true)));
            assert_eq!(record.vector, Some(vec![0.1, 0.2, 0.3, 0.4]));

            TerminalReporter::success("File export golden file structure certified.");
        },
    );
}

// ═══════════════════════════════════════════════════════════════════
// SECTION 5: HARD-LINK FILESYSTEM SNAPSHOT TESTS
// ═══════════════════════════════════════════════════════════════════

/// Helper: seed an Embedded with some test records.
fn seed_snapshot_data(db: &Embedded) {
    for i in 0..5 {
        let mut input = MemoryInput::new("ns/snap", format!("key-{i}"), format!("payload-{i}"));
        input.vector = Some(vec![i as f32, 0.0, 0.0]);
        db.put(input).expect("seed put");
    }
    db.flush().expect("seed flush");
}

/// Helper: count files in a directory (recursive).
fn count_files(path: &std::path::Path) -> usize {
    if !path.exists() {
        return 0;
    }
    let mut total = 0;
    for entry in std::fs::read_dir(path).map_or(vec![], |e| e.flatten().collect()) {
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_file() {
            total += 1;
        } else if ft.is_dir() {
            total += count_files(&entry.path());
        }
    }
    total
}

/// Helper: read every WAL file (`vanta*.wal*`: shards + shard meta) under
/// `<snapshot>/data`, name-sorted, as (name, bytes). These are the files the
/// snapshot reopen replays — they must be a frozen point-in-time image.
fn snapshot_wal_files(snap_root: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    let data = snap_root.join("data");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&data).expect("read snapshot data dir") {
        let entry = entry.expect("snapshot dir entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("vanta") && name.contains(".wal") {
            out.push((
                name,
                fs::read(entry.path()).expect("read snapshot wal file"),
            ));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Helper: read every VantaFile segment (`vstore_L*.vanta`, plus the legacy
/// `vector_store.vanta`) under `<snapshot>/data`, name-sorted, as
/// (name, bytes). The live engine rewrites these in place (tombstone
/// bit-flip on delete, node appends, `compact_level` truncation) — the
/// snapshot image must stay frozen.
fn snapshot_vstore_files(snap_root: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    let data = snap_root.join("data");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&data).expect("read snapshot data dir") {
        let entry = entry.expect("snapshot dir entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        if (name.starts_with("vstore_L") && name.ends_with(".vanta"))
            || name == "vector_store.vanta"
        {
            out.push((
                name,
                fs::read(entry.path()).expect("read snapshot vstore file"),
            ));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn test_hardlink_snapshot_instant() {
    TerminalReporter::suite_banner("HARD-LINK SNAPSHOT INSTANT", 1);
    let mut harness = VantaHarness::new("HARD-LINK SNAPSHOT INSTANT");

    harness.execute(
        "Snapshot creates all data files under snapshots/<name>",
        || {
            let dir = tempdir().expect("tempdir");
            let db = Embedded::open(dir.path()).expect("open db");
            seed_snapshot_data(&db);

            let start = Instant::now();
            let snap = db.create_snapshot("test-snap-1").expect("create snapshot");
            let elapsed = start.elapsed();

            // Snapshot directory must exist and contain files
            assert!(snap.path.exists(), "snapshot path must exist");
            assert!(snap.path.is_dir(), "snapshot path must be a directory");
            let file_count = count_files(&snap.path);
            assert!(
                file_count > 0,
                "snapshot must contain at least one file, got {file_count}"
            );

            // Keep the small fixture within the snapshot latency bound.
            // On Windows (copy fallback) this may be slower, so use a generous limit.
            assert!(
                elapsed.as_secs() < 5,
                "snapshot creation took {elapsed:?} — expected < 5s"
            );

            TerminalReporter::success(&format!(
                "Snapshot created with {file_count} files in {elapsed:?}",
            ));
        },
    );
}

#[test]
fn test_hardlink_snapshot_independence() {
    TerminalReporter::suite_banner("HARD-LINK SNAPSHOT INDEPENDENCE", 1);
    let mut harness = VantaHarness::new("HARD-LINK SNAPSHOT INDEPENDENCE");

    harness.execute(
        "Modifying original data after snapshot leaves snapshot unchanged",
        || {
            let dir = tempdir().expect("tempdir");
            let db = Embedded::open(dir.path()).expect("open db");

            // Seed original data
            let mut input = MemoryInput::new("ns/indep", "key-a", "original payload");
            input.vector = Some(vec![1.0, 0.0, 0.0]);
            db.put(input).expect("put original");
            db.flush().expect("flush original");

            // Snapshot the original state
            let snap = db.create_snapshot("pre-modify").expect("create snapshot");

            // Deterministic isolation probe (no timing dependence): capture the
            // snapshot's own WAL files as of snapshot time. Pre-fix, Unix
            // hard-linked these to the live WAL, so the post-snapshot put below
            // appended to the SAME inode and silently mutated the snapshot —
            // reopening it then replayed the "modified" record. Asserting the
            // bytes are frozen catches the shared inode directly on any
            // platform (the ASan job runs plain `cargo test`, the only CI path
            // that executes this binary — nextest's default filter excludes it).
            let snap_wal_before = snapshot_wal_files(&snap.path);
            assert!(
                !snap_wal_before.is_empty(),
                "snapshot must capture the WAL files (replay source)"
            );

            // Modify the original data
            let mut input2 = MemoryInput::new("ns/indep", "key-a", "modified payload");
            input2.vector = Some(vec![99.0, 0.0, 0.0]);
            db.put(input2).expect("put modified");
            db.flush().expect("flush modified");

            // The snapshot's WAL must be byte-identical: no post-snapshot
            // append may reach it through a shared inode.
            let snap_wal_after = snapshot_wal_files(&snap.path);
            assert_eq!(
                snap_wal_before, snap_wal_after,
                "post-snapshot writes must not mutate the snapshot's WAL files \
                 (appendable files must be copied, not hard-linked)"
            );

            // Unix: direct inode-level proof — the snapshot's WAL files must
            // not be hard links of the live ones (nlink == 1).
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let live_data = dir.path().join("data");
                let snap_data = snap.path.join("data");
                for (name, _) in &snap_wal_before {
                    let live_file = live_data.join(name);
                    if !live_file.exists() {
                        continue;
                    }
                    let snap_meta = fs::metadata(snap_data.join(name)).expect("snap wal metadata");
                    assert_eq!(
                        snap_meta.nlink(),
                        1,
                        "snapshot WAL file {name} is hard-linked (nlink={}) — \
                         post-snapshot appends would mutate the snapshot",
                        snap_meta.nlink()
                    );
                    let live_meta = fs::metadata(&live_file).expect("live wal metadata");
                    assert_ne!(
                        snap_meta.ino(),
                        live_meta.ino(),
                        "snapshot WAL file {name} shares the live inode — not independent"
                    );
                }
            }

            // Verify the snapshot directory still exists and has content
            assert!(
                snap.path.exists(),
                "snapshot path must survive modification"
            );

            // Open a fresh DB pointing to the snapshot directory to verify
            // the snapshot contains the original state
            let snap_db = Embedded::open(&snap.path).expect("open snapshot as db");
            let got = snap_db.get("ns/indep", "key-a").expect("get from snapshot");
            assert_eq!(
                got.map(|r| r.payload),
                Some("original payload".to_string()),
                "Snapshot must contain original payload, not modified one"
            );

            TerminalReporter::success("Snapshot independence confirmed: original data preserved.");
        },
    );
}

/// FIND-318: `vstore_L*.vanta` segments are rewritten **in place** by the
/// live engine after a snapshot (tombstone bit-flip on `delete`, node
/// appends at the write cursor, `compact_level` truncation). A hard link
/// would let those writes mutate the snapshot through the shared inode —
/// the segment family is copied, not linked.
#[test]
fn test_hardlink_snapshot_vstore_independence() {
    TerminalReporter::suite_banner("HARD-LINK SNAPSHOT VSTORE INDEPENDENCE", 1);
    let mut harness = VantaHarness::new("HARD-LINK SNAPSHOT VSTORE INDEPENDENCE");

    harness.execute(
        "In-place vstore mutations after a snapshot leave the snapshot segment unchanged",
        || {
            let dir = tempdir().expect("tempdir");
            let db = Embedded::open(dir.path()).expect("open db");

            // Seed a node so its record lands in vstore_L0.vanta.
            let mut input = MemoryInput::new("ns/vstore", "key-a", "original payload");
            input.vector = Some(vec![1.0, 0.0, 0.0]);
            db.put(input).expect("put original");
            db.flush().expect("flush original");

            let snap = db.create_snapshot("vstore-snap").expect("create snapshot");

            // Deterministic isolation probe (no timing dependence): capture
            // the snapshot's own vstore segments as of snapshot time.
            let snap_vstore_before = snapshot_vstore_files(&snap.path);
            assert!(
                !snap_vstore_before.is_empty(),
                "snapshot must capture the vstore segments"
            );

            // In-place mutation channel 1: tombstone bit-flip rewrites the
            // live segment header at the node's existing offset.
            db.delete("ns/vstore", "key-a").expect("delete");
            // In-place mutation channel 2: a put appends at the write cursor.
            let mut input2 = MemoryInput::new("ns/vstore", "key-b", "post-snapshot payload");
            input2.vector = Some(vec![0.0, 1.0, 0.0]);
            db.put(input2).expect("put post-snapshot");
            db.flush().expect("flush post-snapshot");

            let snap_vstore_after = snapshot_vstore_files(&snap.path);
            assert_eq!(
                snap_vstore_before, snap_vstore_after,
                "post-snapshot in-place vstore writes must not reach the snapshot \
                 (vstore_L*.vanta must be copied, not hard-linked)"
            );

            // Unix: direct inode-level proof — the snapshot's segments must
            // not be hard links of the live ones.
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let live_data = dir.path().join("data");
                let snap_data = snap.path.join("data");
                for (name, _) in &snap_vstore_before {
                    let live_file = live_data.join(name);
                    if !live_file.exists() {
                        continue;
                    }
                    let snap_meta =
                        fs::metadata(snap_data.join(name)).expect("snap vstore metadata");
                    assert_eq!(
                        snap_meta.nlink(),
                        1,
                        "snapshot segment {name} is hard-linked (nlink={}) — \
                         post-snapshot in-place writes would mutate the snapshot",
                        snap_meta.nlink()
                    );
                    let live_meta = fs::metadata(&live_file).expect("live vstore metadata");
                    assert_ne!(
                        snap_meta.ino(),
                        live_meta.ino(),
                        "snapshot segment {name} shares the live inode — not independent"
                    );
                }
            }

            // The snapshot still resolves the deleted node: the in-place
            // tombstone flip must not have leaked through a shared inode.
            let snap_db = Embedded::open(&snap.path).expect("open snapshot as db");
            let got = snap_db
                .get("ns/vstore", "key-a")
                .expect("get from snapshot");
            assert!(
                got.is_some(),
                "snapshot must still resolve the node deleted after the snapshot"
            );

            TerminalReporter::success(
                "Vstore independence confirmed: segment bytes frozen across delete + put.",
            );
        },
    );
}

#[test]
fn test_hardlink_snapshot_multiple() {
    TerminalReporter::suite_banner("HARD-LINK MULTIPLE SNAPSHOTS", 1);
    let mut harness = VantaHarness::new("HARD-LINK MULTIPLE SNAPSHOTS");

    harness.execute("Multiple snapshots are isolated from each other", || {
        let dir = tempdir().expect("tempdir");
        let db = Embedded::open(dir.path()).expect("open db");

        // Phase 1: seed data and snapshot
        seed_snapshot_data(&db);
        let snap1 = db.create_snapshot("phase-1").expect("snapshot 1");
        let count1 = count_files(&snap1.path);

        // Phase 2: add more data and snapshot again
        for i in 5..10 {
            let mut input = MemoryInput::new("ns/snap", format!("key-{i}"), format!("payload-{i}"));
            input.vector = Some(vec![i as f32, 0.0, 0.0]);
            db.put(input).expect("put phase 2");
        }
        db.flush().expect("flush phase 2");
        let snap2 = db.create_snapshot("phase-2").expect("snapshot 2");
        let count2 = count_files(&snap2.path);

        // Phase 3: delete some data and snapshot again
        db.delete("ns/snap", "key-0").expect("delete key-0");
        db.flush().expect("flush phase 3");
        let snap3 = db.create_snapshot("phase-3").expect("snapshot 3");
        let count3 = count_files(&snap3.path);

        // List snapshots and verify all 3 exist
        let names = db.list_snapshots().expect("list snapshots");
        assert!(names.contains(&"phase-1".to_string()), "phase-1 must be listed");
        assert!(names.contains(&"phase-2".to_string()), "phase-2 must be listed");
        assert!(names.contains(&"phase-3".to_string()), "phase-3 must be listed");

        // Each snapshot must have at least one file
        assert!(count1 > 0, "snapshot 1 must have files");
        assert!(count2 > 0, "snapshot 2 must have files");
        assert!(count3 > 0, "snapshot 3 must have files");

        TerminalReporter::success(&format!(
            "All 3 snapshots isolated: phase-1 ({count1} files), phase-2 ({count2} files), phase-3 ({count3} files)",
        ));
    });
}

// ═══════════════════════════════════════════════════════════════════
// SECTION: FIND-25 — SNAPSHOT CONSISTENCY UNDER CONCURRENT WRITES
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_snapshot_consistent_under_concurrent_writes() {
    TerminalReporter::suite_banner("SNAPSHOT CONSISTENCY UNDER CONCURRENT WRITES (FIND-25)", 1);
    let mut harness = VantaHarness::new("SNAPSHOT CONSISTENCY UNDER CONCURRENT WRITES");

    harness.execute(
        "Snapshot taken during active writes reopens with a consistent node set",
        || {
            let dir = tempdir().expect("tempdir");
            let db = Embedded::open(dir.path()).expect("open db");

            // Baseline: committed + flushed BEFORE the snapshot. Every one of
            // these nodes MUST appear in the reopened snapshot.
            const BASELINE: usize = 20;
            for i in 0..BASELINE {
                let mut input = MemoryInput::new("ns/conc", format!("base-{i}"), format!("payload-base-{i}"));
                input.vector = Some(vec![i as f32, 0.0, 0.0]);
                db.put(input).expect("put baseline");
            }
            db.flush().expect("flush baseline");

            // Writers hammer puts while the main thread snapshots. Writers are
            // NOT flushed, so whether their nodes land in the snapshot depends
            // on the quiesce point — but whatever lands must be intact.
            const WRITERS: usize = 4;
            const PER_WRITER: usize = 25;
            let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let mut handles = Vec::new();
            for t in 0..WRITERS {
                let db_clone = db.clone();
                let stop = stop.clone();
                handles.push(std::thread::spawn(move || {
                    for i in 0..PER_WRITER {
                        if stop.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }
                        let key = format!("w{t}-{i}");
                        let mut input =
                            MemoryInput::new("ns/conc", key.clone(), format!("payload-{key}"));
                        input.vector = Some(vec![100.0 + i as f32, 0.0, 0.0]);
                        // Writes may transiently fail under snapshot lock
                        // contention — that's allowed; corruption is not.
                        let _ = db_clone.put(input);
                    }
                }));
            }

            let snap = db.create_snapshot("concurrent").expect("create snapshot");
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
            for h in handles {
                h.join().expect("writer thread panicked");
            }

            // Reopen the snapshot as an independent DB and verify consistency:
            // every retrieved payload is exact (no torn records) and the whole
            // flushed baseline survived.
            let snap_db = Embedded::open(&snap.path).expect("open snapshot as db");
            let mut found_baseline = 0;
            for i in 0..BASELINE {
                let got = snap_db.get("ns/conc", &format!("base-{i}")).expect("get baseline from snapshot");
                match got {
                    Some(r) => {
                        assert_eq!(
                            r.payload,
                            format!("payload-base-{i}"),
                            "torn snapshot: baseline node base-{i} has wrong payload"
                        );
                        found_baseline += 1;
                    }
                    None => panic!("torn snapshot: flushed baseline node base-{i} missing"),
                }
            }

            // Writer nodes present in the snapshot must have exact payloads.
            let mut found_writer_nodes = 0;
            for t in 0..WRITERS {
                for i in 0..PER_WRITER {
                    let key = format!("w{t}-{i}");
                    if let Some(r) = snap_db.get("ns/conc", &key).expect("get writer node") {
                        assert_eq!(
                            r.payload,
                            format!("payload-{key}"),
                            "torn snapshot: writer node {key} has wrong payload"
                        );
                        found_writer_nodes += 1;
                    }
                }
            }

            // The snapshot dir must not recursively contain its own snapshots/
            // subtree (recursion guard on the data_dir walk).
            let nested = snap.path.join("data").join("snapshots");
            assert!(
                !nested.exists() || fs::read_dir(&nested).map(|d| d.count()).unwrap_or(0) == 0,
                "snapshot must not nest its own snapshots/ directory"
            );

            TerminalReporter::success(&format!(
                "Snapshot consistent: {found_baseline}/{BASELINE} baseline nodes, {found_writer_nodes} concurrent-writer nodes captured intact",
            ));
        },
    );
}

// ═══════════════════════════════════════════════════════════════════
// SECTION 6: MCP-34b — PHYSICAL SNAPSHOT RESTORE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_snapshot_restore_roundtrip() {
    TerminalReporter::suite_banner("SNAPSHOT RESTORE ROUNDTRIP (MCP-34b)", 1);
    let mut harness = VantaHarness::new("SNAPSHOT RESTORE ROUNDTRIP");

    harness.execute(
        "snapshot → mutate → close → restore → reopen: pre-snapshot records back, post-snapshot gone",
        || {
            let dir = tempdir().expect("tempdir");
            let db_path = dir.path();

            // Phase 1: seed and snapshot the pre-mutation state.
            // Phase 2: mutate AFTER the snapshot. The scoped drop releases the
            // fs2 lock — the embedded exclusivity contract for restore.
            {
                let db = Embedded::open(db_path).expect("open db");
                seed_snapshot_data(&db); // ns/snap key-0..key-4, payload-i

                db.create_snapshot("restore-me").expect("create snapshot");

                let mut post =
                    MemoryInput::new("ns/snap", "post-snap", "added after snapshot");
                post.vector = Some(vec![9.0, 0.0, 0.0]);
                db.put(post).expect("put post-snapshot");
                db.flush().expect("flush post-snapshot");
                assert!(
                    db.get("ns/snap", "post-snap")
                        .expect("get live")
                        .is_some(),
                    "post-snapshot record must be live before restore"
                );
            }

            // Phase 3: physical restore (no open handles).
            vantadb::storage::StorageEngine::snapshot_restore(db_path, "restore-me")
                .expect("snapshot_restore");

            // Phase 4: reopen over the restored directory and assert.
            let db2 = Embedded::open(db_path).expect("reopen restored db");
            for i in 0..5 {
                let got = db2
                    .get("ns/snap", &format!("key-{i}"))
                    .expect("get pre-snapshot key")
                    .unwrap_or_else(|| panic!("pre-snapshot key-{i} must exist after restore"));
                assert_eq!(
                    got.payload,
                    format!("payload-{i}"),
                    "restored payload mismatch for key-{i}"
                );
            }
            assert!(
                db2.get("ns/snap", "post-snap")
                    .expect("get post-snapshot key")
                    .is_none(),
                "record written after the snapshot must NOT survive restore"
            );

            // All snapshots survive the swap (they are moved back into the
            // fresh data_dir before copy-back).
            let names = db2.list_snapshots().expect("list snapshots");
            assert!(
                names.contains(&"restore-me".to_string()),
                "snapshots must survive the restore swap: {names:?}"
            );

            TerminalReporter::success(
                "Restore roundtrip certified: pre-snapshot state recovered, mutation reverted.",
            );
        },
    );
}

#[test]
fn test_snapshot_restore_rejects_unsafe_names() {
    TerminalReporter::suite_banner("SNAPSHOT RESTORE NAME VALIDATION (MCP-34b)", 1);
    let mut harness = VantaHarness::new("SNAPSHOT RESTORE NAME VALIDATION");

    harness.execute(
        "Path traversal / separators / empty names rejected; unknown name NotFound",
        || {
            let dir = tempdir().expect("tempdir");

            for bad in ["", ".", "..", "../escape", "sub/dir", "back\\slash"] {
                let res = vantadb::storage::StorageEngine::snapshot_restore(dir.path(), bad);
                assert!(res.is_err(), "snapshot_restore must reject name {bad:?}");
            }

            // Valid identifier but nonexistent snapshot → NotFound, not a panic.
            let res = vantadb::storage::StorageEngine::snapshot_restore(dir.path(), "no-such-snap");
            let err = res.expect_err("nonexistent snapshot must fail");
            assert!(
                err.to_string().contains("no-such-snap"),
                "error must identify the missing snapshot: {err}"
            );

            TerminalReporter::success("Snapshot name trust boundary enforced.");
        },
    );
}

#[cfg(feature = "failpoints")]
#[test]
fn test_snapshot_restore_failpoint_aborts_before_swap() {
    TerminalReporter::suite_banner("SNAPSHOT RESTORE FAILPOINT (MCP-34b)", 1);
    let mut harness = VantaHarness::new("SNAPSHOT RESTORE FAILPOINT");

    harness.execute(
        "snapshot_restore_fail makes restore error out without touching data_dir",
        || {
            let dir = tempdir().expect("tempdir");
            let db_path = dir.path();

            {
                let db = Embedded::open(db_path).expect("open db");
                seed_snapshot_data(&db);
                db.create_snapshot("fp-snap").expect("create snapshot");
            }

            fail::cfg("snapshot_restore_fail", "return").expect("arm failpoint");
            let res = StorageEngine::snapshot_restore(db_path, "fp-snap");
            fail::cfg("snapshot_restore_fail", "off").expect("disarm failpoint");

            assert!(res.is_err(), "armed failpoint must abort snapshot_restore");

            // Nothing was displaced: original data_dir intact and usable.
            let db2 = Embedded::open(db_path).expect("reopen after failed restore");
            assert!(
                db2.get("ns/snap", "key-0").expect("get").is_some(),
                "original data must be untouched after failed restore"
            );

            TerminalReporter::success("Failpoint aborted restore cleanly; live data untouched.");
        },
    );
}

#[cfg(feature = "fjall")]
fn force_snapshot_tables(path: &std::path::Path) {
    let db = fjall::Database::builder(path)
        .open()
        .expect("open fjall directly");
    for name in db.list_keyspace_names() {
        db.keyspace(&name, fjall::KeyspaceCreateOptions::default)
            .expect("open keyspace")
            .rotate_memtable_and_wait()
            .expect("flush memtable");
    }
}

#[cfg(feature = "fjall")]
fn snapshot_file_set(root: &std::path::Path) -> std::collections::BTreeSet<std::path::PathBuf> {
    fn visit(
        root: &std::path::Path,
        dir: &std::path::Path,
        files: &mut std::collections::BTreeSet<std::path::PathBuf>,
    ) {
        for entry in fs::read_dir(dir).expect("read directory") {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.is_file() {
                files.insert(
                    path.strip_prefix(root)
                        .expect("relative path")
                        .to_path_buf(),
                );
            }
        }
    }
    let mut files = std::collections::BTreeSet::new();
    visit(root, root, &mut files);
    files
}

#[cfg(feature = "fjall")]
fn snapshot_config(path: &std::path::Path, encrypted: bool) -> vantadb::config::Config {
    let mut config = vantadb::config::Config {
        storage_path: path.to_string_lossy().into_owned(),
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    if encrypted {
        config.encryption_key = Some("31".repeat(32));
    }
    config
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
fn snapshot_live_tree_bytes(
    root: &std::path::Path,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    let mut paths = std::collections::BTreeSet::new();
    for tree in ["data", "keyspaces"] {
        for path in snapshot_file_set(&root.join(tree)) {
            paths.insert(std::path::PathBuf::from(tree).join(path));
        }
    }
    for entry in fs::read_dir(root).expect("root") {
        let entry = entry.expect("entry");
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let journal = name_str
            .strip_suffix(".jnl")
            .is_some_and(|id| !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()));
        if matches!(name_str.as_ref(), "version" | "lock" | ".vanta.schema") || journal {
            paths.insert(name.into());
        }
    }
    paths
        .into_iter()
        .map(|path| {
            let contents = fs::read(root.join(&path)).expect("live file");
            (path, contents)
        })
        .collect()
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
fn assert_no_snapshot_restore_staging(root: &std::path::Path) {
    assert!(!fs::read_dir(root).expect("root").any(|entry| {
        let entry = entry.expect("entry");
        let name = entry.file_name();
        let name = name.to_string_lossy();
        name.starts_with("data.pre_restore_") || name.starts_with("backend.pre_restore_")
    }));
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
fn seed_backend_kind_mismatch(root: &std::path::Path) -> vantadb::config::Config {
    let config = snapshot_config(root, false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open Fjall");
        db.put(record("backend-kind", "a", "before", "test"))
            .expect("put A");
        db.create_snapshot("backend-kind").expect("snapshot");
        db.put(record("backend-kind", "b", "after", "test"))
            .expect("put B");
        db.flush().expect("flush");
    }
    config
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_sdk_backend_mismatch_rejected_before_staging() {
    let dir = tempdir().expect("tempdir");
    let config = seed_backend_kind_mismatch(dir.path());
    let live = snapshot_live_tree_bytes(dir.path());
    let mut wrong = config.clone();
    wrong.backend_kind = BackendKind::RocksDb;
    let mismatch = "snapshot backend 'fjall' does not match requested backend 'rocksdb'";

    for with_report in [false, true] {
        let result = if with_report {
            Embedded::restore_from_with_backend(wrong.clone(), "backend-kind").map(|_| ())
        } else {
            Embedded::restore_from(wrong.clone(), "backend-kind").map(|_| ())
        };
        let error = result.expect_err("mismatched backend must fail");
        assert!(
            matches!(error, vantadb::error::Error::Validation { ref field, ref reason }
                if field == "backend_kind" && reason == mismatch),
            "{error}"
        );
        assert_eq!(snapshot_live_tree_bytes(dir.path()), live);
        assert_no_snapshot_restore_staging(dir.path());
    }

    wrong.backend_kind = BackendKind::InMemory;
    let error = Embedded::restore_from(wrong, "backend-kind").expect_err("in-memory backend");
    assert!(matches!(error, vantadb::error::Error::InvalidInput(_)));
    assert_eq!(snapshot_live_tree_bytes(dir.path()), live);
    assert_no_snapshot_restore_staging(dir.path());
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_pending_backend_mismatch_preserves_marker_and_live_state() {
    let dir = tempdir().expect("tempdir");
    let config = seed_backend_kind_mismatch(dir.path());
    let live = snapshot_live_tree_bytes(dir.path());
    let marker =
        StorageEngine::schedule_snapshot_restore(dir.path(), "backend-kind").expect("schedule");
    let marker_contents = fs::read(&marker).expect("marker");
    let mut wrong = config.clone();
    wrong.backend_kind = BackendKind::RocksDb;
    let error = Embedded::open_with_config(wrong).expect_err("mismatched backend must fail");
    assert!(
        error
            .to_string()
            .contains("snapshot backend 'fjall' does not match requested backend 'rocksdb'"),
        "{error}"
    );
    assert_eq!(fs::read(&marker).expect("marker retained"), marker_contents);
    assert_eq!(snapshot_live_tree_bytes(dir.path()), live);
    assert_no_snapshot_restore_staging(dir.path());

    let db = Embedded::open_with_config(config).expect("open with correct backend");
    assert!(!marker.exists());
    assert_eq!(
        db.get("backend-kind", "a")
            .expect("A")
            .expect("record")
            .payload,
        "before"
    );
    assert!(db.get("backend-kind", "b").expect("B").is_none());
}

#[cfg(all(feature = "rocksdb", feature = "fs2"))]
#[test]
fn test_snapshot_rocksdb_native_checkpoint_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let config = vantadb::config::Config {
        storage_path: dir.path().to_string_lossy().into_owned(),
        backend_kind: BackendKind::RocksDb,
        ..Default::default()
    };
    {
        let db = Embedded::open_with_config(config.clone()).expect("open RocksDB");
        db.put(record("rocks-checkpoint", "a", "before", "test"))
            .expect("put A");
        db.create_snapshot("rocks-native").expect("snapshot");
        db.put(record("rocks-checkpoint", "b", "after", "test"))
            .expect("put B");
        db.flush().expect("flush");
    }
    let db = Embedded::restore_from(config, "rocks-native").expect("restore RocksDB");
    assert_eq!(
        db.get("rocks-checkpoint", "a")
            .expect("A")
            .expect("record")
            .payload,
        "before"
    );
    assert!(db.get("rocks-checkpoint", "b").expect("B").is_none());
}

#[cfg(feature = "fjall")]
fn snapshot_seed(path: &std::path::Path, encrypted: bool) -> vantadb::config::Config {
    let config = snapshot_config(path, encrypted);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        for key in ["a0", "a1", "a2"] {
            db.put(record(
                "snapshot-test",
                key,
                "alpha snapshot original",
                "snapshot",
            ))
            .expect("put");
        }
        db.flush().expect("flush");
    }
    force_snapshot_tables(path);
    config
}

#[cfg(feature = "fjall")]
fn assert_snapshot_tables_exist(path: &std::path::Path) {
    let files = snapshot_file_set(&path.join("keyspaces"));
    assert!(
        files.iter().any(|p| p
            .parent()
            .and_then(|p| p.file_name())
            .is_some_and(|n| n == "tables")),
        "Fjall must have persisted a table before snapshot: {files:?}"
    );
}

#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_captures_fjall_keyspaces() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), false);
    let db = Embedded::open_with_config(config).expect("open");
    assert_snapshot_tables_exist(dir.path());
    let snapshot = db.create_snapshot("tables").expect("snapshot");
    assert!(snapshot.path.join("backend.manifest").is_file());
    assert_eq!(
        fs::read(snapshot.path.join("snapshot.format")).expect("format"),
        b"vantadb-snapshot v1\n"
    );
    let manifest = fs::read(snapshot.path.join("backend.manifest")).expect("manifest");
    let payload = manifest
        .strip_prefix(b"vantadb-snapshot-backend v1\n")
        .expect("manifest header");
    let inventory: serde_json::Value = serde_json::from_slice(payload).expect("inventory");
    assert_eq!(inventory["backend_kind"], "fjall");
    let files = inventory["files"].as_array().expect("files");
    let paths: Vec<_> = files
        .iter()
        .map(|file| file["path"].as_str().expect("path"))
        .collect();
    assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(paths.iter().any(|path| path.starts_with("data/")));
    assert!(paths.iter().any(|path| path.starts_with("backend/")));
    for file in files {
        let path = file["path"].as_str().expect("path");
        assert_eq!(
            fs::metadata(snapshot.path.join(path)).expect("file").len(),
            file["len"].as_u64().expect("length"),
            "{path}"
        );
    }
    assert_eq!(
        snapshot_file_set(&dir.path().join("keyspaces")),
        snapshot_file_set(&snapshot.path.join("backend/keyspaces"))
    );
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
const FOREIGN_ROCKS_ENTRIES: [&str; 4] = ["archive/user.bin", "CURRENT", "MANIFEST-123", "12.sst"];

#[cfg(all(feature = "fjall", feature = "fs2"))]
fn write_foreign_rocks_entries(root: &std::path::Path, contents: &[u8]) {
    fs::create_dir_all(root.join("archive")).expect("foreign archive");
    for name in FOREIGN_ROCKS_ENTRIES {
        fs::write(root.join(name), contents).expect("foreign RocksDB-shaped entry");
    }
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
fn assert_foreign_rocks_entries(root: &std::path::Path, contents: &[u8]) {
    for name in FOREIGN_ROCKS_ENTRIES {
        assert_eq!(
            fs::read(root.join(name)).expect("foreign RocksDB-shaped entry"),
            contents,
            "{name}"
        );
    }
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_foreign_root_files_survive_success_and_scheduled_restore() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    fs::create_dir(dir.path().join("exports")).expect("exports");
    fs::write(dir.path().join("notes.txt"), b"before").expect("notes before");
    fs::write(dir.path().join("exports/file.bin"), b"before").expect("export before");
    write_foreign_rocks_entries(dir.path(), b"before");
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.put(record("foreign", "a", "A", "test")).expect("put");
        let snap = db.create_snapshot("foreign").expect("snapshot");
        assert!(!snap.path.join("backend/notes.txt").exists());
        assert!(!snap.path.join("backend/exports").exists());
        for name in FOREIGN_ROCKS_ENTRIES {
            assert!(!snap.path.join("backend").join(name).exists(), "{name}");
        }
        db.put(record("foreign", "b", "B", "test")).expect("put");
    }
    fs::write(dir.path().join("notes.txt"), b"after").expect("notes after");
    fs::write(dir.path().join("exports/file.bin"), b"after").expect("export after");
    fs::write(dir.path().join("exports/new.bin"), b"new").expect("new export");
    write_foreign_rocks_entries(dir.path(), b"after");
    let future_journal = dir.path().join("999999999.jnl");
    fs::write(&future_journal, b"future journal").expect("future journal");
    StorageEngine::snapshot_restore(dir.path(), "foreign").expect("restore");
    assert!(
        !future_journal.exists(),
        "future journal must not be replayed"
    );
    assert_foreign_rocks_entries(dir.path(), b"after");
    assert_eq!(
        fs::read(dir.path().join("notes.txt")).expect("notes"),
        b"after"
    );
    assert_eq!(
        fs::read(dir.path().join("exports/file.bin")).expect("export"),
        b"after"
    );
    assert_eq!(
        fs::read(dir.path().join("exports/new.bin")).expect("new"),
        b"new"
    );
    {
        let db = Embedded::open_with_config(config.clone()).expect("open restored");
        assert!(db.get("foreign", "b").expect("get").is_none());
        db.put(record("foreign", "c", "C", "test")).expect("put");
        StorageEngine::schedule_snapshot_restore(dir.path(), "foreign").expect("schedule");
    }
    fs::write(dir.path().join("notes.txt"), b"scheduled").expect("notes");
    fs::write(dir.path().join("exports/file.bin"), b"scheduled").expect("export");
    write_foreign_rocks_entries(dir.path(), b"scheduled");
    let db = Embedded::open_with_config(config).expect("apply scheduled restore");
    assert!(db.get("foreign", "c").expect("get").is_none());
    assert_foreign_rocks_entries(dir.path(), b"scheduled");
    assert_eq!(
        fs::read(dir.path().join("notes.txt")).expect("notes"),
        b"scheduled"
    );
    assert_eq!(
        fs::read(dir.path().join("exports/file.bin")).expect("export"),
        b"scheduled"
    );
    assert_eq!(
        fs::read(dir.path().join("exports/new.bin")).expect("new"),
        b"new"
    );
}

#[cfg(all(windows, feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_restore_rejects_case_aliases_of_foreign_entries() {
    for (managed, foreign, is_dir) in [
        ("version", "VERSION", false),
        ("keyspaces", "KEYSPACES", true),
    ] {
        let dir = tempdir().expect("tempdir");
        let config = snapshot_config(dir.path(), false);
        {
            let db = Embedded::open_with_config(config.clone()).expect("open");
            db.create_snapshot("case-alias").expect("snapshot");
        }
        let original = dir.path().join(managed);
        let intermediate = dir.path().join("case-alias-intermediate");
        let alias = dir.path().join(foreign);
        fs::rename(&original, &intermediate).expect("move managed entry");
        fs::rename(&intermediate, &alias).expect("create foreign case alias");
        let probe = if is_dir {
            alias.join("foreign.bin")
        } else {
            alias.clone()
        };
        fs::write(&probe, b"foreign bytes").expect("foreign content");

        assert!(
            StorageEngine::snapshot_restore(dir.path(), "case-alias").is_err(),
            "restore must reject {foreign} rather than overwriting it"
        );
        assert_eq!(fs::read(&probe).expect("foreign content"), b"foreign bytes");
        assert!(!fs::read_dir(dir.path()).expect("root").any(|entry| {
            let name = entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
            name.starts_with("data.pre_restore_") || name.starts_with("backend.pre_restore_")
        }));
        StorageEngine::schedule_snapshot_restore(dir.path(), "case-alias").expect("schedule");
        let error = Embedded::open_with_config(config).expect_err("scheduled restore must fail");
        assert!(
            error
                .to_string()
                .contains("pending snapshot restore failed"),
            "{error}"
        );
        assert!(dir.path().join(".restore_pending").is_file());
        assert_eq!(fs::read(&probe).expect("foreign content"), b"foreign bytes");
    }
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
#[test]
fn test_snapshot_foreign_root_files_survive_restore_rollback() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    fs::create_dir(dir.path().join("exports")).expect("exports");
    fs::write(dir.path().join("notes.txt"), b"before").expect("notes");
    fs::write(dir.path().join("exports/file.bin"), b"before").expect("export");
    write_foreign_rocks_entries(dir.path(), b"before");
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        let snapshot = db.create_snapshot("rollback-foreign").expect("snapshot");
        for name in FOREIGN_ROCKS_ENTRIES {
            assert!(!snapshot.path.join("backend").join(name).exists(), "{name}");
        }
        db.put(record("foreign", "later", "later", "test"))
            .expect("put");
        db.flush().expect("flush");
    }
    fs::write(dir.path().join("notes.txt"), b"after").expect("notes");
    fs::write(dir.path().join("exports/file.bin"), b"after").expect("export");
    write_foreign_rocks_entries(dir.path(), b"after");
    fail::cfg("snapshot_restore_backend_fail", "return").expect("failpoint");
    let result = StorageEngine::snapshot_restore(dir.path(), "rollback-foreign");
    fail::remove("snapshot_restore_backend_fail");
    assert!(result.is_err());
    assert_foreign_rocks_entries(dir.path(), b"after");
    assert_eq!(
        fs::read(dir.path().join("notes.txt")).expect("notes"),
        b"after"
    );
    assert_eq!(
        fs::read(dir.path().join("exports/file.bin")).expect("export"),
        b"after"
    );
    let db = Embedded::open_with_config(config).expect("reopen");
    assert!(db.get("foreign", "later").expect("get").is_some());
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
#[test]
fn test_snapshot_rollback_failure_preserves_staging_and_reports_paths() {
    let dir = tempdir().expect("tempdir");
    {
        let db = Embedded::open(dir.path()).expect("open");
        db.create_snapshot("rollback-failure").expect("snapshot");
        db.put(record("rollback", "later", "later", "test"))
            .expect("put");
        db.flush().expect("flush");
    }
    fail::cfg("snapshot_restore_backend_fail", "return").expect("restore failpoint");
    fail::cfg("snapshot_restore_rollback_fail", "return").expect("rollback failpoint");
    let err =
        StorageEngine::snapshot_restore(dir.path(), "rollback-failure").expect_err("restore fails");
    fail::remove("snapshot_restore_backend_fail");
    fail::remove("snapshot_restore_rollback_fail");
    let message = err.to_string();
    assert!(message.contains("rollback incomplete"), "{message}");
    assert!(message.contains("backend.pre_restore_"), "{message}");
    let staging: Vec<_> = fs::read_dir(dir.path())
        .expect("root")
        .filter_map(|entry| {
            let entry = entry.expect("entry");
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("backend.pre_restore_")
                .then_some(entry.path())
        })
        .collect();
    assert_eq!(staging.len(), 1, "{staging:?}");
    assert!(fs::read_dir(&staging[0]).expect("staging").next().is_some());
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
#[test]
fn test_snapshot_restore_sync_failure_rolls_back_both_trees() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.put(record("sync", "before", "before", "test"))
            .expect("put before");
        db.create_snapshot("sync-failure").expect("snapshot");
        db.put(record("sync", "later", "later", "test"))
            .expect("put later");
        db.flush().expect("flush");
    }
    fail::cfg("snapshot_restore_sync_fail", "return").expect("sync failpoint");
    let result = StorageEngine::snapshot_restore(dir.path(), "sync-failure");
    fail::remove("snapshot_restore_sync_fail");
    assert!(result.is_err(), "sync failure must abort the restore");
    assert!(!fs::read_dir(dir.path()).expect("root").any(|entry| {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        name.starts_with("data.pre_restore_") || name.starts_with("backend.pre_restore_")
    }));
    let db = Embedded::open_with_config(config).expect("reopen");
    assert!(db.get("sync", "before").expect("before").is_some());
    assert!(db.get("sync", "later").expect("later").is_some());
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
#[test]
fn test_snapshot_pending_rollback_failure_requires_manual_recovery() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.create_snapshot("rollback-incomplete").expect("snapshot");
        db.put(record("rollback", "later", "later", "test"))
            .expect("put");
        db.flush().expect("flush");
        StorageEngine::schedule_snapshot_restore(dir.path(), "rollback-incomplete")
            .expect("schedule");
    }
    fail::cfg("snapshot_restore_backend_fail", "return").expect("restore failpoint");
    fail::cfg("snapshot_restore_rollback_fail", "return").expect("rollback failpoint");
    let error = match Embedded::open_with_config(config.clone()) {
        Ok(_) => panic!("pending restore must fail"),
        Err(error) => error,
    };
    fail::remove("snapshot_restore_backend_fail");
    fail::remove("snapshot_restore_rollback_fail");
    let message = error.to_string();
    assert!(message.contains("rollback incomplete"), "{message}");
    assert!(message.contains("manual recovery"), "{message}");
    assert!(message.contains("backend.pre_restore_"), "{message}");
    assert!(!message.contains("to open the current state"), "{message}");
    let marker = dir.path().join(".restore_pending");
    assert!(marker.is_file());
    fs::remove_file(marker).expect("remove marker");
    let error = match Embedded::open_with_config(config) {
        Ok(_) => panic!("orphaned staging must still block the open"),
        Err(error) => error,
    };
    assert!(
        matches!(error, vantadb::error::Error::Validation { ref field, .. } if field == "snapshot_restore"),
        "{error}"
    );
}

#[cfg(all(unix, feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_restore_rejects_symlink_in_live_backend_before_staging() {
    use std::os::unix::fs::symlink;
    let dir = tempdir().expect("tempdir");
    {
        let db = Embedded::open(dir.path()).expect("open");
        db.create_snapshot("backend-symlink").expect("snapshot");
    }
    let keyspace = fs::read_dir(dir.path().join("keyspaces"))
        .expect("keyspaces")
        .map(|entry| entry.expect("entry"))
        .find(|entry| entry.file_type().expect("type").is_dir())
        .expect("keyspace")
        .path();
    let outside = dir.path().join("outside.txt");
    fs::write(&outside, b"untouched").expect("outside");
    let link = keyspace.join("unsafe-link");
    symlink(&outside, &link).expect("symlink");
    assert!(StorageEngine::snapshot_restore(dir.path(), "backend-symlink").is_err());
    assert!(fs::symlink_metadata(&link)
        .expect("link")
        .file_type()
        .is_symlink());
    assert_eq!(fs::read(outside).expect("outside"), b"untouched");
    assert!(!fs::read_dir(dir.path()).expect("root").any(|entry| {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        name.starts_with("data.pre_restore_") || name.starts_with("backend.pre_restore_")
    }));
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
fn assert_pending_consumer_serializes_scheduler(wait_for_busy: bool) {
    use std::sync::{mpsc, Arc, Barrier};
    use std::time::Duration;
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.put(record("pending-race", "a", "A", "test"))
            .expect("put A");
        db.create_snapshot("pending-a").expect("snapshot A");
        db.put(record("pending-race", "b", "B", "test"))
            .expect("put B");
        db.create_snapshot("pending-b").expect("snapshot B");
        db.put(record("pending-race", "c", "C", "test"))
            .expect("put C");
        StorageEngine::schedule_snapshot_restore(dir.path(), "pending-a").expect("schedule A");
    }
    let read = Arc::new(Barrier::new(2));
    let resume = Arc::new(Barrier::new(2));
    let (events, receiver) = mpsc::channel::<&'static str>();
    {
        let read = read.clone();
        let resume = resume.clone();
        fail::cfg_callback("snapshot_pending_after_read", move || {
            read.wait();
            resume.wait();
        })
        .expect("read callback");
    }
    {
        let events = events.clone();
        fail::cfg_callback("snapshot_pending_lock_contended", move || {
            let _ = events.send("contended");
        })
        .expect("contention callback");
    }
    let root = dir.path().to_path_buf();
    let open_config = config.clone();
    let consumer = std::thread::spawn(move || {
        let db = Embedded::open_with_config(open_config).expect("consume A");
        assert!(db.get("pending-race", "a").expect("A").is_some());
        assert!(db.get("pending-race", "b").expect("B").is_none());
    });
    read.wait();
    let scheduler = std::thread::spawn(move || {
        let result = StorageEngine::schedule_snapshot_restore(&root, "pending-b");
        events.send("finished").expect("completion");
        result
    });
    let first = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("scheduler reached lock or finished");
    let scheduled = if wait_for_busy {
        let result = scheduler.join().expect("scheduler");
        resume.wait();
        result
    } else {
        resume.wait();
        scheduler.join().expect("scheduler")
    };
    consumer.join().expect("consumer");
    fail::remove("snapshot_pending_after_read");
    fail::remove("snapshot_pending_lock_contended");
    assert_eq!(first, "contended", "scheduler must wait for consumer");
    if wait_for_busy {
        assert!(
            matches!(&scheduled, Err(vantadb::error::Error::DatabaseBusy(_))),
            "expected DatabaseBusy while consumer holds the lock, got {scheduled:?}"
        );
    }
    match scheduled {
        Ok(_) => assert!(!wait_for_busy, "busy path must not schedule B"),
        Err(vantadb::error::Error::DatabaseBusy(_)) => {
            assert!(
                !dir.path().join(".restore_pending").exists(),
                "A marker must be consumed before retry"
            );
            StorageEngine::schedule_snapshot_restore(dir.path(), "pending-b")
                .expect("retry schedule B");
        }
        Err(error) => panic!("unexpected scheduling error: {error}"),
    }
    assert_eq!(
        fs::read(dir.path().join(".restore_pending")).expect("B marker"),
        b"vantadb-restore-pending v1\npending-b\n"
    );
    let db = Embedded::open_with_config(config).expect("consume B");
    assert!(db.get("pending-race", "a").expect("A").is_some());
    assert!(db.get("pending-race", "b").expect("B").is_some());
    assert!(db.get("pending-race", "c").expect("C").is_none());
    assert!(!dir.path().join(".restore_pending").exists());
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
#[test]
fn test_snapshot_pending_consumer_serializes_scheduler() {
    assert_pending_consumer_serializes_scheduler(false);
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
#[test]
fn test_snapshot_pending_consumer_busy_timeout_can_retry() {
    assert_pending_consumer_serializes_scheduler(true);
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_duplicate_name_never_replaces_previous_capture() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    db.put(record("duplicate", "a", "A", "test")).expect("put");
    let snap = db.create_snapshot("duplicate").expect("first snapshot");
    let before = fs::read(snap.path.join("data/vstore_L0.vanta")).expect("snapshot data");
    db.put(record("duplicate", "b", "B", "test")).expect("put");
    assert!(
        db.create_snapshot("duplicate").is_err(),
        "must reject duplicate"
    );
    assert_eq!(
        fs::read(snap.path.join("data/vstore_L0.vanta")).expect("snapshot data"),
        before
    );
}

#[cfg(all(feature = "fjall", feature = "fs2", feature = "failpoints"))]
#[test]
fn test_snapshot_failed_capture_cannot_be_restored() {
    let dir = tempdir().expect("tempdir");
    {
        let db = Embedded::open(dir.path()).expect("open");
        fail::cfg("snapshot_create_fail", "return").expect("failpoint");
        let result = db.create_snapshot("failed-capture");
        fail::remove("snapshot_create_fail");
        assert!(result.is_err());
    }
    let snap = dir.path().join("data/snapshots/failed-capture");
    let error = StorageEngine::snapshot_restore(dir.path(), "failed-capture")
        .expect_err("incomplete capture");
    if snap.exists() {
        assert!(snap.join("snapshot.format").is_file());
        assert!(
            matches!(error, vantadb::error::Error::Validation { ref field, .. } if field == "snapshot"),
            "{error}"
        );
    } else {
        assert!(
            matches!(error, vantadb::error::Error::NotFound { .. }),
            "{error}"
        );
    }
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_incomplete_or_corrupt_capture_rejected_before_staging() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    let snap = {
        let db = Embedded::open_with_config(config).expect("open");
        db.put(record("incomplete", "a", "A", "test")).expect("put");
        db.create_snapshot("incomplete").expect("snapshot").path
    };
    let format = snap.join("snapshot.format");
    let manifest = snap.join("backend.manifest");
    let format_bytes = fs::read(&format).expect("format marker");
    let manifest_bytes = fs::read(&manifest).expect("manifest");
    let segment = snap.join("data/vstore_L0.vanta");
    let segment_bytes = fs::read(&segment).expect("segment");
    let live = fs::read(dir.path().join("data/vstore_L0.vanta")).expect("live");
    for case in 0..9 {
        match case {
            0 => fs::remove_file(&manifest).expect("remove manifest"),
            1 => fs::write(&manifest, b"unknown\n").expect("corrupt manifest"),
            2 => fs::write(&format, b"unknown\n").expect("corrupt format"),
            3 => fs::remove_file(&format).expect("remove format"),
            4 => fs::remove_file(&segment).expect("remove segment"),
            5 => fs::write(&segment, b"truncated").expect("truncate segment"),
            6 => fs::write(&manifest, b"vantadb-snapshot-backend v1\n{\"backend_kind\":\"fjall\",\"files\":[{\"path\":\"data/../escape\",\"len\":0}]}").expect("unsafe inventory"),
            7 | 8 => {
                let payload = manifest_bytes
                    .strip_prefix(b"vantadb-snapshot-backend v1\n")
                    .expect("manifest header");
                let mut inventory: serde_json::Value =
                    serde_json::from_slice(payload).expect("inventory");
                if case == 7 {
                    inventory["backend_kind"] = serde_json::json!("unknown");
                } else {
                    inventory
                        .as_object_mut()
                        .expect("object")
                        .remove("backend_kind");
                }
                let mut bytes = b"vantadb-snapshot-backend v1\n".to_vec();
                bytes.extend(serde_json::to_vec(&inventory).expect("inventory JSON"));
                fs::write(&manifest, bytes).expect("invalid backend kind");
            }
            _ => unreachable!(),
        }
        for result in [
            StorageEngine::snapshot_restore(dir.path(), "incomplete").map(|_| ()),
            StorageEngine::schedule_snapshot_restore(dir.path(), "incomplete").map(|_| ()),
        ] {
            assert!(
                matches!(result, Err(vantadb::error::Error::Validation { ref field, .. }) if field == "snapshot"),
                "case {case}: {result:?}"
            );
        }
        assert!(!dir.path().join(".restore_pending").exists(), "case {case}");
        assert_eq!(
            fs::read(dir.path().join("data/vstore_L0.vanta")).expect("live"),
            live
        );
        assert!(
            !fs::read_dir(dir.path()).expect("root").any(|entry| {
                let name = entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned();
                name.starts_with("data.pre_restore_") || name.starts_with("backend.pre_restore_")
            }),
            "case {case}"
        );
        fs::write(&format, &format_bytes).expect("restore format");
        fs::write(&manifest, &manifest_bytes).expect("restore manifest");
        fs::write(&segment, &segment_bytes).expect("restore segment");
    }
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_non_numeric_staging_prefixes_are_preserved() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    let backup = dir.path().join("data.pre_restore_backup");
    let notes = dir.path().join("backend.pre_restore_notes");
    fs::create_dir(&backup).expect("backup directory");
    fs::create_dir(&notes).expect("notes directory");
    fs::write(backup.join("user.bin"), b"backup").expect("backup file");
    fs::write(notes.join("user.bin"), b"notes").expect("notes file");
    {
        let db = Embedded::open_with_config(config.clone()).expect("open with foreign names");
        db.create_snapshot("foreign-staging-prefixes")
            .expect("snapshot");
    }
    StorageEngine::snapshot_restore(dir.path(), "foreign-staging-prefixes").expect("restore");
    for read_only in [false, true] {
        let mut options = config.clone();
        options.read_only = read_only;
        let _db = Embedded::open_with_config(options).expect("reopen with foreign names");
    }
    assert_eq!(
        fs::read(backup.join("user.bin")).expect("backup"),
        b"backup"
    );
    assert_eq!(fs::read(notes.join("user.bin")).expect("notes"), b"notes");
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_orphaned_stagings_block_every_open_and_restore() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.create_snapshot("orphan").expect("snapshot");
    }
    let data_stage = dir.path().join("data.pre_restore_123");
    let backend_stage = dir.path().join("backend.pre_restore_456");
    let file_stage = dir.path().join("data.pre_restore_789");
    fs::create_dir(&data_stage).expect("data staging");
    fs::create_dir(&backend_stage).expect("backend staging");
    fs::write(&file_stage, b"file staging").expect("file staging");
    fs::write(data_stage.join("original"), b"data").expect("data");
    fs::write(backend_stage.join("original"), b"backend").expect("backend");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("missing", dir.path().join("backend.pre_restore_101"))
            .expect("symlink staging");
    }
    for read_only in [false, true] {
        let mut options = config.clone();
        options.read_only = read_only;
        let err = match Embedded::open_with_config(options) {
            Ok(_) => panic!("open must reject orphaned staging"),
            Err(err) => err,
        };
        assert!(
            matches!(err, vantadb::error::Error::Validation { ref field, .. } if field == "snapshot_restore"),
            "{err}"
        );
        assert!(err.to_string().contains("data.pre_restore_123"));
        assert!(err.to_string().contains("backend.pre_restore_456"));
        assert!(err.to_string().contains("data.pre_restore_789"));
        #[cfg(unix)]
        assert!(err.to_string().contains("backend.pre_restore_101"));
    }
    let err = StorageEngine::snapshot_restore(dir.path(), "orphan")
        .expect_err("restore must reject staging");
    assert!(
        matches!(err, vantadb::error::Error::Validation { ref field, .. } if field == "snapshot_restore"),
        "{err}"
    );
    assert_eq!(
        fs::read(data_stage.join("original")).expect("data"),
        b"data"
    );
    assert_eq!(
        fs::read(backend_stage.join("original")).expect("backend"),
        b"backend"
    );
    assert_eq!(fs::read(file_stage).expect("file staging"), b"file staging");
    #[cfg(unix)]
    assert!(
        fs::symlink_metadata(dir.path().join("backend.pre_restore_101"))
            .expect("symlink staging")
            .file_type()
            .is_symlink()
    );
}

#[cfg(all(feature = "fjall", not(feature = "fs2")))]
#[test]
fn test_snapshot_restore_and_pending_open_require_fs2() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.create_snapshot("no-fs2").expect("snapshot");
    }
    for result in [
        StorageEngine::snapshot_restore(dir.path(), "no-fs2").map(|_| ()),
        StorageEngine::schedule_snapshot_restore(dir.path(), "no-fs2").map(|_| ()),
    ] {
        assert!(
            matches!(result, Err(vantadb::error::Error::Validation { .. })),
            "{result:?}"
        );
    }
    let marker = dir.path().join(".restore_pending");
    assert!(!marker.exists());
    fs::write(&marker, b"vantadb-restore-pending v1\nno-fs2\n").expect("marker");
    let before = fs::read(dir.path().join(".vanta.lock")).expect("lock metadata");
    let err = match Embedded::open_with_config(config) {
        Ok(_) => panic!("pending open must require fs2"),
        Err(err) => err,
    };
    assert!(
        matches!(err, vantadb::error::Error::Validation { .. }),
        "{err}"
    );
    assert_eq!(
        fs::read(marker).expect("marker"),
        b"vantadb-restore-pending v1\nno-fs2\n"
    );
    assert_eq!(
        fs::read(dir.path().join(".vanta.lock")).expect("lock metadata"),
        before
    );
}

#[cfg(all(feature = "fjall", any(unix, windows)))]
#[test]
fn test_pending_restore_symlink_marker_fails_closed() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.put(record("symlink", "key", "original", "test"))
            .expect("put");
        db.create_snapshot("symlink-marker").expect("snapshot");
    }
    let marker = dir.path().join(".restore_pending");
    let target = dir.path().join("external-marker");
    let lock_contents = fs::read(dir.path().join(".vanta.lock")).expect("lock metadata");

    for target_exists in [false, true] {
        if target_exists {
            fs::write(&target, b"vantadb-restore-pending v1\nsymlink-marker\n")
                .expect("external target");
        }
        #[cfg(unix)]
        let result = std::os::unix::fs::symlink(&target, &marker);
        #[cfg(windows)]
        let result = std::os::windows::fs::symlink_file(&target, &marker);
        if let Err(error) = result {
            #[cfg(windows)]
            if error.raw_os_error() == Some(1314) {
                eprintln!("symlink fixture unavailable: {error}");
                return;
            }
            panic!("create symlink fixture: {error}");
        }

        let read_only = vantadb::config::Config {
            read_only: true,
            ..config.clone()
        };
        let db = Embedded::open_with_config(read_only).expect("read-only open");
        assert!(db.get("symlink", "key").expect("get").is_some());
        drop(db);

        let error = match Embedded::open_with_config(config.clone()) {
            Ok(_) => panic!("writable open must reject a symlink marker"),
            Err(error) => error,
        };
        assert!(
            matches!(error, vantadb::error::Error::Validation { .. }),
            "{error}"
        );
        assert!(
            StorageEngine::schedule_snapshot_restore(dir.path(), "symlink-marker").is_err(),
            "scheduler must not read or replace a symlink marker"
        );
        assert!(fs::symlink_metadata(&marker)
            .expect("marker")
            .file_type()
            .is_symlink());
        if target_exists {
            assert_eq!(
                fs::read(&target).expect("external target"),
                b"vantadb-restore-pending v1\nsymlink-marker\n"
            );
        } else {
            assert!(!target.exists());
        }
        #[cfg(not(feature = "fs2"))]
        assert_eq!(
            fs::read(dir.path().join(".vanta.lock")).expect("lock metadata"),
            lock_contents
        );
        fs::remove_file(&marker).expect("remove fixture symlink");
    }
    #[cfg(feature = "fs2")]
    let _ = lock_contents;
    let db = Embedded::open_with_config(config).expect("open current state");
    assert!(db.get("symlink", "key").expect("get").is_some());
}

#[cfg(all(windows, feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_windows_name_rejected_by_all_entry_points_before_io() {
    const CHILD: &str = "FIND319_WINDOWS_NAME_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let working_dir = tempdir().expect("isolated working directory");
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .current_dir(working_dir.path())
            .env(CHILD, "1")
            .args([
                "--exact",
                "test_snapshot_windows_name_rejected_by_all_entry_points_before_io",
                "--nocapture",
            ])
            .output()
            .expect("run isolated test");
        assert!(
            output.status.success(),
            "isolated test failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    let before = snapshot_file_set(dir.path());
    let working_dir = std::env::current_dir().expect("working directory");
    let drive_relative = match working_dir.components().next() {
        Some(std::path::Component::Prefix(prefix)) => {
            format!("{}example", prefix.as_os_str().to_string_lossy())
        }
        _ => panic!("Windows working directory must have a prefix"),
    };
    for name in [drive_relative.as_str(), "file:stream", "alias.", "alias "] {
        let result = db.create_snapshot(name);
        assert!(
            matches!(result, Err(vantadb::error::Error::InvalidInput(_))),
            "{name:?}"
        );
        let result = StorageEngine::snapshot_restore(dir.path(), name);
        assert!(
            matches!(result, Err(vantadb::error::Error::InvalidInput(_))),
            "{name:?}"
        );
        let result = StorageEngine::schedule_snapshot_restore(dir.path(), name);
        assert!(
            matches!(result, Err(vantadb::error::Error::InvalidInput(_))),
            "{name:?}"
        );
    }
    assert_eq!(snapshot_file_set(dir.path()), before);
}

#[cfg(feature = "fjall")]
fn snapshot_restore_backend_state(encrypted: bool) {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), encrypted);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        assert_snapshot_tables_exist(dir.path());
        db.create_snapshot("state").expect("snapshot");
        assert!(db.delete("snapshot-test", "a0").expect("delete"));
        db.put(record("snapshot-test", "a1", "changed payload", "snapshot"))
            .expect("update");
        db.put(record(
            "snapshot-test",
            "b0",
            "bravo later payload",
            "snapshot",
        ))
        .expect("put B");
        db.flush().expect("flush later state");
    }
    let db = Embedded::restore_from(config, "state").expect("restore");
    for key in ["a0", "a1", "a2"] {
        assert_eq!(
            db.get("snapshot-test", key)
                .expect("get")
                .expect("A exists")
                .payload,
            "alpha snapshot original"
        );
    }
    assert!(db.get("snapshot-test", "b0").expect("get B").is_none());
    assert_eq!(db.count("snapshot-test", None).expect("count"), 3);
    let text = |term: &str| {
        db.search(MemorySearchRequest {
            namespace: "snapshot-test".into(),
            text_query: Some(term.into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("text search")
    };
    assert_eq!(text("alpha").len(), 3);
    assert!(text("bravo").is_empty());
    let vectors = db
        .search(MemorySearchRequest {
            namespace: "snapshot-test".into(),
            query_vector: vec![1.0, 0.0, 0.0],
            top_k: 10,
            ..Default::default()
        })
        .expect("vector search");
    let keys: std::collections::BTreeSet<_> =
        vectors.iter().map(|hit| hit.record.key.as_str()).collect();
    assert_eq!(keys, ["a0", "a1", "a2"].into_iter().collect());
}

#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_restore_restores_backend_state() {
    snapshot_restore_backend_state(false);
}

#[cfg(all(feature = "fjall", feature = "encryption"))]
#[test]
fn test_snapshot_restore_restores_backend_state_encrypted() {
    snapshot_restore_backend_state(true);
}

#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_isolated_from_post_snapshot_delete() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    db.put(record("snapshot-test", "a0", "alpha", "snapshot"))
        .expect("put");
    let snapshot = db.create_snapshot("delete-isolation").expect("snapshot");
    let segment = snapshot.path.join("data/vstore_L0.vanta");
    let before = fs::read(&segment).expect("read segment");
    db.delete("snapshot-test", "a0").expect("delete");
    let after = fs::read(&segment).expect("read segment after delete");
    assert_eq!(
        before, after,
        "post-snapshot tombstone changed snapshot segment"
    );
}

#[cfg(all(feature = "fjall", feature = "failpoints"))]
#[test]
fn test_snapshot_restore_backend_failure_rolls_back_both_trees() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.create_snapshot("rollback").expect("snapshot");
        db.delete("snapshot-test", "a0").expect("delete");
        db.put(record("snapshot-test", "b0", "later", "snapshot"))
            .expect("put B");
        db.flush().expect("flush");
    }
    fail::cfg("snapshot_restore_backend_fail", "return").expect("arm");
    let result = StorageEngine::snapshot_restore(dir.path(), "rollback");
    fail::remove("snapshot_restore_backend_fail");
    assert!(result.is_err());
    for entry in fs::read_dir(dir.path()).expect("root") {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        assert!(
            !name.starts_with("data.pre_restore_") && !name.starts_with("backend.pre_restore_"),
            "{name}"
        );
    }
    assert!(dir
        .path()
        .join("data/snapshots/rollback/backend.manifest")
        .is_file());
    let db = Embedded::open_with_config(config).expect("reopen");
    assert!(db.get("snapshot-test", "a0").expect("get A").is_none());
    assert!(db.get("snapshot-test", "b0").expect("get B").is_some());
}

#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_restore_legacy_without_manifest_restores_data_only() {
    use sha2::{Digest, Sha256};
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), false);
    let snapshot_segment_hash = {
        let db = Embedded::open_with_config(config).expect("open");
        let snapshot = db.create_snapshot("legacy").expect("snapshot");
        let hash = Sha256::digest(
            fs::read(snapshot.path.join("data/vstore_L0.vanta")).expect("snapshot segment"),
        )
        .to_vec();
        fs::remove_file(snapshot.path.join("backend.manifest")).expect("remove manifest");
        fs::remove_file(snapshot.path.join("snapshot.format")).expect("remove format marker");
        db.put(record("snapshot-test", "b0", "later", "snapshot"))
            .expect("put B");
        db.flush().expect("flush");
        hash
    };
    let backend_hashes = || {
        let mut hashes = std::collections::BTreeMap::new();
        for relative in snapshot_file_set(dir.path()) {
            let first = relative.components().next().expect("component").as_os_str();
            if first == "data" || first == ".vanta.lock" {
                continue;
            }
            hashes.insert(
                relative.clone(),
                Sha256::digest(fs::read(dir.path().join(relative)).expect("file")).to_vec(),
            );
        }
        hashes
    };
    let before = backend_hashes();
    assert_ne!(
        snapshot_segment_hash,
        Sha256::digest(fs::read(dir.path().join("data/vstore_L0.vanta")).expect("live segment"))
            .to_vec()
    );
    StorageEngine::snapshot_restore(dir.path(), "legacy").expect("legacy restore");
    assert_eq!(before, backend_hashes());
    assert_eq!(
        snapshot_segment_hash,
        Sha256::digest(
            fs::read(dir.path().join("data/vstore_L0.vanta")).expect("restored segment")
        )
        .to_vec()
    );
}

#[cfg(all(feature = "fjall", feature = "fs2"))]
#[test]
fn test_snapshot_restore_rejects_open_database() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    db.create_snapshot("busy").expect("snapshot");
    let err = StorageEngine::snapshot_restore(dir.path(), "busy")
        .expect_err("open database must be rejected");
    assert!(
        matches!(err, vantadb::error::Error::DatabaseBusy(_)),
        "{err:?}"
    );
}

#[cfg(all(feature = "fjall", feature = "failpoints"))]
#[test]
fn test_snapshot_backend_mirror_retries_when_unstable() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), false);
    let db = Embedded::open_with_config(config).expect("open");
    fail::cfg("snapshot_backend_unstable", "1*return").expect("arm");
    let snapshot = db.create_snapshot("retry");
    let consumed = fail::eval("snapshot_backend_unstable", |_| true).is_none();
    fail::remove("snapshot_backend_unstable");
    let snapshot = snapshot.expect("retry must succeed");
    assert!(consumed, "snapshot must exercise the instability failpoint");
    assert!(snapshot.path.join("backend.manifest").is_file());
    assert_eq!(
        snapshot_file_set(&dir.path().join("keyspaces")),
        snapshot_file_set(&snapshot.path.join("backend/keyspaces"))
    );
}

#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_skips_tmp_files() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    fs::write(dir.path().join("data/leftover.tmp"), b"leftover").expect("plant tmp");
    let snapshot = db.create_snapshot("tmp").expect("snapshot");
    assert!(dir.path().join("data/leftover.tmp").exists());
    assert!(!snapshot.path.join("data/leftover.tmp").exists());
}

#[cfg(feature = "fjall")]
#[test]
fn test_scheduled_restore_applies_on_next_open() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.create_snapshot("scheduled").expect("snapshot");
        db.delete("snapshot-test", "a0").expect("delete");
        db.put(record("snapshot-test", "b0", "later", "snapshot"))
            .expect("put");
        let marker =
            StorageEngine::schedule_snapshot_restore(dir.path(), "scheduled").expect("schedule");
        assert!(marker.is_file());
        assert!(db.get("snapshot-test", "a0").expect("get A").is_none());
        assert!(db.get("snapshot-test", "b0").expect("get B").is_some());
    }
    let db = Embedded::open_with_config(config).expect("reopen");
    assert!(!dir.path().join(".restore_pending").exists());
    assert!(db.get("snapshot-test", "a0").expect("get A").is_some());
    assert!(db.get("snapshot-test", "b0").expect("get B").is_none());
    assert_eq!(db.count("snapshot-test", None).expect("count"), 3);
}

#[cfg(feature = "fjall")]
#[test]
fn test_scheduled_restore_missing_snapshot_errors_without_marker() {
    let dir = tempdir().expect("tempdir");
    let _db = Embedded::open(dir.path()).expect("open");
    let err = StorageEngine::schedule_snapshot_restore(dir.path(), "missing").expect_err("missing");
    assert!(
        matches!(err, vantadb::error::Error::NotFound { .. }),
        "{err}"
    );
    assert!(!dir.path().join(".restore_pending").exists());
}

#[cfg(all(feature = "fjall", feature = "failpoints"))]
#[test]
fn test_scheduled_restore_failure_keeps_live_state_and_marker() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.create_snapshot("scheduled-fail").expect("snapshot");
        db.delete("snapshot-test", "a0").expect("delete");
        db.put(record("snapshot-test", "b0", "later", "snapshot"))
            .expect("put");
        db.flush().expect("flush");
        StorageEngine::schedule_snapshot_restore(dir.path(), "scheduled-fail").expect("schedule");
    }
    fail::cfg("snapshot_restore_backend_fail", "return").expect("arm");
    let err = Embedded::open_with_config(config.clone()).expect_err("restore must fail");
    fail::remove("snapshot_restore_backend_fail");
    assert!(
        err.to_string().contains("pending snapshot restore failed"),
        "{err}"
    );
    let marker = dir.path().join(".restore_pending");
    assert!(marker.is_file());
    fs::remove_file(marker).expect("cancel pending restore");
    let db = Embedded::open_with_config(config).expect("open current state");
    assert!(db.get("snapshot-test", "a0").expect("get A").is_none());
    assert!(db.get("snapshot-test", "b0").expect("get B").is_some());
}

#[cfg(feature = "fjall")]
#[test]
fn test_scheduled_restore_ignored_by_read_only_open() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), false);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open");
        db.create_snapshot("read-only-pending").expect("snapshot");
        db.put(record("snapshot-test", "b0", "later", "snapshot"))
            .expect("put");
        db.flush().expect("flush");
        StorageEngine::schedule_snapshot_restore(dir.path(), "read-only-pending")
            .expect("schedule");
    }
    let marker = dir.path().join(".restore_pending");
    let pending_lock = dir.path().join(".restore_pending.lock");
    fs::remove_file(&pending_lock).expect("simulate a database without the pending lock file");
    let stale_tmp = dir.path().join(".restore_pending.1.2.3.tmp");
    let unrelated = dir.path().join("unrelated.tmp");
    fs::write(&stale_tmp, b"incomplete").expect("plant stale temp");
    fs::write(&unrelated, b"keep").expect("plant unrelated temp");
    let before = fs::read(&marker).expect("marker");
    let read_only = vantadb::config::Config {
        read_only: true,
        ..config.clone()
    };
    let db = Embedded::open_with_config(read_only).expect("read-only open");
    assert!(db.get("snapshot-test", "b0").expect("get B").is_some());
    assert_eq!(before, fs::read(&marker).expect("marker still exists"));
    assert!(
        stale_tmp.is_file(),
        "read-only open must not clean temporary files"
    );
    assert!(
        !pending_lock.exists(),
        "read-only open must not create the pending lock"
    );
    assert!(unrelated.exists());
    drop(db);
    let db = Embedded::open_with_config(config).expect("writable open");
    assert!(!stale_tmp.exists());
    assert!(
        pending_lock.is_file(),
        "writable open creates the pending lock"
    );
    assert_eq!(fs::read(unrelated).expect("unrelated file"), b"keep");
    assert!(!marker.exists());
    assert!(db.get("snapshot-test", "b0").expect("get B").is_none());
}

#[cfg(feature = "fjall")]
#[test]
fn test_scheduled_restore_tmp_cleanup_without_pending_marker() {
    let dir = tempdir().expect("tempdir");
    {
        let _db = Embedded::open(dir.path()).expect("create database");
    }
    let stale_tmp = dir.path().join(".restore_pending.1.2.3.tmp");
    let unrelated = dir.path().join("unrelated.tmp");
    fs::write(&stale_tmp, b"incomplete").expect("plant stale temp");
    fs::write(&unrelated, b"keep").expect("plant unrelated temp");
    let _db = Embedded::open(dir.path()).expect("open without pending restore");
    assert!(!stale_tmp.exists());
    assert_eq!(fs::read(unrelated).expect("unrelated remains"), b"keep");
}

#[cfg(feature = "fjall")]
#[test]
fn test_schedule_restore_concurrent_writers_leave_valid_marker() {
    use std::collections::HashSet;
    use std::sync::{Arc, Barrier};

    let dir = tempdir().expect("tempdir");
    let config = snapshot_config(dir.path(), false);
    let db = Embedded::open_with_config(config.clone()).expect("open");
    for index in 0..8 {
        db.put(record(
            "schedule-race",
            &format!("key-{index}"),
            "snapshot",
            "race",
        ))
        .expect("put");
        db.create_snapshot(&format!("race-{index}"))
            .expect("snapshot");
    }
    db.put(record("schedule-race", "after", "later", "race"))
        .expect("put after snapshots");
    let marker = dir.path().join(".restore_pending");
    let mut winner = String::new();
    for _ in 0..50 {
        let barrier = Arc::new(Barrier::new(8));
        let successful = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|index| {
                    let barrier = barrier.clone();
                    let root = dir.path();
                    scope.spawn(move || {
                        let name = format!("race-{index}");
                        barrier.wait();
                        let success = StorageEngine::schedule_snapshot_restore(root, &name).is_ok();
                        (name, success)
                    })
                })
                .collect();
            handles
                .into_iter()
                .filter_map(|handle| {
                    let (name, success) = handle.join().expect("scheduler thread");
                    success.then_some(name)
                })
                .collect::<HashSet<_>>()
        });
        assert!(
            !successful.is_empty(),
            "at least one scheduler must succeed"
        );
        let contents = fs::read_to_string(&marker).expect("marker");
        let lines: Vec<_> = contents.lines().collect();
        assert_eq!(lines.len(), 2, "malformed marker: {contents:?}");
        assert_eq!(lines[0], "vantadb-restore-pending v1");
        assert!(
            successful.contains(lines[1]),
            "marker names an unsuccessful call: {contents:?}"
        );
        winner = lines[1].to_string();
        for entry in fs::read_dir(dir.path()).expect("root") {
            let name = entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
            assert!(
                !(name.starts_with(".restore_pending.") && name.ends_with(".tmp")),
                "leftover {name}"
            );
        }
    }
    let winner_index: usize = winner
        .strip_prefix("race-")
        .expect("prefix")
        .parse()
        .expect("index");
    drop(db);
    let db = Embedded::open_with_config(config).expect("apply winning restore");
    for index in 0..8 {
        let present = db
            .get("schedule-race", &format!("key-{index}"))
            .expect("get")
            .is_some();
        assert_eq!(
            present,
            index <= winner_index,
            "key-{index}, winner {winner}"
        );
    }
    assert!(db.get("schedule-race", "after").expect("get").is_none());
    assert!(!marker.exists());
}

#[cfg(all(feature = "fjall", feature = "encryption"))]
#[test]
fn test_scheduled_restore_recovers_encrypted_db() {
    let dir = tempdir().expect("tempdir");
    let config = snapshot_seed(dir.path(), true);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open encrypted");
        db.create_snapshot("encrypted-pending").expect("snapshot");
        db.delete("snapshot-test", "a0").expect("delete");
        db.put(record("snapshot-test", "b0", "later", "snapshot"))
            .expect("put");
        db.flush().expect("flush");
        StorageEngine::schedule_snapshot_restore(dir.path(), "encrypted-pending")
            .expect("schedule");
    }
    fs::remove_file(dir.path().join("data/encryption.registry")).expect("remove live registry");
    let db = Embedded::open_with_config(config).expect("restore before resolving encryption");
    for key in ["a0", "a1", "a2"] {
        assert_eq!(
            db.get("snapshot-test", key)
                .expect("get")
                .expect("A")
                .payload,
            "alpha snapshot original"
        );
    }
    assert!(db.get("snapshot-test", "b0").expect("get B").is_none());
    assert!(!dir.path().join(".restore_pending").exists());
}

#[cfg(all(feature = "fjall", feature = "encryption", feature = "fs2"))]
#[test]
fn test_open_locked_encrypted_database_reports_busy_before_missing_key() {
    let dir = tempdir().expect("tempdir");
    let keyed = snapshot_config(dir.path(), true);
    let _db = Embedded::open_with_config(keyed).expect("open encrypted");
    let unkeyed = vantadb::config::Config {
        file_lock_timeout_ms: 5,
        ..snapshot_config(dir.path(), false)
    };
    let err = Embedded::open_with_config(unkeyed).expect_err("locked database");
    assert!(
        matches!(err, vantadb::error::Error::DatabaseBusy(_)),
        "{err}"
    );
}

#[cfg(feature = "fjall")]
#[test]
fn test_scheduled_restore_malformed_marker_fails_closed() {
    let dir = tempdir().expect("tempdir");
    {
        let db = Embedded::open(dir.path()).expect("open");
        db.put(record("snapshot-test", "a0", "original", "snapshot"))
            .expect("put");
        db.flush().expect("flush");
    }
    let marker = dir.path().join(".restore_pending");
    fs::write(&marker, "vantadb-restore-pending v1\n../escape\n").expect("write malformed marker");
    let err = Embedded::open(dir.path()).expect_err("malformed marker");
    assert!(
        err.to_string().contains("pending snapshot restore failed"),
        "{err}"
    );
    assert!(marker.exists());
    fs::remove_file(marker).expect("cancel");
    let db = Embedded::open(dir.path()).expect("open current state");
    assert!(db.get("snapshot-test", "a0").expect("get").is_some());
}
