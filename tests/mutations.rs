use connectomedb::storage::StorageEngine;
use connectomedb::parser::parse_statement;
use connectomedb::executor::{Executor, ExecutionResult};
use tempfile::tempdir;

#[tokio::test]
async fn test_dml_pipeline_e2e() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).unwrap();
    let executor = Executor::new(&storage);

    // 1. INSERT
    let q_insert = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", pais: "VZLA" }"#;
    let (_, stmt_insert) = parse_statement(q_insert).unwrap();
    
    match executor.execute_statement(stmt_insert).await.unwrap() {
        ExecutionResult::Write { affected_nodes, .. } => assert_eq!(affected_nodes, 1),
        _ => panic!("Expected write result"),
    }

    // Verify it was stored
    let node = storage.get(101).unwrap().unwrap();
    assert_eq!(node.get_field("pais").unwrap().as_str().unwrap(), "VZLA");

    // 2. UPDATE
    let q_update = r#"UPDATE NODE#101 SET role = "Admin", pais = "US""#;
    let (_, stmt_update) = parse_statement(q_update).unwrap();
    executor.execute_statement(stmt_update).await.unwrap();

    let node2 = storage.get(101).unwrap().unwrap();
    assert_eq!(node2.get_field("role").unwrap().as_str().unwrap(), "Admin");
    assert_eq!(node2.get_field("pais").unwrap().as_str().unwrap(), "US"); // overwritten

    // 3. RELATE
    // Insert another node first
    let q_insert2 = r#"INSERT NODE#5 TYPE Tarea { nombre: "ConnectomeDB Tarea" }"#;
    let (_, stmt_insert2) = parse_statement(q_insert2).unwrap();
    executor.execute_statement(stmt_insert2).await.unwrap();

    let q_relate = r#"RELATE NODE#101 --"creo"--> NODE#5 WEIGHT 1.0"#;
    let (_, stmt_relate) = parse_statement(q_relate).unwrap();
    executor.execute_statement(stmt_relate).await.unwrap();

    let node3 = storage.get(101).unwrap().unwrap();
    assert_eq!(node3.edges.len(), 1);
    assert_eq!(node3.edges[0].label, "creo");

    // 4. DELETE
    let q_delete = r#"DELETE NODE#101"#;
    let (_, stmt_delete) = parse_statement(q_delete).unwrap();
    executor.execute_statement(stmt_delete).await.unwrap();

    assert!(storage.get(101).unwrap().is_none());
}
