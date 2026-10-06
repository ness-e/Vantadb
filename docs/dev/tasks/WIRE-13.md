---
title: "WIRE-13: IQL agregaciones (`COUNT`/`SUM`/`GROUP BY`)"
kind: task
description: "Agregaciones básicas (COUNT/SUM/GROUP BY) end-to-end (parser→AST→planner→executor) con tests (parse + ejecución + tipos de resultado) y docs/api/IQL.md actualizado; sin breaking; resultado serializable en el AST JSON."
---

# WIRE-13: IQL agregaciones (`COUNT` / `SUM` / `GROUP BY`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 69, F0-expandido — campaign taskId `69`)
- **Fuente:** Backlog `:180` (fila WIRE-13) + plan Task 69
- **Esfuerzo:** 🟡 3-5d · **Prioridad:** 🟡 · **Tipo:** Rust (parser/planner/executor IQL + registry) + `docs/api/`
- **Turns estimados:** 15-30
- **Creado:** 2026-10-06 · **last-synced:** 2026-10-06
- **Estado:** ✅ COMPLETED (steps 1-5 ✅; commit `753b9f78` + progreso `05f20b06`; campaign taskId 69 cerrado)
- **Incógnitas (uphill):** 0 (scope + shape de resultado + interacciones resueltas en §Spec con evidencia) · **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Executor::execute_select` ← HTTP `/api/v2/query`, MCP `query_iql`, bindings `query()`, CLI/TUI REPL; `parse_select` ← `parse_statement` (`grammar.rs:621`) + subqueries (`parse_subquery_condition_inner`, `grammar.rs:440`) |
| Callees | `SelectStatement::into_logical_plan` (`query.rs:254`) → `planner::optimize_and_compile` (catch-all `_ => pending_extensions`, `planner.rs:250`) → `OperatorRegistry::compile` (`planner.rs:438-443`) → `PhysicalAggregate` (nuevo); `cost_estimator::estimate_operator` (arm forzado, patrón `Dedup`/`Offset`); executor `iql_parse_error_message` (mensajes estables de las restricciones) |
| Implicaciones | Contrato IQL público (gramática + AST JSON + `IQL_VERSION` 3→4); `SelectStatement` gana campos aditivos (`aggregates`, `group_by`) → 11 construction sites (grammar + 10 tests en `tests/logic/joins.rs`, 1 en `src/executor.rs` tests) se actualizan; `LogicalOperator` gana variante (adición minor, patrón `Dedup`/`Offset`); snapshot público `tests/api/public-api.txt` (símbolos nuevos); **sin** cambios en storage/WAL/índices; sin cambios de wire on-disk |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/parser/lexer.rs` (179L), `src/parser/grammar.rs` (680L), `src/query.rs` (967L), `src/operator_registry.rs` (593L), `src/planner.rs` (989L), `src/physical_plan/mod.rs` (814L), `src/physical_plan/offset.rs` (55L), `src/physical_plan/dedup.rs` (167L), `src/physical_plan/project.rs` (94L), `src/physical_plan/join.rs` (≤270L), `src/physical_plan/filter.rs` (≤133L), `src/executor.rs` (1-700L funcional + tests Select), `src/cost_estimator.rs:180-319`, `src/parser/mod.rs` (≤1436L + 1543-1622), `src/node/field.rs` (199L), `src/node/unified.rs:1-130`, `docs/api/IQL.md` (424L), `tests/logic/parser.rs` (133L), `tests/logic/joins.rs:55-129`, `tests/logic/snapshots/parser__dql_query_ast.snap`, `tests/api/public_api.rs:1-80`, `tests/operator_registry_extension.rs` (197L)
- **Referencias hacia dentro:** `grammar.rs` importa lexer + `super::{iql_supports, IQL_VERSION_MIN_*}`; `planner.rs:250` catch-all → `pending_extensions` → `operator_registry::compile` (`:438-443`, extensions post-chain DESPUÉS de `Limit`); `cost_estimator.rs:268-290` arms forzados `Dedup`/`Offset` delegan al registry; `executor.rs:92-106` `iql_parse_error_message` (keyed por `ErrorKind` + prefijo del input); `SelectStatement` consumido por `execute_select` (`executor.rs:226-233`), `into_logical_plan` (2 impls), `tests/logic/joins.rs`
- **Referencias entrantes:** `parse_select` ← `parse_statement` + subqueries; `LogicalOperator` consumido por planner/cost/registry/governor/executor (0 usos fuera de `src/` + tests); `RESERVED_KEYWORDS` ← `non_keyword_ident` (aliases) + autocomplete (sin tests que pinen contenido exacto); `IQL_VERSION` referenciado en `docs/api/VERSIONING.md:44`, `docs/api/openapi.yaml:152`, `docs/api/IQL.md:5,15,23-25`
- **Veredicto impacto:** **medio** — gramática + AST públicos (adición compatible: campos nuevos al final + cláusula opt-in + variante aditiva); planner/executor **sin** edición semántica (catch-all ya existe; solo arm forzado en `cost_estimator` y mensajes de error estables en executor); construction sites + snapshot público actualizados en el mismo commit; cero migración de datos

