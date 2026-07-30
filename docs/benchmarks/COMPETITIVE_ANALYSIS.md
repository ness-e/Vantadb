# VantaDB Competitive Analysis

> Generado: 2026-07-30  
> Benchmark: `competitive_bench.py` — 3 motores, 2 datasets, 10K vectores, 100 queries, top-K=10  
> **Versión evaluada:** v0.4.0 (local build, Fase 1 + Fase 2 + Propuesta 1b + Propuesta 4 activas)
> 
> ⚠️ **Regresión detectada vs baseline Jul 29:** VantaDB index time se duplicó (~2.2s → ~4.2s). Ingest QPS cayó ~34-44%. LanceDB/ChromaDB también cayeron ~5-13% (factor sistémico parcial), pero VantaDB es mucho mayor. Probable causa: diferencias en features del build (`maturin build --release` vs `maturin develop --release`), CPU throttling entre corridas, o cambios de código entre commits. Pendiente investigación en sección 3C.

---

## 1. Resultados

### GloVe-100-angular (100d, Cosine)

| Métrica | VantaDB (hoy) | VantaDB (baseline) | LanceDB | ChromaDB | Ganador |
|---------|--------------|-------------------|---------|----------|---------|
| **Ingest (QPS)** | **2,076.3** | 3,157.1 (-34%) | 116,678 | 3,198.8 | LanceDB |
| **Index (ms)** | **4,157.9** | 2,196.2 (+89%) | 1,950.4 | N/A (Inc) | LanceDB |
| **Query (QPS)** | 458.9 | 468.3 (~igual) | 200.9 | **727.9** | ChromaDB |
| **p50 (ms)** | 1.94 | 1.91 (~igual) | 4.528 | **1.051** | ChromaDB |
| **p99 (ms)** | **3.615** | 4.49 (-20%) | 8.936 | 5.839 | **VantaDB** |
| **Recall@10** | **100.00%** | 100.00% | 23.70% | 95.80% | **VantaDB** |
| **Peak RSS (MB)** | 279.1 | 285.6 | 360.5 | **238.4** | ChromaDB |
| **Delta RSS (MB)** | 99.4 | 134.1 (-26%) | — | **38.2** | ChromaDB |

> ⚠️ **Regresión vs baseline:** Index +89%, Ingest -34%. Causa bajo investigación (ver §3C).

### SIFT-128-euclidean (128d, Euclidean)

| Métrica | VantaDB (hoy) | VantaDB (baseline) | LanceDB | ChromaDB | Ganador |
|---------|--------------|-------------------|---------|----------|---------|
| **Ingest (QPS)** | **1,888.1** | 3,357.9 (-44%) | 98,401 | 3,295.9 | LanceDB |
| **Index (ms)** | **4,278.8** | 2,004.2 (+113%) | 2,436.7 | N/A (Inc) | LanceDB |
| **Query (QPS)** | 396.6 | 448.3 (-12%) | 214.0 | **592.7** | ChromaDB |
| **p50 (ms)** | 2.29 | 1.98 (+16%) | 4.047 | **1.423** | ChromaDB |
| **p99 (ms)** | 4.751 | 3.42 (+39%) | 9.428 | **4.484** | ChromaDB |
| **Recall@10** | 99.40% | 99.40% | 63.80% | **99.80%** | ChromaDB |
| **Peak RSS (MB)** | **291.9** | 284.8 | 373.7 | 276.5 | ChromaDB |
| **Delta RSS (MB)** | 99.8 | 134.4 (-26%) | — | **26.2** | ChromaDB |

> ⚠️ **Regresión vs baseline:** Index +113%, Ingest -44%. Causa bajo investigación (ver §3C).

---

## 2. Ventajas de VantaDB

### ✅ Recall perfecto en cosine

- **GloVe 100% / SIFT 99.4%** — incluso con `ef_construction=100` (optimizado para velocidad), VantaDB mantiene recall perfecto en cosine y 99.4% en euclidean
- LanceDB tiene recall pobre en cosine (23.8%) — su índice IVF-PQ sacrifica precisión
- ChromaDB es 1.8× más rápido en queries pero entrega ~95% recall — VantaDB es el único motor con **100% recall** en cosine

