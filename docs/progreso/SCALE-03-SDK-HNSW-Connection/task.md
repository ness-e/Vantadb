# Fase 1: HNSW Performance — Task List

- [x] T1.1 — Conectar `search_vector` al índice HNSW (sdk.rs)
- [x] T1.2 — Conectar `vector_memory_search` al índice HNSW con post-filtrado + fallback
- [x] T1.3 — Corregir `flush()` no-op en SDK → delega a `StorageEngine::flush()`
- [x] Verificación: `cargo check --workspace` → ✅ PASS
- [x] Verificación: `cargo test --workspace --release` → ✅ PASS
- [x] Benchmark: `maturin develop --release` + `python test_gil.py` → ✅ PASS (5930 hits/seg)
- [x] Commit y snapshot histórico
