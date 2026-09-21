# TBH-08 — benches/wal_throughput.rs

## Metadata
- **Plan file:** docs/plans/2026-08-30-testing-bench-harden.md
- **Created:** 2026-08-31T12:35
- **last-synced:** 2026-08-31T12:35
- **Estado:** ✅ COMPLETED
- **Tipo (campaign_detect_task_type):** bench / new-criterion-bench (1 new bench file + Cargo.toml block)
- **Esfuerzo:** 🟡 Medio
- **Prioridad:** MEDIA (Phase 2, gap crítico de benchmarking)

## Impacto mapeado (Regla 0)

### Archivos leídos completos
| Archivo | Líneas | Notas |
|---------|--------|-------|
| `benches/common/mod.rs` | 102 | `synthetic_vectors(count, dim)` → Vec<Vec<f32>> determinístico; `apply_fixed_profile(group)` = warm_up 3s + measure 5s + 0.95 CI. **No se usa `synthetic_vectors`** — bench de WAL mide I/O, no HNSW (un record `UnifiedNode::new(id)` basta para el sweep). |
| `benches/common/gen_dataset.rs` | 31 | Standalone regenerador del dataset binario. **No se modifica.** |
| `Cargo.toml` (workspace) | 680 | Líneas 192-262: 18 `[[bench]] harness = false`. Líneas 170-180: `[dev-dependencies]` con `criterion = "0.8"` (features `html_reports`, `async_tokio`), `tempfile = "3"`. |
| `src/wal.rs` | 1373 | API pública expuesta: `WalWriter::{open, open_with_buffer, append, batch_append, sync, rotate}`, `WalRecord::{Insert, Update, Delete, Checkpoint, Begin, Prepare, Commit, Abort}`. `SyncMode::{Always, Periodic, Never}`. |
| `src/config.rs` | 1772 | `SyncMode` (líneas 86-103) y `VantaConfig` builder `with_sync_mode(SyncMode)`. `SyncMode` NO está re-exportado en `lib.rs` PERO `crate::config` es `pub mod` → benches pueden usar `vantadb::config::SyncMode`. Verificado: `src/bin/crash_helper.rs:3` ya lo usa. |
| `src/lib.rs` | ~250 | Líneas 138 `pub mod wal`, 74 `pub mod config`, 184 `pub use wal::{WalReader, WalRecord, WalWriter}`. |
| `benches/hnsw_pure.rs` | 101 | Patrón criterion de referencia (`criterion_group!` + `criterion_main!` + `sample_size` + `apply_fixed_profile`). |
| `benches/canonical_p99.rs` | 133 | Patrón criterion con `iter_custom` + histogram percentile (p50/p95/p99) impreso vía `println!`. **Inspiración para p99 latency report.** |
| `benches/stress_test.rs` | 61 | Patrón para uso de `tempfile::tempdir()` + `StorageEngine::open`. **No se usa `StorageEngine`** — el bench es a nivel WAL crudo. |

### Referencias hacia adentro (outbound references de los archivos tocados)
- `benches/wal_throughput.rs` → usa `vantadb::wal::{WalWriter, WalRecord}`, `vantadb::config::SyncMode`, `vantadb::node::UnifiedNode` (todos públicos per lib.rs).
- `Cargo.toml` → añade `[[bench]] name = "wal_throughput" harness = false`. **Patrón exacto** de las 18 benches existentes (líneas 192-262).

### Referencias hacia afuera (inbound — quién lee o importa estos archivos)
- `benches/wal_throughput.rs` → será invocado por `cargo bench -p vantadb --bench wal_throughput` y (futuro) `.github/workflows/heavy-bench-nightly-51.yml` cuando TASK-11 lo agregue al matrix.
- `Cargo.toml` → ya consumido por `cargo build/check/bench -p vantadb` (CI fast gate + nightly).

### Veredicto de impacto
**Mínimo, blast radius = 0 archivos aguas abajo.** Cambios son:
1. Nuevo archivo `benches/wal_throughput.rs` (1 file nuevo).
2. 1 `[[bench]]` block nuevo en `Cargo.toml` (1 línea agregada).
3. Ningún archivo existente modificado (Cargo.lock se actualiza solo — añadido por `cargo build`, NO editado a mano).

### SDP (Skill Discovery Protocol)
- `campaign-executor` (base, auto-cargada via MCP) — orquestación de la tarea.
- `incremental-implementation` (regla 6 del agente worker) — slice vertical delgado (1 archivo + 1 línea TOML).
- `test-driven-development` — el bench es el "test" (mide comportamiento); cumple el rol de regression detector.
- `code-simplification` (ponytail full) — bench mínimo, sin sobreingeniería.
- `campaign-load-skills base + 2 lifecycle mappings` (BUILD + VERIFY) ya cubre este caso.

### Gates
- **P (pre-flight):** no requerido (no toca WAL/storage/vector — propiedad de vanta-arch/vanta-engine).
- **D (discovery):** disparado — leído WAL API + benches reference + Cargo.toml + common helpers; veredicto arriba.
- **V (verify):** contrato mecánico (5 checks), ver `## Contrato`.
- **C (commit):** conventional commit `feat(TBH-08):`.

## Contexto

`incremental_bench.rs` usa `skip_wal: true` en 5 sitios (líneas 99, 123, 146, 178, 193). La auditoría multi-agente del 2026-08-30 (gap audit benchmarks) identificó esto como gap crítico: **no podemos saber si un cambio al WAL degrada performance hasta que se mide**. Prioridad MEDIA en el plan.

El bench cierra el gap midiendo el **WAL crudo** (no el engine completo): throughput (ops/sec) y p50/p95/p99 latencias sobre un sweep configurable.

