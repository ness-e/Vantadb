// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! SCH-06 / ADR-0046 — v1 → v2 migration determinism suite.
//!
//! The gate (ADR §Migration): "misma DB v1 ⇒ resultado byte-idéntico". This
//! suite builds a real v1 database (schema header v1 + record nodes without the
//! v2 fields, the exact shape `is_memory_record_node`/`backfill_record_node`
//! key on), copies it, migrates both copies through the same sequence the CLI
//! runs (`records` backfill → schema bump last), and compares the result.
//!
//! Byte scope: the comparison covers the migration's data artifacts — the
//! full on-disk directory **minus** engine-internal write-path journals. The
//! engine stamps `last_accessed = now_ms()` on every WAL append
//! (`src/storage/engine/insert.rs:211`, `get.rs:190`) — a wall-clock value
//! owned by the write path, not by the backfill (`src/migration.rs` is a pure
//! function of stored values). The strict comparison asserts identical file
//! sets and identical bytes for every file; the volatile journal/WAL files are
//! compared by presence + non-emptiness only, and any *other* difference fails
//! the test (see `assert_dirs_byte_identical`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use vantadb::config::Config;
use vantadb::migration::MigrationEngine;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::schema::StorageHeader;
use vantadb::storage::{BackendKind, StorageEngine};

const NS: &str = "mig/ns";

/// Field names of the v1 record shape that are NOT exported through
/// `vantadb::sdk` (only 7 FIELD_* consts are public). Pinned as wire literals
/// on purpose — this suite is a compatibility gate, so a rename upstream must
/// fail here (same precedent as `tests/memory_api.rs:624`).
const FIELD_SUPERSEDED_BY: &str = "__vanta_superseded_by";
const FIELD_SUPERSEDED_AT_MS: &str = "__vanta_superseded_at_ms";
const FIELD_VALID_AT_MS: &str = "__vanta_valid_at_ms";
const FIELD_INVALID_AT_MS: &str = "__vanta_invalid_at_ms";
const FIELD_CONFIDENCE_CLASS: &str = "__vanta_confidence_class";

/// Number of record nodes in the fixture (small: determinism is the point,
/// not volume; the chaos suite exercises the multi-batch path).
const RECORDS: u64 = 32;

fn open_engine(path: &Path) -> StorageEngine {
    let path_str = path.to_string_lossy().into_owned();
    let config = Config {
        storage_path: path_str.clone(),
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(&path_str, Some(config)).expect("open engine")
}

/// Write the v1 schema header (migration-complete marker absent).
fn write_v1_header(dir: &Path) {
    StorageHeader {
        version: 1,
        flags: 0,
        min_compat_version: 1,
    }
    .write_to(&dir.join(".vanta.schema"))
    .expect("write v1 header");
}

/// One record's deterministic expected values (assertions use these).
struct ExpectedRecord {
    id: u128,
    key: String,
    created_at: u64,
    superseded_at: Option<u64>,
    superseded_by: Option<String>,
}

/// Deterministic v1 fixture: `RECORDS` record nodes (every 4th superseded)
/// + one plain graph node that must stay untouched.
fn build_v1_database(dir: &Path) -> Vec<ExpectedRecord> {
    write_v1_header(dir);
    let mut expected = Vec::with_capacity(RECORDS as usize);
    let engine = open_engine(dir);
    for i in 0..RECORDS {
        let created_at = 1_000 + i * 10;
        let superseded = if i % 4 == 3 {
            let by = format!("k{:02}", (i + 1) % RECORDS);
            Some((by, created_at + 5))
        } else {
            None
        };
        let mut node = UnifiedNode::new(1_000 + i as u128);
        node.set_field("__vanta_namespace", FieldValue::String(NS.into()));
        node.set_field("__vanta_key", FieldValue::String(format!("k{i:02}")));
        node.set_field(
            "__vanta_payload",
            FieldValue::String(format!("payload-{i:02}")),
        );
        node.set_field("__vanta_created_at_ms", FieldValue::Int(created_at as i64));
        node.set_field("__vanta_updated_at_ms", FieldValue::Int(created_at as i64));
        node.set_field("__vanta_version", FieldValue::Int(1));
        if let Some((by, at)) = &superseded {
            node.set_field(FIELD_SUPERSEDED_BY, FieldValue::String(by.clone()));
            node.set_field(FIELD_SUPERSEDED_AT_MS, FieldValue::Int(*at as i64));
        }
        engine.insert(&node).expect("insert v1 record");
        expected.push(ExpectedRecord {
            id: 1_000 + i as u128,
            key: format!("k{i:02}"),
            created_at,
            superseded_at: superseded.as_ref().map(|(_, at)| *at),
            superseded_by: superseded.map(|(by, _)| by),
        });
    }
    // Non-record node: the backfill must not touch it (structural filter).
    let mut plain = UnifiedNode::new(2_000);
    plain.set_field("type", FieldValue::String("Doc".into()));
    engine.insert(&plain).expect("insert plain node");
    expected
}

/// Run the CLI's `--format all` sequence for a records-only DB:
/// backfill first, schema bump LAST (ADR §Migration, expand → backfill → bump).
fn migrate_full(path: &Path) -> vantadb::migration::RecordsBackfillReport {
    let report = MigrationEngine::new(path)
        .migrate_records()
        .expect("records backfill");
    StorageHeader::current()
        .write_to(&path.join(".vanta.schema"))
        .expect("bump header to v2");
    report
}

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create dst");
    for entry in fs::read_dir(src).expect("read src") {
        let entry = entry.expect("entry");
        let target = dst.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

/// Recursive (relative path → bytes) map of a directory.
fn dir_bytes_map(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if entry.file_type().expect("file type").is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .expect("rel path")
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, fs::read(&path).expect("read file"));
            }
        }
    }
    let mut map = BTreeMap::new();
    if root.exists() {
        walk(root, root, &mut map);
    }
    map
}

