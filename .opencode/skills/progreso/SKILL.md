---
name: progreso
description: >
  Registers completed tasks from docs/Backlog.md into the canonical
  docs/avance tree (domain files), archives removed backlog items,
  tracks doc coverage, reconciles agent memory (lessons/decisions)
  with the project registry, and maintains cross-references across
  the VantaDB documentation tree. (Legacy name kept for call-site
   compatibility; since 2026-08-23 the canonical tree is docs/avance,
   NOT the legacy progreso tree — which was migrated and removed.)
compatibility: opencode
---

# Progreso Skill — VantaDB (avance-canónico desde 2026-08-23)

> **⚠️ CAMBIO DE CANONICIDAD:** el legacy `progreso/` fue migrado físicamente a
> `docs/avance/historial/` y eliminado. El registro vivo de tareas completadas
> es ahora el árbol **`docs/avance/`**, organizado por dominio. Esta skill
> conserva su nombre por compatibilidad con los call sites (`skill progreso`),
> pero todo destino de escritura es `docs/avance/`.

## File Roles

| File | Role |
|---|---|
| `docs/Backlog.md` | **Active tasks only.** Rows completed are **removed from the table** — the completion record lives in `docs/avance/<dominio>`. Rows removed without completing (❌/nunca hará) go to `docs/avance/historial/backlog-history.md`. No `~~…~~` accumulation |
| `docs/avance/activo/*.md` | **Registro vivo por dominio** (core-engine, bindings, web-frontend, ci-cd, operaciones, desktop). Destino canónico de tareas completadas |
| `docs/avance/auditoria/*.md` | Seguridad y dependencias (SEC/AUD/fuzz/Miri; cargo-deny/dependabot) |
| `docs/avance/decisiones/wontfix.md` | WONTFIX / decisiones |
| `docs/avance/investigaciones.md` | INV-* / research artifacts |
| `docs/avance/historial/backlog-history.md` | **Archivo vivo** de ítems removidos del Backlog sin completar |
| `docs/avance/historial/fuentes/*` | Registry legacy congelado (README, bitácora, ARCHIVO_HISTORICO) — **NO editar, NO escribir** |
| `docs/avance/historial/snapshot-*.md` | Snapshots congelados — **NO editar** |
| `docs/avance/historial/campanas/*.md` | Registros por campaña (congelados al cerrar la campaña) |
| `docs/avance/meta.md` | Cambios de proceso y obediencia |
| `docs/plans/archive/` | Plan files completed/aborted |
| `docs/reports/INDEX.md` | Master registry of review/audit reports (Trigger 4) |
| `docs/CHANGELOG.md` | Release notes per version |

**Invariant:** no task exists in both Backlog.md and the avance domain files simultaneously. Items removed from Backlog.md are archived to `historial/backlog-history.md`, not silently dropped.

## Language split

| Language | Directories |
|---|---|
| **English** (tech source of truth) | `docs/api/`, `docs/architecture/`, `docs/operations/`, `docs/QUICKSTART.md` |
| **Spanish** (planning only) | `docs/VantaDB-MPTS/`, `docs/Backlog.md`, `docs/avance/`, `docs/Investigaciones/`, `docs/CHANGELOG.md` (lower section) |

---

## Trigger 1: Complete a task

Run when a task reaches ✅ in the current session.

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

> **Mantenimiento:** actualizar esta tabla cuando aparezcan archivos fuente o docs nuevos.

### B. Extract task data

ID (e.g. `TSK-09`), name, date, objective, modified files, result.

### C. Check all task sources

| Source | What to do |
|--------|-----------|
| `docs/Backlog.md` | **Remove the ✅ row entirely.** Removed without completing (❌) → ALSO remove row, move to `docs/avance/historial/backlog-history.md` |
| `docs/plans/YYYY-MM-DD-*.md` | Update status tracker + recitation |

### D. Register in avance (dominio)

1. Determinar el archivo de dominio según la tabla de abajo.
2. Grep el ID en `docs/avance/activo/`+`auditoria/` (archivos chicos — sí se pueden leer; los grandes son solo core-engine).
3. Si el ID ya existe → skip/actualizar. Si no, agregar entrada:
   ```
   ### <ID>: Description
   - **Fecha:** YYYY-MM-DD
   - **Objetivo:** One-line summary
   - **Resultado:** ✅
   - **Commit:** <hash si aplica>
   ```
4. Milestone significativo → nota también en `meta.md`.
5. Research/discovery → considerar `docs/avance/investigaciones.md` o `docs/Investigaciones/`.

### Tabla de dominios

