# Index Rebuild Optimization — Execution Plan

> **For sub-agents:** Implementación por fases, cada fase es una tarea independiente delegable a un sub-agente con contexto limitado.

**Goal:** Implementar Propuesta 1b (incremental threshold) + Propuesta 3 (layer-wise) + Propuesta 4 (flatten) del documento INDEX_REBUILD_OPTIMIZATION.md. Dejar Propuesta 2 (NN-Descent) para fase posterior.

**Arquitectura:** Modificaciones en la capa SDK (api.rs) + StorageEngine (ops.rs) + HNSW graph (graph.rs). No cambiar APIs públicas. Cada cambio es backward-compatible.

**Tech Stack:** Rust, DashMap, parking_lot, rayon

---

## Tarea 1: Incremental Insert Threshold en put_batch()

**Files:**
- Modify: `src/sdk/api.rs:220-249`
- Modify: `src/storage/engine/ops.rs:745-991` (no tocar, solo entender flujo)

**Contexto actual:**
`put_batch()` en api.rs línea 220-249:
```rust
engine.batch_insert_with_opts(
    &nodes,
    BatchInsertOptions {
        skip_existing_check: true,
        skip_wal: false,
        skip_hnsw: true,   // ← SIEMPRE true
    },
)?;
// ...
engine.rebuild_vector_index()?;  // ← SIEMPRE rebuild
```

**Cambio:**
Cuando `chunk.len() < INCREMENTAL_THRESHOLD`, pasar `skip_hnsw: false` y **no** llamar `rebuild_vector_index()`. Los inserts van directo al HNSW incrementalmente.

```rust
const INCREMENTAL_THRESHOLD: usize = 1000;

if chunk.len() < INCREMENTAL_THRESHOLD {
    // Batch pequeño → insertar directo al HNSW incrementalmente
    engine.batch_insert_with_opts(&nodes, BatchInsertOptions {
        skip_existing_check: true,
        skip_wal: false,
        skip_hnsw: false,  // ← incremental insert
    })?;
} else {
    // Batch grande → skip HNSW + rebuild al final
    engine.batch_insert_with_opts(&nodes, BatchInsertOptions {
        skip_existing_check: true,
        skip_wal: false,
        skip_hnsw: true,
    })?;
}

// Solo rebuild si hubo batches con skip_hnsw=true
// (trackear con flag local)
```

**Riesgo:** `batch_insert_with_opts(skip_hnsw=false)` ya existe y funciona — solo no se usaba desde SDK. El camino de `hnsw.add()` en línea 987-990 está probado. Sin riesgo de regresión.

**Verificación:**
- `cargo check -p vantadb` — compila
- `cargo test -p vantadb test_put_batch` — tests existentes pasan
- Insert manual de 100 nodos → no hay llamada a rebuild_vector_index()
- Insert manual de 10000 nodos → rebuild_vector_index() se llama como antes

---

## Tarea 2: Tests para Incremental Insert

**Files:**
- Create: `src/storage/engine/tests/incremental.rs`

**Qué testear:**
1. Insert 500 nodos (under threshold) → no rebuild, recall >= 99%
2. Insert 2000 nodos (over threshold) → rebuild happens, recall = 100% (como hoy)
3. Insert incremental mantiene recall comparable al rebuild
4. UPSERT (mismo ID) funciona con incremental insert

**Verificación:**
- `cargo test -p vantadb test_incremental` — todos pasan
- Test de recall incremental vs rebuild: delta < 1%

---

## Tarea 3: Configuración expuesta para threshold

**Files:**
- Modify: `src/sdk/api.rs:180-252`
- Modify: `src/config.rs` (si existe estructura de config)

**Cambio:**
Exponer `incremental_threshold` en la configuración del SDK para que el usuario pueda ajustarlo:
- Default: 1000
- 0 = siempre rebuild (comportamiento actual, backward compatible)
- usize::MAX = siempre incremental

**Verificación:**
- `cargo check -p vantadb` — compila
- Tests de configuración pasan

---

## Tarea 4: Flatten + RWLock Neighbor Lists (Propuesta 4)

**Files:**
- Modify: `src/index/graph.rs` — separar neighbor lists de HnswNode
- Modify: `src/index/search.rs` — usar nueva estructura
- Modify: `src/index/serialize.rs` — serialización compatible
- Modify: `src/storage/archive.rs` — traverse_graph, reindex_nodes

**Diseño:**
```rust
pub(crate) struct HnswNeighborIndex {
    /// RWLock por neighbor list — concurrencia granular
    pub lists: Vec<parking_lot::RwLock<NeighborVec>>,
    /// Mapping de node_id a índice en lists[]
    pub id_to_idx: DashMap<u128, usize>,
}
```

HnswNode pierde `neighbors: Vec<NeighborVec>` y gana `neighbor_idx: Option<usize>` (posición en HnswNeighborIndex).

**Riesgo:** Refactor mayor. Asegurar que serialization_order, traverse_graph, serialize, deserialize, ivf, y flat_search se actualicen.

