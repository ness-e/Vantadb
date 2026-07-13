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
| `docs/Backlog.md` | Active tasks (state: ✅ or ❌ in `Status` column) |
| `docs/progreso/README.md` | Completed task history + milestones + audits |
| `docs/CHANGELOG.md` | Release notes per version (keepachangelog format) |
| `docs/Investigaciones/` | Research artifacts (not tasks) |
| `docs/Investigaciones.md` | Index of research artifacts |

**Invariant:** No task exists in both Backlog.md and progreso/README.md simultaneously.

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

If a new technical capability was added (not just an internal bugfix), add a cross-reference from the relevant Spanish MPTS to the English doc.

### B. Extract task data

From the task you just completed: ID (e.g. `TSK-09`), name, date, objective, modified files, result.

### C. Check all task sources

Completed tasks may come from 3 sources. Check ALL:

| Source | What to do |
|--------|-----------|
| `docs/Backlog.md` | Find the ✅ row, delete it |
| `docs/bitacora.md` | Mark the issue as resuelto ✅ |
| `docs/plans/YYYY-MM-DD-*.md` | Update status tracker + recitation |

### D. Migrate to progreso (sin duplicados)

1. Read `docs/progreso/README.md` — buscá el ID de la tarea en todas las secciones.
2. Si el ID **ya existe** en progreso → skip (no duplicar). Si es información nueva (commit, fecha) → actualizá la entrada existente.
3. Si el ID **no existe**, agregá entrada en **`## Tareas Completadas`** (sección según fuente) con:
   ```
   ### <ID>: Description
   - **Fuente:** Backlog / Bitácora / Plan
   - **Fecha:** YYYY-MM-DD
   - **Objetivo:** One-line summary
   - **Resultado:** ✅
   - **Ids:** `ID`
   ```
3. If the task was a significant milestone, also add a note under the **Executive Summary** or **Recent Progress** section.
4. If the task was a research/discovery, consider adding to `docs/Investigaciones/` instead of or in addition to progreso.

### E. Register in CHANGELOG (user-visible changes only)

Only add to `docs/CHANGELOG.md` if the task introduces a new feature, breaking change, public bugfix, new CLI command, etc. NOT every individual task.

### F. Validate doc coverage

```pwsh
pwsh scripts/validate-docs-coverage.ps1
```

If it reports gaps, document the missing surface before proceeding.

### G. Notify

Tell the user that Backlog.md, bitacora.md, plan file and progreso/README.md were updated and validation passed. Do NOT commit — wait for explicit instruction.

---

## Trigger 2: Start a new task

Before generating a new plan:

1. Read `docs/progreso/README.md` — check if the previous task was already migrated.
2. If not, run **Trigger 1** first to flush it.
3. Find the task in `docs/Backlog.md`, `docs/bitacora.md`, or the active plan file. If status is ❌, change it to 🟡 (or leave it and update after completion).
4. Proceed with the new work.

---

## Trigger 3: Monthly/fase maintenance

1. Backlog: move tasks inactive >30 days to ⏸️ Icebox or ❌ No Hacer.
2. progreso: deduplicate entries, fix stale cross-links.
3. Investigaciones: verify index matches actual files, prune orphans.
4. Cross-check: no task exists in both Backlog.md and progreso/README.md.

---

## Definition of Done (pre-commit checklist)

- [ ] Compiles (`cargo check --workspace` or `cargo nextest run --no-run`)
- [ ] Tests pass (`cargo nextest run --profile audit --workspace --build-jobs 2`)
- [ ] Affected docs updated (see Trigger 1.A table)
- [ ] MPTS cross-reference added if new technical feature
- [ ] `scripts/validate-docs-coverage.ps1` passes clean
