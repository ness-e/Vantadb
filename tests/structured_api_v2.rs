use connectomedb::storage::StorageEngine;
use connectomedb::executor::{Executor, ExecutionResult};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_structured_api_v2_ids() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    let executor = Executor::new(&storage);

    // Test de Relate ID
    executor.execute_hybrid("(INSERT :neuron {:label \"S1\"})").await.unwrap();
    executor.execute_hybrid("(INSERT :neuron {:label \"S2\"})").await.unwrap();
    
    // Necesitamos los IDs. Buscamos en el motor L1 Cache (recién insertados)
    let s1_id;
    let s2_id;
    {
        let cortex = storage.cortex_ram.read().unwrap();
        s1_id = *cortex.iter().find(|(_, n)| n.get_field("label").unwrap().as_str() == Some("S1")).unwrap().0;
        s2_id = *cortex.iter().find(|(_, n)| n.get_field("label").unwrap().as_str() == Some("S2")).unwrap().0;
    }

    let relate_query = format!("RELATE #{} TO #{} WITH weight 0.8 LABEL \"test_rel\"", s1_id, s2_id);
    let res = executor.execute_hybrid(&relate_query).await.unwrap();
    
    if let ExecutionResult::Write { node_id, .. } = res {
        assert_eq!(node_id, Some(s1_id), "RELATE no devolvió el ID del origen");
    }

    // Test de Insert Message ID
    // Simular un Thread
    executor.execute_hybrid("(INSERT :neuron {:type \"Thread\" :id 999})").await.unwrap();
    let msg_query = "INSERT MESSAGE \"Hola Mundo\" AS ROLE \"user\" TO THREAD #999";
    let msg_res = executor.execute_hybrid(msg_query).await.unwrap();

    if let ExecutionResult::Write { node_id, .. } = msg_res {
        assert!(node_id.is_some(), "INSERT MESSAGE no devolvió el ID del mensaje creado");
    }

    println!("✅ El API Estructurado (v2) funciona. Todos los IDs capturados.");
}
