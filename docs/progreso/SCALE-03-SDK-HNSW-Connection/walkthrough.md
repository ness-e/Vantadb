# Walkthrough: Fase 1 — HNSW Performance (Python SDK < 20ms)

## Objetivo

Eliminar la latencia de ~200ms p50 del Python SDK en búsquedas vectoriales,
alcanzando el target de < 20ms p50.

---

## Root Cause Real

La investigación reveló que **las optimizaciones del reporte ejecutivo (SIMD, sqrt,
OnceLock, BFS serialization) ya estaban implementadas**. El problema era anterior y
más fundamental: dos funciones del SDK nunca fueron conectadas al índice HNSW.

| Función | Ubicación | Problema |
|---|---|---|
| `search_vector()` | `src/sdk.rs:2846` | Brute-force O(N) con comentario literal: *"For now, delegate to a safe implementation that compiles"* |
| `vector_memory_search()` | `src/sdk.rs:2186` | Scan lineal O(N) via `records_for_namespace()` |
| `flush()` | `src/sdk.rs:2867` | No-op silencioso — nunca sincronizaba al disco |

El índice `CPIndex::search_nearest` existía, era correcto y funcional — se usaba
desde el executor de IQL/LISP — pero el SDK de Python nunca lo llamaba.

---

## Cambios Implementados

### `src/sdk.rs` — 3 modificaciones quirúrgicas

#### 1. `search_vector()` — O(N) → O(log N)

```diff
-   // "For now, delegate to a safe implementation that compiles"
-   for node in engine.scan_nodes()? {          // FULL TABLE SCAN
-       let score = cosine_sim_f32(vector, nv); // sin HNSW
-   }
+   let hnsw = engine.hnsw.read();
+   let vs = engine.vector_store.read();
+   let results = hnsw.search_nearest(vector, None, None, u128::MAX, top_k, Some(&*vs));
```

#### 2. `vector_memory_search()` — O(N) → HNSW + post-filtrado

- Budget de búsqueda HNSW = `min(top_k × 10, 500)` — compensa el filtrado por namespace
- Post-filtra por `namespace` y `filters` en candidatos devueltos por HNSW
- **Fallback a scan lineal** cuando HNSW retorna 0 resultados del namespace objetivo
  (garantía de correctitud para namespaces pequeños o índice recién inicializado)

#### 3. `flush()` — no-op → flush real

```diff
-   // Explicit flush is a no-op for now but satisfies the SDK boundary.
-   Ok(())
+   self.engine_handle()?.flush()
```

Ahora delega a `StorageEngine::flush()` que sincroniza el backend KV (Fjall) y el
archivo de vectores MMap al disco.

---

## Resultados de Verificación

### Compilación
```
cargo check --workspace → ✅ Finished in 7.46s (0 errors)
```

### Tests unitarios
```
cargo test --package vantadb --release --lib
→ test result: ok. 37 passed; 0 failed; finished in 2.48s ✅
```

### Tests de integración
```
cargo test --package vantadb --release --test '*'
→ test result: ok. 13 passed; 0 failed; finished in 1.69s ✅ (backend_tests)
→ test result: ok. 1 passed; 0 failed; finished in 0.62s ✅ (basic_node)
```

### Benchmark Python SDK (`test_gil.py`)
```
Ingesting 1000 records...
DB background thread completed 5930 search hits (en 1 segundo).
CPU work efficiency while DB is running: 102.51%
SUCCESS: GIL is released!
```

**Throughput real:** 5930 búsquedas/segundo = **~0.169ms por búsqueda**

| Métrica | Antes (brute-force) | Después (HNSW) | Target |
|---|:---:|:---:|:---:|
| p50 latencia estimada | ~200ms | **~0.17ms** | < 20ms |
| Algoritmo | O(N) scan | O(log N) HNSW | — |
| GIL liberado | ✅ (ya era correcto) | ✅ | ✅ |

---

## Commit

```
cb4e0af  perf(sdk): connect search_vector and vector_memory_search to HNSW index
```

---

## Notas para Fase 2

- La limitación del budget de filtrado `min(top_k*10, 500)` puede ser insuficiente
  para namespaces con muy baja densidad en datasets masivos. Solución futura: incrustar
  el namespace en el bitset de nodos HNSW para filtrar **durante** el traversal.
- El `vector_memory_search` conserva el fallback lineal como red de seguridad.
  En Fase 2 se puede deprecar ese fallback una vez se implemente el filtrado en traversal.
