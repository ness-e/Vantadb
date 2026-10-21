---
title: "FIND-552 - La proyeccion (FETCH) destruye los metadatos que necesitan los post-filtros"
kind: task
status: active
description: "Que AS OF y el filtro de confianza se apliquen aunque la query proyecte campos: hoy FETCH (name) anula el filtro valid-time."
tags: [physical-plan, executor, critica, correctness]
---

# FIND-552: La proyeccion (FETCH) destruye los metadatos que necesitan los post-filtros

`PhysicalProject::next` **elimina** todo campo no proyectado (`node.relational = projected`). Los filtros temporales se aplican **post-plan** (`executor.rs:257-264` -> `filter_valid_at`) y `node_is_valid_at` lee `__vanta_valid_at_ms` / `__vanta_created_at_ms`; si no estan, `let Some(valid_at) = valid_at else { return true; }` ⇒ **sin metadatos = siempre valido**. Esos dos campos no estan en la lista de proyeccion ⇒ desaparecen ⇒ el predicado temporal se convierte en `true` para todas las filas: `FROM Doc AS OF 1500 FETCH (name)` devuelve nodos **no validos** en t=1500. El filtro valid-time (SCH-03 / ADR-046 D3) queda **anulado sin error ni warning**.

El mismo mecanismo, segundo caso: `filter_low_confidence_summaries` (`executor.rs:280-303`, solo en `execute_query`) depende de `type` y `confidence_score`; con `FETCH (name)` el campo `type` desaparece ⇒ el filtro de confianza de `SemanticSummary` deja de filtrar.

Ojo con `FIND-559` (la clausula `ROLE` lee `_owner_role`, que la proyeccion tambien borra ⇒ `ROLE` con proyeccion devuelve 0 filas siempre): la allowlist de metadata del sistema lo arregla de paso.

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-552 (critica))
- **Esfuerzo:** 🟠 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 5-7
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T12:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 4 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | src/planner.rs, src/executor.rs, src/sdk/api/graph.rs |
| Callees | src/physical_plan/project.rs, src/executor.rs (filter_valid_at, filter_low_confidence_summaries) |
| Implicaciones | la proyeccion deja de ser una proyeccion pura: conserva una allowlist de metadata del sistema. Es el precio de que los filtros temporales y de confianza se apliquen despues del plan. |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** src/physical_plan/project.rs (completo), src/executor.rs (la parte de filtros post-plan: 232-350), src/physical_plan/filter.rs (para saber que campos lee cada filtro)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** project.rs -> crate::node::UnifiedNode (relational); executor.rs -> physical_plan::{project,filter,vector,scan}
- **Archivos que referencian a los editados (referencias entrantes):** src/planner.rs (compila el Project), src/executor.rs (execute_select/execute_query), src/sdk/api/graph.rs (query)
- **Veredicto impacto:** medio: allowlist de metadata del sistema en la proyeccion. Hay que enumerar TODOS los campos de sistema que leen los filtros (__vanta_valid_at_ms, __vanta_invalid_at_ms, __vanta_created_at_ms, type, confidence_score, _owner_role) para no perder ninguno.

## Contrato
"`FROM Doc AS OF <t> FETCH (name)` **no** devuelve nodos con `__vanta_valid_at_ms` posterior a `<t>` (hoy los devuelve todos); y el filtro de confianza de `SemanticSummary` sigue filtrando con `FETCH`. Sin cambios en lo que devuelve un `SELECT`/`FROM` **sin** proyeccion."

## Spec (SDD)

N/A — no es feature-add ni symbolos publicos nuevos: es la correccion de un
comportamiento existente que ya tiene contrato documentado (el del backlog).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 1) un `SELECT *` o un `FROM` **sin** `FETCH` devuelve exactamente lo mismo que antes (no cambiar el camino sin proyeccion); 2) la proyeccion sigue **quitando** los campos de usuario que el usuario no pide (no basta con no borrar nada: hay que borrar los de usuario y conservar solo la allowlist del sistema); 3) `AS OF` sobre un nodo **sin** metadatos de validez sigue valiendo (no se inventa la semantica de un nodo legacy); 4) el filtro de confianza solo aplica a `SemanticSummary` (no generalizarlo).
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit --lib physical_plan`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-552: La proyeccion (FETCH) destruye los metadatos que necesitan los post-filtros` |
| `lastAction` | (se actualiza en cada step) |
| `result` | ⬜ PENDING |
| `nextAction` | Step 1 (ver `## Steps`) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + los tests del cambio en verde |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica (nunca auto-reporte) |
| **Release** | No aplica (sin release en esta iteracion) |

## Herramientas necesarias
- cargo (check, clippy, fmt) — con `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`
- codegraph_codegraph_explore (blast radius)

## Skills cargadas (SDP):** test-driven-development (el fix necesita un test que fije el
comportamiento roto antes del fix) + systematic-debugging (causa raiz, no sintoma) +
security-and-hardening (cuando el hallazgo es de integridad de datos).

## Investigation Notes
- El hallazgo viene de la auditoria profunda del 2026-10-20/21 y su evidencia (archivo:línea)
  esta en la fila del backlog. Verificar la evidencia ANTES de tocar nada: si el codigo ya
  esta corregido, documentarlo y cerrar la fila (sin editar).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 |
| % completado | 0% |

## Steps

1. **Verificar la evidencia** (sin editar): confirmar en `project.rs:31-42` que
   se borran los campos no proyectados y en `executor.rs:330-335` que la ausencia de
   `__vanta_valid_at_ms` implica "valido".
2. **Test primero**: un test que inserte nodos con `__vanta_valid_at_ms` en el futuro y haga
   `FROM ... AS OF <t> FETCH (campo)` y compruebe que **no** los devuelve (hoy los devuelve).
   Y un test equivalente para el filtro de confianza de `SemanticSummary`.
3. **Fix**: mover `filter_valid_at` y `filter_low_confidence_summaries` **dentro** del plan
   (por encima del `Project`), **o** que `PhysicalProject` conserve una allowlist explicita de
   metadata del sistema. Elegir una (preferible la allowlist: no cambia el plan) y **declarar la
   allowlist en un solo sitio** (un `const` con los nombres).
4. **Verificar**: fmt, clippy, la suite del lib completa, y los tests nuevos en verde.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
