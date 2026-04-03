use connectomedb::storage::StorageEngine;
use connectomedb::executor::{Executor, ExecutionResult};
use connectomedb::query::{Statement, InsertStatement};
use connectomedb::error::ConnectomeError;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_devil_advocate_trust_conflict() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    let executor = Executor::new(&storage);

    // Creamos un vector base (Ej: "La capital de Francia es París")
    let vector_base = vec![0.1_f32, 0.2, 0.3, 0.4];

    // 1. Incumbent: Insertamos un nodo con Alta Confianza (0.9)
    let insert_incumbent = Statement::Insert(InsertStatement {
        node_id: 1,
        node_type: "Fact".to_string(),
        fields: std::collections::HashMap::new(),
        vector: Some(vector_base.clone()),
    });
    executor.execute_statement(insert_incumbent).await.unwrap();

    // Promovemos la confianza heurísticamente (simulando hits y tiempo)
    let mut node1 = storage.get(1).unwrap().unwrap();
    node1.trust_score = 0.9;
    storage.insert(&node1).unwrap();

    // 2. Challenger: Intentamos insertar un nodo que habla del mismo tema (Mismo vector o muy similar), 
    // pero con una confianza inferior (0.2). El Devil's Advocate debe detectar el conflicto semántico
    // (Similitud del vector ~ 1.0 > 0.95) y rechazarlo porque Challenger Trust (0.2) < Incumbent Trust (0.9)
    let insert_challenger = Statement::Insert(InsertStatement {
        node_id: 2,
        node_type: "FactDisputed".to_string(),
        fields: std::collections::HashMap::new(),
        vector: Some(vector_base.clone()),
    });
    // Por defecto `UnifiedNode::new()` le asigna `trust_score = 0.5`. 0.5 < 0.9, así que debería rechazarlo igual.

    let result = executor.execute_statement(insert_challenger).await;
    
    assert!(result.is_err(), "Sovereignty Failed: Permitió insertar una contradicción de baja confianza");
    
    if let Err(ConnectomeError::Execution(msg)) = result {
        assert!(msg.contains("Sovereignty Rejected"), "Debe devolver error de Soberanía. Recibido: {}", msg);
    } else {
        panic!("Tipo de error esperado incorrecto");
    }
}
