//! Storage Chaos & Data Integrity Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::error::VantaError;
use vantadb::executor::Executor;
use vantadb::query::{InsertStatement, RelateStatement, Statement};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn chaos_integrity_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (CHAOS INTEGRITY)");

    harness.execute("Topological Axioms: Ghost Node Prevention", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let db_path = dir.path().to_str().unwrap();
            let storage = Arc::new(StorageEngine::open(db_path).unwrap());
            let executor = Executor::new(&storage);

            TerminalReporter::sub_step("Setting up valid base nodes (1, 2)...");
            executor.execute_statement(Statement::Insert(InsertStatement {
                node_id: 1, node_type: "Test".to_string(), fields: std::collections::BTreeMap::new(), vector: None,
            })).await.unwrap();
            executor.execute_statement(Statement::Insert(InsertStatement {
                node_id: 2, node_type: "Test".to_string(), fields: std::collections::BTreeMap::new(), vector: None,
            })).await.unwrap();

            TerminalReporter::sub_step("Attempting RELATE to non-existent Ghost Node 999...");
            let relate_ghost = Statement::Relate(RelateStatement {
                source_id: 1, target_id: 999, label: "likes".to_string(), weight: None,
            });
            let result_ghost = executor.execute_statement(relate_ghost).await;
            
            assert!(result_ghost.is_err());
            if let Err(VantaError::Execution(msg)) = result_ghost {
                assert!(msg.contains("Topological Axiom violated"));
            } else { panic!("Expected Topological Axiom error"); }
            
            TerminalReporter::success("Ghost node relation correctly blocked.");
        });
    });

    harness.execute("Topological Axioms: Tombstone Resilience", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
            let executor = Executor::new(&storage);

            executor.execute_statement(Statement::Insert(InsertStatement {
                node_id: 1, node_type: "Test".to_string(), fields: std::collections::BTreeMap::new(), vector: None,
            })).await.unwrap();
            executor.execute_statement(Statement::Insert(InsertStatement {
                node_id: 2, node_type: "Test".to_string(), fields: std::collections::BTreeMap::new(), vector: None,
            })).await.unwrap();

            TerminalReporter::sub_step("Deleting Node 2 (creating tombstone)...");
            executor.execute_statement(Statement::Delete(vantadb::query::DeleteStatement { node_id: 2 })).await.unwrap();

            TerminalReporter::sub_step("Attempting RELATE to deleted Node 2...");
            let relate_tombstone = Statement::Relate(RelateStatement {
                source_id: 1, target_id: 2, label: "likes".to_string(), weight: None,
            });
            let result_tombstone = executor.execute_statement(relate_tombstone).await;
            
            assert!(result_tombstone.is_err());
            TerminalReporter::success("Relation to tombstone correctly blocked.");
        });
    });
}
