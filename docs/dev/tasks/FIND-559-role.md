---
title: "FIND-559 - La clausula ROLE: mantener el fail-closed y arreglar el silencio"
kind: task
status: completed
description: "Decidir el comportamiento de la poda de ROLE y hacerlo visible: contar los nodos podados, avisar cuando la poda se come el 100 % del resultado y documentar el campo _owner_role. Decision del owner (2026-10-21): fail-closed + aviso."
tags: [iql, rbac, rol, documentacion, decision-owner]
---

# FIND-559: la cláusula ROLE, fail-closed y con aviso

- **Origen:** `docs/dev/Backlog.md` (`FIND-559`) + la decision del owner del 2026-10-21
  ("fail-closed + aviso") + el brief de `docs/dev/reviews/2026-10-21-encryption-envelope-por-pagina.md`.
- **Rama:** `develop` · **Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
- **Estado:** 🟢 COMPLETED — Verificación mecánica ejecutada y review P2-01 aprobado.
- **Decision del owner (2026-10-21):** **mantener el fail-closed** (un nodo sin `_owner_role` se poda) y
  **arreglar el silencio**: contar los podados y avisar cuando la poda se come el 100 % del
  resultado, mas documentar `_owner_role`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | el path IQL de `src/executor.rs` (`execute_plan`, el prune de `ROLE`), la CLI, el MCP, los bindings |
| Callees | `src/executor.rs:672-684` (el prune actual, antes de este task) |
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
| `lastAction` | Step 5 (verificacion) — 5 tests nuevos en verde, `fmt` y `clippy -D warnings` limpios, suite del lib verde (1 timeout de carga, ajeno) |
| `result` | ✅ IMPLEMENTADO — la poda **sigue fail-closed** (el resultado no cambia) + **un aviso informativo por consulta** cuando la poda se come el 100 % de los candidatos + `docs/api/IQL.md` documenta `_owner_role` y la semantica fail-closed |
| `nextAction` | Review P2-01 (agente DISTINTO del implementador) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda. La unica simplificacion (documentada en `## Notas`, con su `ponytail:`) es el **canal**
del aviso: va por `tracing::warn!` y no por un campo nuevo en la superficie publica. Techo
conocido: si algun dia hace falta llevar el aviso al cliente HTTP/MCP, el camino es un campo
nuevo en el resultado — no convertirlo en un `Error`.

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

- El prune actual (`src/executor.rs:672-684` antes de este task) descarta todo nodo sin
  `_owner_role` salvo `required_role == "admin"`. **Nadie escribe ese campo**: `grep _owner_role` en `src/` no encuentra
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
| Pendientes de ejecución (downhill) | 0 (4/4 steps completados) + review P2-01 |
| % completado | 100% |

## Steps

1. ✅ **Verificar la evidencia**: confirmada. El prune actual (`src/executor.rs:672-684`, antes
   de este task) descarta todo nodo sin `_owner_role` salvo `required_role == "admin"`, **sin
   avisar**; y `grep _owner_role src/` no encuentra **ningun escritor** (solo el benchmark
   `benches/high_density.rs:113` y los tests) ⇒ `ROLE "user"` sobre una base sin etiquetar
   devuelve **0 filas sin decir por que**.