## Contrato

"Agregaciones básicas (`COUNT`/`SUM`/`GROUP BY`; alcance exacto en DISCOVERY) end-to-end con tests (parse + ejecución + tipos de resultado) y `docs/api/IQL.md` actualizado; sin breaking; resultado serializable en el AST JSON (patrón IQL.md `:303-339`)."

Comando de verificación (cierre): `cargo nextest run --profile audit -p vantadb --lib` + `--test parser` + `--test executor` + `--test operator_registry_extension` + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + gates docs + snapshot público.

## Spec (SDD — Phase 1b: feature-add, símbolos/contratos públicos nuevos)

> Gate P/D: la solución agrega contrato público (cláusulas IQL + campos AST + variante `LogicalOperator` + tipos nuevos). El plan Task 69 delegó expresamente a DISCOVERY: scope (pre-mortem #1: "fijar mínimo COUNT/SUM/GROUP BY"), shape de resultado (#2: "decidir shape antes de codear") e interacción con LIMIT (#3: "orden de cláusulas declarado") → se resuelven **por evidencia** (opción 2 del gate mecánico spec-first, precedente WIRE-12), sin ronda `question` adicional.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Superficie gramatical | A) extender `SELECT` (SQL-parity: `SELECT COUNT(*), SUM(x) FROM t GROUP BY g`) · B) extender `FROM` + `FETCH COUNT(*)` (FETCH documentado como "return only these fields", lista de idents) · C) statement nuevo `AGGREGATE` (duplica WHERE/JOIN/pipeline) | ✅ **A** — `SELECT` ya es "the read surface with projections" (`IQL.md:116`); SQL-parity es el criterio canónico de la casa (`LIMIT`/`OFFSET` = SQL order, WIRE-12 §Spec #2). B/C rechazados: reutilizan/duplican superficie con menos naturalidad |
| 2 | Alcance de funciones | A) `COUNT(*)` + `COUNT(field)` + `SUM(field)` · B) solo `COUNT(*)` · C) + `AVG`/`MIN`/`MAX`/`HAVING` | ✅ **A** — contrato pide COUNT/SUM; `COUNT(field)` = semántica SQL no-NULL, ~3 líneas extra; C diferido explícitamente por stop condition del plan ("COUNT+GROUP BY mínimo + FIND del resto") → FIND-316 |
| 3 | Shape de resultado | A) `UnifiedNode` sintético por grupo (campos: group key + agregados), reusa todo el path de serialización HTTP/MCP/bindings · B) nuevo `ExecutionResult::Aggregated` (rompe `execute_plan → Vec<UnifiedNode>` y toca executor + sdk + bindings) | ✅ **A** — el stream Volcano es `UnifiedNode` (`query.rs:570-577`); B multiplica ediciones y rompe consumidores de `Read`. Nodos sintéticos: `id = 0`, solo campos de salida (group key + `count`/`count_<f>`/`sum_<f>`) |
| 4 | Orden de salida de grupos | A) first-seen (orden de aparición del scan — contrato de `Dedup` "first-seen wins") · B) orden por key (requiere `Ord` en `FieldValue` — **no existe**, solo `Hash + Eq`, `node/field.rs:6-32`) | ✅ **A** — B exigiría derivar `Ord` en un tipo core (fuera de scope) o sort por string serializado (invención). First-seen es determinista dado el orden del scan |
| 5 | Mecanismo de operador | A) registry C2S6 (variante + `PhysicalAggregate` + `register`) · B) brazo nombrado built-in en planner (edita el match probado; contradice `operator_registry.rs:4-8`) | ✅ **A** — el plan cita el registry como habilitador real (`operator_registry.rs:222`); catch-all del planner ya rutea variantes desconocidas (`planner.rs:250`); `Dedup`/`Offset` precedentes; cero ediciones semánticas a planner/executor |
| 6 | Interacción con `LIMIT`/`OFFSET` | A) rechazo en parse (Failure estable) · B) semántica input-limit (extension post-chain limita la ENTRADA → `LIMIT 5` limitaría filas antes de agregar: resultado silenciosamente incorrecto) · C) group-pagination dentro del operador (inventa semántica "primeros N grupos por key" + requiere orden de grupos que `Ord` no soporta) | ✅ **A** — B es incorrecto silencioso (jamás); C inventa semántica no pedida. Rechazo con mensaje estable = patrón de estrictez WIRE-12/SCH-03 ("fail loud, never silent drop"). Paginación de agregados → FIND-316 |
| 7 | Interacción con `AS OF` | A) rechazo en parse · B) filtrar valid-time dentro del operador (duplica `node_is_valid_at` — `executor.rs:292-307` — y rompe la leaf-discipline de `physical_plan`, que solo depende de `query`/`node`/`error`) | ✅ **A** — el filtro valid-time corre **post-plan** (`executor.rs:281-289`): sobre filas sintéticas sería un no-op silencioso → COUNT incorrecto. Precedente exacto: subquery + `AS OF` = Failure (`grammar.rs:441-448`, SCH-03). Valid-time-aware aggregation → FIND-316 |
| 8 | `GROUP BY` sin agregados | A) parse Failure ("GROUP BY requires at least one aggregate") · B) tratarlo como DISTINCT (semántica nueva) | ✅ **A** — contrato mínimo; B no pedido (scope discipline) |
| 9 | Multi-field `GROUP BY a, b` | A) rechazo en parse ("single field only") · B) `Vec<String>` + key compuesta | ✅ **A** — mínimos primero (stop condition); B → FIND-316. Guard obligatorio: sin él `GROUP BY a, b` degradaría silenciosamente a `a` (trailing ignorado, clase FIND-312) |
| 10 | `GROUP BY` en `FROM`/`MATCH` | A) rechazo en parse ("aggregation is a SELECT surface") · B) dejar que quede como trailing silencioso (clase FIND-312) | ✅ **A** — `FROM t GROUP BY g` es el error de usuario más probable (FROM es la superficie de lectura); dejarlo silencioso devuelve "todo" sin agrupar (resultado incorrecto silencioso). Guard targetizado (mismo espíritu que `reject_trailing_after_pagination`, WIRE-12) |
| 11 | Tipos de salida | `COUNT` → `Int` siempre; `SUM`: `Int` si todos los aportes son Int (con `saturating_add`), `Float` si hay mezcla Int/Float, `Null` si ningún aporte numérico; valores no numéricos/missing/Null se **ignoran** (sin coerción — precedente type-strict `IQL.md:62-70`); sin GROUP BY + input vacío → 1 fila (`COUNT` 0 / `SUM` null — SQL) | ✅ decidido-por-evidencia (SQL; type-strict de la casa) |
| 12 | Nombres de campos de salida | group key → nombre exacto del campo; `COUNT(*)` → `count`; `COUNT(f)` → `count_<f>`; `SUM(f)` → `sum_<f>` (deterministas, usables desde JSON/bindings; sin `AS` alias en este alcance) | ✅ decidido-por-evidencia (sin aliasing en el scope; nombres derivados documentados en `IQL.md`) |
| 13 | Versionado | Bump `IQL_VERSION` 3→4 + `IQL_VERSION_MIN_AGGREGATION = 4` · sin bump | ✅ bump — "Bump when the grammar changes in a way consumers must detect (new clause)" (`IQL.md:13-30`; patrón PROFILE/AS OF/PAGINATION) |
| 14 | Keyword `GROUP` | A) agregar `GROUP` a `RESERVED_KEYWORDS` · B) no agregar | ✅ **A** — **requerido**: el alias opcional de `FROM t` (`opt(non_keyword_ident)`) consumiría `GROUP` como alias y `BY g` quedaría trailing → query SIN agrupar silenciosa. `BY` NO se reserva (precedente `RANK BY`); `COUNT`/`SUM` tampoco (no aparecen en posición de alias; mantiene compat de aliases uppercase) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) queries existentes sin agregación parsean idéntico (campos `aggregates: []`/`group_by: None`); (2) `LogicalOperator::Limit/Offset/Dedup` conservan shape y semántica; (3) `planner::optimize_and_compile` y `executor` **no** editan su semántica (solo arm forzado en `cost_estimator` + mensajes de error); (4) registry aditivo: `register("aggregate", …)` sin duplicados; (5) `RESERVED_KEYWORDS` solo gana `GROUP` (aliases lowercase intactos); (6) sin push (Regla 7); snapshot público actualizado 1 vez en el mismo commit; (7) nodos sintéticos de agregación nunca se insertan en storage (solo output del operador)
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --lib` · `--test parser` · `--test executor` · `--test operator_registry_extension` · `cargo fmt -p vantadb -- --check` · `cargo clippy -p vantadb --all-targets -- -D warnings` · `node scripts/docs/check-links.mjs` + `check-docs.mjs`
- **Deuda pendiente:** ninguna al abrir (FIND-316 derivado: AVG/MIN/MAX/HAVING + multi-field GROUP BY + paginación de grupos + valid-time-aware aggregation)

## Recitation (canónico)

```
=== RECITATION ===
Objetivo activo: WIRE-13 — IQL agregaciones COUNT/SUM/GROUP BY end-to-end
Estado: completed (steps 1-5 ✅; commit 753b9f78 + progreso 05f20b06; campaign taskId 69 cerrado)
Última acción: review adversarial r2 ✅ APPROVE (C-1/M-1/M-2/M-3 + L-1..L-4 resueltos y re-probados; R-1 Low pre-existente → subcase FIND-312; N-2 atendido en docs) → verify final (lib 2440/2440, integración 20/20, fmt/clippy, public-api 1/1, gates docs) → commit LOCAL 753b9f78 → campaign completed → skill progreso (05f20b06) → learnings/decisions escritos
Resultado: OK
Próxima acción: ninguno — tarea cerrada. Próxima del plan: la decide el orquestador.
Contrato: ver ## Contrato
Invariantes: sin agregación → parseo idéntico; planner/executor sin edición semántica; registry aditivo; sin push
Deuda: ninguna (FIND-316 diferido; R-1 anotado en FIND-312)
Próxima tarea si completa: siguiente pendiente del plan (SRV-10 ya cerró en ca3f00da)
last-synced: 2026-10-06
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** Sin deuda nueva — variante + campos aditivos; el código nuevo cierra un gap real (0 hits de agregación en IQL). Deuda diferida registrada como FIND-316 (resto de agregaciones: AVG/MIN/MAX/HAVING, multi-field GROUP BY, paginación de grupos, valid-time-aware) — ninguna introducida por este PR.

