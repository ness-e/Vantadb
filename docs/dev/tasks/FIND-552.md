---
title: "FIND-552 - La proyeccion (FETCH) destruye los metadatos que necesitan los post-filtros"
kind: task
status: review
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
- **Estado:** 🟡 EN REVISION (implementado y verificado; falta la review P2-01 de otro agente)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes (4/4 completados) + review P2-01

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

## Evidencia verificada (step 1 — antes de tocar nada)

- `src/physical_plan/project.rs:31-43` (pre-fix): `PhysicalProject::next` hace
  `node.relational = projected` con **solo** los campos de `self.fields` ⇒ todo
  campo no proyectado desaparece.
- `src/executor.rs:344-359` (pre-fix): `node_is_valid_at` lee
  `FIELD_VALID_AT_MS` → fallback `FIELD_CREATED_AT_MS`; si no hay ninguno,
  `let Some(valid_at) = valid_at else { return true; }` ⇒ **sin metadatos =
  siempre valido**.
- `src/executor.rs:301-324` (pre-fix): `filter_low_confidence_summaries` lee
  `node.relational.get("type")` ⇒ sin `type` no detecta el `SemanticSummary`.
- `src/executor.rs:660-673` (pre-fix): el prune de `ROLE` lee
  `node.relational.get("_owner_role")` ⇒ sin el campo, `role_match` es `false`
  ⇒ se poda **toda** la fila (salvo `ROLE "admin"`).
- `confidence_score` **no** es un campo relacional: es un campo de la struct
  `UnifiedNode` (`src/node/unified.rs:42`, persistido en la cabecera del
  nodo) ⇒ la proyeccion no lo destruye; lo que falta es `type`.
- El orden real es: `Scan` → `Filter` → `Sort` → **`Project`** → `Limit`
  (`src/planner.rs:444-456`) y los post-filtros se aplican **despues** del
  plan (`src/executor.rs:277-297` para `SELECT`/`QUERY`, `src/executor.rs:657-676`
  para el prune de `ROLE`) ⇒ los tres leen la fila **ya proyectada**.

## Decisión (step 2): **allowlist de metadata del sistema** en la proyección

