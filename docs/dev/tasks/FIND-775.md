---
title: "FIND-775: parseTasks() lee el formato tabla del plan real (50 tareas, no 0)"
kind: task
status: stable
description: "parseTasks() devolvía 0 de 35 tareas del plan activo porque solo reconocía `### Task N:` y los planes reales son tablas markdown, así que get_next_task, el claim y HARD-07 quedaban apagados. Se añade el lector de tablas, el guard de `status:` (un plan superseded no entrega tareas) y se cierra el falso positivo updated:true con un taskId inexistente."
---

# FIND-775: `parseTasks()` lee el formato tabla (el real) y deja de mentir en el write

## Metadata

- **Plan file:** `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md` (W12)
- **Fuente:** `docs/dev/Backlog.md` fila `FIND-775` (Alta, 2026-11-26)
- **Origen:** auditoría del harness (21 `Task not found` de una sesión con waves)
- **Tipo:** JS (task-system, parser + contrato de escritura) + tests
- **Zona:** `.opencode/task-system/mcp/**` (exclusiva de esta tarea)
- **Estado:** ✅ COMPLETED (2026-11-26)

## Síntoma medido

```
node -e "…parseTasks(plan activo).length"   →  0        (antes)
```

Causa: `parseTasks` parte el texto por `### Task N:` (`.opencode/task-system/mcp/parsers.mjs`, §`parseTasks`).
El plan activo no tiene ni un header de esos: sus 50 tareas son **filas** de tabla
(35 en la tabla principal, 15 en la tabla de waves W10–W14). `findTaskById` y
`updateState` tienen el mismo supuesto, así que ningún read ni write del task-system
tocaba este plan.

Segundo defecto del mismo sitio: `updateTaskStateCore` compara el contenido *con*
Campaign ID insertado contra el original; si el plan no tenía `Campaign ID:`, la
inserción hace `updated !== original` y un `taskId` inexistente devuelve
`updated:true` con la tarea intacta (falso positivo de escritura).

## Contrato

1. `parseTasks` devuelve **50** con el plan activo y **0** con un plan `status: superseded`.
2. El formato `### Task N:` sigue funcionando igual (los 4 suites actuales en verde).
3. Un `taskId` inexistente devuelve `updated:false` (+ `warning`) y **no escribe**.
4. `node --test .opencode/task-system/mcp/*.test.mjs` verde, con test **nuevo** que
   parsea el plan real y afirma 50, y otro con un plan superseded que afirma 0.

## Blast Radius

