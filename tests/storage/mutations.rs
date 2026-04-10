//! DML Pipeline & Mutations Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};
use tempfile::tempdir;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::parser::parse_statement;
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn dml_mutations_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (DML MUTATIONS)");

    harness.execute("Pipeline: INSERT -> GET Cycle", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            let q_insert = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", pais: "VZLA" }"#;
            let (_, stmt_insert) = parse_statement(q_insert).unwrap();

            match executor.execute_statement(stmt_insert).await.unwrap() {
                ExecutionResult::Write { affected_nodes, .. } => assert_eq!(affected_nodes, 1),
                _ => panic!("Expected write result"),
            }

            let node = storage.get(101).unwrap().unwrap();
            assert_eq!(node.get_field("pais").unwrap().as_str().unwrap(), "VZLA");
            TerminalReporter::success("Parse-to-Insert pipeline validated.");
        });
    });

    harness.execute("Pipeline: UPDATE & Atomicity", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            // Initial insert
            let q_insert = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", pais: "VZLA" }"#;
            let (_, stmt_insert) = parse_statement(q_insert).unwrap();
            executor.execute_statement(stmt_insert).await.unwrap();

            let q_update = r#"UPDATE NODE#101 SET role = "Admin", pais = "US""#;
            let (_, stmt_update) = parse_statement(q_update).unwrap();
            executor.execute_statement(stmt_update).await.unwrap();

            let node = storage.get(101).unwrap().unwrap();
            assert_eq!(node.get_field("role").unwrap().as_str().unwrap(), "Admin");
            assert_eq!(node.get_field("pais").unwrap().as_str().unwrap(), "US");
            TerminalReporter::success("Partial node updates committed successfully.");
        });
    });

    harness.execute("Pipeline: RELATE & Topology Integrity", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            let q_i1 = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros" }"#;
            let q_i2 = r#"INSERT NODE#5 TYPE Tarea { nombre: "VantaDB" }"#;
            executor.execute_statement(parse_statement(q_i1).unwrap().1).await.unwrap();
            executor.execute_statement(parse_statement(q_i2).unwrap().1).await.unwrap();

            let q_relate = r#"RELATE NODE#101 --"creo"--> NODE#5 WEIGHT 1.0"#;
            executor.execute_statement(parse_statement(q_relate).unwrap().1).await.unwrap();

            let node = storage.get(101).unwrap().unwrap();
            assert_eq!(node.edges.len(), 1);
            assert_eq!(node.edges[0].label, "creo");
            TerminalReporter::success("Directed relation established through DML.");
        });
    });

    harness.execute("Pipeline: Physical DELETE logic", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            executor.execute_statement(parse_statement(r#"INSERT NODE#101 TYPE X {}"#).unwrap().1).await.unwrap();
            executor.execute_statement(parse_statement(r#"DELETE NODE#101"#).unwrap().1).await.unwrap();

            assert!(storage.get(101).unwrap().is_none());
            TerminalReporter::success("Node excision complete.");
        });
    });
}