## Definition of Done (3 niveles)

- **Task:** contrato verificado con tests parse+ejecución+tipos (RED→GREEN) + `docs/api/IQL.md` sincronizado (Regla 3) + fmt/clippy scoped verdes + snapshot público refrescado
- **Commit:** atómico, `feat(iql): WIRE-13 — …`, solo paths del scope, verificación mecánica previa (nunca auto-reporte)
- **Release:** changelog minor — lo maneja release-plz (no manual); sin cambios de wire on-disk

## Herramientas necesarias
- `cargo check/nextest/clippy/fmt`, `campaign_verify_cmd`, `pwsh dev-tools/heavy-test-lock.ps1` (pruebas pesadas serializadas)
- codegraph / codebase-memory-mcp (blast radius), `pwsh dev-tools/ocr-review.ps1`
- **Skills cargadas (SDP):** campaign-executor + progreso (base auto) · api-and-interface-design + documentation-and-adrs (pins de policy "API docs") · test-driven-development + incremental-implementation (lifecycle BUILD) · rust-write-tests (dominio Rust tests) · source-driven-development (validación contra código real). `writing-guidelines`/`writing-plans` (base docs) evaluadas: subsumidas por `documentation-skill` para este update de API reference.

## Investigation Notes (DISCOVERY — evidencia)

