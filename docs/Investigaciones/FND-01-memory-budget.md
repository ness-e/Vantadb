# FND-01: Regla de presupuesto de memoria (compute/storage separation) + benchmark OOM + back-pressure

**Estado:** ✅ Resuelto (benchmark + regla normativa; fix de código documentado como pendiente)
**Fecha:** 2026-08-16
**Prioridad:** 🟡 (P20a)
**Fuente:** docs/Backlog.md:483
**Contrato DoD:** "regla + benchmark que la sustente" — ✅ regla en `.opencode/rules/memory-budget.md` + benchmark `benches/memory_budget.rs`
**Alcance:** `src/index/` (lectura), `src/storage/` (solo lectura DISCOVERY), `src/metrics/` (solo lectura), `benches/` (bench NUEVO), `.opencode/rules/` (regla NUEVA), `docs/Investigaciones/`
**Archivos tocados:** `benches/memory_budget.rs` (nuevo), `Cargo.toml` (entrada `[[bench]]`, ver §7), `.opencode/rules/memory-budget.md` (nuevo), `.opencode/rules/README.md` (índice), `docs/Investigaciones/FND-01-memory-budget.md` (este reporte)

---

## 1. Inventario RAM vs disco (archivo:línea)

| Componente | Residencia | Ubicación |
|---|---|---|
| **HNSW graph** — `HnswNode` (id, bitset, `vec_data`, `neighbor_lists`) en `DashMap nodes` | **RAM (100%)** — heap | `src/index/graph.rs:145-158` (default `IndexBackend::InMemory`, graph.rs:419) |
| **FlatIndex** — `Mutex<Vec<FlatEntry>>` | **RAM** | `src/index/flat.rs:63-66` |
| **VStore (vector store)** — append de vectores a archivo, mmap | **Disco (mmap)** — páginas residentes medidas | `src/storage/vfile.rs`, `vfile_mmap.rs` (`mmap_resident_bytes()`, `get_resident_bytes` vfile_mmap.rs:225) |
| **KV backend** — Fjall (default) / RocksDB / InMemory, LSM | **Disco (LSM paginado)** | `src/backends/` (`fjall_backend.rs`, `rocksdb_backend.rs`, `in_memory.rs`), `src/lsm.rs` |
| **Volatile hot-node cache** | **RAM con cap** — `total_memory/4/1536` nodos | `src/storage/engine/insert.rs:303-311` |
| **WAL** | **Disco** (sharded) | `src/wal.rs` |
| **Cardinality stats / edge / scalar index** | **RAM con cap** — `MAX_CARDINALITY_PAIRS` | `src/config.rs`, `src/storage/engine/insert.rs` |
| **CacheWarmer co-access** | **RAM con cap** — `MAX_CO_ACCESS_PAIRS` (1M) | `src/cache_warmer.rs:30` |

**Respuesta a la pregunta del backlog:** HNSW es **siempre RAM** (config `mmap_hnsw=true` hace zero-copy del vector desde el vstore mmap, pero el grafo + neighbor lists viven en heap); el LSM del KV backend **sí pagina a disco**; el vstore es mmap. Existe guard de RSS (`check_memory_pressure`, stats.rs:98, llamado al inicio de `insert()` en insert.rs:34) y `MemoryGovernor` (src/memory_governor.rs) con watermarks.

## 2. Benchmark de memoria (OOM bajo carga)

**Archivo:** `benches/memory_budget.rs` — engine full-stack (`StorageEngine::open` en tempdir, backend Fjall), batches crecientes de vectores 1536d, tras cada batch: 10k reads (mix de lectura) + `flush()` (dispara `record_memory_breakdown` → RSS real del proceso) + muestreo de RSS real vs estimación lógica del guard.

**Comando:**
```
MEMORY_BUDGET_SCALE=lite cargo bench -p vantadb --bench memory_budget
```
**Entorno:** Windows 11 (win32), 12 cores AVX2, RAM 31.78 GiB (34,120,724,480 B), bench profile (opt + debuginfo). Dataset determinístico (seed 0x9E37... de `benches/common`).

**Escala documentada (reducida):** el run full `[10k, 25k, 50k, 100k]` proyecta ~40-60 min (la tasa de insert degrada con el tamaño del grafo: el batch a 20k acumuló 258s). Se corrió `lite` `[5k, 10k, 20k]` — lo importante es la **tendencia** RSS vs dataset, no el absoluto.

**Resultados (run limpio, 2 runs con ruido ±5%):**

| nodos | insert (s) | RSS real (MiB) | estimación lógica guard (MiB) | `physical_rss` guard (mmap, MiB) | pressure_ratio |
|---|---|---|---|---|---|
| 5,000 | 128.2 | 101.59 | 289.10 | 0 | 0.000 |
| 10,000 | 146.8 | 158.19 | 322.20 | 0 | 0.000 |
| 20,000 | 258.1 | 354.02 | 452.40 | 54.41 | 0.002 |

**Tendencia RSS vs dataset:** slope 5k→10k ≈ 11.6 KB/nodo; slope 10k→20k ≈ 20.0 KB/nodo (densificación del grafo HNSW: neighbor lists). Slope conservador de diseño: **~20 KB/nodo** (1536d).

**Extrapolación (máquina de referencia, RAM 31.78 GiB):**