### ✅ Mejora dramática en ingesta: 17.2× vs baseline documentado

De 184 QPS → **3,157 QPS** en GloVe, de 240 QPS → **3,358 QPS** en SIFT. El gap con LanceDB se redujo de 650× a ~33×, **VantaDB supera a ChromaDB** en GloVe (3,157 vs 3,406 QPS — empate técnico) y le pisa los talones en SIFT (3,358 vs 3,530 QPS).

### ✅ Optimizaciones aplicadas (Jul 2026)

| # | Optimización | Fase | Impacto |
|---|-------------|------|---------|
| 1 | `ShardedWal::batch_append()` group-by-shard | F1 | WAL: 6,174ms → ~18ms/1K (325×) |
| 2 | `metadata.clone()` eliminado en serialization | F1 | ~200K allocs menos por 10K records |
| 3 | `put_batch()` → `engine.batch_insert_with_opts()` | F1 | 2.3× vs loop per-item put() |
| 4 | `ef_construction`: 400 → 100 | F1 | 4× menos distancia en HNSW rebuild |
| 5 | `select_neighbors` simplificado (sin diversity) | F1 | 2.5× rebuild más rápido |
| 6 | `BatchInsertOptions` con `skip_hnsw` + rebuild diferido | F1 | Pipeline puro ~32K QPS teóricos |
| 7 | **HNSW rebuild paralelo con rayon** | **F2** | **3.5× index (GloVe 7.7s→2.2s)** |
| 8 | **add_with_level + thread-local RNG** | **F2** | Evita contención Mutex RNG |
| 9 | **InsertMode incremental (Propuesta 1b)** | **F3** | **4-10× en inserts <50 nodos, recall 100%** |
| 10 | **NN-Descent (Propuesta 2)** | ❌ **REVERTIDA** | **Regresión catastrófica 7-1,332×** — revertida en commit f1b9ee03 |
| 11 | **Propuesta 4: flat_threshold + index_type** | **P4** | Activa pero no afecta rebuild. Pendiente verificar. |

### ✅ Pipeline puro de insert: ~31ms/1K ≈ 32K QPS teóricos

---

## 3. Deficiencias Restantes y Plan de Mejora

### 🟡 INGESTA: mejora lograda, target no alcanzado

VantaDB inserta **3,157 QPS** (GloVe) / **3,358 QPS** (SIFT) — 17.2× y 14.0× sobre baseline. **Target 2,200 QPS ALCANZADO.** Aún lejos de LanceDB (~100K QPS), pero **empata técnicamente a ChromaDB** (3,157 vs 3,406 QPS GloVe, 3,358 vs 3,530 QPS SIFT) con mejor recall.

> **Logro principal:** El pipeline de insert puro (sin HNSW rebuild) mide ~31ms/1K = ~32K QPS teóricos. Con Fase 2 (parallel rebuild), el rebuild bajó de 7.7s→2.2s (86% del tiempo vs 96%), y el gap con ChromaDB se redujo de 2.5× a 5-7%.

#### Optimizaciones Aplicadas (Jul 2026)

| # | Acción | Archivos | Ganancia |
|---|--------|----------|----------|
| 1 | **ShardedWal::batch_append()** group-by-shard (WAL 325×) | `src/wal_sharded.rs` | WAL: 6,174ms → ~18ms/1K |
| 2 | **metadata.clone()** eliminado en `memory_record_to_node_owned()` | `src/sdk/serialization/mod.rs` | ~200K allocs menos |
| 3 | **put_batch() → batch_insert_with_opts()** con skip_hnsw | `src/sdk/api.rs` + `src/storage/engine/ops.rs` | 2.3× vs loop per-item |
| 4 | **ef_construction 400 → 100** | `src/index/graph.rs:251` | 4× menos distancia HNSW |
| 5 | **select_neighbors simplificado** (top-M sin diversity) | `src/index/search.rs` | 2.5× rebuild más rápido |

### 🔴 HNSW REBUILD LENTO (prioridad alta)

VantaDB tarda **2.2s** (GloVe) / **2.0s** (SIFT) en rebuild index vs LanceDB **2.2-2.6s**. **VantaDB supera a LanceDB** en SIFT (2.0s vs 2.6s) y empata en GloVe (2.2s vs 2.2s). A 10K vectores, rebuild domina ~86% del tiempo total (vs 96% en Fase 1).

