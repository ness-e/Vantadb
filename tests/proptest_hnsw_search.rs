//! 🔬 Property-based testing (proptest) para HNSW search correctness.
//!
//! Ejecuta: `cargo test proptest_hnsw_search`
//!
//! Invariantes verificados:
//! 1. **Identidad**: vector insertado se encuentra como top-1 con score ≈ 1.0
//! 2. **Score descendente**: resultados se ordenan por score descendente
//! 3. **Monotonicidad top-k**: resultados de k pequeño son prefijo de k grande
//! 4. **Índice vacío**: search_vector devuelve vacío cuando no hay vectores
//! 5. **top_k = 0**: retorna vacío incluso con datos insertados
//! 6. **Exclusión post-eliminación**: nodo eliminado no aparece en resultados
//! 7. **Múltiples vectores**: con N vectores insertados, top_k ≤ N respeta el límite

use proptest::prelude::*;
use tempfile::TempDir;
use vantadb::config::VantaConfig;
use vantadb::BackendKind;
use vantadb::{VantaEmbedded, VantaMemoryInput};

const VEC_DIM: usize = 4;

fn vec_strategy() -> impl Strategy<Value = Vec<f32>> {
    proptest::collection::vec(-1.0f32..1.0, VEC_DIM)
}

fn multi_vec_strategy() -> impl Strategy<Value = Vec<Vec<f32>>> {
    proptest::collection::vec(vec_strategy(), 1..=10)
}

fn setup_db() -> (TempDir, VantaEmbedded) {
    let dir = TempDir::new().unwrap();
    let config = VantaConfig {
        storage_path: dir.path().to_string_lossy().to_string(),
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).unwrap();
    (dir, db)
}

fn l2_norm_sq(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Prop 1 — Identidad: un vector debe encontrar su copia en el índice
    /// como top-1 con score ≈ 1.0 (cosine similarity de un vector consigo mismo).
    #[test]
    fn prop_hnsw_identity_query(vec in vec_strategy()) {
        // Skip zero-norm vectors: a zero vector falls back to Euclidean distance
        // where self-distance is 0, not 1.0.
        prop_assume!(l2_norm_sq(&vec) > f32::EPSILON);

        let (_dir, db) = setup_db();
        let input = VantaMemoryInput {
            vector: Some(vec.clone()),
            ..VantaMemoryInput::new("hnsw", "identity", "payload")
        };
        db.put(input).unwrap();

        let hits = db.search_vector(&vec, 5).unwrap();
        prop_assert!(!hits.is_empty(), "identity vector must appear in results");
        // Cosine: identical vectors → similarity 1.0 (within float tolerance)
        prop_assert!(
            hits[0].distance > 0.99,
            "self-similarity should be ~1.0, got {}",
            hits[0].distance
        );
    }

    /// Prop 2 — Score descendente: los resultados de search_vector deben estar
    /// ordenados por score de mayor a menor (más similar primero).
    #[test]
    fn prop_hnsw_scores_descending(
        vectors in multi_vec_strategy(),
        query_vec in vec_strategy(),
    ) {
        let (_dir, db) = setup_db();
        for (i, v) in vectors.iter().enumerate() {
            let input = VantaMemoryInput {
                vector: Some(v.clone()),
                ..VantaMemoryInput::new("hnsw", format!("n{}", i), "payload")
            };
            db.put(input).unwrap();
        }

        let hits = db.search_vector(&query_vec, vectors.len()).unwrap();
        for window in hits.windows(2) {
            prop_assert!(
                window[0].distance >= window[1].distance - f32::EPSILON,
                "scores must be descending: {} < {}",
                window[0].distance,
                window[1].distance
            );
        }
    }

    /// Prop 3 — Monotonicidad top-k: aumentar top_k nunca descarta resultados
    /// que ya aparecían. Los resultados con k pequeño deben ser un prefijo
    /// de los resultados con k grande (mismos primeros elementos, mismo orden).
    ///
    /// Nota: esta propiedad se cumple porque `small` y `large` se consultan
    /// sobre la misma instancia de `VantaEmbedded` (mismo `ef_search`).
    /// Si las consultas usaran distintos `ef_search`, la naturaleza aproximada
    /// de HNSW podría no garantizar el prefijo. Para vectores pocos (n ≤ 10)
    /// y `ef_search = 100`, la monotonicidad se sostiene en la práctica.
    #[test]
    fn prop_hnsw_top_k_monotonicity(
        vectors in multi_vec_strategy(),
        query_vec in vec_strategy(),
    ) {
        let (_dir, db) = setup_db();
        for (i, v) in vectors.iter().enumerate() {
            let input = VantaMemoryInput {
                vector: Some(v.clone()),
                ..VantaMemoryInput::new("hnsw", format!("n{}", i), "payload")
            };
            db.put(input).unwrap();
        }

        let n = vectors.len();
        let k_small = if n > 1 { n / 2 } else { n };
        let k_large = n;

        let small = db.search_vector(&query_vec, k_small).unwrap();
        let large = db.search_vector(&query_vec, k_large).unwrap();

        // Each element in `small` must appear at the same position in `large`
        for (i, hit) in small.iter().enumerate() {
            if i < large.len() {
                prop_assert_eq!(
                    hit.node_id,
                    large[i].node_id,
                    "top_k={} result index {} must match top_k={}",
                    k_small,
                    i,
                    k_large
                );
            }
        }
    }

    /// Prop 4 — Índice vacío: search_vector sobre un índice sin datos
    /// debe devolver una lista vacía (sin errores).
    #[test]
    fn prop_hnsw_empty_index(query_vec in vec_strategy()) {
        prop_assume!(l2_norm_sq(&query_vec) > f32::EPSILON);

        let (_dir, db) = setup_db();
        let hits = db.search_vector(&query_vec, 10).unwrap();
        prop_assert!(hits.is_empty(), "empty index must return empty results");
    }

    /// Prop 5 — top_k = 0: incluso con datos, search_vector con top_k = 0
    /// debe devolver vacío.
    #[test]
    fn prop_hnsw_zero_top_k(vec in vec_strategy()) {
        let (_dir, db) = setup_db();
        let input = VantaMemoryInput {
            vector: Some(vec.clone()),
            ..VantaMemoryInput::new("hnsw", "zero_topk", "payload")
        };
        db.put(input).unwrap();

        let hits = db.search_vector(&vec, 0).unwrap();
        prop_assert!(hits.is_empty(), "top_k=0 must return empty results");
    }

    /// Prop 6 — Exclusión post-eliminación: insertar un solo vector,
    /// eliminarlo, y verificar que search_vector devuelve vacío.
    #[test]
    fn prop_hnsw_deletion_excludes(
        vec in vec_strategy(),
    ) {
        prop_assume!(l2_norm_sq(&vec) > f32::EPSILON);

        let (_dir, db) = setup_db();
        let input = VantaMemoryInput {
            vector: Some(vec.clone()),
            ..VantaMemoryInput::new("hnsw", "to_delete", "payload")
        };
        db.put(input).unwrap();

        // Antes de eliminar: el vector aparece
        let before = db.search_vector(&vec, 5).unwrap();
        prop_assert!(!before.is_empty(), "should find result before deletion");

        // Eliminar (único vector)
        let deleted = db.delete("hnsw", "to_delete").unwrap();
        prop_assert!(deleted, "delete must return true for existing record");

        // Después de eliminar: el índice queda vacío → search_vector vacío
        let after = db.search_vector(&vec, 5).unwrap();
        prop_assert!(after.is_empty(), "deleted vector must not appear in results");
    }

    /// Prop 7 — Múltiples vectores: con N vectores insertados, top_k <= N
    /// debe retornar exactamente top_k resultados (o menos si hay duplicados).
    /// Con N vectores y top_k = N, todas las distancias deben ser finitas.
    #[test]
    fn prop_hnsw_multiple_vectors_respects_limit(
        ref vectors in multi_vec_strategy(),
    ) {
        let n = vectors.len();
        let (_dir, db) = setup_db();
        let mut keys: Vec<String> = Vec::with_capacity(n);
        for (i, v) in vectors.iter().enumerate() {
            let key = format!("mv_{}", i);
            let input = VantaMemoryInput {
                vector: Some(v.clone()),
                ..VantaMemoryInput::new("hnsw", &key, "payload")
            };
            db.put(input).unwrap();
            keys.push(key);
        }

        // Con top_k = n, deben devolverse ≤ n resultados
        // Usamos el primer vector como query
        let query = &vectors[0];
        let hits = db.search_vector(query, n).unwrap();
        prop_assert!(
            hits.len() <= n,
            "results ({}) must not exceed top_k ({})",
            hits.len(),
            n
        );

        // Todas las distancias deben ser finitas (no NaN, no Inf)
        for hit in &hits {
            prop_assert!(hit.distance.is_finite(), "distance must be finite, got {}", hit.distance);
        }
    }
}