Elegida: la **allowlist** (la opción que el propio task file preferia: "no
cambia el plan").

- **No (a) mover los filtros dentro del plan**: el plan se compila en
  `src/planner.rs::optimize_and_compile` — fichero **excluido** de este task
  (trabajo en curso de otros agentes) — y el prune de `ROLE` no es un
  operador del plan sino una comprobacion dentro del bucle de
  `Executor::execute_plan`. Mover los filtros exigiria tocar el planner,
  reordenar operadores y tocar el CBO/`cost_estimator` (tambien excluidos):
  un cambio de plan con el doble de superficie de riesgo.
- **Sí (b) la allowlist**: el `Project` sigue siendo el ultimo operador del
  plan; el cambio es local a `PhysicalProject` + las dos lecturas del
  executor; es aditivo y reversible; y no toca el CBO.

La allowlist se declara en **un solo sitio** (`SYSTEM_METADATA_FIELDS` en
`src/physical_plan/project.rs`). Los nombres de los dos campos no
`__vanta_` (`type`, `_owner_role`) son consts (`FIELD_TYPE`,
`FIELD_OWNER_ROLE`) **usados por la propia allowlist y por el executor** ⇒
la lista y las lecturas no pueden divergir.

## Implementación (step 3)

| Fichero | Cambio |
|---|---|
| `src/physical_plan/project.rs` | `SYSTEM_METADATA_FIELDS` (allowlist, un solo sitio) + `FIELD_TYPE` / `FIELD_OWNER_ROLE`; `PhysicalProject::next` conserva la allowlist **ademas** de los campos proyectados (sigue haciendo `node.relational = projected` ⇒ los campos de usuario no pedidos se siguen quitando) |
| `src/physical_plan/mod.rs` | re-export de `SYSTEM_METADATA_FIELDS`, `FIELD_TYPE`, `FIELD_OWNER_ROLE` + test unitario de la proyeccion |
| `src/executor.rs` | `filter_low_confidence_summaries` y el prune de `ROLE` leen los nombres desde los consts (fuente unica); 4 tests de executor nuevos |

Sin tocar: `src/parser/**`, `src/planner.rs`, `src/physical_plan/vector.rs`,
`src/cost_estimator.rs`, `src/index/**`, `src/crypto*`, `src/storage/**`,
`src/config.rs`, `src/lib.rs`, `benches/**`, `tests/encryption_chaos.rs`.

## Tests nuevos

| Test | Tipo | Que fija |
|---|---|---|
| `src/executor.rs::tests::as_of_still_filters_when_the_query_projects_fields` | integracion (Rust, motor real) | `AS OF` + `FETCH` respeta la ventana de validez **y** deja pasar al nodo sin metadatos de validez (RED: devolvia `[1, 2]`, esperaba `[1]`) |
| `src/executor.rs::tests::low_confidence_summaries_are_filtered_when_the_query_projects_fields` | integracion | el filtro de confianza de `SemanticSummary` sigue filtrando con `FETCH` y no toca los `Doc` sanos (RED: devolvia `[1]`, esperaba `[]`) |
| `src/executor.rs::tests::role_prune_still_applies_when_the_query_projects_fields` | integracion | la poda por `ROLE` sigue aplicando con `FETCH` (FIND-559) (RED: devolvia `[]`, esperaba `[1]`) |
| `src/executor.rs::tests::query_without_projection_still_returns_every_field` | integracion | invariante 1: una query **sin** proyeccion no pierde ningun campo (invariante de no-regresion: verde en RED por diseno, el camino sin proyeccion no se toca) |
| `src/physical_plan/mod.rs::tests::test_physical_project_keeps_system_metadata_but_drops_user_fields` | unit | la proyeccion conserva la metadata del sistema **y sigue quitando** los campos de usuario no pedidos (`secret` eliminado) (RED: "system metadata `type` must survive the projection") |

Ciclo TDD: RED reproducido antes del fix (4 de los 5 tests fallan con la
evidencia exacta de arriba), GREEN con el cambio minimo (la allowlist),
REFACTOR (los nombres de campo pasan a consts compartidos; `cargo fmt`).

## Invariantes: verificados / pendientes

| # | Invariante | Estado | Evidencia |
|---|---|---|---|
| 1 | un `SELECT *` o un `FROM` **sin** `FETCH` devuelve exactamente lo mismo que antes | ✅ | sin proyeccion **no hay** operador `Project` (`src/query.rs:371-379` y `:619-621`) ⇒ ese camino no se toca. Test `query_without_projection_still_returns_every_field` + la suite AS OF preexistente (`as_of_filters_nodes_by_valid_window_with_known_t`, `as_of_after_where_also_applies_and_includes_unscoped_nodes`, `select_as_of_filters_namespace_scan_records`) en verde |
| 2 | la proyeccion sigue **quitando** los campos de usuario que el usuario no pide | ✅ | `PhysicalProject::next` sigue haciendo `node.relational = projected` con (pedidos + allowlist); el campo `secret` se elimina en `test_physical_project_keeps_system_metadata_but_drops_user_fields` |
| 3 | `AS OF` sobre un nodo **sin** metadatos de validez sigue valiendo | ✅ | la proyeccion no *añade* metadatos: un nodo sin `__vanta_valid_at_ms`/`__vanta_created_at_ms` sigue sin ellos ⇒ `node_is_valid_at` sigue devolviendo `true`. Cubierto en `as_of_still_filters_when_the_query_projects_fields` (el nodo 9 entra en el resultado) |
| 4 | el filtro de confianza solo aplica a `SemanticSummary` | ✅ | el filtro **no se ha tocado** (sigue leyendo `type == "SemanticSummary" && confidence_score < 0.4`); el `Doc` sano sobrevive a `FETCH` en `low_confidence_summaries_are_filtered_when_the_query_projects_fields` |
| 5 | la allowlist esta declarada en un solo sitio | ✅ | `SYSTEM_METADATA_FIELDS` en `src/physical_plan/project.rs`; el executor referencia `FIELD_TYPE`/`FIELD_OWNER_ROLE` (los mismos consts que la allowlist) ⇒ lista y lecturas no pueden divergir |
| — | **delta visible al usuario** (a confirmar por el revisor) | ⚠️ | una query **proyectada** ahora devuelve tambien `type` (y `__vanta_valid_at_ms`/`__vanta_invalid_at_ms`/`__vanta_created_at_ms` si el nodo las tiene) ⇒ el `NodeRecord.fields` de `FETCH name` lleva mas campos que antes. **No es una fuga nueva**: sin proyeccion ya se devolvian todos. Es el precio de que los post-filtros se apliquen despues del plan (la opcion que el task file sanciona) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 1) un `SELECT *` o un `FROM` **sin** `FETCH` devuelve exactamente lo mismo que antes (no cambiar el camino sin proyeccion); 2) la proyeccion sigue **quitando** los campos de usuario que el usuario no pide (no basta con no borrar nada: hay que borrar los de usuario y conservar solo la allowlist del sistema); 3) `AS OF` sobre un nodo **sin** metadatos de validez sigue valiendo (no se inventa la semantica de un nodo legacy); 4) el filtro de confianza solo aplica a `SemanticSummary` (no generalizarlo).
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit --lib physical_plan`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-552: La proyeccion (FETCH) destruye los metadatos que necesitan los post-filtros` |
| `lastAction` | Implementado + verificado (fmt/clippy/tests); task file actualizado; **falta la review P2-01** |
| `result` | 🟡 IMPLEMENTADO — la proyeccion conserva una allowlist de metadata del sistema (`SYSTEM_METADATA_FIELDS`), de modo que `AS OF` y el filtro de confianza de `SemanticSummary` se siguen aplicando con `FETCH` (y la poda por `ROLE`, FIND-559); la proyeccion sigue quitando los campos de usuario no pedidos |
| `nextAction` | Review P2-01 por un agente distinto (`## Review`) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda introducida por este task. Queda **una linea de documentacion**
pendiente (no es deuda de codigo): `docs/api/IQL.md:105` describe `FETCH` como
"Projection: return only these fields" y, con la allowlist, la proyeccion
devuelve ademas la metadata del sistema. `docs/api/**` estaba fuera del scope de
ficheros de este task ⇒ lo decide `vanta-docs` o el revisor (ver
`## NOTICED BUT NOT TOUCHING` en `## Notas`).

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
| Pendientes de ejecución (downhill) | 0 (4/4 steps hechos) |
| % completado | 100% (implementado; review P2-01 pendiente) |

