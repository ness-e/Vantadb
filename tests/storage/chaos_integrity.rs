#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::error::VantaError;
use vantadb::executor::Executor;
use vantadb::query::{InsertStatement, RelateStatement, Statement};
use vantadb::storage::StorageEngine;

#[test]
fn chaos_integrity_certification() {
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
        .unwrap();

    executor1
        .execute_statement(Statement::Insert(InsertStatement {
            node_id: 2,
            node_type: "Test".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }))
        .unwrap();

    s1.step("Attempting illegal relation to non-existent ID 999");
    let relate_ghost = Statement::Relate(RelateStatement {
        source_id: 1,
        target_id: 999,
        label: "likes".to_string(),
        weight: None,
    });
    let result_ghost = executor1.execute_statement(relate_ghost);

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
        .unwrap();

    executor2
        .execute_statement(Statement::Insert(InsertStatement {
            node_id: 2,
            node_type: "Test".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }))
        .unwrap();

    executor2
        .execute_statement(Statement::Delete(vantadb::query::DeleteStatement {
            node_id: 2,
        }))
        .unwrap();

    s2.step("Attempting relation to deleted (Tombstoned) node");
    let relate_tombstone = Statement::Relate(RelateStatement {
        source_id: 1,
        target_id: 2,
        label: "likes".to_string(),
        weight: None,
    });
    let result_tombstone = executor2.execute_statement(relate_tombstone);

    assert!(
        result_tombstone.is_err(),
        "Axiom Failure: Relation to tombstone was not blocked"
    );
    s2.success("Tombstone integrity verified.");
    s2.finish(true);

    // Final Report for this suite
    TerminalReporter::print_certification_summary();
}

#[test]
fn chaos_integrity_failpoints_certification() {
    #[cfg(feature = "failpoints")]
    {
        TerminalReporter::suite_banner("FAILPOINT INJECTION & RESILIENCE AXIOMS", 1);

        let _scenario = vantadb::FailScenario::setup();

        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        // 1. Inicialización y escritura inicial exitosa
        let storage = Arc::new(StorageEngine::open(db_path).unwrap());
        let executor = Executor::new(&storage);

        // 2. Activar inyección de fallo en WAL
        vantadb::cfg_failpoint("wal_append_fail", "return").unwrap();

        // 3. Comprobar que la base de datos rechaza la operación limpiamente
        let result = executor.execute_statement(Statement::Insert(InsertStatement {
            node_id: 42,
            node_type: "Chaos".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }));

        assert!(
            result.is_err(),
            "Se esperaba error debido a inyección de fallo en el WAL"
        );

        // Desactivar inyección
        vantadb::remove_failpoint("wal_append_fail");

        // 4. Comprobar auto-recuperación y escrituras posteriores
        let recovery_result = executor.execute_statement(Statement::Insert(InsertStatement {
            node_id: 42,
            node_type: "Chaos".to_string(),
            fields: std::collections::BTreeMap::new(),
            vector: None,
        }));
        assert!(
            recovery_result.is_ok(),
            "El motor debería recuperarse tras desactivar el failpoint"
        );

        TerminalReporter::print_certification_summary();
    }
}