/// Engine-internal journals carry the write-path wall clock (`last_accessed`)
/// and are excluded from the strict byte comparison — see the module docs.
fn is_volatile_engine_journal(rel: &str) -> bool {
    rel.contains("wal") || rel.contains("journal") || rel.ends_with("db.lock")
}

/// Strict comparison: same file set everywhere; byte-equality for every file
/// except volatile engine journals (present + non-empty). Any other mismatch
/// fails with the first differing paths listed.
fn assert_dirs_byte_identical(a: &Path, b: &Path) {
    let (ma, mb) = (dir_bytes_map(a), dir_bytes_map(b));
    let keys_a: Vec<&String> = ma.keys().collect();
    let keys_b: Vec<&String> = mb.keys().collect();
    assert_eq!(
        keys_a, keys_b,
        "migrated copies must contain the same files"
    );
    let volatile: Vec<&String> = ma
        .keys()
        .filter(|rel| is_volatile_engine_journal(rel))
        .collect();
    eprintln!("SCH-06 volatile engine journals (excluded from strict bytes): {volatile:?}");
    let mut strict_mismatches = Vec::new();
    for (rel, bytes_a) in &ma {
        let bytes_b = &mb[rel];
        if is_volatile_engine_journal(rel) {
            assert!(
                !bytes_a.is_empty() && !bytes_b.is_empty(),
                "volatile journal {rel} must be non-empty in both copies"
            );
            continue;
        }
        if bytes_a != bytes_b {
            let first_diff = bytes_a
                .iter()
                .zip(bytes_b.iter())
                .position(|(x, y)| x != y)
                .unwrap_or(bytes_a.len().min(bytes_b.len()));
            strict_mismatches.push(format!(
                "{rel} (len {} vs {}, first diff at byte {first_diff})",
                bytes_a.len(),
                bytes_b.len()
            ));
        }
    }
    assert!(
        strict_mismatches.is_empty(),
        "migration output must be byte-identical; differing files: {strict_mismatches:#?}"
    );
}

fn fresh_fixture() -> (TempDir, Vec<ExpectedRecord>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let expected = build_v1_database(dir.path());
    (dir, expected)
}

// ─── C1: doble corrida byte-idéntica sobre copia ─────────────────────────

#[test]
fn migration_v1_to_v2_is_byte_identical_across_copies() {
    let (source, _expected) = fresh_fixture();
    let copy_a = tempfile::tempdir().expect("copy a");
    let copy_b = tempfile::tempdir().expect("copy b");
    copy_dir_all(source.path(), copy_a.path());
    copy_dir_all(source.path(), copy_b.path());
    // Sanity: the two copies start byte-identical (comparison machinery).
    assert_dirs_byte_identical(copy_a.path(), copy_b.path());

    let report_a = migrate_full(copy_a.path());
    let report_b = migrate_full(copy_b.path());

    assert_eq!(report_a.scanned, RECORDS, "every record node is scanned");
    assert_eq!(
        report_a.backfilled, RECORDS,
        "every v1 record is backfilled"
    );
    assert_eq!(report_a.already_v2, 0);
    assert_eq!(report_b.scanned, report_a.scanned);
    assert_eq!(report_b.backfilled, report_a.backfilled);

    assert_dirs_byte_identical(copy_a.path(), copy_b.path());
}

// ─── C1: valores de referencia del backfill (ADR §Migration verbatim) ────