## Steps

1. ✅ **Verificar la evidencia** (sin editar): confirmada — `src/physical_plan/project.rs:31-43`
   borra los campos no proyectados; `src/executor.rs:344-359` devuelve `true` cuando
   falta `__vanta_valid_at_ms`/`__vanta_created_at_ms`; `src/executor.rs:301-324` lee
   `type`; `src/executor.rs:660-673` lee `_owner_role`. Detalle en `## Evidencia verificada`.
2. ✅ **Test primero**: 5 tests nuevos, **4 en RED** antes del fix (ver `## Tests nuevos`
   con la salida exacta del fallo) + 1 invariante de no-regresion verde en RED por diseno.
3. ✅ **Fix**: **allowlist de metadata del sistema** en la proyeccion
   (`SYSTEM_METADATA_FIELDS` en `src/physical_plan/project.rs`), declarada en un solo
   sitio; los nombres de campo del executor salen de los mismos consts. Decision y
   alternativa descartada en `## Decisión (step 2)`.
4. ✅ **Verificar**: `cargo fmt -p vantadb --check` exit 0 y `rustfmt --check` exit 0 sobre
   los 3 ficheros de este task; `cargo clippy -p vantadb --all-targets -- -D warnings`
   **exit 0** (crate entero); los 5 tests nuevos en verde (`5 passed`); `physical_plan` +
   `executor::tests` completos en verde (`126 passed` con `--profile ci-windows`); suite
   completa del lib en verde (`2524 passed, 0 failed`).

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)
- **Checklist para el revisor:**
  1. ¿La allowlist de metadata del sistema en `PhysicalProject` es la decision correcta
     frente a mover los post-filtros dentro del plan (`src/planner.rs` esta excluido de
     este task)? Ver `## Decisión (step 2)`.
  2. ¿La allowlist esta **completa**? Los post-filtros que leen la fila ya proyectada son
     tres: valid-time (`__vanta_valid_at_ms`, `__vanta_created_at_ms`,
     `__vanta_invalid_at_ms`), confianza (`type` + `confidence_score`, que es un campo de
     la struct, no relacional) y `ROLE` (`_owner_role`). ¿Falta algun filtro post-plan
     que lea un campo relacional? (grep: `src/executor.rs` -> `node.relational.get`).
  3. ¿El **delta visible al usuario** es aceptable? Una query proyectada devuelve ahora
     tambien `type` (y `__vanta_*` si existen) en el `NodeRecord.fields`. Ver la ultima
     fila de `## Invariantes: verificados / pendientes`.
  4. ¿La invariante 3 (`AS OF` sobre un nodo sin metadatos de validez sigue valiendo) esta
     realmente cubierta, o hace falta un caso explicito para el nodo **v1** (con
     `__vanta_created_at_ms` pero sin `__vanta_valid_at_ms`)? El test actual cubre el nodo
     **sin** metadatos; el v1 con `created_at` esta cubierto por
     `select_as_of_filters_namespace_scan_records` (sin proyeccion).
  5. ¿El scope respetado el del task (solo `src/physical_plan/project.rs`,
     `src/physical_plan/mod.rs`, `src/executor.rs`)? `src/parser/**`, `src/planner.rs`,
     `src/physical_plan/vector.rs`, `src/cost_estimator.rs`, `src/wal.rs`, `src/index/**`,
     `src/crypto*`, `src/storage/**`, `src/config.rs`, `src/lib.rs`, `benches/**` y
     `tests/encryption_chaos.rs` **no** se han tocado.

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7). **Este
  agente NO hace commits** (los hace el orquestador).