/// Prop 8 — Normalización consistente: insertar vectores con distintas
/// magnitudes no debe causar errores ni resultados NaN/Inf.
/// (Test fuera del macro proptest! porque usa lógica condicional
/// con vectores fijos.)
#[test]
fn prop_hnsw_mixed_normalization() {
    let (_dir, db) = setup_db();

    // Insertar vectores con distintas magnitudes
    let raw_vectors = [
        vec![1.0, 0.0, 0.0, 0.0],
        vec![2.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.5, 0.5, 0.5, 0.5],
        vec![0.1, 0.2, 0.3, 0.4],
    ];
    for (i, v) in raw_vectors.iter().enumerate() {
        let input = VantaMemoryInput {
            vector: Some(v.clone()),
            ..VantaMemoryInput::new("hnsw", format!("mixed_{}", i), "payload")
        };
        db.put(input).unwrap();
    }

    // Query con vector normalizado
    let query = vec![
        std::f32::consts::FRAC_1_SQRT_2,
        std::f32::consts::FRAC_1_SQRT_2,
        0.0,
        0.0,
    ];
    let hits = db.search_vector(&query, 10).unwrap();
    assert!(
        !hits.is_empty(),
        "mixed normalization should return results"
    );
    for hit in &hits {
        assert!(
            hit.distance.is_finite(),
            "distance must be finite, got {}",
            hit.distance
        );
        assert!(
            (-1.0..=1.0).contains(&hit.distance),
            "cosine similarity must be in [-1, 1], got {}",
            hit.distance
        );
    }

    // Query con vector normalizado de la misma dirección que dos vectores
    // insertados: [1,0,0,0] y [2,0,0,0] son colineales.
    // Cosine similarity es invariante a escala, así que ambos deberían tener
    // score ≈ 1.0 frente a query [1,0,0,0], pero la naturaleza aproximada
    // de HNSW puede retornar solo uno de ellos (por eso la cota ≥ 1).
    let q = vec![1.0, 0.0, 0.0, 0.0];
    let hits = db.search_vector(&q, 10).unwrap();
    assert!(!hits.is_empty(), "collinear query should return results");

    // Al menos un resultado debe tener score muy cercano a 1.0
    // (los vectores [1,0,0,0] y [2,0,0,0] son colineales con la query)
    let close_matches: Vec<&vantadb::VantaSearchHit> =
        hits.iter().filter(|h| h.distance > 0.99).collect();
    assert!(
        !close_matches.is_empty(),
        "at least one collinear vector should have score near 1.0, got {} close matches",
        close_matches.len()
    );
}
