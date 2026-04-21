//! Schema Evolution & Metadata Integrity Suite
//!
//! This suite validates that VantaDB maintains data integrity as node structures
//! evolve over time, ensuring forward and backward compatibility.

#[path = "common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use tempfile::tempdir;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

// ─── HELPER: Open Engine ──────────────────────────────────────

fn open_engine(path: &str) -> StorageEngine {
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config)).unwrap()
}

// ─── TEST: Metadata Evolution ─────────────────────────────────

#[test]
fn test_metadata_schema_evolution() {
    TerminalReporter::suite_banner("SCHEMA EVOLUTION & METADATA CERTIFICATION", 2);
    let mut session = VantaSession::begin("Dynamic Schema Evolution");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // PHASE 1: Legacy Data Creation
    session.step("Phase 1: Creating 'Legacy' nodes (v1 Schema)");
    {
        let engine = open_engine(db_path);
        let mut node = UnifiedNode::new(1);
        node.set_field("version", FieldValue::String("1.0".into()));
        node.set_field("tag", FieldValue::String("stable".into()));
        engine.insert(&node).unwrap();
        session.step("Phase 1: Node 1 saved with v1 properties.");
    }

    // PHASE 2: Evolution (Adding New Fields)
    session.step("Phase 2: Upgrading schema and adding v2 fields");
    {
        let engine = open_engine(db_path);
        let mut node = engine.get(1).unwrap().expect("Node 1 lost!");

        // Verify v1 data still exists
        assert_eq!(node.get_field("version").unwrap().as_str().unwrap(), "1.0");

        // Add v2 data
        node.set_field("version", FieldValue::String("2.0".into()));
        node.set_field("new_feature_flag", FieldValue::String("enabled".into()));
        node.set_field("author", FieldValue::String("VantaArchitect".into()));

        engine.insert(&node).unwrap();
        session.step("Phase 2: Node 1 evolved to v2 successfully.");
    }

    // PHASE 3: Partial Schema Deletion
    session.step("Phase 3: Testing property removal and consistency");
    {
        let engine = open_engine(db_path);
        let mut node = engine.get(1).unwrap().unwrap();

        // Remove a legacy property
        node.relational.remove("tag");
        engine.insert(&node).unwrap();

        let final_node = engine.get(1).unwrap().unwrap();
        assert!(final_node.get_field("tag").is_none());
        assert_eq!(
            final_node.get_field("version").unwrap().as_str().unwrap(),
            "2.0"
        );
        assert_eq!(
            final_node
                .get_field("new_feature_flag")
                .unwrap()
                .as_str()
                .unwrap(),
            "enabled"
        );
    }

    session
        .success("Schema evolution verified: VantaDB handles dynamic property changes gracefully.");
    session.finish(true);
}

// ─── TEST: Bulk Migration Simulation ──────────────────────────

#[test]
fn test_bulk_property_migration() {
    let mut session = VantaSession::begin("Bulk Metadata Migration");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    session.step("Seeding 1000 nodes for migration test");
    let engine = open_engine(db_path);
    for i in 0..1000 {
        let mut node = UnifiedNode::new(i);
        node.set_field("status", FieldValue::String("pending".into()));
        engine.insert(&node).unwrap();
    }

    session.step("Performing bulk 'Status' migration to 'active'");
    for i in 0..1000 {
        let mut node = engine.get(i).unwrap().unwrap();
        node.set_field("status", FieldValue::String("active".into()));
        engine.insert(&node).unwrap();
    }

    session.step("Verifying migration result");
    for i in 0..1000 {
        let node = engine.get(i).unwrap().unwrap();
        assert_eq!(
            node.get_field("status").unwrap().as_str().unwrap(),
            "active"
        );
    }

    session.success("Bulk migration complete without data loss.");
    session.finish(true);
}

// ─── SUMMARY ──────────────────────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