- **FIND-559** (poda por `ROLE` con proyeccion) queda **arreglado de paso** por la misma
  allowlist, con test propio (`role_prune_still_applies_when_the_query_projects_fields`).
  El agente de FIND-559 debe confirmar que su fila puede cerrarse con esta evidencia.
- `cargo fmt -p vantadb --check` ⇒ **exit 0**; `rustfmt --check` sobre los 3 ficheros de
  este task ⇒ **exit 0**.
- `cargo clippy -p vantadb --all-targets -- -D warnings` ⇒ **exit 0** (crate entero verde).
  Durante la sesion hubo errores transitorios en ficheros de **otros agentes**
  (`src/parser/grammar.rs`, `src/wal.rs:1399` `clippy::suspicious_open_options`); ambos
  desaparecieron cuando esos agentes cerraron su trabajo. Con `--message-format=short` se
  verifico ademas que **no hay ni un solo diagnostico** en `src/physical_plan/project.rs`,
  `src/physical_plan/mod.rs` ni `src/executor.rs`.
- **Superficie publica**: los 3 consts (`SYSTEM_METADATA_FIELDS`, `FIELD_TYPE`,
  `FIELD_OWNER_ROLE`) son **`pub(crate)`**, no `pub` ⇒ la superficie publica del crate no
  cambia y el golden file `tests/api/public-api.txt` (gate `tests/api/public_api.rs`, job
  `public-api-snapshot`) **sigue valido sin regenerar**. (La primera version los tenia
  `pub`; se corrigio antes de cerrar el task.)
- Suite del lib: `cargo nextest run -p vantadb --lib` ⇒ **2524 tests run: 2524 passed
  (1 slow), 2 skipped, 0 failed, 0 aborted**.
- Subconjunto del area tocada: `cargo nextest run -p vantadb --profile ci-windows --lib -E
  'test(physical_plan) or test(executor::tests)'` ⇒ **126 passed, 0 failed** (el perfil
  `ci-windows` de `.config/nextest.toml` limita `test-threads = 2` justo para evitar el
  `os error 1455` de este entorno; con el perfil por defecto y 3 agentes compilando a la
  vez hay `ABORT` de Windows por stack overflow que **no** son de este cambio).
- Nota de entorno: con 3 agentes usando el `target/` compartido a la vez, el enlazador
  falla con `LNK1104` (el `.exe` esta en uso por el run de test de otro agente). Se
  reintenta hasta que el lock queda libre; ninguna verificacion de este task se ha
  saltado.
- **NOTICED BUT NOT TOUCHING:**
  - `src/physical_plan/vector.rs`, `src/cost_estimator.rs`, `src/planner.rs`,
    `src/parser/**`, `src/wal.rs` — trabajo en curso de otros agentes.
  - `docs/api/IQL.md:105` — la tabla de IQL describe `FETCH` como "Projection:
    return only these fields"; con la allowlist la proyeccion devuelve ademas la
    metadata del sistema (`type` y `__vanta_*` si existen). **Falta una linea
    aclarandolo** (una linea, no mas): lo decide `vanta-docs` o el revisor, no
    este agente (`docs/api/**` estaba fuera del scope de ficheros de este task).
  - `src/sdk/api/graph.rs:328-333` — el `NodeRecord` expone `fields` tal cual; si en el
    futuro se quiere que la metadata del sistema **no** salga al usuario, el sitio natural
    es ahi (fuera del alcance de este task).
  - `src/sdk/serialization/graph_types.rs:102-106` — `unified_to_record` no filtra campos
    `__vanta_`; sin proyeccion ya devolvian todos (no es un cambio de este task).
