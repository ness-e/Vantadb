// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! SCH-06 / ADR-046 — migration chaos suite (failpoints + reopen + indexes).
//!
//! Contract: a failure during the v1 → v2 records backfill must never lose or
//! corrupt data, must never bump the schema header (the bump is the
//! migration-complete marker, ADR §Migration), and a re-run must complete the
//! migration deterministically. Recovery is exercised through the engine's
//! existing failpoints — no production instrumentation was added.
//!
//! Scenarios:
//! 1. `storage_insert_fail` on the **second** batch (`1*off->return`, 1.1k
//!    records = 2 chunks): the first chunk commits, the backfill aborts
//!    mid-migration → reopen + re-run + index rebuild must be whole.
//! 2. `wal_append_fail` on the **first** batch-append group (`return`): the
//!    write path fails inside chunk 0 before any group lands → the sharded
//!    WAL stays coherent → reopen + re-run must be whole.
//! 3. Witness (documented limitation, FIND in the task file): a transient
//!    failure **between** shard groups (`1*off->return`) — the shape a real
//!    crash mid-batch can leave — makes the ERR-011 guard refuse to recover
//!    ("aborting recovery instead of silently dropping data") until the
//!    opt-in `vanta-cli wal salvage` path (FIND-109) truncates each shard to
//!    the coherent prefix. The guard is deliberate: the test pins that the
//!    failure is loud, never a silent data drop.
//!
//! Requires the `failpoints` feature (`required-features` on the `[[test]]`).

use std::path::Path;

use tempfile::TempDir;
use vantadb::config::Config;
use vantadb::migration::MigrationEngine;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::schema::StorageHeader;
use vantadb::storage::{BackendKind, StorageEngine};
use vantadb::{Embedded, MemoryListOptions, MemorySearchRequest};

const NS: &str = "chaos-mig";
/// Two backfill chunks for scenario 1: `src/migration.rs` BACKFILL_BATCH = 1000.
const RECORDS: u64 = 1_100;
const FIRST_CHUNK: u64 = 1_000;
/// Small fixture: enough records for one multi-shard batch.
const SMALL_RECORDS: u64 = 8;

const FIELD_VALID_AT_MS: &str = "__vanta_valid_at_ms";