#### Fase 2 Implementada (Jul 2026) — HNSW Rebuild Paralelo

| # | Acción | Estado | Archivo | Descripción |
|---|--------|--------|---------|-------------|
| 6 | **HNSW rebuild paralelo con rayon** | ✅ IMPLEMENTADO | `src/storage/archive.rs` + `src/index/graph.rs` | `rebuild_hnsw_from_vstore_with_segment()` two-phase: scan secuencial → `rayon::into_par_iter()` con thread-local RNG. `#[cfg(feature = "rayon")]` gateado. |
| 7 | **add_with_level() + random_layer_from_config()** | ✅ IMPLEMENTADO | `src/index/graph.rs` | Evita contención en `rng: Mutex<StdRng>` — cada thread tiene su propio `thread_rng()` |
| 8 | **Layer-wise bulk insert** | ⏳ Pendiente | — | Saltar rebuild intermedio, insert por capas |

> **Benchmarks Fase 2 completados.** Rebuild real: GloVe 7.7s → **2.2s** (3.5×), SIFT 6.4s → **2.0s** (3.2×). Ingesta real: GloVe **3,157 QPS** (2.7× vs Fase 1), SIFT **3,358 QPS** (2.5× vs Fase 1).

### 🟢 QUERY LATENCIA COMPETITIVA (prioridad media-baja)

ChromaDB es ~1.8× más rápido en queries que VantaDB. Sin embargo, ChromaDB usa HNSW incremental (inserción directa sin rebuild) y sacrifica ~5% recall para ganar velocidad.

**Mejora propuesta:**

| # | Acción | Impacto estimado |
|---|--------|-----------------|
| 9 | Exponer `ef_search` como parámetro en `search_memory()` | Permite tuning recall↔velocidad |
| 10 | Auto-tune `ef_search` según tamaño del dataset | 5-20% mejora en latencia media |
| 11 | SIMD check: verificar AVX2/SSE en búsqueda coseno | 10-15% si falta |

### 🔵 DIFUSIÓN DE RECALL EN LANCEDB (hallazgo confirmado)

LanceDB tiene recall 23.8% en cosine y 63.6% en euclidean. IVF-PQ no está bien tuneado para datasets pequeños. VantaDB puede usar esto como argumento de venta: **recall perfecto predecible** a velocidades competitivas.

### 🟡 INDEX REBUILD LENTO (prioridad media)

VantaDB tarda **33.5s** en rebuild index (medido, sintético 10K). LanceDB: 2.6s.

> **Nota:** El tiempo de rebuild crece más que linealmente con el tamaño del dataset porque reconstruye el HNSW completo desde cero en un solo hilo. A 10K vectores ya toma 33s — a 100K podría escalar a minutos. LanceDB usa IVF-PQ que escala O(N·centroids) en vez de O(N²·log N) del HNSW single-threaded.

**Mejora propuesta:**

| # | Acción | Impacto estimado |
|---|--------|-----------------|
| 4 | `rebuild_index()` con construcción paralela del HNSW (multi-hilo en capa Rust) | **3-5x** |
| 5 | Index incremental: mutar el HNSW en vez de reconstruirlo completo tras cada ingesta | **10-20x** (elimina necesidad de rebuild) |

### 🟢 QUERY LATENCIA COMPETITIVA (prioridad media-baja)

ChromaDB es ~1.5-2x más rápido en queries que VantaDB. Sin embargo, ChromaDB usa HNSW incremental (insersión directa sin rebuild) con defaults tuneados para velocidad.

**Causa:** `ef_search` default en VantaDB puede ser conservador. ChromaDB sacrifica un ~5% de recall para ganar velocidad.

**Mejora propuesta:**

| # | Acción | Impacto estimado |
|---|--------|-----------------|
| 6 | Exponer `ef_search` como parámetro en `search_memory()` de la Python API | **Permite tuning** recall ↔ velocidad |
| 7 | Auto-tune `ef_search` según tamaño del dataset (ej: `ef_search = sqrt(N)`) | **5-20%** mejora en latencia media |
| 8 | SIMD acceleration check: verificar que AVX2/SSE se usa en la búsqueda coseno | Mantener o **10-15%** si falta |

