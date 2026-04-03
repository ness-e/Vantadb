use connectomedb::storage::StorageEngine;
use connectomedb::node::{UnifiedNode, NeuronType};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_dynamic_memory_promotion() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());

    let node_id = 12345;
    
    // 1. Insertar un LTNeuron (solo Disco)
    {
        let mut node = UnifiedNode::new(node_id);
        node.neuron_type = NeuronType::LTNeuron;
        node.hits = 48; // Casi llegando al umbral de 50
        storage.insert(&node).unwrap();
    }

    // Verificar que NO está en RAM inicialmente
    {
        let cortex = storage.cortex_ram.read().unwrap();
        assert!(!cortex.contains_key(&node_id), "El LTNeuron no debería estar en RAM al inicio");
    }

    // 2. Realizar consultas (get) para subir los hits
    // Primer Get: hits pasa de 48 a 49 (Todavía LTN)
    let _ = storage.get(node_id).unwrap();
    {
        let cortex = storage.cortex_ram.read().unwrap();
        assert!(!cortex.contains_key(&node_id), "No debería promoverse con 49 hits");
    }

    // Segundo Get: hits pasa de 49 a 50 -> Gatilla Promoción
    let _ = storage.get(node_id).unwrap();

    // 3. Verificar que ahora el nodo reside en el Cortex RAM (STN)
    {
        let cortex = storage.cortex_ram.read().unwrap();
        assert!(cortex.contains_key(&node_id), "El nodo debería haber sido promovido a RAM al alcanzar 50 hits");
        
        let promoted_node = cortex.get(&node_id).unwrap();
        assert_eq!(promoted_node.neuron_type, NeuronType::STNeuron, "El tipo de neurona debería haber cambiado a STNeuron");
    }
}