**Verificación:**
- `cargo check -p vantadb` — compila
- `cargo test -p vantadb` — todos los tests pasan
- Benchmark rebuild time: 1.5-2× mejora

---

## Checkpoint 1: After Tasks 1-3
- [ ] `cargo check -p vantadb` pasa
- [ ] Tests de put_batch pasan
- [ ] Insert 100 nodos: tiempo ~20-50ms (sin rebuild)
- [ ] Insert 10000 nodos: tiempo ~2.2s (rebuild como hoy)
- [ ] Recall > 99% en ambos casos

## Checkpoint 2: After Task 4
- [ ] `cargo test -p vantadb` completo pasa
- [ ] Benchmark rebuild: 1.0-1.5s (vs 2.0-2.2s hoy)
- [ ] Serialization roundtrip preserva neighbor lists
- [ ] Búsqueda post-flatten produce mismos resultados

---

## Work Items Registrados (2026-07-31) — Fuera del plan original

### WI-1: Fix bug B2 — comparador invertido en select_neighbors ✅ COMPLETADO

**Archivos:** `src/index/search.rs` (L458-465), test `test_select_neighbors_keeps_best_scores` (L833-857)

**Descripción:** La optimización a `select_nth_unstable_by` (commit 1379343b) usó comparador normal (`a.0 < b.0`) contra `NodeSimMin::Ord` REVERSED (graph.rs:300-308) → seleccionaba los m **PEORES** vecinos → grafo degradado (build +21%).

**Fix:** comparador invertido a `b.0.partial_cmp(&a.0)` — `vec[0..m]` = m mejores, mantiene O(n) partial sort.

**Verificación:** test-first — FAILED antes (devolvía scores 0.1, 0.3), PASS después (0.9, 0.7). `cargo test -p vantadb --lib index::search` = 31 passed.

**Delegado a:** vanta-engine (24 toolcalls). No commiteado.

### WI-2: Fix harness hnsw_recall_ef — medía flat search, no HNSW ✅ COMPLETADO

**Archivos:** `benches/hnsw_recall_ef.rs` (L15, L103, L161-164)

**Descripción:** `flat_threshold: Some(10000)` con N=10000 → `use_flat_search()` = true → ef ignorado. Config real del bench: m=16, m_max0=32, ef_construction=200 (los "M=32, efC=100" de la doc eran incorrectos).

**Fix:** `flat_threshold: None` (fuerza HNSW real) + `AutoTune::set_ef(1)` por iteración (bypass auto_tuner global, espeja param_sweep.rs:237).

**Verificación:** `cargo check -p vantadb --bench hnsw_recall_ef` OK. Primer HNSW real medido: recall 0.229→0.9975 (ef 10→400).

**Delegado a:** vanta-engine. No commiteado.

### WI-3: Fix competitive_bench.py — doble build + baseline no comparable ✅ COMPLETADO

**Archivos:** `benchmarks/competitive_bench.py` (header L5-28, `bench_vantadb(..., batch_size=0)` L213, `--batch-size` flag L592, summary L694-701)

**Descripción:** put_batch con batch ≥1000 dispara rebuild HNSW dentro del timer Ingest; rebuild_index() rebuild de nuevo en Index. Baseline pre-regresión no comparable (git diff +152/−29: sin normalización coseno, JIT, warmup, median-of-3).

**Fix:** flag `--batch-size` (default 0 = comportamiento actual; `999` = incremental, sin rebuild dentro de Ingest). Delta (0−999) aísla el hidden rebuild. Caveat de baseline impreso en summary.

**Verificación:** `ast.parse` OK, `--help` muestra el flag.

**Delegado a:** vanta-tuner (15 toolcalls). No commiteado.

### Estado de verificación (bench 2026-07-31, entorno sucio)

- Recorrido de recall por ef con harness corregido: **ef_10 0.229 / ef_20 0.403 / ef_50 0.639 / ef_100 0.832 / ef_200 0.958 / ef_400 0.9975** — primer HNSW real medido.
- Search 7× más rápido que flat (ef_10 42ms vs 310ms).
- ⚠️ Build 9.12s INVALID (CPU 60-90% + soak térmico tras compile 4m49s). Pendiente re-medición limpia.
- ⚠️ ef_200 ≈ ef_100 en latencia (~1.22ms) con recall distinto — sugiere early-exit del break condition; requiere confirmación.

=== RECITATION ===
Campaign ID: 1557758b-2940-498b-8de7-53dd638bb39e
Objetivo activo: Completar INV-002 — Memory Telemetry Correction (investigación)
Estado: completed
Última acción: Delegó a vanta-tuner, revisó resultado, ejecutó skill progreso Trigger 1, auto-commit b2583288
Resultado: ✅
Próxima acción: Fase 2 del esquema (implementar gauges por categoría) como task futuro — requiere decisión del usuario
Contrato: MEMORY_TELEMETRY.md actualizado + src/ intacto + docs coverage gaps pre-existentes documentados
Próxima tarea si completa: INV-003 (Tokio Blocking Audit) si se continúa backlog
=== END RECITATION ===
