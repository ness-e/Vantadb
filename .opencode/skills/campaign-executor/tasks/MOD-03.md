# MOD-03 — `trigger_compaction()` es stub que solo loguea

> Plan: docs/plans/2026-08-23-backlog-triage.md · Task 14 · 🟡 · max 1h
> Fuente: docs/reviews/modulos/core.md M-1

## Contrato

Sin método público que solo loguee: o delega a la operación real (compact/vacuum)
o deprecated con doc honesta apuntando a la vía real. Suite verde.

## Discovery (evidencia)

1. **Stub confirmado** — `src/storage/engine/maintenance.rs:22-48`: cuenta
   tombstones iterando headers, loguea warn si >20% hardcodeado, retorna
   `Ok(())`. Nunca compacta. El log dice "offline compaction triggered".
2. **Callers** (`rg -n trigger_compaction`): solo 5 tests unitarios en
   `src/storage/engine/tests/maintenance.rs:346-403`. Cero callers de
   producción; NO está expuesto en `src/sdk/api.rs`, ni bindings, ni MCP.
   → Nadie depende del no-op barato.
3. **Operación real existe** — `merge_segments()` (`maintenance.rs:868`):
   mide fragmentación contra `config.segment_optimizer.vacuum_threshold_pct`
   (default 15.0) y delega a `compact_layout_bfs()` (`maintenance.rs:584`),
   que reescribe el VantaFile en orden BFS salteando tombstones
   (`src/storage/archive.rs::compact_layout`, reclaim real de bytes).
   Ya usada por `run_pipeline`.

### Decisión: DELEGAR (no deprecar)

- No hay callers que esperen un no-op barato → la objeción "compaction es
  costosa" no aplica a nadie real.
- Delegar hace que el nombre diga la verdad ("trigger compaction" dispara
  compactación) — recomendación ideal del review (core.md R3/M-1) sobre el
  renombre mínimo.
- API pública estable (`Result<()>` preservado): cero semver impact.
- Threshold pasa de 20% hardcodeado al configurado — más consistente con el
  resto del sistema (`merge_segments`, CONFIGURATION.md).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** maintenance.rs (imports, trigger_compaction
  22-48, compact_layout_bfs 584-643, vacuum 780-860, merge_segments 868-962),
  archive.rs (compact_layout, traverse_graph, reindex_nodes), tests/mod.rs
  completo (fixtures), tests/maintenance.rs 1-100 + 300-419, init.rs 1-90,
  antilocality_layout.rs 1-75 (patrón disk-backed engine), mod.rs VacuumReport.
- **Referencias entrantes:** 5 tests (`test_trigger_compaction_*`) — todos
  `expect Ok(())`; delegación los mantiene verdes. Docs: glosario/compaction.md
  (snippet del stub + tabla), reviews (solo lectura histórica).
- **Referencias salientes:** `FLAG_TOMBSTONE`, `tracing`, imports existentes —
  tras delegar, el body ya no usa ninguno (imports quedan para otros fns del
  mismo archivo).
- **Veredicto:** cambio contenido a 2 archivos (+1 doc glossary). Blast radius
  bajo; sin Gate D.

## Steps

- ✅ S1 (RED): test `test_trigger_compaction_reclaims_disk_space_on_high_fragmentation`
  — falló contra stub (`before=67108864 after=67108864`), confirmando el no-op.
- ✅ S2 (GREEN): body delegado a `merge_segments()`; 6/6 tests trigger_compaction
  verdes; suite completa `cargo nextest run -p vantadb` 2052/2052 ✅.
- ✅ S3 (cierre): glosario sincronizado; fmt+clippy -D warnings OK;
  commit `28ce0a57`.

## Context Save Point

- **Colateral NO commiteado:** `tests/cli_tests.rs:1185` tenía llamada a
  `cmd_server` con 9 args contra firma de 10 (`allow_insecure` nuevo). Fix
  aplicado en worktree pero NO commiteado: la firma de 10 args es trabajo sin
  commit de otra tarea (src/cli_handlers/server.rs modificado en worktree).
  En HEAD ambos lados siguen consistentes (9 args). El fix viaja con la tarea
  que commitea server.rs.
- **Hallazgo para Backlog (FIND-*):** `merge_segments()`/`vacuum()` cuentan
  tombstones iterando `hnsw.nodes`, pero `delete()` normal hace
  `remove_hnsw_entry()` → headers tombstoned huérfanos en el VantaFile son
  invisibles al threshold-check (solo compact_layout_bfs los reclama si se
  invoca directo). Decisión de diseño (escanear vfile vs índice) es territorio
  Engine/Arch — fila FIND candidata.