### 🔵 DIFUSIÓN DE RECALL EN LANCEDB (hallazgo)

LanceDB tiene recall 23.7% en cosine y 61.7% en euclidean con 10K vectores. Esto no es culpa de VantaDB, pero sugiere que **el IVF-PQ de LanceDB no está bien tuneado para datasets pequeños**. VantaDB puede usar esto como argumento de venta: recall predecible y configurable.

---

## 3A. Resultados Finales de Benchmark (Post-Optimización)

Benchmarks ejecutados con build local (v0.4.0, Jul 2026) usando datasets reales de ann-benchmarks: 10K vectores, 100 queries, top-K=10.

### GloVe-100-angular (100d, Cosine)

| Engine   | Ingest QPS | Index (ms) | Query QPS | p50 (ms) | p99 (ms) | Recall@10 | Peak RSS |
|----------|-----------|------------|-----------|---------|---------|-----------|----------|
| VantaDB  | **2,076.3** | 4,157.9 | 458.9 | 1.940 | 3.615 | **100.00%** | 279.1 MB |
| LanceDB  | 116,678 | **1,950.4** | 200.9 | 4.528 | 8.936 | 23.70% | 360.5 MB |
| ChromaDB | 3,198.8 | N/A (Inc) | **727.9** | **1.051** | 5.839 | 95.80% | **238.4 MB** |

### SIFT-128-euclidean (128d, Euclidean)

| Engine   | Ingest QPS | Index (ms) | Query QPS | p50 (ms) | p99 (ms) | Recall@10 | Peak RSS |
|----------|-----------|------------|-----------|---------|---------|-----------|----------|
| VantaDB  | 1,888.1 | 4,278.8 | 396.6 | 2.290 | 4.751 | 99.40% | 291.9 MB |
| LanceDB  | 98,401 | **2,436.7** | 214.0 | 4.047 | 9.428 | 63.80% | 373.7 MB |
| ChromaDB | **3,295.9** | N/A (Inc) | **592.7** | **1.423** | **4.484** | **99.80%** | **276.5 MB** |

> ⚠️ **Regresión detectada vs baseline Jul 29:** VantaDB index time se duplicó (~2.2s → ~4.2s).  
> **VantaDB pierde liderazgo en index time:** LanceDB ahora es 2.1× más rápido en GloVe (1.95s vs 4.16s).  
> **ChromaDB mantiene ventaja en queries** (~1.5-1.6× VantaDB).  
> **Recall se mantiene:** 100% GloVe, 99.40% SIFT.

## 3B. Post-Mortem — Por Qué No Se Alcanzó el Factor 30-300x

### Lo que se cambió (Fase 1)

| # | Acción | Estado | Ganancia real |
|---|--------|--------|-------------|
| 1a | `put_batch_raw` expuesto en API Python | ✅ | Zero-copy numpy |
| 1b | `put_batch()` → `engine.batch_insert_with_opts()` | ✅ | 2.3× vs per-item |
| 1c | Benchmark usa `put_batch_raw(vectors=ndarray)` | ✅ | Elimina loop `.tolist()` |
| 2 | Auto-flush deshabilitado en batch_insert | ✅ | Menos flushing |
| 3 | ShardedWal batch_append real (group-by-shard) | ✅ | WAL 325× más rápido |
| 4 | metadata.clone() eliminado | ✅ | ~200K allocs menos |
| 5 | ef_construction: 400 → 100 | ✅ | 4× rebuild |
| 6 | select_neighbors simplificado (top-M) | ✅ | 2.5× rebuild |
| 7 | skip_hnsw + rebuild diferido | ✅ | Pipeline puro ~32K QPS |
| **8** | **HNSW rebuild paralelo con rayon (Fase 2)** | **✅** | **3.5× index time** |

### Dónde está realmente el cuello de botella (hoy)

El bottleneck actual NO es el pipeline de insert (que corre a ~32K QPS teóricos). Era la **reconstrucción del HNSW** — y con Fase 2 (parallel rebuild), se redujo drásticamente:

