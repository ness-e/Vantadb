use connectomedb::storage::StorageEngine;
use connectomedb::executor::{Executor, ExecutionResult};
use connectomedb::query::{Statement, RelateStatement, InsertStatement};
use connectomedb::error::ConnectomeError;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_ghost_node_and_tombstone_axioms() {
    // 1. Setup
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    let executor = Executor::new(&storage);

    // 2. Setup Nodes
    // Insertamos Node 1
    let insert_stmt = Statement::Insert(InsertStatement {
        node_id: 1,
        node_type: "Test".to_string(),
        fields: std::collections::HashMap::new(),
        vector: None,
    });
    executor.execute_statement(insert_stmt).await.unwrap();

    // Insertamos Node 2
    let insert_stmt2 = Statement::Insert(InsertStatement {
        node_id: 2,
        node_type: "Test".to_string(),
        fields: std::collections::HashMap::new(),
        vector: None,
    });
    executor.execute_statement(insert_stmt2).await.unwrap();

    // 3. Prueba 1: Falso Positivo (Ghost Node)
    // El Node 999 no existe. Intentamos RELATE 1 -> 999.
    // Aunque el Bloom Filter no es expuesto explícitamente y falla rápido, 
    // forzamos la violación asegurándonos que el Get siempre aborte la relación.
    let relate_ghost = Statement::Relate(RelateStatement {
        source_id: 1,
        target_id: 999, // Ghost
        label: "likes".to_string(),
        weight: None,
    });

    let result_ghost = executor.execute_statement(relate_ghost).await;
    assert!(result_ghost.is_err(), "Axioma 1 falló al atrapar un Ghost Node!");
    if let Err(ConnectomeError::Execution(msg)) = result_ghost {
        assert!(msg.contains("Axioma Topológico violado"), "Mensaje incorrecto: {}", msg);
    } else {
        panic!("Tipo de error esperado incorrecto");
    }

    // 4. Prueba 2: Lápida (Tombstone test)
    // Borramos a Node 2 para forzar paso al Shadow Archive
    let delete_stmt = Statement::Delete(connectomedb::query::DeleteStatement { node_id: 2 });
    executor.execute_statement(delete_stmt).await.unwrap();

    // Intentamos relacionar Node 1 -> Node 2 (difunto)
    let relate_tombstone = Statement::Relate(RelateStatement {
        source_id: 1,
        target_id: 2, // Tombstoned
        label: "likes".to_string(),
        weight: None,
    });

    let result_tombstone = executor.execute_statement(relate_tombstone).await;
    assert!(result_tombstone.is_err(), "Axioma 1 falló al atrapar una Lápida!");
    if let Err(ConnectomeError::Execution(msg)) = result_tombstone {
        assert!(msg.contains("reside en el Shadow Archive"), "Mensaje de difunto incorrecto: {}", msg);
    } else {
        panic!("Tipo de error esperado incorrecto para difunto");
    }
}