| nodos | RSS proyectado | % RAM |
|---|---|---|
| 100k | ~2.0 GB | ~6% |
| 500k | ~10 GB | ~31% |
| 1M | ~20 GB | ~63% |
| **~1.6M** | **~31.8 GB** | **~100% → OOM** |

## 3. Análisis — clasificación: ✅ **CONFIRMADO**

**El riesgo OOM es real y el guard existente no lo detecta:**

1. **RSS crece sin límite con el dataset** — HNSW 100% RAM residente, slope ~20 KB/nodo. 1M nodos ≈ 20 GB; ~1.6M nodos ≈ RAM total → el SO mata el proceso.
2. **Blind spot del guard (hallazgo crítico):** `check_memory_pressure` usa `effective_bytes()` = `physical_rss.unwrap_or(logical)`. Con `mmap_hnsw=true` (default), `physical_rss` = suma de `mmap_resident_bytes()` de vstores + backend HNSW. A 20k nodos: **54.41 MiB** contra **354.02 MiB de RSS real** → subestimación **~6.5×**. El guard cree que hay 0.2% de presión cuando el proceso usa 1.1% de la RAM y crece lineal.
3. **El RSS real ya se mide pero no se usa:** `flush()` → `record_memory_breakdown` (`src/metrics/core/mod.rs:402`) captura el RSS real del proceso (`_get_rss_virt`, mod.rs:471 — Win32 QueryWorkingSetEx / Mach task_info / sysinfo), expuesto en `operational_metrics_snapshot().process_rss_bytes`. Pero `check_memory_pressure` no lo consulta.
4. La estimación lógica (`hnsw.estimate_memory_bytes + vstore + cache`) es **conservadora en escala chica** (289 MiB vs 102 MiB a 5k) pero nunca se usa cuando hay mmap-residente presente — justamente cuando más importa.

## 4. Decisión

- **Guard existe pero es ineficaz para el caso real.** El fix correcto NO es "agregar un guard" (ya existe) sino **alimentar `check_memory_pressure` con el RSS real del proceso** (usar `memory_breakdown_snapshot().process_rss_bytes` con fallback a la estimación lógica) — cambio de ~10 líneas en `src/storage/engine/stats.rs`.
- **Fuera de alcance de FND-01:** `src/storage/` es solo lectura para DISCOVERY → el fix se documenta como pendiente (§6). No se implementa back-pressure nuevo sin spec (instrucción de la tarea: "NO rediseño grande de back-pressure sin spec; documentá el gap").
- **Salida normativa:** regla `.opencode/rules/memory-budget.md` (nueva área) — inventario RAM/disco, RSS real como señal de presión (must/must-not), back-pressure antes de OOM, caps obligatorios para estructuras residentes nuevas.

## 5. Regla

`.opencode/rules/memory-budget.md` (Status: 🟡 En revisión) — 4 reglas:
1. Conocer qué vive en RAM y qué en disco (inventario normativo).
2. Medir el RSS real del proceso, no estimaciones parciales (must-not: `physical_rss` mmap como única señal; subestimación 6.5× medida).
3. Back-pressure antes de que el SO mate el proceso (rechazo con `VantaError::ResourceLimit` sobre `rss_threshold`).
4. Toda estructura residente nueva lleva un cap declarado (patrón: volatile cache / co-access / cardinality).

Índice actualizado en `.opencode/rules/README.md` (fila 12).

## 6. Fixes / pendientes

| ID | Qué | Archivo | Esfuerzo |
|---|---|---|---|
| **FND-01-F1** | Wire del RSS real en `check_memory_pressure`: `effective` debe ser `memory_breakdown_snapshot().process_rss_bytes` (o fallback lógico), no `physical_rss` mmap. Requiere spec mínima + test de `pressure_ratio` con RSS (patrón ya existente en `src/storage/engine/tests/stats.rs`) | `src/storage/engine/stats.rs` (~10 líneas) | 🟢 1 hr |
| FND-01-F2 | Re-evaluar `DEFAULT_RSS_THRESHOLD=0.80` vs máquina real: con RSS real como señal, 0.80 de 31.78 GiB deja ~6.3 GiB de margen al SO; validar que la evicción (eviction_ratio 0.20) alcanza antes de rechazar | `src/config.rs` | 🟢 30 min |
| FND-01-F3 | Correr escala full `[10k,25k,50k,100k]` (≈40-60 min) en CI heavy o máquina de referencia para fijar baseline absoluto | `benches/memory_budget.rs` | 🟡 2 hr |
| FND-01-F4 | Señal de reapertura: si F1 no se implementa, re-evaluar al llegar a ~500k nodos / 10 GB RSS en producción — el guard actual no protegerá | — | — |

## 7. Notas

- **Desviación de alcance documentada:** `Cargo.toml` recibió la entrada `[[bench]] name = "memory_budget" harness = false` (línea ~254). Es requisito del harness Criterion (todos los benches del repo la tienen; sin `harness=false` el bench corre en modo test y nunca ejecuta el workload). Cambio aditivo, no toca código.
- **Verificación mecánica:** `cargo check --benches -p vantadb` ✅ (bench compila, sin warnings propios); `cargo check -p vantadb` ✅ (código core intacto). No se corrió el verify full (fmt/clippy/nextest) porque no se tocó código core y el lead commitea.
- Los warnings de `src/sdk/search/debug_ops.rs` (5 unused imports) son pre-existentes, fuera de alcance.