```
GloVe 10K — Fase 1 (single-threaded):      GloVe 10K — Fase 2 (parallel rayon):
  7.7s total                                   2.2s total
  ├── Insert pipeline = ~0.3s  ← 4%            ├── Insert pipeline = ~0.3s  ← 14%
  └── rebuild_vector_index() = ~7.4s ← 96%     └── rebuild_vector_index() = ~1.9s ← 86%

  Resultado: 1,187 QPS                         Resultado: 3,157 QPS  (2.7×)
```

### Fase 2 Completada — Resultados Reales (Jul 2026)

| # | Acción | Estado | Impacto real |
|---|--------|--------|-------------|
| 8 | **HNSW rebuild paralelo con rayon** | ✅ Completo | **3.5× index** (GloVe 7.7s→2.2s), **3.2×** (SIFT 6.4s→2.0s) |
| 9 | **add_with_level() + thread-local RNG** | ✅ Completo | Evita contención Mutex RNG en rebuild paralelo |
| 10 | **Layer-wise bulk insert skip** | ⏳ Pendiente | **2-3×** adicional estimado |

> **Fase 2 logró 2.7× en ingesta total (GloVe: 1,187→3,157 QPS) y 3.5× en index time. VantaDB ahora iguala o supera a LanceDB en index time, y está a un 5-7% de ChromaDB en ingesta bruta — con 100% recall vs 95-96% de ChromaDB.**

### Competencia

**LanceDB (~100K QPS):** append-only columnar. Su ventaja es arquitectónica, no de implementación — no necesita WAL, KV store, edge index, ni HNSW. VantaDB hace 6× más operaciones por insert.

**ChromaDB (~3K QPS):** HNSWlib incremental (C++). Su rebuild es instantáneo porque no reconstruye — inserta incrementalmente. VantaDB podría emular esto con index worker thread (P5 del COMPAT original).

---
## 3C. Post-Regresión — Benchmark Jul 30, 2026

> **Contexto:** Tras revertir Propuesta 2 (NN-Descent, que causó regresión catastrófica de 7-1,332×), se rebuildéo `vantadb_py` con `maturin build --release` y se instaló el wheel local. Los benchmarks competitivos se ejecutaron para verificar el estado actual.

### Delta vs Baseline (Jul 29, Fase 2)

#### GloVe-100-angular

| Métrica | Jul 29 (F2) | Jul 30 (hoy) | Δ | Diagnóstico |
|---------|------------|-------------|---|-------------|
| **Ingest QPS** | 3,157.1 | 2,076.3 | **-34%** 🔴 | Puede ser build flags, throttling |
| **Index (ms)** | 2,196.2 | 4,157.9 | **+89%** 🔴 | **Principal preocupación** |
| **Query QPS** | 468.3 | 458.9 | -2% 🟢 | Dentro de ruido |
| **Recall@10** | 100.00% | 100.00% | 0% ✅ | Sin cambio |

#### SIFT-128-euclidean

| Métrica | Jul 29 (F2) | Jul 30 (hoy) | Δ | Diagnóstico |
|---------|------------|-------------|---|-------------|
| **Ingest QPS** | 3,357.9 | 1,888.1 | **-44%** 🔴 | Puede ser build flags, throttling |
| **Index (ms)** | 2,004.2 | 4,278.8 | **+113%** 🔴 | **Se duplicó** |
| **Query QPS** | 448.3 | 396.6 | -12% 🟡 | Leve, consistente con throttling |
| **Recall@10** | 99.40% | 99.40% | 0% ✅ | Sin cambio |

### Causas posibles

| # | Hipótesis | Probabilidad | Explicación |
|---|-----------|-------------|-------------|
| 1 | **Build flags diferentes** | 🟠 Alta | El baseline usó `maturin develop --release` (link simbólico). Hoy usamos `maturin build --release` + copia manual del `.pyd`. Podría faltar feature flag `rayon`, `simd`, u optimización del compilador. |
| 2 | **CPU throttling térmico** | 🟡 Media | Tras NN-Descent benchmarks (506s sostenidos a 100% CPU), el sistema puso los cores en throttling. LanceDB/ChromaDB también cayeron ~5-13% consistente con esto. VantaDB cae más porque rebuild es CPU-bound. |
| 3 | **Background load** | 🟡 Media | Windows Update, antivirus, u otro proceso background compitiendo por CPU. |
| 4 | **Feature flag desactivado** | 🟠 Alta | Si `rayon` no está activo en el build, el rebuild cae a single-threaded y explicaría el 2×. Verificar con `cargo metadata --features` o inspeccionar el Cargo.toml del wheel. |
| 5 | **Cambio de código no contabilizado** | 🟢 Baja | No hubo cambios en rebuild/insert entre commits. La reversión de Propuesta 2 restauró `archive.rs` Phase 2 a parallel insert. |

