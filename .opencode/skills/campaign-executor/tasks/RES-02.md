# RES-02 — backup/restore físico completo (research)

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-fixes-research.md (Task 8)
- **Creado:** 2026-08-25
- **Estado:** ✅ COMPLETED — research completo, doc persistido por el lead

## Impacto mapeado (Regla 0)
Research read-only: leídos storage/engine/mod.rs, sdk/builder.rs, vantadb-mcp/tools.rs, wal_archiver.rs, tests/fjall_cold_copy_restore.rs, serialization/impl_export.rs. Cero código tocado. Output: docs/research/res02-backup-restore.md.

## Steps
### Step 1: Estado actual verificado (file:line) ✅
create_snapshot ×2 variantes, list_snapshots, snapshot_restore NO existe (0 ocurrencias), PITR dead code.
### Step 2: Alternativas comparadas ✅
(a) restore físico swap-dir RECOMENDADO · (b) PITR DEFER (dead code + sin wiring) · (c) lógico export/import YA EXISTE.
### Step 3: Plan S1-S5 + hallazgos ruteados ✅
S1 quiesce+flush create_snapshot · S2 core snapshot_restore · S3 SDK+docs · S4 tests · S5 CLI/MCP MCP-34b. Ruteado: MCP-34b, FIND-25, FIND-26.

## Context Save Point
- **Fecha:** 2026-08-25
- **Artefacto:** docs/research/res02-backup-restore.md
- **Ruteado al Backlog:** MCP-34b, FIND-25, FIND-26
- **Próxima tarea:** cierre de campaña (lead)