| Dirección | Módulos / símbolos |
|-----------|--------------------|
| Callers de `parseTasks` | `campaign_get_next_task` (`campaign-server.mjs:538`), `campaign_validate_scope` (`:429`), `campaign_plan_status` (`:1301`,`:1483`), `findInProgressTasks` (`:93`), `updateTaskStateCore` (`:972`,`:993`), budget/claim (`:1096`), `:1204`, `:2197` |
| Callers de `updateState`/`findTaskById` | solo `updateTaskStateCore` (`:975-976`) |
| Callers de `extractState` | `parseTasks` (único); `normalizeState` se extrae de su cuerpo sin cambiar comportamiento |
| Propiedad ajena | `docs/dev/plans/**` (lo edita el owner) · `dev-tools/**` (FIND-788/789) · `.github/**` (FIND-788/789) · `vantadb-mcp/**`, `vanta-memory/**`, `src/**` → **no se tocan** |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/task-system/mcp/parsers.mjs` (184 L) ·
  `parsers.test.mjs` (206 L) · `hardening.test.mjs` (204 L) · `review-gate.test.mjs` (175 L) ·
  `state-persistence.test.mjs` (220 L) · `campaign-server.mjs` §helpers (25-134), §`updateTaskStateCore`
  (860-1020), §`campaign_get_next_task` (420-580) · `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md` (93 L) ·
  `docs/dev/Backlog.md` fila `FIND-775` · `docs/dev/tasks/FIND-782.md` (formato de referencia) ·
  `dev-tools/git-commit-isolated.ps1` (cabecera) · `.githooks/pre-commit` (cabecera)
- **Referencias hacia dentro:** `parseTasks` importado en una sola línea (`campaign-server.mjs:12`);
  `findTaskById`/`updateState` solo desde `updateTaskStateCore` y los tests de `mcp/`.
- **Referencias entrantes:** `grep` de `parseTasks|findTaskById|updateState` en `.opencode/**` → solo
  `mcp/` (server + 4 suites). `dev-tools/check-harness-contract.mjs` y `C0-unified.mjs` no los citan.
- **Veredicto impacto:** medio, confinado a `mcp/parsers.mjs` + `mcp/campaign-server.mjs`. El riesgo
  real es de **compatibilidad de formato**, no de concurrencia: por eso el contrato se fija primero
  con tests que reproducen el plan real y el formato viejo.

## Spec (decisiones)

| # | Decisión | Alternativa descartada | Por qué |
|---|----------|------------------------|---------|
| 1 | La tabla se reconoce por su **header**: columna `ID` (una tarea por fila) o `Filas`/`Tareas` (lista de IDs en la celda) | escanear cualquier celda con un token `XXX-12` | la tabla principal cita `FIND-770` en la columna *Prerequisito*: escanear todo duplica tareas (35 + 2 = 37, no 50) |
| 2 | `status:` del frontmatter se evalúa **en `parseTasks`**: `superseded`/`archived`/`rejected`/`deprecated` → `[]` | filtrar en `findPlanFile` (mtime) | un guard en la función por la que pasan **todos** los callers cubre también el budget y `findInProgressTasks`; el orden por mtime queda como deuda declarada (§Deuda) |
| 3 | Fila de tabla **sin** columna de estado → `⬜ PENDING`, y `updateState` sobre ella es **no-op** que reporta `updated:false` | inventar el estado o añadir columna al plan | el plan lo edita el owner; escribir un token que el formato no tiene sería otra mentira |
| 4 | `stateApplied` (el token objetivo está realmente en el bloque/fila) es la prueba de escritura; sin ella `updated:false` | confiar en `updated !== original` | es exactamente el falso positivo: la inserción de `Campaign ID` también cambia el contenido |
| 5 | `findTaskById` reconoce también la fila de tabla (mismo primitivo de existencia que `parseTasks`) | guard con `parseTasks` en el server | ya se rompió una vez (regresión F1, `review-gate.test.mjs` T5): dos primitivos de existencia divergen |

## Steps

- [x] 1. Medir el antes (`parseTasks` = 0) y el baseline de suites (55 tests, 55 pass).
- [x] 2. RED: tests nuevos (plan real = 50 · plan superseded = 0 · taskId inexistente sin write).
- [x] 3. GREEN: `parsers.mjs` — tablas, `normalizeState`, `extractPlanStatus`, fila en `findTaskById`/`updateState`.
- [x] 4. GREEN: `campaign-server.mjs` — guard de existencia + `stateApplied` antes de escribir.
- [x] 5. VERIFY: `node --test .opencode/task-system/mcp/*.test.mjs` (antes/después) + medición de 50.
- [x] 6. Review P2-01 ronda 1: `changes-required` con 3 findings → 2 bugs reales corregidos (write no
      column-scoped · tabla `| ID |` sin columnas de tarea) + nits de docs. 2 fixtures nuevos.
- [x] 7. Cierre: fila fuera del Backlog, entrada en `docs/dev/avance/activo/operaciones.md`, commit local.

## Verify (medido)

```
node --test .opencode/task-system/mcp/*.test.mjs   →  55/55 antes  ·  62/62 después (7 nuevos, 0 fail)
parseTasks(plan activo)                            →  0 → 50 (50 ids únicos, FIND-758 … FIND-737)
parseTasks(2026-11-23-release-v0.10.0-gates.md)    →  0   (status: superseded)
parseTasks(plan activo con status: superseded)     →  0
parseTasks(archive/2026-10-20-wiring-encryption)   →  0   (era 12 "tareas" que eran riesgos)
```

## Review (P2-01 — tier fast: `.opencode/**` + `docs/dev/**`)

Ronda 1 (agente distinto, `vanta-review`, 2026-11-26): **`changes-required`** — los 5 puntos del
contrato pasaron, con 3 findings:

| # | Finding | Estado |
|---|---------|--------|
| 1 | El write de tabla no estaba acotado a la columna: `STATE_TOKEN_RE` se aplicaba a la fila entera, así que la primera celda que citara un token (la prosa de un plan los cita) se corrompía y el estado no cambiaba — con `updated:true`. | ✅ Corregido: `findTaskById` devuelve `stateCol`/`state`; `updateState` reescribe solo esa celda; `stateApplied` compara el estado **leído de vuelta** por el parser (`normalizeState`), no un substring. Fixture: "el write toca SOLO la celda de estado". |
| 2 | Un header `\| ID \|` se trataba como tabla de tareas fuera lo que fuera el resto: `archive/2026-10-20-wiring-encryption-feature.md` (`\| ID \| Riesgo \| Mitigación \|`) daba 12 tareas fantasma. | ✅ Corregido: la vía `ID` exige al menos una columna de tarea. Medido 12 → 0. Fixture propio. |
| 3 | El diff de `Backlog.md` arrastraba la eliminación de la sección `FIND-789` (worktree compartido, otro carril). | ✅ No entró al commit: el archivo se reconstruyó desde `git show HEAD:` menos **solo** mi fila (`1 deletion(-)`); la eliminación ajena queda intacta en el worktree para su carril. |
| — | Nits: `status: active` en un task file cerrado · "las tablas de waves" (es una) · warning del write nombraba solo la tabla | ✅ Los tres aplicados |

Ronda 2 (re-verificación de los 3 findings, mismo reviewer): **`approve`** — repro del finding 1 corregido
(la celda que cita `⬜ PENDING` sobrevive, el estado se escribe y se lee de vuelta), archive de riesgos
**12 → 0**, `git diff HEAD -- docs/dev/Backlog.md` vacío, **62/62** tests. Sin findings nuevos.
Deuda no bloqueante que confirma: `findPlanFile` por mtime y `status: completed` fuera de la lista de inactivos.

**Nota de cierre:** la fila `FIND-775` del Backlog **ya no está en HEAD** — la retiró el commit de otro
carril `6e4119c8d` (path-scoped commit que se llevó mi edición del worktree, exactamente la clase que
`FIND-792` acaba de registrar). Ese commit contiene **una sola** eliminación en `Backlog.md`, así que el
resultado es el buscado; mi commit no vuelve a tocar esa fila (incluir el path es no-op) y la verificación
por contenido (`-ExpectAbsent`) sigue midiendo lo mismo.

## Deuda declarada

- `findPlanFile` sigue eligiendo por **mtime de 24 h**; con el guard de `status:` en
  `parseTasks` un plan superseded ya no entrega tareas, pero el archivo sigue siendo
  el "más reciente" y participa del guard de ambigüedad. Cambiar el orden (status antes
  que mtime) toca el reloj de las waves: **fila separada**, no se mezcla con el parser.
- El plan activo **no tiene columna de estado**: `parseTasks` lo lee (50 `⬜ PENDING`),
  pero el claim no puede escribir hasta que el owner añada esa columna. `updateState`
  lo dice (`updated:false` + warning) en vez de fingir un write.
- `status: completed` **no** está en la lista de planes inactivos (solo `superseded`,
  `archived`, `deprecated`, `rejected`): un plan terminado sigue siendo legible para
  resúmenes, y el falso positivo que lo motivaba lo cierra la regla de columna de tarea.
- `C0-unified.mjs` (perfil de workflow) no se revisó en esta tarea: fuera de contrato.