fn open_engine(path: &Path) -> vantadb::error::Result<StorageEngine> {
    let path_str = path.to_string_lossy().into_owned();
    let config = Config {
        storage_path: path_str.clone(),
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(&path_str, Some(config))
}

fn open_engine_or_panic(path: &Path) -> StorageEngine {
    open_engine(path).expect("open engine")
}

fn write_v1_header(dir: &Path) {
    StorageHeader {
        version: 1,
        flags: 0,
        min_compat_version: 1,
    }
    .write_to(&dir.join(".vanta.schema"))
    .expect("write v1 header");
}

/// v1 fixture: `records` record nodes (no v2 fields) + one plain node.
fn build_v1_database(dir: &Path, records: u64) {
    write_v1_header(dir);
    let engine = open_engine_or_panic(dir);
    for i in 0..records {
        let created_at = 1_000 + i;
        let mut node = UnifiedNode::new(1_000 + i as u128);
        node.set_field("__vanta_namespace", FieldValue::String(NS.into()));
        node.set_field("__vanta_key", FieldValue::String(format!("k{i:04}")));
        node.set_field(
            "__vanta_payload",
            FieldValue::String(format!("chaos payload {i:04}")),
        );
        node.set_field("__vanta_created_at_ms", FieldValue::Int(created_at as i64));
        node.set_field("__vanta_updated_at_ms", FieldValue::Int(created_at as i64));
        node.set_field("__vanta_version", FieldValue::Int(1));
        engine.insert(&node).expect("insert v1 record");
    }
    let mut plain = UnifiedNode::new(9_500);
    plain.set_field("type", FieldValue::String("Doc".into()));
    engine.insert(&plain).expect("insert plain node");
}

/// Every record readable with its original payload; returns (v2_count, v1_count).
fn assert_no_loss_and_count_backfilled(dir: &Path, records: u64) -> (u64, u64) {
    let engine = open_engine_or_panic(dir);
    let mut v2 = 0u64;
    let mut v1 = 0u64;
    for i in 0..records {
        let node = engine
            .get(1_000 + i as u128)
            .unwrap_or_else(|e| panic!("get record {i}: {e}"))
            .unwrap_or_else(|| panic!("record {i} lost after crash"));
        assert_eq!(
            node.get_field("__vanta_payload"),
            Some(&FieldValue::String(format!("chaos payload {i:04}"))),
            "record {i}: payload corrupted"
        );
        if node.get_field(FIELD_VALID_AT_MS).is_some() {
            v2 += 1;
        } else {
            v1 += 1;
        }
    }
    assert!(
        engine.get(9_500).expect("get plain").is_some(),
        "plain node lost after crash"
    );
    (v2, v1)
}

fn assert_header_is_v1(dir: &Path) {
    let header = StorageHeader::read_from(&dir.join(".vanta.schema"))
        .expect("read header")
        .expect("header present");
    assert_eq!(
        header.version, 1,
        "aborted migration must not bump the header (bump = complete marker)"
    );
}

fn finish_migration(dir: &Path, records: u64) {
    let report = MigrationEngine::new(dir)
        .migrate_records()
        .expect("re-run completes the backfill");
    assert_eq!(
        report.backfilled + report.already_v2,
        records,
        "re-run accounts for every record (no double-write, no gap)"
    );
    StorageHeader::current()
        .write_to(&dir.join(".vanta.schema"))
        .expect("bump header to v2");
}

/// Post-recovery index integrity: rebuild all derived indexes from the
/// recovered records and serve them through the SDK (list + text search).
fn assert_indexes_serve_all_records(dir: &Path, records: u64) {
    let db = Embedded::open(dir).expect("open recovered db");
    db.rebuild_index().expect("rebuild indexes");
    let page = db
        .list(
            NS,
            MemoryListOptions {
                limit: (records as usize) + 10,
                ..Default::default()
            },
        )
        .expect("list");
    assert_eq!(
        page.records.len() as u64,
        records,
        "namespace/payload indexes must serve every recovered record"
    );
    let hits = db
        .search(MemorySearchRequest {
            namespace: NS.to_string(),
            text_query: Some("chaos".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("search recovered");
    assert!(!hits.is_empty(), "text index serves recovered records");
    let audit = db
        .audit_text_index(Some(NS))
        .expect("audit text index after recovery");
    assert!(audit.passed, "text index must be consistent: {audit:?}");
}

#[test]
fn chaos_migration_failpoint_aborts_and_recovers_with_index_integrity() {
    let _scenario = vantadb::FailScenario::setup();

    // ── Scenario 1: storage_insert_fail on the SECOND chunk ─────────────
    // `1*off->return`: the first batch prelude passes, the second errors.
    let dir_a = TempDir::new().expect("tempdir a");
    build_v1_database(dir_a.path(), RECORDS);
    fail::cfg("storage_insert_fail", "1*off->return").expect("arm storage_insert_fail");
    let aborted = MigrationEngine::new(dir_a.path()).migrate_records();
    fail::remove("storage_insert_fail");
    assert!(
        aborted.is_err(),
        "the second batch must abort the migration, got {aborted:?}"
    );

    // Header untouched: the bump is the migration-complete marker.
    assert_header_is_v1(dir_a.path());
    assert!(
        MigrationEngine::new(dir_a.path())
            .records_backfill_pending()
            .expect("pending check"),
        "the backfill must still be pending after the abort"
    );

    // No loss; the first chunk committed before the abort (deterministic
    // scan order: lowest node ids first).
    let (v2, v1) = assert_no_loss_and_count_backfilled(dir_a.path(), RECORDS);
    assert_eq!(v2, FIRST_CHUNK, "first chunk committed before the abort");
    assert_eq!(v1, RECORDS - FIRST_CHUNK, "second chunk untouched");

    // Re-run completes; then indexes serve everything.
    finish_migration(dir_a.path(), RECORDS);
    let (v2, v1) = assert_no_loss_and_count_backfilled(dir_a.path(), RECORDS);
    assert_eq!((v2, v1), (RECORDS, 0), "re-run backfills the remainder");
    assert_indexes_serve_all_records(dir_a.path(), RECORDS);

    // ── Scenario 2: wal_append_fail on the first batch-append group ─────
    // The sharded WAL groups a batch per shard; failing the first group call
    // (failpoint `return` = every call while armed, disarmed right after)
    // aborts the batch before any group lands → shard counts stay coherent.
    let dir_b = TempDir::new().expect("tempdir b");
    build_v1_database(dir_b.path(), SMALL_RECORDS);
    fail::cfg("wal_append_fail", "return").expect("arm wal_append_fail");
    let aborted = MigrationEngine::new(dir_b.path()).migrate_records();
    fail::remove("wal_append_fail");
    assert!(
        aborted.is_err(),
        "a WAL failure inside a batch must abort the migration, got {aborted:?}"
    );
    assert_header_is_v1(dir_b.path());
    // Reopen must succeed: the abort left no shard-count imbalance.
    assert_no_loss_and_count_backfilled(dir_b.path(), SMALL_RECORDS);

    finish_migration(dir_b.path(), SMALL_RECORDS);
    let (v2, v1) = assert_no_loss_and_count_backfilled(dir_b.path(), SMALL_RECORDS);
    assert_eq!(
        (v2, v1),
        (SMALL_RECORDS, 0),
        "re-run completes after WAL crash"
    );
    assert_indexes_serve_all_records(dir_b.path(), SMALL_RECORDS);

    // ── Scenario 3: failure BETWEEN shard groups (ERR-011 witness) ──────
    // `1*off->return` lets shard 0's group land and fails shard 1's — the
    // shape a real crash mid-batch-append can leave. The ERR-011 guard then
    // refuses recovery (loud, explicit) instead of silently replaying a
    // truncated shard; the documented recovery path is the opt-in
    // `vanta-cli wal salvage` (FIND-109). Witnessed here so the behaviour
    // cannot silently change.
    let dir_c = TempDir::new().expect("tempdir c");
    build_v1_database(dir_c.path(), SMALL_RECORDS);
    fail::cfg("wal_append_fail", "1*off->return").expect("arm wal_append_fail");
    let aborted = MigrationEngine::new(dir_c.path()).migrate_records();
    fail::remove("wal_append_fail");
    assert!(
        aborted.is_err(),
        "the inter-group WAL failure must abort the migration, got {aborted:?}"
    );
    let reopen = open_engine(dir_c.path());
    match reopen {
        Ok(_) => panic!(
            "ERR-011 witness moved: the incoherent shard layout was recovered \
             automatically — remove this witness and re-verify the guard tests"
        ),
        Err(err) => {
            let msg = err.to_string();
            assert!(
                msg.contains("WAL shard") && msg.contains("aborting recovery"),
                "expected the ERR-011 fail-loud message, got: {msg}"
            );
        }
    }
}
