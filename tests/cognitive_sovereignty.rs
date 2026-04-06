use connectomedb::storage::StorageEngine;
use connectomedb::executor::Executor;
use connectomedb::query::{Statement, InsertStatement};

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
        fields: std::collections::BTreeMap::new(),
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
        fields: std::collections::BTreeMap::new(),
        vector: Some(vector_base.clone()),
    });
    // Por defecto `UnifiedNode::new()` le asigna `trust_score = 0.5`. 0.5 < 0.9, así que debería rechazarlo igual.

    let result = executor.execute_statement(insert_challenger).await;
    
    assert!(result.is_ok(), "El Devil's Advocate debería permitir inserción en Superposición en vez de fallar hard");
    
    if let Ok(connectomedb::executor::ExecutionResult::Write { message, .. }) = result {
        assert!(message.contains("Superposition"), "Debe notificar que entró en UncertaintyZone");
    } else {
        panic!("Resultado incorrecto, se esperaba un Write result de Superposition");
    }

    // Verificar que efectivamente está en el uncertainty buffer usando Node#2's ID mapping si aplica
    let q_size = storage.uncertainty_buffer.quantum_zones.read().len();
    assert_eq!(q_size, 1, "Debería haber una zona cuántica activa");
}

#[tokio::test]
async fn test_nmi_hard_urgency_trigger() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    let executor = Executor::new(&storage);

    let vector_base = vec![0.5_f32, 0.5, 0.5, 0.5];

    // 1. Incumbent
    let insert1 = Statement::Insert(InsertStatement {
        node_id: 10,
        node_type: "Fact".to_string(),
        fields: std::collections::BTreeMap::new(),
        vector: Some(vector_base.clone()),
    });
    executor.execute_statement(insert1).await.unwrap();

    let mut node10 = storage.get(10).unwrap().unwrap();
    node10.trust_score = 0.9; 
    node10.semantic_valence = 0.4;
    storage.insert(&node10).unwrap();

    // 2. Challenger
    let insert2 = Statement::Insert(InsertStatement {
        node_id: 20,
        node_type: "DisputedFact".to_string(),
        fields: std::collections::BTreeMap::new(),
        vector: Some(vector_base.clone()),
    });
    executor.execute_statement(insert2).await.unwrap();

    // Check it's in zone
    let mut q_zones = storage.uncertainty_buffer.quantum_zones.write();
    assert_eq!(q_zones.len(), 1, "Should be 1 superposition");
    
    // Elevate valence of challenger to make it the NMI winner
    if let Some(neuron) = q_zones.get_mut(&10) {
        neuron.candidates[1].semantic_valence = 0.99; // Challenger val
    }
    drop(q_zones);

    // 3. Simulate Memory Pressure
    use connectomedb::governor::ALLOCATED_BYTES;
    use std::sync::atomic::Ordering;
    
    let pressure_bytes = (2.0 * 1024.0 * 1024.0 * 1024.0 * 0.95) as usize; // 95% of 2GB
    ALLOCATED_BYTES.store(pressure_bytes, Ordering::SeqCst);

    // 4. Trigger Query -> Triggers NMI
    let dummy_insert = Statement::Insert(InsertStatement {
        node_id: 999,
        node_type: "Dummy".to_string(),
        fields: std::collections::BTreeMap::new(),
        vector: None,
    });
    let _ = executor.execute_statement(dummy_insert).await;

    // Reset allocated bytes to avoid interfering with other tests
    ALLOCATED_BYTES.store(0, Ordering::SeqCst);

    // 5. Assertions
    let q_size = storage.uncertainty_buffer.quantum_zones.read().len();
    assert_eq!(q_size, 0, "NMI should have purged the uncertainty buffer");

    // Check if the challenger survived (ID 20)
    let winner_node_opt = storage.get(20).unwrap();
    assert!(winner_node_opt.is_some(), "Winner node 20 should have been re-inserted into STN");
    
    if let Some(winner_node) = winner_node_opt {
        assert_eq!(winner_node.semantic_valence, 0.99, "Survivor must correctly retain its highest valence");
    }
}
