---
title: "FIND-559 - La clausula ROLE: mantener el fail-closed y arreglar el silencio"
kind: task
status: active
description: "Decidir el comportamiento de la poda de ROLE y hacerlo visible: contar los nodos podados, avisar cuando la poda se come el 100 % del resultado y documentar el campo _owner_role. Decision del owner (2026-10-21): fail-closed + aviso."
tags: [iql, rbac, rol, documentacion, decision-owner]
---

# FIND-559: la cláusula ROLE, fail-closed y con aviso

- **Origen:** `docs/dev/Backlog.md` (`FIND-559`) + la decision del owner del 2026-10-21
  ("fail-closed + aviso") + el brief de `docs/dev/reviews/2026-10-21-encryption-envelope-por-pagina.md`.
- **Rama:** `develop` · **Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
- **Estado:** ⬜ PENDING
- **Decision del owner (2026-10-21):** **mantener el fail-closed** (un nodo sin `_owner_role` se poda) y
  **arreglar el silencio**: contar los podados y avisar cuando la poda se come el 100 % del
  resultado, mas documentar `_owner_role`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | el path IQL de `src/executor.rs` (`execute_plan`, el prune de `ROLE`), la CLI, el MCP, los bindings |
| Callees | `src/executor.rs:634-646` (el prune actual) |
| Implicaciones | el **resultado** no cambia (la poda se mantiene) ⇒ **no hay ruptura de contrato**; lo que se anade es observabilidad y documentacion |

## Impacto mapeado (Regla 0)

- **Archivos leidos (completos):** `src/executor.rs` (la parte del prune de `ROLE` y los filtros
  post-plan), `docs/api/IQL.md` (la seccion de `ROLE`).
- **Archivos referenciados hacia dentro:** `src/executor.rs` -> `crate::query::LogicalPlan`,
  `crate::node::UnifiedNode`, `crate::console` (o `tracing`).
- **Archivos que referencian a los editados:** `src/sdk/api/graph.rs` (ejecuta IQL),
  `src/server/handlers.rs` (query_iql), `vantadb-mcp/src/handlers/tools.rs` (query_iql).
- **Veredicto impacto:** **bajo**. No cambia el resultado de ninguna query; solo anade un contador y
  un aviso. El `src/executor.rs` esta compartido con otras tasks de este ciclo ⇒ **relee el fichero
  antes de editarlo** y no toques nada fuera del prune de `ROLE` y sus contadores.

## Contrato

"Con un plan que lleva `ROLE \"user\"`:
(1) el conjunto de filas devuelto **es el mismo que hoy** (la poda se mantiene: un nodo sin
`_owner_role` no se devuelve);
(2) un aviso (una vez por consulta) cuando **todos** los candidatos fueron podados por el
`ROLE` — es decir, cuando la poda se come el 100 % del resultado, que es el caso que hoy
devuelve 0 filas sin decir por que;
(3) `docs/api/IQL.md` documenta el campo `_owner_role`, que es de donde sale la etiqueta, y la
semantica fail-closed."

## Spec (SDD)

N/A — no es feature-add ni simbolos publicos nuevos. La semantica de la poda **no cambia**
(decision del owner).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (a) el **resultado** de una query con `ROLE` no cambia (la poda se
  mantiene: fail-closed); (b) el aviso **no** puede convertir un resultado legitimo en un error ni
  viceversa (es informativo, no un `Error`); (c) no puedepitfires por consulta: una vez por
  consulta, no por nodo; (d) el aviso tiene que distinguir "la poda se comio todo" de "el filtro
  de `AS OF` se comio todo" (que es un caso distinto y ya documentado); (e) nada de `unsafe`; (f)
  el camino **sin** `ROLE` no se toca (cero sobrecarga cuando no hay prune).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-559` |
| `lastAction` | (se actualiza en cada step) |
| `result` | ⬜ PENDING |
| `nextAction` | Step 1 |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + fmt + clippy `-D warnings` + los tests nuevos |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica |
| **Release** | No aplica |

## Herramientas necesarias

cargo (check, clippy, fmt) · codegraph_codegraph_explore (blast radius)

## Skills cargadas (SDP):** test-driven-development (fijar el comportamiento actual antes de
anadir el aviso) + code-simplification (un contador y un aviso, no un framework de avisos).

## Investigation Notes

- El prune actual (`src/executor.rs:634-646`) descarta todo nodo sin `_owner_role` salvo
  `required_role == "admin"`. **Nadie escribe ese campo**: `grep _owner_role` en `src/` no encuentra
  ningun escritor (solo un benchmark, `benches/high_density.rs:113`); los registros de memoria solo
  lo llevan si el usuario lo pone a mano en el `metadata` (`src/sdk/serialization/mod.rs:621-623`).
- El caso que mas duele: una base cuyos registros nunca se etiquetaron ⇒ `ROLE "user"` devuelve
  **0 filas sin decir por que** (hoy no hay ninguna pista de que la poda fue el motivo).
- **`FIND-552`/`FIND-559` (parte de proyeccion) ya esta resuelto**: la allowlist de metadata del
  sistema de `PhysicalProject` incluye `_owner_role` ⇒ con `FETCH`, `ROLE` **vuelve a filtrar**
  (antes devolvia 0 filas siempre). Esta task es **la otra mitad**: la semantica del nodo sin la
  etiqueta y el silencio.
- Las **2 variantes exactas** del texto para `docs/api/IQL.md` estan en el brief del documento de
  diseno (`docs/dev/reviews/2026-10-21-encryption-envelope-por-pagina.md`, seccion del brief de
  `FIND-559`): una para la opcion fail-closed (la elegida) y otra para fail-open.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 |
| % completado | 0% |

## Steps

1. **Verificar la evidencia**: confirmar que el prune actual descarta sin avisar y que nadie
   escribe `_owner_role` (leer `src/executor.rs:634-646` y el `grep`).
2. **Test primero**: (a) un test que fije que el **resultado** no cambia (la poda se mantiene);
   (b) un test que fije que el aviso aparece cuando la poda se come el 100 %; (c) un test que fije
   que **no** hay aviso cuando la poda es parcial; (d) un test que fije que no hay aviso sin
   `ROLE` en el plan. Deben **fallar** (b) y (c) antes del fix.
3. **Fix**: contar los nodos podados por el prune de `ROLE` y emitir **un** aviso por consulta
   cuando el total podado cubra el total de candidatos (el 100 %), sin convertirlo en error.
4. **Documentar** `docs/api/IQL.md` con la variante fail-closed del texto del brief, y anotar el
   enlace al runbook de claves si aplica.
5. **Verificar**: fmt, clippy `-D warnings`, la suite del lib completa y los 4 tests nuevos en
   verde; y **anadir la entrada de avance** en `docs/dev/avance/activo/core-engine.md`.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

## Notas

- `src/executor.rs` lo comparten otras tasks de este ciclo: **releelo antes de editar** y limita el
  cambio al prune de `ROLE` y sus contadores.
- No cambies la semantica de la poda: la decision del owner es fail-closed + aviso.
