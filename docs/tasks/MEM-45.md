# TASK-3: MEM-45 — Auto-sync scheduler (re-ingest programado del wiki)

## Metadata
- **Plan file:** `docs/plans/2026-08-22-vanta-final-cierre.md`
- **Creado:** 2026-08-22T00:00
- **last-synced:** 2026-08-22T00:00
- **Estado:** COMPLETED

## Blast Radius
- **Nuevo:** `vanta-memory/src/ingest/auto_sync.rs` (módulo aislado).
- **Editado:** `vanta-memory/src/ingest/mod.rs` (1 línea: `pub mod auto_sync;`).
- **Depende de:** MEM-16 `utils/managed_timer.rs` (`Clock`/`FakeClock`/`ManagedTimer`), MEM-30 `ingest::worker::run_with_progress`, MEM-28 `vantadb::wiki::WikiStore` busy guard (`WikiState::is_busy`, 409→`ExecutionConflict`), MEM-31 `ProgressTracker`/`run_id`.
- **Callers entrantes:** ninguno (scheduler pull-based; el owner llama `tick()`). No rompe nada existente.
- **NO toca:** core `vantadb` (wal/vector/storage/wiki), plan file, docs/reviews.

## Impacto mapeado (Regla 0)
- Archivos leídos completos: `utils/managed_timer.rs` (267L), `utils/timer_scanner.rs`, `ingest/mod.rs` (235L), `ingest/worker.rs` (run/run_with_progress/ingest_body), `ingest/callback.rs` (ProgressTracker), core `src/wiki/store.rs` (transiciones) + `src/wiki/state.rs` (is_busy/busy_error), TDAM `auto-sync-scheduler.ts` (328L), tests `tests/ingest.rs` (fixtures).
- Referencias hacia dentro: `vantadb::wiki::{WikiStore, scan_local_sources}`, `crate::utils::managed_timer::*`, `crate::ingest::*`.
- Referencias entrantes: ninguna (API nueva, solo tests).
- Veredicto: cambio aditivo de bajo riesgo — cero modificaciones a código existente salvo el registro del módulo.

## Contrato
"`cargo check -p vanta-memory` pasa; tests D19 con FakeClock: (a) intervalo configurable dispara re-ingest del wiki; (b) respeta busy guard (no re-ingesta si pending/processing); (c) disabled by default; (d) usa run_id fresco por build (paquetes tardíos descartados). Verify: cargo check/nextest/fmt/clippy -p vanta-memory exit 0."

## Herramientas
- codegraph, cargo (terminal), campaign MCP.

## Steps
### Step 1: DISCOVERY + task file
- **Archivos:** este task file
- **Acción:** explorar piezas MEM-16/30/28/31 + ref TDAM; mapear impacto.
- **Verify:** task file con Regla 0 completa
- **Estado:** ✅ COMPLETED

### Step 2: Implementar `auto_sync.rs` + registro en mod.rs
- **Archivos:** `vanta-memory/src/ingest/auto_sync.rs`, `vanta-memory/src/ingest/mod.rs`
- **Acción:** `AutoSyncConfig` (enabled=false default, intervalo clamp ≥60s), `AutoSyncScheduler` pull-based sobre `ManagedTimer<C>`, detección de cambios por FNV-1a por archivo (sin watcher FS, cap documentado), busy guard pre-run, re-ingest vía `worker::run_with_progress` (run_id fresco del store).
- **Verify:** `cargo check -p vanta-memory`
- **Estado:** ✅ COMPLETED

### Step 3: Tests D19 con FakeClock (a-d)
- **Archivos:** `vanta-memory/src/ingest/auto_sync.rs` (`#[cfg(test)]`)
- **Acción:** (a) intervalo dispara re-ingest al detectar cambio; (b) wiki pending → tick devuelve Busy sin re-ingestar; (c) default disabled → tick no-op; (d) dos builds → run_id distintos y paquete tardío del run viejo descartado por tracker.
- **Verify:** `cargo nextest run -p vanta-memory auto_sync`
- **Estado:** ✅ COMPLETED

### Step 4: Verify mecánico completo + cierre
- **Archivos:** task file
- **Acción:** check · nextest full · fmt --check · clippy `-D warnings`; actualizar task file; recitation.
- **Verify:** los 4 comandos exit 0
- **Estado:** ✅ COMPLETED

## Dependencias
- Tasks 1 (MEM-43) y 2 (MEM-44): ✅ completadas — piezas commiteadas.

## Notas
- D40: sobre ManagedTimer/Clock, cero threads. Stop condition respetada: watcher FS exige deps nuevas → hash periódico simple documentado (`ponytail:` cap).
- Primera pasada debida sin baseline → re-ingesta de reconciliación (documentado en código).
- Tras skip por Busy NO se actualizan hashes (el cambio se detecta en la próxima pasada).

## Context Save Point
- **Fecha:** 2026-08-22T12:00
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** pull-based `tick()` sobre ManagedTimer/Clock (cero threads); primera pasada = reconciliación (sin baseline → re-ingesta); FNV-1a stdlib sobre `scan_local_sources` (mismos guards de traversal/budget); tras skip por Busy NO se actualizan hashes; interval clamp ≥ 60s.
- **Problemas conocidos:** ninguno — verify mecánico completo exit 0 (check · nextest 460/460 · fmt · clippy -D warnings). Nota: una sesión previa dejó el task file con steps ✅ pero SIN código; esta sesión implementó Steps 2-4 realmente.
- **Próxima tarea:** Task 4 — MEM-46 (embeddings L1)
