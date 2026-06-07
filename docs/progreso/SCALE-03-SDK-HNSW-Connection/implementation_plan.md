# Fase 1: HNSW Scalability & Performance — Python SDK < 20ms

## Root Cause Analysis

La investigación del código reveló que **la causa raíz del Python SDK ~200ms p50 NO es el overhead del FFI ni el GIL**, sino un **bug de arquitectura crítico en el SDK de Rust**: dos funciones de búsqueda vectorial ignoran por completo el índice HNSW y realizan brute-force O(N).

### Bug P0-A: `search_vector` — Brute Force O(N) declarado explícito

**Archivo:** [`src/sdk.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs#L2846-L2865)

```rust
pub fn search_vector(&self, vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>> {
    let engine = self.engine_handle()?;
    // "For now, delegate to a safe implementation that compiles" ← COMENTARIO ORIGINAL
    let mut hits = Vec::new();
    for node in engine.scan_nodes()? {  // ← FULL TABLE SCAN — O(N)
        if let VectorRepresentations::Full(nv) = &node.vector {
            let score = cosine_sim_f32(vector, nv);  // ← SIN HNSW
            hits.push((node.id, score));
        }
    }
    // ...
}
```

Esta función es llamada directamente desde `db.search()` en Python (lib.rs:709-721).

### Bug P0-B: `vector_memory_search` — Brute Force O(N) via `records_for_namespace`

**Archivo:** [`src/sdk.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs#L2186-L2229)

```rust
fn vector_memory_search(&self, namespace: &str, ...) -> Result<Vec<VantaMemorySearchHit>> {
    let mut hits = Vec::new();
    for record in self.records_for_namespace(namespace, filters)? {  // ← O(N) SCAN
        // Calcula distancia contra CADA nodo del namespace
    }
}
```

Esta función es llamada desde `db.search_memory()` en Python.

### Estado de las Optimizaciones del Reporte Ejecutivo

| Optimización del Reporte | Estado Real |
|---|---|
| Cachear `std::env::var` con `OnceLock` | ✅ **Ya implementado** (index.rs:80-85) |
| Eliminar `sqrt()` en hot path L2 | ✅ **Ya implementado** (euclidean_distance_squared_f32) |
| Aceleración SIMD via `wide::f32x8` | ✅ **Ya implementado** (index.rs:191-209) |
| BFS Serialization order para MMap | ✅ **Ya implementado** (storage.rs:967-1208) |
| **Conectar SDK a HNSW** | ❌ **PENDIENTE — root cause real** |

El reporte ejecutivo identificó síntomas secundarios que ya estaban corregidos. **El problema central era un stub de implementación nunca completado.**

---

## Propuesta de Implementación

### T1.1 — Conectar `search_vector` al índice HNSW [P0]

**Impacto esperado:** De O(N) scan lineal → O(log N) HNSW traversal.  
**Complejidad:** Baja — `CPIndex::search_nearest` ya existe y es correcto.

#### [MODIFY] [src/sdk.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs) — `search_vector`

Reemplazar el brute-force por una llamada directa a `hnsw.search_nearest()`:

```rust
pub fn search_vector(&self, vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>> {
    let engine = self.engine_handle()?;
    let hnsw = engine.hnsw.read();
    let vs = engine.vector_store.read();
    let raw_results = hnsw.search_nearest(
        vector,
        None,   // quantized_1bit: no aplica aquí
        None,   // quantized_3bit: no aplica aquí
        u128::MAX,  // sin filtro de bitset
        top_k,
        Some(&*vs), // vector_store para MMap path
    );
    Ok(raw_results
        .into_iter()
        .map(|(node_id, distance)| VantaSearchHit { node_id, distance })
        .collect())
}
```

**Análisis de impacto en cascada:**
- `engine.hnsw.read()` + `engine.vector_store.read()` son `RwLock`, compatibles con búsquedas concurrentes.
- `search_nearest` ya maneja ambos backends (InMemory y MMap).
- La función de Python `search()` (lib.rs:709) llama directamente a esto — no requiere cambios en el binding.

---

### T1.2 — Conectar `vector_memory_search` al índice HNSW [P0]

**Situación:** `db.search_memory()` llama a `vector_memory_search()` que hace scan lineal. El HNSW retorna `node_id`s — hay que post-filtrar por namespace y cargar los `VantaMemoryRecord` correspondientes.

**Diseño de la solución:**

El HNSW no conoce namespaces (son una abstracción del SDK). La estrategia correcta es:
1. Buscar en HNSW los `top_k * BUDGET_FACTOR` candidatos más cercanos.
2. Para cada `node_id`, cargar el nodo y verificar que pertenece al namespace correcto.
3. Aplicar filtros de metadata y devolver los primeros `top_k` que pasen.

El `BUDGET_FACTOR` compensa el filtrado post-búsqueda. Valor inicial: `min(top_k * 10, 500)`.

```rust
fn vector_memory_search(&self, namespace: &str, query_vector: &[f32], 
    filters: &VantaMemoryMetadata, top_k: usize, distance_metric: DistanceMetric
) -> Result<Vec<VantaMemorySearchHit>> {
    if query_vector.is_empty() || top_k == 0 { return Ok(Vec::new()); }

    let engine = self.engine_handle()?;
    
    // Paso 1: HNSW search con budget ampliado para compensar filtrado
    let budget = (top_k * 10).min(500).max(top_k);
    let hnsw = engine.hnsw.read();
    let vs = engine.vector_store.read();
    let candidates = hnsw.search_nearest(
        query_vector, None, None, u128::MAX, budget, Some(&*vs)
    );
    drop(hnsw); drop(vs);

    // Paso 2: Post-filtrado por namespace y metadata
    let mut hits = Vec::with_capacity(top_k);
    for (node_id, raw_score) in candidates {
        if hits.len() >= top_k { break; }
        if let Some(node) = engine.get(node_id)? {
            if let Some(record) = memory_record_from_node(node) {
                if record.namespace == namespace && matches_memory_filters(&record, filters) {
                    let score = if distance_metric == DistanceMetric::Euclidean {
                        -(-raw_score).max(0.0).sqrt()
                    } else {
                        raw_score
                    };
                    hits.push(VantaMemorySearchHit { score, record, explanation: None });
                }
            }
        }
    }
    Ok(hits)
}
```

> [!IMPORTANT]
> **Limitación del filtrado post-búsqueda:** Si el namespace tiene muy pocos registros o los filtros son muy restrictivos, el budget de `top_k * 10` puede ser insuficiente. Para la Fase 1 esto es aceptable — el comportamiento es degradarse a menos resultados, nunca a incorrectos. Una solución futura (Fase 2) es incrustar el namespace en el `bitset` de los nodos para filtrar durante el traversal HNSW (O(1) por nodo).

---

### T1.3 — Corrección de `flush()` en SDK [P1]

**Archivo:** [`src/sdk.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs#L2867-L2878)

El `flush()` del SDK (`VantaEmbedded`) es actualmente un **no-op** con comentario `"no-op for now"`. Esto puede causar pérdida de datos al cerrar sin llamar `close()`.

```rust
pub fn flush(&self) -> Result<()> {
    // ACTUAL: retorna Ok(()) sin hacer nada
    // FIX: delegar al engine real
    self.engine_handle()?.flush()
}
```

---

### T1.4 — Microbenchmark Python SDK [P1]

Crear `test_gil.py` de referencia para medir la latencia p50/p99 antes y después con 10K vectores 128d:

```python
# Medir: db.search(query, top_k=10) × 1000 iteraciones
# Reportar: p50, p95, p99, throughput QPS
```

El archivo `test_gil.py` ya existe en el workspace — lo ampliaremos con mediciones comparativas.

---

## Análisis de Riesgos

| ID | Riesgo | P | I | Mitigación |
|---|---|:---:|:---:|---|
| R1 | El HNSW retorna `node_id`s que no existen en el namespace solicitado, causando resultados vacíos | 2 | 4 | Budget 10x asegura cobertura suficiente para namespaces típicos |
| R2 | `engine.hnsw.read()` y `engine.vector_store.read()` adquiridos en `search_vector` pueden interbloquear con writes | 1 | 5 | Los RwLock permiten múltiples lectores simultáneos — no hay deadlock |
| R3 | El comportamiento de `vector_memory_search` cambia para namespaces con muy baja densidad | 3 | 2 | Fallback al scan lineal si HNSW retorna 0 candidatos del namespace |
| R4 | Cambiar `flush()` de no-op a real puede impactar tests que asumen que es barato | 2 | 2 | Revisar tests antes de hacer el cambio |

---

## Plan de Verificación

### Compilación
```powershell
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Tests existentes
```powershell
cargo test --workspace --release -- --nocapture 2>&1 | tail -20
```

### Benchmark Python SDK (ejecutar manualmente)
```powershell
# Instalar SDK primero
cd vantadb-python
maturin develop --release

# Luego en la raíz
python test_gil.py
```

**Criterio de aceptación:** Python SDK p50 < 20ms para búsqueda vectorial en 10K vectores 128d.

---

## Secuencia de Implementación

1. **T1.1** → Modificar `search_vector` en `src/sdk.rs` → `cargo check`
2. **T1.2** → Modificar `vector_memory_search` en `src/sdk.rs` → `cargo check`  
3. **T1.3** → Corregir `flush()` en `src/sdk.rs`
4. Ejecutar `cargo test --workspace` (automático si tienes permiso, o lo corres tú)
5. Ejecutar `maturin develop --release` + `python test_gil.py` para certificar latencia
6. Commit y snapshot histórico
