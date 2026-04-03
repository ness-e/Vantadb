use connectomedb::storage::StorageEngine;
use connectomedb::governance::sleep_worker::SleepWorker;
use connectomedb::node::{UnifiedNode, NeuronType};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_thrashing_prevention_grace_period() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());

    // Inyectar un nodo nuevo (0 hits)
    let node_id = 1;
    let mut node = UnifiedNode::new(node_id);
    node.neuron_type = NeuronType::STNeuron;
    node.hits = 0; 
    storage.insert(&node).unwrap();

    // Iniciar SleepWorker
    let worker_storage = storage.clone();
    tokio::spawn(async move {
        SleepWorker::start(worker_storage).await;
    });

    // Esperar 12 segundos (Suficiente para activar un ciclo REM de 10s base)
    println!("💤 Esperando 12s para ver si el worker respeta el periodo de gracia...");
    sleep(Duration::from_secs(12)).await;

    // Verificar que el nodo SIGUE EN RAM porque no han pasado 60s
    {
        let cortex = storage.cortex_ram.read().unwrap();
        assert!(cortex.contains_key(&node_id), "El nodo fue expulsado prematuramente antes del periodo de gracia (Thrashing)");
    }

    println!("✅ El periodo de gracia funciona. El nodo permaneció en RAM.");
}
