//! Integration test for MEM-62 round-trip: export MD → import MD preserves
//! the records byte-for-byte (idempotency via content-hash). Runs against an
//! in-memory VantaEmbedded so it doesn't need the `fjall` feature.

use std::collections::BTreeMap;

use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaValue};
use vantadb::storage::BackendKind;

#[test]
fn round_trip_put_export_import_get() {
    let db = VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    })
    .expect("open in-memory db");

    // Put 3 records with mixed metadata shapes.
    let mut meta1 = VantaMemoryMetadata::new();
    meta1.insert("author".into(), VantaValue::String("alice".into()));
    meta1.insert("version".into(), VantaValue::Int(1));
    db.put(VantaMemoryInput {
        namespace: "agent/team".into(),
        key: "intro".into(),
        payload: "Welcome to the team.".into(),
        metadata: meta1,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("put intro");

    let mut meta2 = VantaMemoryMetadata::new();
    meta2.insert(
        "tags".into(),
        VantaValue::ListString(vec!["a".into(), "b".into()]),
    );
    db.put(VantaMemoryInput {
        namespace: "agent/team".into(),
        key: "handbook".into(),
        payload: "Always commit before merging.".into(),
        metadata: meta2,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("put handbook");

    let mut meta3 = VantaMemoryMetadata::new();
    meta3.insert("priority".into(), VantaValue::Float(0.5));
    db.put(VantaMemoryInput {
        namespace: "agent/team".into(),
        key: "oncall".into(),
        payload: "Weekly rotations: alice → bob.".into(),
        metadata: meta3,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("put oncall");

    // Walk the records and render them as MD strings using the same shape
    // that cli_handlers::export_md::render_record_md produces.
    let mut md_files: BTreeMap<String, String> = BTreeMap::new();
    for ns in ["agent/team"] {
        let page = db
            .list(
                ns,
                vantadb::sdk::VantaMemoryListOptions {
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
    let db2 = VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
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
        Some(VantaValue::String(s)) if s == "alice"
    ));

    let got2 = db2
        .get("agent/team", "handbook")
        .expect("get handbook")
        .expect("handbook exists");
    assert!(matches!(
        got2.metadata.get("tags"),
        Some(VantaValue::ListString(xs)) if xs == &vec!["a".to_string(), "b".to_string()]
    ));

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}
