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

use vantadb::index::{CPIndex, HnswConfig, VectorRepresentations};
use vantadb::node::{
    DiskNodeHeader, DistanceMetric, FieldValue, FilterBitset, NodeFlags, UnifiedNode,
};
use vantadb::sdk::*;
use vantadb::storage::{BackendKind, StorageEngine};
use vantadb::wal::{WalReader, WalRecord, WalWriter};
use vantadb::VantaEmbedded;
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

fn str_value(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

fn record(namespace: &str, key: &str, payload: &str, category: &str) -> VantaMemoryInput {
    let mut input = VantaMemoryInput::new(namespace, key, payload);
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
        };
        let index = CPIndex::new_with_config(config);

        for (id, vec) in &dataset {
            index.add(
                *id,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
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
        for run in 0..3 {
            let index = CPIndex::new_with_config(HnswConfig::default());
            for (id, vec) in &dataset {
                index.add(
                    *id,
                    FilterBitset::all_set(),
                    VectorRepresentations::Full(vec.clone()),
                    0,
                );
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
            TerminalReporter::info(&format!("  Run {}: Recall@10 = {:.4}", run + 1, recall));
            recalls.push(recall);
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

    harness.execute("Export produces schema_version=1", || {
        let source_dir = tempdir().expect("source tempdir");
        let export_path = source_dir.path().join("export.jsonl");

        let source = VantaEmbedded::open(source_dir.path()).expect("open source");
        source
            .put(record("ns/snap", "a", "snapshot payload", "test"))
            .expect("put");
        source.flush().expect("flush");

        source
            .export_namespace(&export_path, "ns/snap")
            .expect("export");

        // Read the raw JSONL and verify schema_version field
        let content = fs::read_to_string(&export_path).expect("read export");
        let line = content.trim();
        let parsed: serde_json::Value = serde_json::from_str(line).expect("parse export line");
        assert_eq!(
            parsed["schema_version"].as_u64(),
            Some(1),
            "Export schema_version must be 1, got: {:?}",
            parsed["schema_version"]
        );

        TerminalReporter::success("Export format schema_version=1 snapshot confirmed.");
    });

    harness.execute(
        "Round-trip: export → import → re-export produces identical JSONL",
        || {
            let source_dir = tempdir().expect("source tempdir");
            let target_dir = tempdir().expect("target tempdir");
            let export1 = source_dir.path().join("round1.jsonl");
            let export2 = target_dir.path().join("round2.jsonl");

            {
                let source = VantaEmbedded::open(source_dir.path()).expect("open source");
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
                let target = VantaEmbedded::open(target_dir.path()).expect("open target");
                let import = target.import_file(&export1).expect("import");
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

        let source = VantaEmbedded::open(source_dir.path()).expect("open source");
        source
            .put(record("ns/upd", "a", "original payload", "test"))
            .expect("put original");
        source.flush().expect("flush");
        source.export_all(&export_path).expect("export");

        let target = VantaEmbedded::open(target_dir.path()).expect("open target");
        target
            .put(record("ns/upd", "a", "stale payload", "test"))
            .expect("seed stale");

        let import = target.import_file(&export_path).expect("import");
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

    harness.execute(
        "All VantaMemoryExportLine fields survive round-trip",
        || {
            let dir = tempdir().expect("tempdir");
            let export_path = dir.path().join("fidelity.jsonl");

            let db = VantaEmbedded::open(dir.path()).expect("open");
            let mut input = VantaMemoryInput::new("ns/fidelity", "key-f", "fidelity payload");
            input
                .metadata
                .insert("int_field".to_string(), VantaValue::Int(42));
            input
                .metadata
                .insert("float_field".to_string(), VantaValue::Float(2.72));
            input
                .metadata
                .insert("bool_field".to_string(), VantaValue::Bool(true));
            input.vector = Some(vec![0.5, 0.5, 0.5]);
            input.ttl_ms = Some(86400000);
            db.put(input).expect("put");
            db.flush().expect("flush");

            db.export_all(&export_path).expect("export");

            let restore_dir = tempdir().expect("restore dir");
            let restored = VantaEmbedded::open(restore_dir.path()).expect("open restored");
            restored.import_file(&export_path).expect("import");

            let record = restored
                .get("ns/fidelity", "key-f")
                .expect("get")
                .expect("record");
            assert_eq!(record.payload, "fidelity payload");
            assert_eq!(record.metadata.get("int_field"), Some(&VantaValue::Int(42)));
            assert_eq!(
                record.metadata.get("float_field"),
                Some(&VantaValue::Float(2.72))
            );
            assert_eq!(
                record.metadata.get("bool_field"),
                Some(&VantaValue::Bool(true))
            );
            assert_eq!(record.vector, Some(vec![0.5, 0.5, 0.5]));

            TerminalReporter::success("All export line fields survive round-trip.");
        },
    );
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

        let target = VantaEmbedded::open(target_dir.path()).expect("open target");
        let import = target.import_file(&export_path).expect("import");
        assert_eq!(import.inserted, 3);
        assert_eq!(import.skipped, 4);
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

            let config = vantadb::config::VantaConfig {
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
                assert!(
                    hnsw.nodes.contains_key(&101),
                    "WAL replay must recover node 101"
                );
                assert!(
                    hnsw.nodes.contains_key(&102),
                    "WAL replay must recover node 102"
                );
                assert!(
                    hnsw.nodes.contains_key(&103),
                    "WAL replay must recover node 103"
                );
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
    let config = vantadb::config::VantaConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(path.to_str().unwrap(), Some(config))
        .expect("open StorageEngine")
}

/// Path to the VantaFile inside a storage engine's data directory.
fn vf_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("data").join("vector_store.vanta")
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

    harness.execute(
        "VantaFile bytes are deterministic for identical input",
        || {
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
                "VantaFile sizes must match for identical inputs"
            );

            // Skip the first 16 bytes (VantaHeader: magic + version + timestamp).
            // The write-cursor at bytes 16..24 and all subsequent node data
            // must be byte-identical.
            assert_eq!(
                bytes1[16..],
                bytes2[16..],
                "VantaFile data after header must be deterministic"
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

            TerminalReporter::success("VantaFile serialization is deterministic.");
        },
    );
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

            // Node 0: Full(f32) vector — persisted in VantaFile
            let full_data = vec![0.25, 0.50, 0.75, 1.00];
            let mut node_full = UnifiedNode::with_vector(100, full_data.clone());
            node_full
                .relational
                .insert("type".into(), FieldValue::String("full".into()));
            engine.insert(&node_full).expect("insert full");

            // Node 1: SQ8(i8[], scale) vector — NOT persisted in VantaFile (vec_len=0)
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
            // VantaFile stores vec_len=0 for non-Full vectors.
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
                        .insert(&UnifiedNode::with_vector(i as u128, vec![i as f32; 4]))
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

                // Verify raw VantaFile has tombstone flag set for node 1
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

            // Phase 2: reopen — tombstone persisted in VantaFile
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

            let db = VantaEmbedded::open(dir.path()).expect("open");
            let mut input = VantaMemoryInput::new("ns/golden", "golden-key", "golden payload");
            input
                .metadata
                .insert("score".to_string(), VantaValue::Float(99.5));
            input
                .metadata
                .insert("active".to_string(), VantaValue::Bool(true));
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
            assert_eq!(parsed["schema_version"].as_u64(), Some(1));
            assert_eq!(parsed["namespace"].as_str(), Some("ns/golden"));
            assert_eq!(parsed["key"].as_str(), Some("golden-key"));
            assert_eq!(parsed["payload"].as_str(), Some("golden payload"));
            assert!(parsed["created_at_ms"].as_u64().unwrap_or(0) > 0);
            assert!(parsed["updated_at_ms"].as_u64().unwrap_or(0) > 0);
            assert_eq!(parsed["version"].as_u64(), Some(0));
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

            // metadata sub-object
            let meta = &parsed["metadata"];
            assert_eq!(meta["score"].as_f64(), Some(99.5));
            assert_eq!(meta["active"].as_bool(), Some(true));

            // Round-trip: import into fresh DB
            let restore_dir = tempdir().expect("restore dir");
            let restored = VantaEmbedded::open(restore_dir.path()).expect("open restored");
            let import = restored.import_file(&export_path).expect("import");
            assert_eq!(import.inserted, 1);
            assert_eq!(import.errors, 0);

            let record = restored
                .get("ns/golden", "golden-key")
                .expect("get")
                .expect("record");
            assert_eq!(record.payload, "golden payload");
            assert_eq!(record.metadata.get("score"), Some(&VantaValue::Float(99.5)));
            assert_eq!(record.metadata.get("active"), Some(&VantaValue::Bool(true)));
            assert_eq!(record.vector, Some(vec![0.1, 0.2, 0.3, 0.4]));

            TerminalReporter::success("VantaFile export golden file structure certified.");
        },
    );
}
