---
name: progreso
description: >
  Migrates completed tasks between docs/Backlog.md and
  docs/progreso/README.md, tracks doc coverage, and maintains
  cross-references across the VantaDB documentation tree.
compatibility: opencode
---

# Progreso Skill — VantaDB

## File Roles

| File | Role |
|---|---|
| `docs/Backlog.md` | Active tasks (state: ✅ or ❌ in `Status` column). Rows completed are struck-through (`~~…~~`) with a completion note — never deleted |
| `docs/progreso/README.md` | Completed task history + milestones + audits |
| `docs/progreso/BACKLOG_HISTORY.md` | Items removed from Backlog.md (historical record, keeps backlogs auditable) |
| `docs/reports/INDEX.md` | Master registry of review/audit reports — one row per report (fecha, modo, archivo, QG, findings, estado). Source of truth for Trigger 4 |
| `docs/CHANGELOG.md` | Release notes per version (keepachangelog format) |
| `docs/Investigaciones/` | Research artifacts (not tasks) |

**Invariant:** No task exists in both Backlog.md and progreso/README.md simultaneously. Items removed from Backlog.md are archived to BACKLOG_HISTORY.md, not silently dropped.

## Language split

| Language | Directories |
|---|---|
| **English** (tech source of truth) | `docs/api/`, `docs/architecture/`, `docs/operations/`, `docs/QUICKSTART.md` |
| **Spanish** (planning only) | `docs/VantaDB-MPTS/`, `docs/Backlog.md`, `docs/progreso/`, `docs/Investigaciones/`, `docs/CHANGELOG.md` (lower section) |

Spanish MPTS documents must cross-reference the English technical doc they correspond to:
`> **Referencia técnica en inglés:** \`docs/api/EMBEDDED_SDK.md\``

---

## Trigger 1: Complete a task

Run this when a task reaches ✅ in the current session.

### A. Doc impact analysis

For each modified file, verify the corresponding doc is updated:

| Modified file | Doc to verify |
|---|---|
| `src/sdk.rs` | `docs/api/EMBEDDED_SDK.md` |
| `src/config.rs` or `src/cli.rs` | `docs/operations/CONFIGURATION.md` |
| `src/error.rs` | `docs/api/EMBEDDED_SDK.md` (VantaError section) |
| `vantadb-python/src/lib.rs` | `docs/api/PYTHON_SDK.md` |
| `src/cli_server.rs` | `docs/api/HTTP_API.md` |
| `vantadb-mcp/src/` | `docs/api/MCP.md` |
| `vantadb-wasm/src/lib.rs` | `vantadb-ts/README.md` |

> **Mantenimiento:** Esta tabla debe actualizarse cuando se agreguen nuevos archivos fuente o nuevos docs. Si encontrás un archivo modificado que no está en la tabla, agregalo.

If a new technical capability was added (not just an internal bugfix), add a cross-reference from the relevant Spanish MPTS to the English doc.

### B. Extract task data

From the task you just completed: ID (e.g. `TSK-09`), name, date, objective, modified files, result.

### C. Check all task sources

Completed tasks may come from 3 sources. Check ALL:

| Source | What to do |
|--------|-----------|
| `docs/Backlog.md` | Find the ✅ row, **táchalo** (`~~…~~`) + add completion note with date. If the task is removed without completing (❌/nunca hará), move the row to `docs/progreso/BACKLOG_HISTORY.md` |
| *(bitácora legacy — migrada a plan files)* | Verificar que el issue esté marcado en el plan file activo |
| `docs/plans/YYYY-MM-DD-*.md` | Update status tracker + recitation |

### D. Migrate to progreso (sin duplicados)

1. Search `docs/progreso/README.md` — usá `grep` para localizar el ID de la tarea (no lo leas completo; son 3K+ líneas ~60K tokens). Grep primero; solo leé las secciones que coincidan.
2. Si el ID **ya existe** en progreso → skip (no duplicar). Si es información nueva (commit, fecha) → actualizá la entrada existente.
3. Si el ID **no existe**, agregá entrada en la sección correspondiente (`## Tasks Completed` para inglés, `## Tareas Completadas` para español) con:
   ```
   ### <ID>: Description
   - **Fuente:** Backlog / Bitácora / Plan
   - **Fecha:** YYYY-MM-DD
   - **Objetivo:** One-line summary
   - **Resultado:** ✅
   - **Ids:** `ID`
   ```
