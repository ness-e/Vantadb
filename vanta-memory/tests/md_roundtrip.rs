// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration test for MEM-62 round-trip: export MD → import MD preserves
//! the records byte-for-byte (idempotency via content-hash). Runs against an
//! in-memory Embedded so it doesn't need the `fjall` feature.

use std::collections::BTreeMap;

use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata, Value};
use vantadb::storage::BackendKind;

#[test]
fn round_trip_put_export_import_get() {
    let db = Embedded::open_with_config(Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    })
    .expect("open in-memory db");

    // Put 3 records with mixed metadata shapes.
    let mut meta1 = MemoryMetadata::new();
    meta1.insert("author".into(), Value::String("alice".into()));
    meta1.insert("version".into(), Value::Int(1));
    db.put(MemoryInput {
        namespace: "agent/team".into(),
        key: "intro".into(),
        payload: "Welcome to the team.".into(),
        metadata: meta1,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put intro");

    let mut meta2 = MemoryMetadata::new();
    meta2.insert(
        "tags".into(),
        Value::ListString(vec!["a".into(), "b".into()]),
    );
    db.put(MemoryInput {
        namespace: "agent/team".into(),
        key: "handbook".into(),
        payload: "Always commit before merging.".into(),
        metadata: meta2,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put handbook");

    let mut meta3 = MemoryMetadata::new();
    meta3.insert("priority".into(), Value::Float(0.5));
    db.put(MemoryInput {
        namespace: "agent/team".into(),
        key: "oncall".into(),
        payload: "Weekly rotations: alice → bob.".into(),
        metadata: meta3,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put oncall");

    // Walk the records and render them as MD strings using the same shape
    // that cli_handlers::export_md::render_record_md produces.
    let mut md_files: BTreeMap<String, String> = BTreeMap::new();
    for ns in ["agent/team"] {
        let page = db
            .list(
                ns,
                vantadb::sdk::MemoryListOptions {
                    limit: 100,
                    ..Default::default()
                },
            )
            .expect("list");
        for r in &page.records {
            md_files.insert(
                format!("{ns}/{}.md", r.key),
                vanta_memory::seed::test_render_md(r),
            );
        }
    }
    assert_eq!(md_files.len(), 3, "should have rendered 3 MD files");

    // Now simulate import: drop the namespace from a fresh in-memory db,
    // then import via import_md_dir.
    let db2 = Embedded::open_with_config(Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    })
    .expect("open in-memory db #2");

    // Write each MD to a tempdir then call import_md_dir.
    let tmp = std::env::temp_dir().join(format!("vanta-md-roundtrip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("mkdir tmp");
    for (path, content) in &md_files {
        let full = tmp.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("mkdir parent");
        }
        std::fs::write(&full, content).expect("write md");
    }

    let counts = vanta_memory::seed::import_md_dir(&db2, &tmp).expect("import_md_dir");
    assert_eq!(counts.created, 3, "fresh import: 3 created");
    assert_eq!(counts.updated, 0);
    assert_eq!(counts.unchanged, 0);

    // Re-import → idempotent: all records should report unchanged.
    let counts2 = vanta_memory::seed::import_md_dir(&db2, &tmp).expect("re-import");
    assert_eq!(counts2.created, 0, "re-import: nothing new");
    assert_eq!(counts2.updated, 0, "re-import: nothing updated");
    assert_eq!(counts2.unchanged, 3, "re-import: all unchanged");

    // Spot-check the round-tripped payload is preserved.
    let got = db2
        .get("agent/team", "intro")
        .expect("get intro")
        .expect("intro exists");
    assert_eq!(got.payload, "Welcome to the team.");
    assert!(matches!(
        got.metadata.get("author"),
        Some(Value::String(s)) if s == "alice"
    ));

    let got2 = db2
        .get("agent/team", "handbook")
        .expect("get handbook")
        .expect("handbook exists");
    assert!(matches!(
        got2.metadata.get("tags"),
        Some(Value::ListString(xs)) if xs == &vec!["a".to_string(), "b".to_string()]
    ));

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn round_trip_preserves_v2_semantics_and_strips_wikilink_block() {
    use vantadb::sdk::ConfidenceClass;

    let db = Embedded::open_with_config(Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    })
    .expect("open in-memory db");

    db.put(MemoryInput::new("agent/team", "parent", "parent body"))
        .expect("put parent");
    let mut child = MemoryInput::new("agent/team", "child", "child body");
    child.confidence_class = Some(ConfidenceClass::Derived);
    child.derived_from = Some(vec!["parent".into()]);
    child.valid_at_ms = Some(1_234);
    db.put(child).expect("put child");

    // Render with the fixture renderer (same shape as the CLI handler).
    let mut md_files: BTreeMap<String, String> = BTreeMap::new();
    let page = db
        .list(
            "agent/team",
            vantadb::sdk::MemoryListOptions {
                limit: 100,
                ..Default::default()
            },
        )
        .expect("list");
    for r in &page.records {
        md_files.insert(
            format!("agent/team/{}.md", r.key),
            vanta_memory::seed::test_render_md(r),
        );
    }
    assert!(
        md_files["agent/team/child.md"].contains("- [[agent/team/parent]]"),
        "derived parent renders as a wikilink"
    );

    let tmp = std::env::temp_dir().join(format!("vanta-md-v2-roundtrip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("mkdir tmp");
    for (path, content) in &md_files {
        let full = tmp.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("mkdir parent");
        }
        std::fs::write(&full, content).expect("write md");
    }

    let db2 = Embedded::open_with_config(Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    })
    .expect("open in-memory db #2");

    let counts = vanta_memory::seed::import_md_dir(&db2, &tmp).expect("import");
    assert_eq!(counts.created, 2);
    assert_eq!(counts.links_unresolved, 0, "parent link resolves");

    let got = db2
        .get("agent/team", "child")
        .expect("get child")
        .expect("child exists");
    assert_eq!(
        got.payload, "child body",
        "related block stripped from payload"
    );
    assert_eq!(got.confidence_class, ConfidenceClass::Derived);
    assert_eq!(got.derived_from, vec!["parent".to_string()]);
    assert_eq!(
        got.valid_at_ms, 1_234,
        "explicit validity start round-trips"
    );
    assert!(
        got.confidence > 0.0 && got.confidence <= 1.0,
        "computed derivation score round-trips"
    );
    assert_eq!(got.quarantined_at_ms, None);

    // Idempotent re-import: projection-stable, nothing rewritten.
    let again = vanta_memory::seed::import_md_dir(&db2, &tmp).expect("re-import");
    assert_eq!(again.created, 0);
    assert_eq!(again.updated, 0);
    assert_eq!(again.unchanged, 2);

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}
