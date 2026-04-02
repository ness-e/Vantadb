use iadbms::index::{cosine_similarity, CPIndex};

#[test]
fn test_cosine_similarity() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let sim = cosine_similarity(&a, &b);
    assert!((sim - 1.0).abs() < f32::EPSILON, "Identical vectors should have similarity 1.0");

    let c = vec![0.0, 1.0, 0.0];
    let sim_orthogonal = cosine_similarity(&a, &c);
    assert!(sim_orthogonal.abs() < f32::EPSILON, "Orthogonal vectors should have similarity 0.0");
    
    let d = vec![-1.0, 0.0, 0.0];
    let sim_opposite = cosine_similarity(&a, &d);
    assert!((sim_opposite - (-1.0)).abs() < f32::EPSILON, "Opposite vectors should have similarity -1.0");
}

#[test]
fn test_hnsw_greedy_search() {
    let mut index = CPIndex::new();
    
    // Inserciones (El bitset 0 es un catch-all / no-fiiter en este test)
    index.add(1, 0, Some(vec![1.0, 0.0, 0.0]));
    index.add(2, 0, Some(vec![0.8, 0.2, 0.0])); // Cerca de 1
    index.add(3, 0, Some(vec![0.0, 1.0, 0.0])); // Ortogonal a 1
    index.add(4, 0, Some(vec![0.0, 0.8, 0.2])); // Cerca de 3

    // Hacemos una busqueda emulando un vector mas cerca de 3 y 4
    let query = vec![0.0, 0.9, 0.1];
    
    let results = index.search_nearest(&query, 0, 2);
    
    // Debería recuperar primero a 4 y luego a 3 o viceversa, descartando 1 y 2 asumiendo su alta lejanía
    assert_eq!(results.len(), 2);
    let top_match = results[0].0;
    
    assert!(top_match == 3 || top_match == 4, "Debería encontrar los vecinos ortogonales al vector 1 primario");
}
