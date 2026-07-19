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

#[cfg(not(target_os = "windows"))]
mod hnsw_proptests {
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
            prop_assume!(l2_norm_sq(&vec) > f32::EPSILON);

            let (_dir, db) = setup_db();
            let input = VantaMemoryInput {
                vector: Some(vec.clone()),
                ..VantaMemoryInput::new("hnsw", "identity", "payload")
            };
            db.put(input).unwrap();

            let hits = db.search_vector(&vec, 5).unwrap();
            prop_assert!(!hits.is_empty(), "identity vector must appear in results");
            prop_assert!(
                hits[0].distance > 0.99,
                "self-similarity should be ~1.0, got {}",
                hits[0].distance
            );
        }

        /// Prop 2 — Score descendente
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

        /// Prop 3 — Monotonicidad top-k
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

        /// Prop 4 — Índice vacío
        #[test]
        fn prop_hnsw_empty_index(query_vec in vec_strategy()) {
            prop_assume!(l2_norm_sq(&query_vec) > f32::EPSILON);

            let (_dir, db) = setup_db();
            let hits = db.search_vector(&query_vec, 10).unwrap();
            prop_assert!(hits.is_empty(), "empty index must return empty results");
        }

        /// Prop 5 — top_k = 0
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

        /// Prop 6 — Exclusión post-eliminación
        #[test]
        fn prop_hnsw_deletion_excludes(vec in vec_strategy()) {
            prop_assume!(l2_norm_sq(&vec) > f32::EPSILON);

            let (_dir, db) = setup_db();
            let input = VantaMemoryInput {
                vector: Some(vec.clone()),
                ..VantaMemoryInput::new("hnsw", "to_delete", "payload")
            };
            db.put(input).unwrap();

            let before = db.search_vector(&vec, 5).unwrap();
            prop_assert!(!before.is_empty(), "should find result before deletion");

            let deleted = db.delete("hnsw", "to_delete").unwrap();
            prop_assert!(deleted, "delete must return true for existing record");

            let after = db.search_vector(&vec, 5).unwrap();
            prop_assert!(after.is_empty(), "deleted vector must not appear in results");
        }

        /// Prop 7 — Múltiples vectores respeta límite
        #[test]
        fn prop_hnsw_multiple_vectors_respects_limit(
            ref vectors in multi_vec_strategy(),
        ) {
            let n = vectors.len();
            let (_dir, db) = setup_db();
            for (i, v) in vectors.iter().enumerate() {
                let key = format!("mv_{}", i);
                let input = VantaMemoryInput {
                    vector: Some(v.clone()),
                    ..VantaMemoryInput::new("hnsw", &key, "payload")
                };
                db.put(input).unwrap();
            }

            let query = &vectors[0];
            let hits = db.search_vector(query, n).unwrap();
            prop_assert!(
                hits.len() <= n,
                "results ({}) must not exceed top_k ({})",
                hits.len(),
                n
            );

            for hit in &hits {
                prop_assert!(hit.distance.is_finite(), "distance must be finite, got {}", hit.distance);
            }
        }
    }

    /// Prop 8 — Normalización consistente
    #[test]
    fn prop_hnsw_mixed_normalization() {
        let (_dir, db) = setup_db();

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

        let q = vec![1.0, 0.0, 0.0, 0.0];
        let hits = db.search_vector(&q, 10).unwrap();
        assert!(!hits.is_empty(), "collinear query should return results");

        let close_matches: Vec<&vantadb::VantaSearchHit> =
            hits.iter().filter(|h| h.distance > 0.99).collect();
        assert!(
            !close_matches.is_empty(),
            "at least one collinear vector should have score near 1.0, got {} close matches",
            close_matches.len()
        );
    }
}

#[cfg(target_os = "windows")]
mod hnsw_proptests {
    #[test]
    fn proptest_hnsw_search_skipped_on_windows() {
        eprintln!("Skipping HNSW proptests on Windows (pagefile error 1455)");
        eprintln!("To force: set VANTADB_PROPTEST_WINDOWS=1, increase pagefile, rebuild");
    }
}
