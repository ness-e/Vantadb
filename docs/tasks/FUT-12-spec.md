# Task FUT-12-spec — spec WAL fsync-batching opt-in (SOLO SPEC)

> **Plan:** `docs/plans/2026-09-04-durability-release-readiness.md` Task 9, Wave 2 · **Ruta:** vanta-arch · **Branch:** develop
> **Contrato:** spec escrita con objetivo / diseño group-commit / aceptación (≥10× + ventana declarada y testeable) / límites (default intacto) + registrada para futura implementación. **Cero código productivo.**
> **Decisiones usuario YA tomadas (NO re-preguntar):** batching 100% opt-in (default intacto) · group-commit ventana tiempo/tamaño reutilizando `batch_append` · aceptación ≥10× batch + ventana de pérdida declarada y testeable.

## SDP

`SDP: campaign-executor, progreso, ponytail, writing-guidelines, writing-plans, spec-driven-development, interview-me, idea-refine (phase DEFINE) + vanta-arch core: documentation-and-adrs, api-and-interface-design, database-design (arch §6 obligatorio; interview-me/idea-refine no se ejercen — decisiones ya tomadas)`

## Spec (pre-llenada — decisiones del usuario, no re-preguntar)

| # | Decisión | Opción elegida (usuario) | Evidencia |
|---|----------|--------------------------|-----------|
| 1 | Opt-in vs default | 100% opt-in, default intacto | Plan Task 9 + Gate P 2026-09-04 |
| 2 | Mecanismo | group-commit ventana tiempo/tamaño, reutiliza `batch_append` | Plan Task 9 |
| 3 | Aceptación | ≥10× batch + ventana de pérdida declarada y testeable | Plan Task 9 |
| 4 | Ubicación spec | `docs/architecture/adr/ADR-038-*` (hay 44 ADRs, storage ADRs existen: 001/002/004/020/022/DRV-014/015/COMP-026 → convención ADR, no artifacts) | `ls docs/architecture/adr` |

## Impacto mapeado (Regla 0 — spec-only, sin edits en src/)

- **Leídos:** `src/wal.rs:305-394` (`append`, `batch_append`, `maybe_sync` Always|Never|Periodic, `sync` = flush+sync_data) · `src/wal_sharded.rs:198-235` (`batch_append` agrupa por shard, 1 lock+1 write_all+1 maybe_sync por shard, move sin clones) · `src/config.rs:85-103` (`SyncMode` Always/Periodic-default/Never) · `benches/wal_throughput.rs` (rig 3 SyncMode × 4 batch sizes, base de medición) · `ADR-022` (consolida DRV-014/015; Phase 1 group-commit **no implementado**) · `DRV-014` (batch-append 3-5× tradeoff) · `DRV-015` (roadmap Phase 1 group-commit + Phase 2 io_uring, riesgos R1/R2/R5) · `docs/_templates/adr.md`.
- **Referencias hacia dentro:** `WalWriter::batch_append` ← `ShardedWal::batch_append` ← `insert.rs`/`txn.rs`/`delete.rs`; `maybe_sync` ← `append`/`batch_append`; `SyncMode` ← wal_sharded/wal_shipping/config/benches.
- **Referencias entrantes:** `engine.rs` → `ShardedWal`; `storage/wal.rs` → `ShardedWal`.
- **Veredicto:** SPEC-ONLY → impacto runtime 0. Futura implementación tocará `wal.rs` + `wal_sharded.rs` + `config.rs` + `benches/` + `wal_resilience`/`chaos_integrity`. Esta tarea NO toca `src/` (prohibido por contrato).

## Steps

- [x] **S1 — DISCOVERY:** leer wal/config/ADRs/bench, decidir ubicación ADR-038, crear este task file.
- [x] **S2 — EJECUCIÓN:** redactar `docs/architecture/adr/ADR-038-wal-fsync-batching-opt-in.md` (objetivo/diseño/aceptación/límites).
- [x] **S3 — CIERRE:** verify docs-only (`git status` solo 2 files, `scripts/validate-docs-coverage.ps1` si aplica) + commit `docs(adr): spec WAL fsync-batching opt-in (FUT-12-spec)` + Task 9 → COMPLETO en plan file (sin stagear plan de más) + 1-2 lessons + RESULTADO.

## Gates evaluados

- P: no (decisiones usuario ya tomadas, scope spec-only acotado) · D: no (blast radius 0 runtime, contrato del plan es ley) · V: no (sin verify de código; verify docs-only en S3) · C: pendiente en S3 (colaterales: ninguno previsto; si `git status` muestra ajenos — `.opencode` submodule modificado pre-existente — NO stagear).