- **Estado real verificado (2026-10-06, post-WIRE-12 `22585e1b`):** `rg "COUNT|SUM|GROUP|AVG" src/parser/` = **0 hits**; `rg "Aggregate" src/` = solo `cli_handlers/export_md.rs` (ajeno). NO existe pipeline de agregación IQL. Habilitadores reales verificados: Volcano (`executor.rs:528-584`) + `OperatorRegistry` C2S6 (`operator_registry.rs:120-218`, extensions pre-registradas `dedup`/`offset`, compiladas post-chain en `planner.rs:438-443`) + `namespace_stats` como agregación SDK fuera de IQL (precedente de intención).
- **La premisa del compact ("habilitado por el pipeline de agregación existente") era falsa en su interpretación literal:** el habilitador es el **registry**, no un pipeline de agregación previo (`[a verificar en DISCOVERY]` → verificado y corregido).
- **Patrón de extensión (C2S6, `IQL.md:400-417`):** variante + archivo físico en `physical_plan/` + una línea `register`; el planner catch-all (`planner.rs:250`) las compila **después** de `Limit`/`Project`/`Sort` → la interacción con paginación se decide en §Spec #6 (rechazo).
- **`FieldValue` es `Hash + Eq` pero NO `Ord`** (`node/field.rs:6-32`): orden de grupos = first-seen (mismo contrato que `Dedup`), no orden por key.
- **Alias guard:** `opt(non_keyword_ident)` en `parse_select` (`grammar.rs:499`) consumiría `GROUP` como alias → `GROUP` entra a `RESERVED_KEYWORDS` (§Spec #14). `BY` no (precedente `RANK BY`: `RANK` reservada, `BY` no).
- **Filtros post-plan:** `AS OF` (`executor.rs:281-289`) y `ROLE` (solo en `FROM`/`MATCH`, no en `SELECT`) filtran después del plan → para agregación: `AS OF` se rechaza en parse (§Spec #7); `ROLE` no aplica (no existe en `SELECT`).
- **Strictness de parse:** mensajes estables keyed por `(ErrorKind, prefijo del input)` en `iql_parse_error_message` (`executor.rs:92-106`). Las restricciones nuevas usan `ErrorKind::Fail` (distinto de `Verify`) + prefijo del clause para mensajes distinguibles.
- **Construction sites de `SelectStatement` (11):** `grammar.rs:571` (real) + `src/executor.rs:1447` (test) + `tests/logic/joins.rs` (10). Snapshot `parser__dql_query_ast.snap` es de `Query` (FROM) — **no cambia** (agregación es superficie SELECT).
- **Snapshot público:** `tests/api/public-api.txt` compara la superficie completa; símbolos nuevos (`AggregateFunc`, `LogicalOperator::Aggregate`, `PhysicalAggregate`, `AggregateCompiler/Cost`, `IQL_VERSION_MIN_AGGREGATION`) exigen refresh oficial (`VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api --run-ignored ignored-only`, con `CARGO_TARGET_DIR` fresco por el lock del exe del server MCP — nota WIRE-12).
- **Refs de versión a actualizar (Regla 3):** `docs/api/VERSIONING.md:44` (`IQL_VERSION = 3` → 4), `docs/api/openapi.yaml:152` (`= 3` → 4), `docs/api/IQL.md` (frontmatter `:5`, body `:15`, tabla `:23-25`). `UPGRADE.md:99` no se toca (fila histórica 0.7→0.8.0).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 (scope, shape, LIMIT/AS OF, orden, versionado → §Spec 14 decisiones) |
| Pendientes | 5 steps |
| % completado | 0% (DISCOVERY ✅) |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — N/A justificado: sin input externo nuevo en trust boundary (IQL ya valida en el borde parser); la strictness **aumenta** (rechazos loud de combinaciones no soportadas); sin auth/deps/FFI/red. `SUM` usa `saturating_add` (sin panic/wrap).
- [x] **PERFORMANCE** — Evaluado: el operador de agregación es blocking (drena el child en `open()`), O(n) sobre el stream ya materializado por Volcano; no se toca hot path de búsqueda/serialización ni se altera ningún operador existente. `ponytail:` agregación materializa grupos en memoria (techo: cardinalidad de grupos; upgrade: streaming con hash parcial si un bench lo justifica — Regla 9: sin benchmark no hay claim).

## Steps

### Step 1: RED — tests de contrato (parse + AST + registry + physical + ejecución)
- **Archivos:** `src/parser/mod.rs` (tests), `src/query.rs` (tests), `src/operator_registry.rs` (tests), `src/physical_plan/aggregate.rs` (nuevo — tests), `src/executor.rs` (tests), `tests/logic/parser.rs` (integración)
- **Acción:** escribir tests que fallan por la razón correcta (API inexistente → compile-fail): parse `COUNT(*)`/`COUNT(f)`/`SUM(f)`/`GROUP BY`; strictness (mixed fields, GROUP BY sin agregado, multi-field, AS OF+agg, LIMIT/OFFSET+agg, FROM+GROUP BY); version gate; plan emite `Aggregate` sin `Project`/`Limit`; registry `"aggregate"` (name + compile + cost); physical (count/sum/grupos/first-seen/vacío/reopen); ejecución end-to-end (count all, count field, sum int/float/mixed/skip non-numeric, group string/int, missing→null group, sin GROUP BY vacío → 1 fila, WHERE+agg, JOIN+agg); AST JSON de `SelectStatement` con `aggregates`/`group_by`
- **Verify:** `cargo check -p vantadb --tests` → **falló** con E0433 (`AggregateFunc`), E0599 (`Aggregate`), E0560/E0609 (`aggregates`/`group_by`), E0425 (`AggregateCompiler`, `IQL_VERSION_MIN_AGGREGATION`) = RED probado ✅
- **Estado:** ✅ COMPLETED

### Step 2: GREEN — gramática + AST + versión
- **Archivos:** `src/parser/lexer.rs` (`GROUP` reservada), `src/parser/grammar.rs` (`parse_aggregate_func` + `GROUP BY` + restricciones Fail + guard `FROM`), `src/parser/mod.rs` (IQL_VERSION=4 + MIN_AGGREGATION + tests), `src/query.rs` (`AggregateFunc` + `output_name()` + campos `aggregates`/`group_by` + emisión en `into_logical_plan`), construction sites (`tests/logic/joins.rs`, tests de `src/executor.rs`), `src/executor.rs` (`iql_parse_error_message`: ramas `Fail`)
- **Acción:** cláusulas estrictas (peek→función requerida); `GROUP BY` en posición SQL (tras WHERE, antes de `WITH TEMPERATURE`); rechazos loud (Fail kind) para combinaciones no soportadas; `RESERVED_KEYWORDS` + `GROUP`; bump de versión
- **Verify:** `cargo check -p vantadb --tests` ✅ exit 0; parse tests verdes en la corrida lib (Step 3)
- **Estado:** ✅ COMPLETED

### Step 3: GREEN — pipeline (variante + físico + registry + cost)
- **Archivos:** `src/query.rs` (`LogicalOperator::Aggregate`), `src/physical_plan/aggregate.rs` (nuevo), `src/physical_plan/mod.rs`, `src/operator_registry.rs` (compiler + cost + register + tests), `src/cost_estimator.rs` (arm forzado)
- **Acción:** `PhysicalAggregate` Volcano (drena child en `open()`, acumula por grupo first-seen, emite nodos sintéticos `id=0`); registro `"aggregate"`; arm de cost que delega al registry (patrón `Dedup`/`Offset`); ejecución end-to-end verde
- **Verify:** `cargo nextest run --profile audit -p vantadb --lib` → **2435/2435** ✅ (+36 tests nuevos; 2 skipped pre-existentes) · integración (`--test parser --test executor --test joins --test operator_registry_extension`, no-cli) → **20/20** ✅
- **Estado:** ✅ COMPLETED

### Step 4: Docs — `docs/api/IQL.md` + AST JSON + refs de versión
- **Archivos:** `docs/api/IQL.md`, `docs/api/VERSIONING.md`, `docs/api/openapi.yaml`
- **Acción:** versión 4 en frontmatter/tabla; sintaxis + semántica de agregaciones (nombres de campos, tipos, first-seen, restricciones); ejemplo AST JSON; refs `IQL_VERSION` 3→4
- **Verify:** `node scripts/docs/check-links.mjs` ✅ · `check-docs.mjs` ✅ · `gen-index.mjs --write` ✅ · `scripts/validate-docs-coverage.ps1` ✅ 0 gaps
- **Estado:** ✅ COMPLETED

### Step 5: Cierre — verify contrato + OCR + review P2-01 + commit + campaña
- **Archivos:** `docs/dev/tasks/WIRE-13.md`, `docs/dev/Backlog.md` (FIND-316), `tests/api/public-api.txt` (refresh)
- **Acción:** suite scoped con LOCK + fmt + clippy; snapshot público refrescado; OCR delegation; review adversarial (vanta-review, tier adversarial por `src/parser/**`); commit LOCAL `feat(iql):`; cierre campaign taskId `69`; `skill progreso`
- **Verify:** fmt ✅ · clippy ✅ · lib 2440/2440 ✅ · integración 20/20 ✅ · public_api 1/1 ✅ · docs gates ✅ · OCR sin Critical/High ✅ · review P2-01 adversarial ronda 1 REQUEST CHANGES → ronda 2 ✅ APPROVE · commit LOCAL `753b9f78` · campaign taskId 69 completed · progreso `05f20b06`
- **Estado:** ✅ COMPLETED

## Dependencias
- WIRE-12 ✅ COMPLETED (`22585e1b`) — base releída desde HEAD (paginación + registry Offset).
- **Coordinación:** SRV-10 puede estar en vuelo (cifrado server — área disjunta); pathspec SIEMPRE; lock de pruebas pesadas obligatorio.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (sesión `ses_eef119650ffeZv2X1omz1Hzc6j`, 2026-10-06) — tier **adversarial** (el diff toca `src/parser/**`)
- **Enfoque:** mecanismo `Aggregate` (registry C2S6 post-chain); paths silenciosos (trailing/orden/subqueries/AST programático); semántica del operador; docs vs código; invariantes
- **Cómo se probó:** re-ejecutó lib 2435/2435, integración 20/20, fmt/clippy, gates docs; `git diff src/planner.rs` vacío; public-api +74/−0; probes adversariales (7 queries silenciosas, colisión de key, AST programático) con lock adquirido/liberado; probes temporales borrados
- **Hallazgos y resolución (ronda 1 → 🔴 REQUEST CHANGES):**
  - 🔴 **C-1 — cláusulas tras la superficie de agregación se descartaban en silencio** (`GROUP BY c WHERE ...` → WHERE descartado, count incorrecto; `ORDER BY count LIMIT 1` → descartado sin error; `WITH TEMPERATURE 1.5 GROUP BY c` → GROUP BY descartado; `GROUP BY` malformado/doble → descartado). **Resuelto:** `reject_trailing_input` (rename de `reject_trailing_after_pagination`) ahora aplica a statements strict = paginación **o** `SELECT` con agregados (mismo patrón WIRE-12, tolera `;`, nivel statement → subqueries intactas) + 5 asserts de parser + 1 test e2e.
  - 🟡 **M-1 — colisión `GROUP BY <key>` vs nombre de salida de agregado pisaba la key** (`GROUP BY count` + `COUNT(*)`). **Resuelto:** rechazo en parse (Fail, mensaje estable extendido) + test + mención en IQL.md.
  - 🟡 **M-2 — path AST programático ignora `limit`/`as_of_ms`/`projections` con agregados.** **Resuelto:** `debug_assert!` del invariante en `into_logical_plan` + invariante documentado en los campos de `SelectStatement`.
  - 🟡 **M-3 — tests e2e declarados ausentes (JOIN+agg, COUNT(field)).** **Resuelto:** ambos agregados (el JOIN e2e requiere el campo relacional `id` del Address, espejo del helper de `joins.rs`).
  - 🟢 **L-1 — docs sobre-reclamaban compatibilidad v1–3** (`GROUP` ahora reservada). **Resuelto:** matizado en `IQL.md` §Language Version + §Aggregations.
  - 🟢 **L-2 — fallback frágil del mapper** (`Fail` sin prefijo heredaba el mensaje de mixed). **Resuelto:** mixed ahora falla al inicio del statement (prefijo `SELECT`, col 1) y el fallback es genérico ("invalid aggregation syntax").
  - 🟢 **L-3 — `GROUP BY a.city` no resuelve alias.** **Resuelto (docs):** límite declarado (match exacto por nombre, igual que proyecciones/WHERE).
  - 🟢 **L-4 — `close()` no propagaba al child (convención).** **Resuelto:** alineado con `PhysicalSort` (child cerrado en `close()`, no en `open()`).
  - Nits (NaN en keys/SUM, cast u64→i64): pre-existentes/documentables — sin acción (Low se descarta).
- **Veredicto ronda 1:** 🔴 REQUEST CHANGES → fixes aplicados → re-verify (lib **2440/2440**, integración 20/20, fmt/clippy, public-api 1/1, gates docs).
- **Ronda 2 (delta, misma sesión):** ✅ **APPROVE** — el revisor re-probó cada fix con evidencia propia: C-1 cerrado en los 7 paths originales + variantes (`;`/`;;` tolerados, subqueries canónicas intactas, `FROM … garbage` sigue OK = FIND-312 intacto); M-1 rechaza exactamente las colisiones y conserva keys no colisionantes; L-2 col 1 + fallback sin capturas falsas (6 sitios `Fail` enumerados, todos mapeados); L-4 lifecycle consistente (close sin open, re-open resetea). Lib 2440/2440, integración 20/20, fmt/clippy/gates docs verdes, golden +74/−0.
- **Residuales (no bloquean):** R-1 (Low, pre-existente clase FIND-312): trailing interno en subquery sin outer agregado → descartado silencioso; anotado como subcase en la fila FIND-312. N-1 (Nit): mensajes de trailing con rendering nom crudo — sugerencia futura de mapper (no aplicada, post-approve). N-2 (Nit): precisión doc del `;` — **atendido** en `IQL.md` (solo docs; gates re-verdes post-fix).
- **Checklist anti-hábitos tóxicos:** sin self-review (contexto fresco en ambas rondas) ✅ · sin fixes por el revisor ✅ · evidencia re-ejecutada ✅ · sin confiar en el resumen (cada fix re-probado) ✅ · sin tocar archivos ajenos ✅ · veredicto binario ✅

## Notas
- **Premisa corregida en DISCOVERY:** "pipeline de agregación existente" NO existía; el habilitador real es `OperatorRegistry` + Volcano (evidencia §Investigation Notes).
- **OCR delegation (advisory):** `pwsh dev-tools/ocr-review.ps1 -Format json` → rule groups 2 (`**/*.rs`: cost_estimator/operator_registry/tests) y 3 (query-DSL project rules: parser/physical_plan/query/executor) revisados contra el diff; sin Critical/High; Medium/Low: ninguno nuevo (FIND-316 ya ticketed). `dev-tools/heavy-test-lock.ps1` aparece como untracked ajeno (BENCH-02) — no se toca ni commitea.
- **Coordinación SRV-10 (committed durante la tarea):** SRV-10 cerró en `ca3f00da` mientras WIRE-13 estaba en vuelo; su commit incluyó `docs/index.md`/`docs/api/index.md`/`llms.txt` regenerados que contenían la descripción nueva de IQL (v4) — contaminación benigna de regeneración (el texto pertenece a esta tarea y `docs/api/IQL.md` entra en el commit de WIRE-13). Sin solape de archivos de código (SRV-10 tocó `src/server/**` + `vantadb-server/`).
- **Convención de posición de error:** `iql_error_position` (`executor.rs`) reporta el offset 0-based del fallo — `SELECT COUNT(*) FROM Invoice LIMIT 5` → col 29 (donde arranca `LIMIT`); los tests pinean esa convención (los mensajes estables son la interfaz, la columna es informativa).
- **Restricción de entorno (verificación):** el server MCP local corre desde `target/debug/vanta-cli.exe` (lock Windows) → `cargo nextest` con default features no puede reconstruir ese bin. Mitigación WIRE-12: suite **lib** con default features + targets de integración con `--no-default-features` quitando solo `cli`; para el snapshot público, `CARGO_TARGET_DIR` fresco (`target/wire13api`). Heavy tests SIEMPRE con `pwsh dev-tools/heavy-test-lock.ps1 acquire/release` (adquirido tras espera por SRV-10/BENCH-02; liberado al terminar).
- **PROHIBIDO tocar (respetado):** `opencode.jsonc`, master plan, `docs/pipeline-state.json`; sin push (Regla 7).