### Próximos pasos (investigación)

1. Verificar features activas en el wheel instalado: `vantadb_py` Cargo.toml → `[features]`
2. Comparar flags de compilación entre `maturin develop --release` y `maturin build --release`
3. Re-ejecutar solo VantaDB (sin LanceDB/ChromaDB) para aislar ruido de otros motores
4. Si se confirma feature faltante, rebuildear con flags correctos y re-benchmark

---

## 4. Animales y Paredes (tl;dr)

- **Regresión detectada en benchmark Jul 30:** index time **2× más lento** (4.2s vs 2.2s), ingest QPS **cayó 34-44%** vs baseline Fase 2.
- **Causa más probable:** diferencias en build flags entre `maturin develop --release` (baseline) y `maturin build --release` (hoy). Posible feature faltante (rayon).
- **LanceDB ahora lidera en index time** (1.95s vs 4.16s GloVe). ChromaDB mantiene liderazgo en queries.
- **Recall se mantiene perfecto:** 100% cosine, 99.40% euclidean.
- **Pipeline puro de insert:** ~32K QPS teóricos — **sin cambios estructurales**.
- **Target 2,200 QPS ALCANZADO** en baseline ✅, pero **hoy estamos en 2,076** (justo debajo del target).

### Próximos pasos

| Prioridad | Acción | Impacto |
|-----------|--------|---------|
| 🔴 P0 | **Investigar regresión index time 2×** — comparar build flags develop vs build, verificar feature `rayon` | Crítico — recuperar rendimiento perdido |
| 🟢 P1 | **✅ InsertMode incremental completado** — Propuesta 1b | **4-10× en inserts <50 nodos** |
| 🟢 P2 | Flatten + RWLock neighbor lists (Propuesta 4) | **1.5-2×** en rebuild paralelo |
| 🟢 P3 | SIMD check AVX2/SSE en búsqueda coseno | **10-15%** query speed |

---

## 5. Datos Crudos

### Benchmark 10K — Fase 2 + Propuesta 1b + Propuesta 4 (Jul 30, 2026 — Post NN-Descent revert)

Benchmark con build `maturin build --release` tras revertir Propuesta 2 (NN-Descent). Features activas en build: verificables en `vantadb-python/Cargo.toml`.

#### GloVe-100-angular

| Engine | Ingest QPS | Index (ms) | Query QPS | p50 (ms) | p99 (ms) | Recall@10 | Peak RSS (MB) |
|--------|-----------|------------|-----------|---------|---------|-----------|----------|
| **VantaDB** | 2,076.3 | 4,157.9 | 458.9 | 1.940 | 3.615 | **100.00%** | 279.1 |
| LanceDB | **116,678** | **1,950.4** | 200.9 | 4.528 | 8.936 | 23.70% | 360.5 |
| ChromaDB | 3,198.8 | N/A (Inc) | **727.9** | **1.051** | 5.839 | 95.80% | **238.4** |

#### SIFT-128-euclidean

| Engine | Ingest QPS | Index (ms) | Query QPS | p50 (ms) | p99 (ms) | Recall@10 | Peak RSS (MB) |
|--------|-----------|------------|-----------|---------|---------|-----------|----------|
| VantaDB | 1,888.1 | 4,278.8 | 396.6 | 2.290 | 4.751 | 99.40% | 291.9 |
| LanceDB | **98,401** | **2,436.7** | 214.0 | 4.047 | 9.428 | 63.80% | 373.7 |
| ChromaDB | 3,295.9 | N/A (Inc) | **592.7** | **1.423** | **4.484** | **99.80%** | **276.5** |