## Contrato (verificable mecánicamente)

```
1. benches/wal_throughput.rs existe y compila
2. benches/wal_throughput.rs contiene `criterion_group!` + `criterion_main!` válidos
3. Cargo.toml tiene `[[bench]] name = "wal_throughput" harness = false`
4. cargo check -p vantadb --benches → exit 0
5. cargo build -p vantadb --benches → exit 0
6. cargo fmt --check → exit 0
7. Sweep del bench cubre: SyncMode x batch sizes [1, 100, 1k, 10k]
8. Bench usa Throughput::Elements para reportar ops/sec
```

## Spec del bench

| Dimensión | Valores |
|-----------|---------|
| Sync mode | `Always` (fsync por write) × `Periodic` (default) × `Never` (sin fsync) |
| Batch size (records/iter) | 1, 100, 1_000, 10_000 |
| Records por iter (total) | 10_000 (constante, así ops/sec es comparable) |
| Record payload | `WalRecord::Insert(UnifiedNode::new(id_counter))` — nodo vacío, ~24 bytes serializado |
| Temp dir | `tempfile::tempdir()` por `iter_custom` |
| Métrica primary | `criterion::Throughput::Elements(N)` → ops/sec |
| Métrica latency | `iter_custom` mide wall-time total / N → p50/p95/p99 vía `canonical_p99.rs::percentile` pattern |
| Profile | `common::apply_fixed_profile` (warm 3s, measure 5s, 0.95 CI) |
| `sample_size` | 10 (matches `hnsw_pure.rs`) |

## Archivos a crear/modificar

| Archivo | Acción |
|---------|--------|
| `benches/wal_throughput.rs` | CREAR |
| `Cargo.toml` (workspace) | AÑADIR 1 `[[bench]]` block al final de la lista (línea ~263, después de `memory_budget`) |
| `Cargo.lock` | regenerado por `cargo build` |

## Steps

### Step 1: Crear task file ✅
- **Acción:** este archivo. **Estado:** ✅

### Step 2: PLAN — diseñar sweep ✅
- **Acción:** spec arriba (3 SyncMode × 4 batch sizes = 12 muestras; mas `iter_custom` recrea tempdir cada iteración para que fsync separe correcto). **Estado:** ✅

### Step 3: ACT — crear `benches/wal_throughput.rs` ⬜ PENDING
- **Acción:** escribir bench mínimo que cumpla contrato.
- **Ponytail reflex:** bench mide lo que dice medir. NO medir 20 variantes (skip sharded WAL, skip checkpont, skip transactions — esos viven en benches dedicados o en `wal.rs` tests). El sweep es exactamente lo que el plan especifica.
- **Verify:** step 4.

### Step 4: ACT — añadir `[[bench]]` en Cargo.toml ⬜ PENDING
- **Acción:** Edit tool. 1 línea al final del bloque de benches.
- **Verify:** `grep "name = \"wal_throughput\"" Cargo.toml` → 1 match.

### Step 5: VERIFY ⬜ PENDING
- `cargo check -p vantadb --benches`
- `cargo build -p vantadb --benches`
- `cargo fmt --check`

### Step 6: COMMIT ⬜ PENDING
- `git add benches/wal_throughput.rs Cargo.toml Cargo.lock .opencode/skills/campaign-executor/tasks/TBH-08.md`
- `git commit -m "feat(TBH-08): add wal_throughput bench (sweep WAL x fsync x batch sizes)"`

## Dependencias

- Ninguna. TBH-08 es independiente de las otras tareas activas del plan.
- **Pre-existente no relacionado:** `vantadb-mcp/tests/context_tests.rs:70` no compila (FIND-MCP-001). Usar `-p vantadb` evita el bug.

## Notas

- **Ponytail reflex:**
  - NO usar `StorageEngine` (overhead engine enmascara WAL puro — el plan pide "WAL throughput").
  - NO medir checkpoint / transactions (bench dedicado en otra tarea; keep simple).
  - NO medir sharded WAL (ya cubierto por `wal_sharded.rs` tests internos; sweep crece sin valor).
  - `iter_custom` (no `iter`) — recrea tempdir cada iteración así el archivo se crea fresh y el fsync se mide de verdad (sino la 2da iter arranca sobre archivo existente con header escrito y los números no son comparables).
  - `sample_size(10)` igual que `hnsw_pure.rs` / `canonical_p99.rs` (3s warm × 5s measure × 10 samples = ~80s por muestra × 12 muestras = ~16min bench; aceptable, pero se puede bajar a `sample_size(5)` si el wall time explota — evaluar post-first-run).
- **Self-check (post-create):** la primera ejecución del bench debe poder ejecutarse sin errores. Si falla el bench, la regresión es detectable.
- **Per-rule Regla 9:** este bench ES la medición que faltaba. Cumple Regla 9 (No optimizar sin medir) cerrando el agujero de medición.

## Context Save Point
- **Fecha:** 2026-08-31
- **Branch:** develop
- **CI pendiente:** no (cambio aditivo, no toca WAL/storage/vector ni core types)
- **Decisiones:**
  - Sweep: 3 SyncMode × 4 batch sizes = 12 muestras (no más, no menos — spec del plan).
  - Payload: `WalRecord::Insert(UnifiedNode::new(id))` (nodo vacío, ~24 bytes serializado — representativo del write-amplification mínimo del WAL).
  - Total records/iter: 10_000 (constante → Throughput::Elements comparable).
  - `iter_custom` (no `iter`) — recrea tempdir cada iter (necesario para medir fsync cold).
  - NO usar `StorageEngine` (overhead engine enmascara WAL puro).
- **Próxima tarea:** handoff al orquestador.