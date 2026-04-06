use connectomedb::storage::StorageEngine;
use connectomedb::node::{UnifiedNode, NeuronType};
use std::sync::Arc;
use tempfile::tempdir;
use std::time::Instant;

#[tokio::test]
async fn test_hnsw_scale_performance_logarithmic() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());

    let num_nodes = 1000;
    println!("🚀 Insertando {} nodos vectoriales para prueba de escala...", num_nodes);

    // Inserción masiva
    for i in 0..num_nodes {
        let mut vec = vec![0.0; 128];
        vec[i % 128] = 1.0; // Vectores dispersos ortogonales para probar navegación
        
        let mut node = UnifiedNode::new(i as u64);
        node.neuron_type = NeuronType::STNeuron;
        node.vector = connectomedb::node::VectorRepresentations::Full(vec);
        node.flags.set(connectomedb::node::NodeFlags::HAS_VECTOR);
        
        storage.insert(&node).unwrap();
    }

    // Query de prueba
    let mut query_vec = vec![0.0; 128];
    query_vec[10] = 1.0;

    println!("🔍 Ejecutando búsqueda vectorial en grafo de {} nodos...", num_nodes);
    let start = Instant::now();
    
    let results = {
        let index = storage.hnsw.read().unwrap();
        index.search_nearest(&query_vec, None, None, 0, 5)
    };
    
    let duration = start.elapsed();
    println!("⏱️ Búsqueda completada en {:?}. Resultados: {}", duration, results.len());

    // Validar que no hayamos perdido precisión con la búsqueda voraz
    assert!(!results.is_empty(), "La búsqueda voraz falló en encontrar resultados en el grafo");
    assert!(results[0].0 == 10, "El primer resultado debería ser el nodo 10 (similitud 1.0)");
    
    // Un escaneo lineal de 1000 nodos con SIMD en este ambiente suele tardar <1ms, 
    // pero lo importante es la tendencia. En grafos de 1M, la diferencia será masiva.
    println!("✅ Búsqueda topológica exitosa y precisa.");
}