#### Delta vs Baseline Fase 2 (Jul 29)

| Métrica | GloVe Δ | SIFT Δ |
|---------|---------|--------|
| **VantaDB Ingest QPS** | **-34%** 🔴 | **-44%** 🔴 |
| **VantaDB Index (ms)** | **+89%** 🔴 | **+113%** 🔴 |
| **VantaDB Query QPS** | -2% 🟢 | -12% 🟡 |
| **VantaDB Recall** | 0% ✅ | 0% ✅ |
| **LanceDB Index (ms)** | -13% 🟢 | -8% 🟢 |
| **ChromaDB Ingest QPS** | -6% 🟡 | -7% 🟡 |

### Histórico de optimización (VantaDB ingesta, GloVe-100-angular 10K)

| Fecha/Commit | Estado | QPS | Mejora |
|---|---|---|---|---|
| Baseline documentado | PyPI v0.2.0, loop per-item | 184 | — |
| Commit 0dd147d1 (P1-P4) | Sin `put_batch_raw`, sin fix .pyd | 119.8 | -35% (regresión) |
| Post-fix .pyd + api.rs | batch_insert_with_opts() implementado | 394.6 | 2.1× |
| Post-ef_construction=100 | 4× rebuild más rápido | ~1,667* | 4.2× |
| Post-select_neighbors simplificado | 2.5× rebuild más rápido | ~3,600** | 9× |
| **Final Fase 1** | **Benchmark real GloVe 10K** | **1,187** | **6.5×** |
| **Fase 2: parallel rebuild** | **rayon + thread-local RNG** | **3,157** | **17.2×** |
| **Pipeline puro (sin rebuild)** | **Solo insert** | **~32,000** | **174×** |
| **Fase 3: InsertMode incremental** | **InsertMode::Auto, threshold 1000** | **4-10× (batches <50)** | **—** |
| **Jul 30 (post revert)** | **Tras revertir NN-Descent** | **2,076** | **11.3×** 🔴 |

\*Estimaciones de vanta-tuner antes de benchmark real. La discrepancia se debe a que otros overheads (vstore, KV, metadata) limitan el QPS real.

> **Condiciones:** Windows 11, SSD NVMe, 32GB RAM. Build local v0.4.0 con `maturin build --release` + instalación del wheel. Datasets de ann-benchmarks HDF5.

---

## 6. Historial Consolidado de Optimizaciones

> Esta sección resume todas las optimizaciones aplicadas, probadas y documentadas en los archivos de optimización ahora eliminados (`COMPAT_INGESTA_OPTIMIZACION.md`, `INDEX_REBUILD_OPTIMIZATION.md`, `RECOVERY_PLAN.md`). El contenido ha sido consolidado aquí para mantener un registro histórico único.

### 6.1 Optimizaciones Aplicadas y en Código

| # | Optimización | Documento Origen | Impacto Medido |
|---|-------------|-----------------|----------------|
| 1 | `put_batch_raw` expuesto en API Python (zero-copy numpy) | COMPAT_INGESTA | Elimina overhead FFI |
| 2 | `put_batch()` → `engine.batch_insert_with_opts()` | COMPAT_INGESTA | 2.3× vs per-item loop |
| 3 | Auto-flush deshabilitado en batch_insert | COMPAT_INGESTA | Menos flushing frecuente |
| 4 | ShardedWal batch_append real (group-by-shard) | COMPAT_INGESTA | WAL 6,174ms → ~18ms/1K (325×) |
| 5 | metadata.clone() eliminado en serialization | COMPAT_INGESTA | ~200K allocs menos por 10K records |
| 6 | ef_construction: 400 → 100 | COMPAT_INGESTA | 4× menos distancia en HNSW rebuild |
| 7 | select_neighbors simplificado (top-M sin diversity) | COMPAT_INGESTA | 2.5× rebuild más rápido |
| 8 | skip_hnsw + rebuild diferido (BatchInsertOptions) | COMPAT_INGESTA | Pipeline puro ~32K QPS teóricos |
| 9 | HNSW rebuild paralelo con rayon | INDEX_REBUILD | 3.5× index (GloVe 7.7s → 2.2s) |
| 10 | add_with_level() + thread-local RNG | INDEX_REBUILD | Evita contención Mutex RNG en rebuild paralelo |
| 11 | InsertMode incremental (threshold 1000) | INDEX_REBUILD | 4-10× en inserts <50 nodos, recall 100% |
| 12 | Layer-wise bulk insert (sort por nivel) | INDEX_REBUILD | Mejor calidad de entry points en batches 100-1000 |
| 13 | HnswNeighborIndex — flatten + RWLock | INDEX_REBUILD | Rebuild 1.88s → 760ms (2.47×) |
| 14 | try_add_and_get_if_full() (1 acceso DashMap vs 3) | RECOVERY_PLAN | Index time mejoró 12-27% |
| 15 | Fase 0 mitigación sistémica | RECOVERY_PLAN | Liberar disco, matar procesos, RAYON_NUM_THREADS=4 |
| 16 | **target-cpu=native reactivado** | INVESTIGACIÓN 2026 | **+20-50% en kernels SIMD** |
| 17 | **select_nth_unstable_by** (O(n log n) → O(n)) | INVESTIGACIÓN 2026 | **~7× menos comparaciones en select_neighbors** |

