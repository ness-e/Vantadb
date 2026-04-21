//! Storage Chaos & Data Integrity Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::error::VantaError;
use vantadb::executor::Executor;
use vantadb::query::{InsertStatement, RelateStatement, Statement};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn chaos_integrity_certification() {
    TerminalReporter::suite_banner("TOPOLOGICAL INTEGRITY & CHAOS AXIOMS", 2);

    // ─── AXIOM 1: Ghost Node Prevention ──────────────────────────

    let mut s1 = VantaSession::begin("Ghost Node Prevention");
    s1.step("Initializing storage and executor");

    let dir1 = tempdir().unwrap();
    let db_path1 = dir1.path().to_str().unwrap();
    let storage1 = Arc::new(StorageEngine::open(db_path1).unwrap());
    let executor1 = Executor::new(&storage1);

    s1.step("Seeding valid base nodes (1, 2)");
    executor1
        .execute_statement(Statement::Insert(InsertStatement {
            node_id: 1,
            node_type: "Test".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }))
        .await
        .unwrap();

    executor1
        .execute_statement(Statement::Insert(InsertStatement {
            node_id: 2,
            node_type: "Test".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }))
        .await
        .unwrap();

    s1.step("Attempting illegal relation to non-existent ID 999");
    let relate_ghost = Statement::Relate(RelateStatement {
        source_id: 1,
        target_id: 999,
        label: "likes".to_string(),
        weight: None,
    });
    let result_ghost = executor1.execute_statement(relate_ghost).await;

    assert!(
        result_ghost.is_err(),
        "Axiom Failure: Relation to ghost node was not blocked"
    );
    if let Err(VantaError::Execution(msg)) = result_ghost {
        assert!(
            msg.contains("Topological Axiom violated"),
            "Wrong error message for ghost node"
        );
    } else {
        panic!("Expected Execution error for Topological Axiom");
    }

    s1.success("Ghost node protection verified.");
    s1.finish(true);

    // ─── AXIOM 2: Tombstone Resilience ───────────────────────────

    let mut s2 = VantaSession::begin("Tombstone Resilience");
    s2.step("Initializing storage context");

    let dir2 = tempdir().unwrap();
    let storage2 = Arc::new(StorageEngine::open(dir2.path().to_str().unwrap()).unwrap());
    let executor2 = Executor::new(&storage2);

    s2.step("Seeding and then deleting target node (ID 2)");
    executor2
        .execute_statement(Statement::Insert(InsertStatement {
            node_id: 1,
            node_type: "Test".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }))
        .await
        .unwrap();

    executor2
        .execute_statement(Statement::Insert(InsertStatement {
            node_id: 2,
            node_type: "Test".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }))
        .await
        .unwrap();

    executor2
        .execute_statement(Statement::Delete(vantadb::query::DeleteStatement {
            node_id: 2,
        }))
        .await
        .unwrap();

    s2.step("Attempting relation to deleted (Tombstoned) node");
    let relate_tombstone = Statement::Relate(RelateStatement {
        source_id: 1,
        target_id: 2,
        label: "likes".to_string(),
        weight: None,
    });
    let result_tombstone = executor2.execute_statement(relate_tombstone).await;

    assert!(
        result_tombstone.is_err(),
        "Axiom Failure: Relation to tombstone was not blocked"
    );
    s2.success("Tombstone integrity verified.");
    s2.finish(true);

    // Final Report for this suite
    TerminalReporter::print_certification_summary();
}