#[test]
fn migration_backfills_adr_reference_values_on_every_record() {
    let (dir, expected) = fresh_fixture();
    let report = migrate_full(dir.path());
    assert_eq!(report.backfilled, RECORDS);

    let engine = open_engine(dir.path());
    for record in &expected {
        let node = engine
            .get(record.id)
            .expect("get")
            .expect("record node present");
        assert_eq!(
            node.get_field(FIELD_VALID_AT_MS),
            Some(&FieldValue::Int(record.created_at as i64)),
            "{}: valid_at := created_at",
            record.key
        );
        match record.superseded_at {
            Some(at) => assert_eq!(
                node.get_field(FIELD_INVALID_AT_MS),
                Some(&FieldValue::Int(at as i64)),
                "{}: invalid_at := superseded_at",
                record.key
            ),
            None => assert!(
                node.get_field(FIELD_INVALID_AT_MS).is_none(),
                "{}: open window stays open",
                record.key
            ),
        }
        assert_eq!(
            node.get_field(FIELD_SUPERSEDED_BY),
            record
                .superseded_by
                .as_ref()
                .map(|by| FieldValue::String(by.clone()))
                .as_ref(),
            "{}: supersession metadata untouched",
            record.key
        );
        assert_eq!(
            node.get_field(FIELD_CONFIDENCE_CLASS),
            Some(&FieldValue::String("Asserted".into())),
            "{}: class := Asserted",
            record.key
        );
        assert_eq!(
            node.confidence_score, 1.0,
            "{}: confidence := D_a",
            record.key
        );
    }

    // Non-record node: untouched (structural filter).
    let plain = engine.get(2_000).expect("get plain").expect("plain node");
    assert!(
        plain.get_field(FIELD_VALID_AT_MS).is_none(),
        "non-record nodes must not be backfilled"
    );
    assert!(plain.get_field(FIELD_CONFIDENCE_CLASS).is_none());
}

// ─── C1: dry-run no escribe + re-run idempotente ─────────────────────────

#[test]
fn migration_dry_run_writes_nothing_and_rerun_skips_already_v2() {
    let (dir, _expected) = fresh_fixture();
    let before = dir_bytes_map(dir.path());
    let schema_before = before
        .get(".vanta.schema")
        .expect("schema header present")
        .clone();
    let vstore_before = before.get("vector_store.vanta").cloned();

    let mut engine = MigrationEngine::new(dir.path());
    engine.set_dry_run(true);
    let dry = engine.migrate_records().expect("dry-run");
    assert_eq!(dry.backfilled, RECORDS, "dry-run reports what would change");

    // Data artifacts unchanged: the header is not rewritten and the record
    // data file is untouched. (Opening the engine may still compact fjall's
    // own LSM bookkeeping — engine recovery, not migration output.)
    let after = dir_bytes_map(dir.path());
    assert_eq!(
        after.get(".vanta.schema"),
        Some(&schema_before),
        "dry-run must not rewrite the schema header"
    );
    if let Some(before) = vstore_before {
        assert_eq!(
            after.get("vector_store.vanta"),
            Some(&before),
            "dry-run must not rewrite the record data"
        );
    }

    // Data-level no-op: the v2 marker is still absent.
    let raw = open_engine(dir.path());
    let node = raw.get(1_000).expect("get").expect("node");
    assert!(
        node.get_field(FIELD_VALID_AT_MS).is_none(),
        "dry-run must not backfill records"
    );
    drop(raw);

    // Real backfill (no header bump yet — still pending).
    let first = MigrationEngine::new(dir.path())
        .migrate_records()
        .expect("backfill");
    assert_eq!(first.backfilled, RECORDS);
    assert_eq!(first.already_v2, 0);

    // Idempotence: a re-run recalculates nothing (v2 marker present).
    let second = MigrationEngine::new(dir.path())
        .migrate_records()
        .expect("re-run");
    assert_eq!(second.backfilled, 0);
    assert_eq!(second.already_v2, RECORDS);

    // The values are still exactly the reference values (no double-write).
    let raw = open_engine(dir.path());
    let node = raw.get(1_000).expect("get").expect("node");
    assert_eq!(
        node.get_field(FIELD_VALID_AT_MS),
        Some(&FieldValue::Int(1_000))
    );
    assert!(
        raw.get(2_000).expect("get plain").is_some(),
        "plain node must survive re-runs"
    );
}

// ─── D7 boundary: un binario v2 lee una DB v1 sin migrar (normalización) ─

#[test]
fn v2_binary_reads_unmigrated_v1_database_through_normalization() {
    let (dir, expected) = fresh_fixture();
    // No migration: the v2 read boundary normalizes the v1 node directly
    // (ADR §D7 #1 — the SDK read path is the normalization point).
    let engine = open_engine(dir.path());

    let node = engine.get(1_000).expect("get").expect("record node");
    let record = vantadb::sdk::record_from_node(&node).expect("v2 reader normalizes a v1 node");
    assert_eq!(
        record.valid_at_ms, expected[0].created_at,
        "v1 normalization on read: valid_at := created_at"
    );
    assert_eq!(
        record.invalid_at_ms, None,
        "v1 nodes carry no invalid_at (D2 default None; supersede aligns it on write)"
    );
    assert_eq!(
        record.confidence,
        vantadb::sdk::default_confidence(),
        "v1 node reads D_a, not the legacy node-level 0.5"
    );
    assert_eq!(record.quarantined_at_ms, None);

    // Supersession metadata survives the v1 read (it is v1-era state).
    let node = engine.get(1_003).expect("get").expect("record node");
    let superseded = vantadb::sdk::record_from_node(&node).expect("normalize");
    assert_eq!(superseded.superseded_by.as_deref(), Some("k04"));
    assert_eq!(superseded.superseded_at_ms, Some(1_035));
}