2. ✅ **Test primero** (RED→GREEN en 2 slices): 5 tests nuevos (4 de contrato + 1 de politica).
   - **RED #1 (compile)**: `role_prune_notice_policy` no compila (`cannot find function
     role_prune_notice in module super`) ⇒ la politica de aviso no existia.
   - **RED #2 (comportamental)**: con la politica implementada pero **sin** el cableado en el
     executor, `role_prune_warns_once_when_it_prunes_every_scanned_record` **FALLA** con
     `the notice must name the missing field:` (log capturado vacio) ⇒ el aviso no existia.
   - **Nota honesta**: (c) "no hay aviso con poda parcial" y (d) "no hay aviso sin `ROLE`" **no
     pueden fallar antes del fix** (una asercion de "no hay aviso" es trivialmente cierta
     mientras no exista el aviso): son **guardas** contra una implementacion ingenua que avise en
     cada poda. El RED de la rama "poda parcial ⇒ sin aviso" vive en el test de politica
     (`role_prune_notice_policy`), que si fallo (RED #1).
3. ✅ **Fix**: contadores de la poda en `execute_plan` (`role_scanned` / `role_pruned` /
   `role_untagged`) + **un** `tracing::warn!` por consulta cuando `pruned == scanned` (y
   `scanned > 0`). El aviso **no** es un `Error` y **no** cambia el resultado: la poda sigue
   siendo fail-closed (fail-closed + aviso, la decision del owner del 2026-10-21).
4. ✅ **Documentar** `docs/api/IQL.md`: fila de la tabla de componentes reemplazada por la
   **variante A (fail-closed)** del brief, verbatim, + la nota nueva debajo de la tabla. No
   aplica runbook de claves (esta task no toca el cifrado).
5. ✅ **Verificar**: `cargo fmt --all -- --check` **exit 0**; `cargo clippy -p vantadb
   --all-targets -- -D warnings` **exit 0**; suite del lib `cargo nextest run -p vantadb
   --profile audit --lib` → **2533 tests: 2532 passed, 2 skipped, 1 timeout**
   (`index::core::tests::concurrent_insert_preserves_hnsw_invariants`: test de stress de
   concurrencia **ajeno a este task**, pasa en aislamiento en 82,9 s; el timeout es de **carga**
   de la maquina — hay otros agentes compilando en paralelo); los 6 tests de `role_prune` en
   verde. Entrada de avance anadida en `docs/dev/avance/activo/core-engine.md`.

## Tests nuevos (5)

Todos en `src/executor.rs` (`mod tests`):

| Test | Que fija |
|---|---|
| `role_prune_keeps_the_fail_closed_result` | (a) el **resultado no cambia**: el nodo etiquetado se ve, el sin etiquetar se poda |
| `role_prune_warns_once_when_it_prunes_every_scanned_record` | (b) el aviso aparece con la poda al 100 % (con los contadores `pruned 3 of 3` y el campo `_owner_role`) y se emite **una sola vez** por consulta, no una por nodo |
| `role_prune_does_not_warn_when_it_prunes_only_part_of_the_records` | (c) **no** hay aviso con la poda parcial |
| `role_prune_does_not_warn_without_a_role_clause` | (d) **no** hay aviso sin `ROLE` en el plan |
| `role_prune_notice_policy` | la politica del aviso: 100 % ⇒ aviso; parcial ⇒ nada; plan vacio ⇒ nada (el caso del filtro de `AS OF`, que corre **despues** del plan) |

El aviso se observa por el log (`tracing::warn!`), capturado con un `tracing_subscriber::fmt`
con `MakeWriter` propio (mismo patron que `src/storage/engine/tests/encryption.rs`).

## Invariantes: verificados

| # | Invariante | Como se verifica |
|---|---|---|
| a | el **resultado** de una query con `ROLE` no cambia | `role_prune_keeps_the_fail_closed_result` + el test preexistente `role_prune_still_applies_when_the_query_projects_fields` (FIND-552) siguen verdes; el prune se limito a **contar**, la condicion de poda es la misma |
| b | el aviso **no** es un `Error` ni cambia el resultado | el aviso es un `tracing::warn!` (log); `execute_plan` sigue devolviendo `Ok(results)` con las mismas filas; los tests (b)/(c)/(d) comprueban las filas **y** el log a la vez |
| c | **una vez por consulta**, no por nodo | el aviso se emite **despues** del bucle, con los contadores acumulados; `role_prune_warns_once_when_it_prunes_every_scanned_record` asserta `matches("pruned").count() == 1` con 3 registros podados |
| d | el aviso distingue "la poda se comio todo" de "el filtro de `AS OF` se comio todo" | el filtro de `AS OF` (`filter_valid_at`) corre **despues** del plan (`execute_query`/`execute_select`), asi que no puede disparar el aviso; y `role_prune_notice_policy` fija que `scanned == 0` (plan vacio) ⇒ sin aviso. El mensaje nombra la clausula `ROLE` y el campo `_owner_role` |
| e | nada de `unsafe` | el cambio no introduce `unsafe` (ni lo toca) |
| f | el camino **sin** `ROLE` no se toca | los 3 contadores solo se incrementan dentro del `if let Some(required_role) = &plan.enforce_role`; sin `ROLE` el camino es identico al de antes (0 iteraciones extra, 0 Formateo) |


## Review (P2-01)

- **Reviewer:** Orquestador (sesión 87987f03)
- **Veredicto:** ✅ APROBADO
- **Fecha:** 2026-10-23
- **Checklist para el revisor (verificado mecánicamente):**
  1. [x] La **semántica de la poda** sigue siendo fail-closed: verificado en `src/executor.rs:707-723`, la condición de descarte es idéntica y solo se contabiliza.
  2. [x] El aviso es **informativo** (`tracing::warn!`) y `execute_plan` retorna `Ok(results)`.
  3. [x] Se emite **una vez por consulta** tras finalizar el procesamiento del plan (L736-742), nunca por nodo.
  4. [x] Distingue poda al 100% de `AS OF` (que se ejecuta posterior al plan). Si `scanned == 0`, no emite advertencia.
  5. [x] El camino sin `ROLE` no ejecuta sobrecarga (contadores condicionados a `enforce_role`).
  6. [x] `docs/api/IQL.md` documenta la variante A (fail-closed) y el campo `_owner_role`.
  7. [x] Alcance respetado: sin cambios espurios fuera de `src/executor.rs` y `docs/api/IQL.md`.

### Evidencia de tests
- 6/6 tests en `executor::tests::role_prune`: ✅ todos en verde (0.58s)
- `cargo clippy -p vantadb --lib -D warnings`: ✅ 0 warnings (5.56s)
- `cargo fmt --check`: ✅ limpio

## Notas

- `src/executor.rs` lo comparten otras tasks de este ciclo: **releelo antes de editar** y limita el
  cambio al prune de `ROLE` y sus contadores.
- No cambies la semantica de la poda: la decision del owner es fail-closed + aviso.
- **Regla de git:** este agente **NO hace commits** (los hace el orquestador); el push requiere
  instruccion explicita del owner (AGENTS.md Regla 7).
- **Ficheros tocados por este task:** `src/executor.rs` (la poda + sus contadores + el aviso + 5
  tests), `docs/api/IQL.md` (variante A del brief), este task file y
  `docs/dev/avance/activo/core-engine.md`. Nada mas.
- **Ponytail (lite):** el aviso va por `tracing::warn!` — el idiom del repo para "avisado, nunca
  silencioso" (el mismo patron que uso FIND-485 para el vector sintetico) — en vez de un canal
  nuevo de avisos: tocar la superficie publica (`ExecutionResult`) habria roto el contrato que
  este task promete no romper. `ponytail:` la granularidad del aviso (una linea de log por
  consulta) es la unica que el brief pide; si algun dia hace falta llevar el aviso al cliente
  HTTP/MCP, el camino es un campo nuevo en el resultado, no un `Error`.
- **Ojo con el reparto de trabajo en el arbol:** durante la sesion, `src/storage/vfile.rs`,
  `src/sdk/builder.rs`, `src/agentic/thread.rs`, `src/tui/**` y `src/storage/engine/init.rs`
  cambiaron bajo los pies (otros agentes de este ciclo). El `fmt`/`clippy`/`nextest` de esta task
  se re-verificaron **con el arbol ya estable**; el unico rojo que queda (el timeout de
  `index::core::tests::concurrent_insert_preserves_hnsw_invariants`) es de **carga** y es ajeno.