### 6.2 Lo Que se Probó y NO Funcionó

| # | Propuesta | Documento Origen | Resultado | Decisión |
|---|-----------|-----------------|-----------|----------|
| 1 | Propuesta 1a: Deferred shrink (1a.4) | INDEX_REBUILD | **Regresión +53%** (2.024s → 3.106s). Listas de vecinos crecen sin límite durante rebuild paralelo → shrink final O(m²) domina. | ❌ DESCATADA |
| 2 | Propuesta 2: NN-Descent Bulk Build | INDEX_REBUILD | **Regresión catastrófica 7-1,332×**. 200v: 0.558-1.470s vs ~0.076s parallel insert. 1000v: 506s vs ~0.380s. | ❌ REVERTIDA (commit f1b9ee03) |
| 3 | Propuesta 5: Index worker thread | COMPAT_INGESTA | Complejidad alta (~200 líneas), riesgo de shutdown race. No implementada. | ⏳ DIFERIDA |
| 4 | `target-cpu=native` desactivado por codegen time | INVESTIGACIÓN 2026 | Se comentó porque aumentaba codegen time. Para benchmarks el beneficio es directo. | ✅ REACTIVADO |

### 6.3 Causas de Regresión Documentadas (Jul 2026)

| # | Causa | Impacto | Fix |
|---|-------|---------|-----|
| 1 | HnswNeighborIndex: 3 accesos DashMap vs 1 en connect_layer_neighbors | +12-27% index time | try_add_and_get_if_full() |
| 2 | CPU throttling: E-cores al 69%, SSD 8.2% libre | ~1.3-1.6× | Liberar disco, RAYON_NUM_THREADS=4, enfriar sistema |
| 3 | Procesos anómalos (UninstallMonitor, imsctadn) | ~1.05-1.15× | Matar procesos, deshabilitar scheduled tasks |
| 4 | 21 instancias VS Code compitiendo | Significativo | Cerrar instancias no esenciales |

### 6.4 Próximas Optimizaciones Pendientes (de la Investigación 2026)

Ver `docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md` para el plan detallado.

| # | Acción | Prioridad | Impacto Esperado |
|---|--------|-----------|-----------------|
| 1 | A2: get_neighbors_ref() — eliminar clones en hot path | ALTA | ~10% menos allocs |
| 2 | A4: Sweep paramétrico (M, efC, efS) | ALTA | Identificar config óptima |
| 3 | A3: Ground truth pre-computado HDF5 | MEDIA | Elimina O(n²) en benchmarks |
| 4 | B5: thread_local RNG (pendiente de profiling) | MEDIA | Elimina lock en inserts paralelos |
| 5 | B1: Extraer branches vector_store/metric del while loop | MEDIA | ~5-10% search throughput |
| 6 | B4: Prefetch batch en search_layer | MEDIA | ~10-20% en search con VantaFile |
| 7 | C2: Benchmarks multi-thread | MEDIA | Medir throughput real |
| 8 | A5: cargo-criterion + HTML reports | BAJA | Visualización de regresiones |
