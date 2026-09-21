# FND-01 — Regla de presupuesto de memoria (compute/storage separation) + benchmark OOM + back-pressure

**Estado:** ✅ COMPLETO (wave follow-ups: F1 wire RSS real 3f6b4c31; F2/F3 pendientes en reporte)
**Prioridad:** 🟡 (P20a)
**Fuente:** docs/Backlog.md:483
**Contrato DoD:** "regla + benchmark que la sustente"
**Archivos clave:** `src/index/hnsw.rs`, `src/storage/`, `src/metrics/`, `benches/memory_budget.rs`, `.opencode/rules/memory-budget.md`, `docs/Investigaciones/FND-01-memory-budget.md`

---

## Impacto mapeado (Regla 0)

**Archivos leídos completos (DISCOVERY):**
- `src/index/graph.rs` — `HnswNode` (línea 145): DashMap `nodes` + `vec_data: VectorRepresentations` + `neighbor_lists` — **100% RAM residente**. `estimate_memory_bytes()` disponible. Backend default `IndexBackend::InMemory`.
- `src/index/flat.rs` — `FlatIndex` (`Mutex<Vec<FlatEntry>>`, línea 63-66) — RAM residente.
- `src/storage/vfile.rs` + `vfile_mmap.rs` — vstore **mmap-backed a disco** (append vectors), `mmap_resident_bytes()` mide páginas residentes; `AlignedBytes` shim sin memmap2.
- `src/backends/` — KV backend en disco (Fjall default / RocksDB fallback / InMemory); `src/lsm.rs` paging.
- `src/storage/engine/stats.rs:98` — `check_memory_pressure()`: **GUARD EXISTENTE** — compara `rss_threshold` (default 0.80) contra `effective_bytes()` (lógico: `hnsw.estimate_memory_bytes + vstore + cache*1536`, o físico solo-mmap); sobre umbral → evicta cold nodes + `Err(ResourceLimit)` (rechaza escritura).
- `src/storage/engine/insert.rs:34` — `insert()` llama `check_memory_pressure()` al inicio; `insert.rs:303-311` volatile_cache cap (`total_memory/4/1536` nodos).
- `src/storage/engine/maintenance.rs:52` — `flush()` público → `record_memory_breakdown` (mide RSS real del proceso).
- `src/memory_governor.rs` — `MemoryGovernor` watermarks (0.75), sincronizado vía `set_used_bytes` en `check_memory_pressure`.
- `src/metrics/core/mod.rs:471` — `_get_rss_virt()` → `get_native_memory()` (Win32 QueryWorkingSetEx / Mach task_info) + sysinfo fallback; `operational_metrics_snapshot()` / `memory_breakdown_snapshot()` públicos.
- `src/config.rs` — `memory_limit: Option<u64>` (default → `HardwareCapabilities::total_memory`), `rss_threshold` default 0.80, `eviction_ratio` 0.20.
- `benches/canonical_p99.rs` — patrón benchmark (seed 42, Criterion, `common::apply_fixed_profile`).
- `benches/common/mod.rs` — `synthetic_vectors(count, dim)` determinístico (seed 0x9E37...).
- `.opencode/rules/README.md` — formato de reglas (R1-R6), índice.
- `docs/Investigaciones/FND-02-multi-index-locks.md` — formato de reporte.

**Referencias hacia dentro (quién depende de lo que voy a tocar):**
- `benches/memory_budget.rs` (NUEVO) — sin dependencias entrantes; compilado por `cargo bench -p vantadb`.
- `.opencode/rules/memory-budget.md` (NUEVO) — lazy-load vía tabla de índice en `.opencode/rules/README.md` (debo añadir fila al índice).
- `docs/Investigaciones/FND-01-memory-budget.md` (NUEVO) — referencia desde plan/backlog.
- `.opencode/skills/campaign-executor/tasks/FND-01.md` (NUEVO) — task system.

**Referencias hacia fuera (de lo que dependo):**
- Bench depende de: `vantadb::storage::StorageEngine::open` (init.rs:29, pub), `StorageEngine::insert` (pub), `StorageEngine::flush` (pub, maintenance.rs:52), `StorageEngine::get_memory_stats` (pub, stats.rs:54), `vantadb::node::UnifiedNode::with_vector` (pub), `vantadb::metrics::memory_breakdown_snapshot()` (pub), `common::synthetic_vectors`.
- Regla referencia: `src/index/graph.rs`, `src/storage/engine/stats.rs`, `src/metrics/core/mod.rs`, `src/config.rs`.

**Veredicto de impacto:** Aditivo. No modifico código core — solo creo bench nuevo (benches/), regla nueva (rules/) y reporte (docs/). `cargo check -p vantadb` inafectado; `cargo check --benches -p vantadb` valida el bench.

**Hallazgo clave:** el guard existente (`check_memory_pressure`) NO usa el RSS real del proceso — usa estimación lógica (HNSW+vstore+cache) o residente solo-mmap. RSS real (que `record_memory_breakdown` sí captura) incluye WAL buffers, backend, text index, allocator overhead. El benchmark debe cuantificar el delta.

---

## Steps

- [x] S1. Escribir `benches/memory_budget.rs` (engine full-stack en tempdir, batches crecientes de vectores 1536d, flush + muestreo RSS real vs estimación lógica, escala configurable vía env) — + entrada `[[bench]] harness=false` en Cargo.toml (requisito Criterion)
- [x] S2. Compilar bench: `cargo check --benches -p vantadb` ✅ + run real `MEMORY_BUDGET_SCALE=lite cargo bench -p vantadb --bench memory_budget` ✅ (escala reducida documentada: full proyecta 40-60 min)
- [x] S3. Análisis: **CONFIRMADO** — RSS crece ~20 KB/nodo (HNSW RAM) y el guard usa `physical_rss` mmap (54 MiB) vs RSS real (354 MiB) a 20k nodos → subestimación 6.5×, blind spot
- [x] S4. Regla normativa `.opencode/rules/memory-budget.md` (4 reglas) + fila 12 en índice README
- [x] S5. Reporte `docs/Investigaciones/FND-01-memory-budget.md` (inventario, benchmark, decisión CONFIRMADO, fix pendiente FND-01-F1)
- [x] S6. Verify final: `cargo check -p vantadb` ✅ (core intacto) + `cargo check --benches -p vantadb` ✅

## Context Save Point

Tarea completa. NO se hizo git commit (lead commitea). NO se usó campaign_update_task_state (instrucción explícita). El fix de código (FND-01-F1: wire RSS real en check_memory_pressure, `src/storage/engine/stats.rs`) queda como pendiente — fuera de alcance por read-only de src/storage/.