4. If the task was a significant milestone, also add a note under the **Executive Summary** or **Recent Progress** section.
5. If the task was a research/discovery, consider adding to `docs/Investigaciones/` instead of or in addition to progreso.

### E. Register in CHANGELOG (user-visible changes only)

Only add to `docs/CHANGELOG.md` if the task introduces a new feature, breaking change, public bugfix, new CLI command, etc. NOT every individual task.

### F. Validate doc coverage

```pwsh
pwsh scripts/validate-docs-coverage.ps1
```

If it reports gaps, document the missing surface before proceeding.

### G. Notify

Tell the user that Backlog.md, plan file and progreso/README.md were updated and validation passed. Commit policy:
- **Standalone** (no campaign-executor): no commit — esperar instrucción
- **Desde campaign-executor**: el executor maneja commits automáticos (el progreso no hace commit directo)
- Si aplica, registrar decisión: `campaign_memory_write(file="decisions", entry="progreso: migración de <ID> completada")`

---

## Trigger 2: Start a new task

Before generating a new plan:

1. Grep `docs/progreso/README.md` para el ID de la tarea previa (no leer completo — 3K+ líneas ~60K tokens). Si no está → correr **Trigger 1** primero.
2. Find the task in `docs/Backlog.md` or the active plan file (`docs/plans/`). If status is ❌, change it to 🟡 (or leave it and update after completion).
3. Proceed with the new work.

---

## Trigger 3: Monthly/fase maintenance

1. Backlog: move tasks inactive >30 days to ⏸️ Icebox or ❌ No Hacer (archiving removed rows to `docs/progreso/BACKLOG_HISTORY.md`).
2. progreso: deduplicate entries, fix stale cross-links.
3. Investigaciones: verify index matches actual files, prune orphans.
4. Cross-check: no task exists in both Backlog.md and progreso/README.md.

---

## Trigger 4: Sync reportes (review/audit reports ↔ backlog)

Run this **at session start** (alongside the reading of Backlog.md) and **after
any `/review` or `/audit` run**. Reports in `docs/reviews/` and
`docs/audit-reports/` are artifacts — they only have value when their findings
are tracked.

### A. Check the registry

1. If `docs/reports/INDEX.md` exists: read it.
2. If it's missing, list `docs/reviews/*.md` and `docs/audit-reports/*.md` and
   flag that the registry hasn't been created (first sync creates it).

### B. Flag orphans

For every report file whose `YYYYMMDD-HHMMSS` is **newer** than its INDEX.md row
(or that has no row):

- Add the row to `docs/reports/INDEX.md` (fecha | modo | archivo | QG | C/H/M/L/I | estado | resumen).
- Check `docs/Backlog.md` for a `## Hallazgos pendientes de reportes` section.
  If the report has findings ≥ medium and no backlog row references them
  (`REVIEW-NN` / `AUD-NN` derived from that report), add them. Otherwise append
  `✔ findings ya se siguen en <IDs>` to the INDEX row.

### C. Report to user

Summarize: "N reportes en INDEX, M reportes nuevos sincronizados, K hallazgos
pendientes derivados al backlog". Never silently skip a report that is newer
than its registry entry — that's the exact desync this trigger exists to catch.

---

## Definition of Done (pre-commit checklist)

Ver el standing quality bar en [`.opencode/references/definition-of-done.md`](../../references/definition-of-done.md) — aplica para releases y cambios del sistema.

Para tareas de código (referencia rápida):
- [ ] Compiles (`cargo check --workspace` or `cargo nextest run --no-run`)
- [ ] Tests pass (`cargo nextest run --profile audit --workspace --build-jobs 2`)
- [ ] Affected docs updated (see Trigger 1.A table)
- [ ] MPTS cross-reference added if new technical feature
- [ ] `scripts/validate-docs-coverage.ps1` passes clean
- [ ] **Certify gate recomendado:** `skill unified-review --mode certify --profile vantadb` para validación completa pre-push
  - Si no es posible (cambio chico): mínimo `just verify-quick`

## Campaign Memory Integration

Al completar una tarea, registrar la decisión si es relevante:

```python
# Registrar migración de tarea
campaign_memory_write(
    file="decisions",
    entry="progreso: migrada <ID> de Backlog a progreso. Archivos tocados: <paths>"
)

# Si fue una decisión arquitectónica
campaign_memory_write(
    file="decisions",
    entry="progreso: <ID> implicó tradeoff entre <X> y <Y>. Se eligió <X> por <razón>"
)
```