| ID / dominio | Archivo en `docs/avance/` |
|---|---|
| DRV-*, COMP-*, VFY-*, storage/WAL/HNSW/ACID/IQL | `activo/core-engine.md` |
| Bindings Python/WASM/TS/MCP/adapters | `activo/bindings.md` |
| Web frontend/SEO/UX | `activo/web-frontend.md` |
| CI/CD, release, docker, wheels | `activo/ci-cd.md` |
| Ops: backup/restore/API/enterprise | `activo/operaciones.md` |
| Desktop (Tauri) | `activo/desktop.md` |
| SEC-*, AUD-*, fuzz, Miri, FFI | `auditoria/seguridad.md` |
| Deps: cargo-deny, dependabot, advisories | `auditoria/dependencias.md` |
| WONTFIX / decisiones | `decisiones/wontfix.md` |
| Investigaciones (INV-*) | `investigaciones.md` |
| No-ops / SKIPs | `historial/no-ops.md` |
| Cambios de proceso | `meta.md` |

### D2. Archive completed plans (cuando el plan file termina)

Cuando todas las tareas están ✅ (o ❌ ABORTADO):

0. **Retrospectiva Start/Stop/Continue + UNA acción medible** contra baseline
   (North Star: >90% first-try, ver `campaign_eval_summary`). Si pipeline-run
   step 8 ya la produjo, copiala.
1. Mover plan file **Y su `.budget.json`** a `docs/plans/archive/`
   (`git mv docs/plans/<plan>.md docs/plans/<plan>.budget.json docs/plans/archive/`).
   El budget es obligatorio: sin él, un `campaign_get_next_task` sobre el plan
   archivado regenera un budget vacío que pisa el histórico real (incidente
   2026-08-25, 5 JSONs huérfanos).
2. Verificar que no queden `*.budget.json` huérfanos en `docs/plans/` raíz:
   todo `<X>.budget.json` en la raíz cuyo `docs/plans/archive/<X>.md` exista
   debe moverse también.
3. Nota de archivo en `docs/avance/meta.md` (fecha, N/M completadas, retrospectiva).
4. Las filas del Backlog ya se eliminaron en paso C.

> Task files (`tasks/<ID>.md`) no se archivan — quedan STALE tras archivar el
> plan; el registro vive en avance.

### E. Register in CHANGELOG (user-visible changes only)

Only new features, breaking changes, public bugfixes, CLI commands. NOT every task.

### F. Validate doc coverage

```pwsh
pwsh scripts/check-avance-coverage.ps1
pwsh scripts/validate-docs-coverage.ps1
```

### G. Notify

Report: Backlog.md, plan file y avance actualizados + validación OK. Commit policy:
- **Standalone:** no commit — esperar instrucción
- **Desde campaign-executor:** el executor maneja commits
- Decisión relevante → `campaign_memory_write(file="decisions", ...)`

---

## Trigger 2: Start a new task

1. Grep el ID previo en `docs/avance/` antes de planificar (evita duplicar trabajo ya registrado).
2. Find the task in Backlog.md o plan activo. ❌ → 🟡 si se retoma.
3. Proceed.

---

## Trigger 3: Monthly/fase maintenance

1. Backlog: tasks inactive >30 days → ⏸️ Icebox o ❌ (removidos → `historial/backlog-history.md`).
2. avance: deduplicate entries, fix stale cross-links.
3. Investigaciones: index vs files reales, prune orphans.
4. Cross-check: ninguna tarea en Backlog Y avance a la vez.

---

## Trigger 4: Sync reportes (review/audit reports ↔ backlog)

Igual que antes pero sin referencia a progreso: reports nuevos en `docs/reviews/`
→ fila en `docs/reports/INDEX.md` → hallazgos ≥ medium derivados al Backlog.
Nunca silenciar un report más nuevo que su fila del INDEX.

---

## Trigger 5: Postmortem (falla / incidente)

Sin cambios respecto a v1: triggers (task ❌, verify 2× mismo error, incidente,
STALL > appetite, fix 3+ intentos), plantilla 10 minutos, persistir en
`.opencode/task-system/memory/lessons.md` vía `campaign_memory_write(file="lessons",
entry="YYYY-MM-DD | POSTMORTEM <ID> | <contexto> | <lección> | <acción>")`.
Sin persistencia no hay cierre.

---

## Trigger 6: Reconciliación de memorias (agente ↔ proyecto)

Comparar entradas nuevas de `memory/lessons.md`+`decisions.md` contra
`docs/Backlog.md` (activa) y `docs/avance/` (completada). Divergencias → hallazgo
REC-<NN> en Backlog o nota pendiente en `meta.md`. Nunca silenciar una entrada
cuyo ID no existe en el proyecto.

> Marker: `reconcil|Reconciliación|reconciliation`.

---

## Definition of Done (pre-commit checklist)

Ver [`.opencode/references/definition-of-done.md`](../../references/definition-of-done.md).

Para tareas de código:
- [ ] Compiles + tests pass (`cargo nextest run --profile audit --workspace --build-jobs 2`)
- [ ] Affected docs updated (Trigger 1.A)
- [ ] `scripts/validate-docs-coverage.ps1` + `check-avance-coverage.ps1` clean
- [ ] **Certify gate recomendado:** `skill unified-review --mode certify --profile vantadb`

## Campaign Memory Integration

```python
campaign_memory_write(
    file="decisions",
    entry="progreso: registrada <ID> en docs/avance/<dominio>. Archivos tocados: <paths>"
)
```
