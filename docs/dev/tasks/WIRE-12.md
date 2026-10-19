---
title: "WIRE-12: IQL `LIMIT`/`OFFSET` — paginación end-to-end"
kind: task
description: "LIMIT <n> y OFFSET <n> funcionan end-to-end (parser→AST→planner→executor) con tests (parse + ejecución + interacción RANK BY/AS OF) y docs/api/IQL.md; sin breaking de queries existentes."
---

# WIRE-12: IQL `LIMIT` / `OFFSET`

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 68, F0-expandido — campaign taskId `68`)
- **Fuente:** Backlog `:181` (fila WIRE-12) + plan Task 68
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟡 · **Tipo:** Rust (parser/planner/executor IQL) + `docs/api/`
- **Turns estimados:** 15-30
- **Creado:** 2026-10-06 · **last-synced:** 2026-10-06
- **Estado:** ⏳ IN PROGRESS (steps 1-4 ✅; Step 5 = review P2-01 + commit + cierre)
- **Incógnitas (uphill):** 0 (resueltas en DISCOVERY §Spec con evidencia) · **Pendientes (downhill):** 1 step (cierre)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Executor::execute_hybrid` ← HTTP `/api/v2/query`, MCP `query_iql`, bindings `query()`, CLI/TUI REPL; `parse_statement` ← `cli_handlers/data.rs:435` (`query_is_mutating`) |
| Callees | `Query::into_logical_plan` / `SelectStatement::into_logical_plan` (`src/query.rs:241,452`) → `planner::optimize_and_compile` (`src/planner.rs:165`) → Volcano (`PhysicalScan/Filter/Sort/Project/Limit` + registry `Dedup`); `cost_estimator::estimate_plan` (admisión OLD-21, `src/executor.rs:530`); `governor.apply_temperature_limits` (solo lee `Traverse`) |
| Implicaciones | Contrato IQL público (gramática + AST JSON + versionado `IQL_VERSION`); snapshots insta del AST (`tests/logic/snapshots/parser__dql_query_ast.snap`); construcciones de `Query`/`SelectStatement` en tests (`tests/logic/joins.rs`, `src/query.rs`, `src/executor.rs`) requieren los campos nuevos; `LogicalOperator` es enum público (`src/lib.rs:130 pub mod query`) → variante nueva = adición minor documentada (patrón C2S6 `Dedup`); **sin** cambios en storage/WAL/índices; sin cambios de wire on-disk |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/parser/lexer.rs` (178L), `src/parser/grammar.rs` (577L), `src/query.rs` (≤745L), `src/physical_plan/project.rs` (93L), `src/physical_plan/mod.rs` (750L), `src/operator_registry.rs` (≤510L), `src/cost_estimator.rs` (527L), `src/executor.rs` (1-779L; resto = tests), `src/planner.rs:1-480` (parte funcional; no se modifica), `docs/api/IQL.md` (373L), `tests/logic/parser.rs` (133L), `tests/logic/snapshots/parser__dql_query_ast.snap` (51L), `dev-tools/heavy-test-lock.ps1`
- **Referencias hacia dentro:** `grammar.rs` importa lexer (`ws/ident/parse_number/...`) + `super::{iql_supports, IQL_VERSION_MIN_*}`; `planner.rs` matchea `LogicalOperator::Limit{top_k}` (`:234`) y aplica `PhysicalLimit` (`:427-432`) antes de extensiones del registry (`:438-443`); `cost_estimator.rs:243` matchea `Limit`; `operator_registry.rs:49` mapea `"limit"`; `physical_plan/mod.rs:20` re-exporta `PhysicalLimit`
- **Referencias entrantes:** `parse_statement` ← `executor.rs:179` + `cli_handlers/data.rs:435`; `Query`/`SelectStatement` construidos en `grammar.rs:183,499`, `tests/logic/joins.rs` (10 sitios), `src/query.rs` tests (5), `src/executor.rs` tests (2); `LogicalOperator` consumido por planner/cost/registry/governor/executor (grep completo: 0 usos fuera de `src/` + tests)
- **Veredicto impacto:** **medio** — gramática + AST públicos (adición compatible: campos nuevos al final + cláusula opt-in), variante `Offset` aditiva (patrón C2S6), planner/executor **no** editados en su semántica (solo arm forzado del match exhaustivo en `cost_estimator`, como `Dedup`); snapshots y construction sites actualizados en el mismo commit; cero migración de datos

## Contrato

"`LIMIT <n>`/`OFFSET <n>` funcionan end-to-end (parser→AST→planner→executor) con tests (parse + ejecución + interacción con RANK BY/AS OF) y `docs/api/IQL.md` actualizado; sin breaking de queries existentes (LIMIT sigue siendo reservada, ahora consumida)."

Comando de verificación (cierre): `cargo nextest run --profile audit -p vantadb --lib` + `--test parser` + `--test executor` + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + gates docs.

## Spec (SDD — Phase 1b: feature-add, símbolos/contratos públicos nuevos)

> Gate P/D: la solución agrega contrato público (cláusulas IQL + campos AST + variante `LogicalOperator`). El plan Task 68 delegó expresamente la decisión de OFFSET a DISCOVERY "con evidencia" (pre-mortem #1) → se resuelve **por evidencia** (opción 2 del gate mecánico spec-first), sin ronda `question` adicional.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Mecanismo de OFFSET | A) nuevo operador `Offset{skip}` vía registry C2S6 (aplicado post-chain) + ensanchar la ventana `Limit` a `n+m` cuando ambos están presentes → skip-then-take correcto sin tocar matches probados · B) extender `Limit{top_k}` con campo `offset` → cambia el shape de una variante **pública existente** + edita el arm built-in probado (contradice "built-ins keep their existing match arms", `operator_registry.rs:4-8`) · C) variante nueva + brazo nombrado en planner → viola "new operators: variant + physical file + one register line" (`docs/api/IQL.md:355-366`) | ✅ **A decidido-por-evidencia**: las extensiones del registry se aplican **después** de `Limit` (`planner.rs:427-443`); `Offset` post-chain + ventana `n+m` da skip-then-take con **cero edición** de los matches probados (planner/executor). Precedente aditivo: `Dedup` (C2S6 §2.4 compat: "variante añadida → minor + nota CHANGELOG"). `Dedup`/`Offset` coexisten como extensiones registradas |
| 2 | Orden canónico de cláusulas | `LIMIT n OFFSET m` (SQL/SQLite/Postgres) · `OFFSET m LIMIT n` (rechazado: orden invertido = Failure estricto, nunca drop silencioso) | ✅ decidido-por-evidencia (ref: SQL estándar; patrón estricto de `AS OF`, `grammar.rs:99-103`) |
| 3 | Posiciones de `AS OF` vs paginación | Mantener 2 posiciones y que `LIMIT n AS OF t` sea trailing silencioso (malo) · agregar 3ª posición tras paginación con merge-duplicado (mismo patrón "two canonical positions" de SCH-03) | ✅ decidido-por-evidencia (ref: `grammar.rs:96-136`): se agrega la 3ª posición; `LIMIT 5 AS OF t` y `AS OF t LIMIT 5` parsean ambos; duplicado → `Failure` (mensaje estable existente) |
| 4 | Strictness de valores | `LIMIT abc`/`LIMIT` (sin número) → error de parse · drop silencioso (comportamiento actual de tokens no consumidos) | ✅ decidido-por-evidencia (ref: strictness de `AS OF`): peek del keyword → número requerido → `Failure(Verify)` |
| 5 | Versionado IQL | Bump `IQL_VERSION` 2→3 + `IQL_VERSION_MIN_PAGINATION = 3` (patrón `PROFILE`/`AS OF`) · sin bump (consumidores no pueden feature-detectar) | ✅ decidido-por-evidencia (ref: `parser/mod.rs:11-33` + `IQL.md:13-30`: "Bump when the grammar changes in a way consumers must detect (new clause)") |
| 6 | Semántica con búsqueda vectorial | LIMIT aplica al stream final (post-fusión RRF / post-ventana vectorial) · subir la ventana vectorial hardcoded (5) desde el LIMIT | ✅ decidido-por-evidencia (ref: `physical_plan/vector.rs:80` `search_nearest(..., 5, ...)`): se documenta que la ventana vectorial actual (5) acota las consultas vector-only; subirla = cambio de hot path fuera de scope (→ FIND-313) |
| 7 | Interacción con filtros post-plan (`AS OF`, `ROLE`) | LIMIT in-plan (post-sort) y filtros post-plan → página puede acortarse · mover filtros antes del LIMIT (requiere operador de intervalo valid-time, cambio mayor) | ✅ decidido-por-evidencia (ref: `executor.rs:219-238,274-300`): se mantiene la arquitectura (AS OF/ROLE post-plan) y se documenta la garantía de "página corta = fin del walk" ya vigente en `IQL.md:296-301` |
| 8 | Valores borde | `LIMIT 0` → vacío (SQL válido); `OFFSET 0` → no-op (no se emite operador); overflow `n+m` → `saturating_add` (wasm32) | ✅ decidido-por-evidencia (ref: SQL; aritmética `usize` portable) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) `LIMIT` sigue en `RESERVED_KEYWORDS` y no puede usarse como alias — la cláusula ahora se **consume** (antes quedaba como trailing silencioso); (2) queries existentes sin paginación parsean idéntico (campos `None`); (3) `LogicalOperator::Limit{top_k}` conserva su shape y semántica ("cap at N"); (4) `planner::optimize_and_compile` y `executor` **no** editan su semántica (solo arm forzado en `cost_estimator`, patrón `Dedup`); (5) registry aditivo: `register` de "offset" pre-hecho, sin duplicados; (6) `AS OF` duplicado sigue siendo error; (7) sin push (Regla 7); snapshots del AST actualizados 1 vez en el mismo commit.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --lib` · `cargo nextest run --profile audit -p vantadb --test parser` · `cargo nextest run --profile audit -p vantadb --test executor` · `cargo fmt -p vantadb -- --check` · `cargo clippy -p vantadb --all-targets -- -D warnings` · `node scripts/docs/check-links.mjs` + `check-docs.mjs`
- **Deuda pendiente:** ninguna al abrir (2 FINDs derivados: FIND-312 trailing-input general, FIND-313 ventana vectorial hardcoded)

## Recitation (canónico)

```
=== RECITATION ===
Objetivo activo: WIRE-12 — IQL LIMIT/OFFSET end-to-end
Estado: in-progress (steps 1-5 casi cerrados → re-review ronda 2 + commit)
Última acción: review adversarial ronda 1 → REQUEST CHANGES (snapshot public-api stale) → fixes: refresh oficial del snapshot (+compare 1/1), reject trailing-after-pagination (+tests), refs IQL_VERSION en VERSIONING/openapi; lib 2399/2399, integración 19/19, fmt/clippy/docs gates verdes
Resultado: PARTIAL
Próxima acción: re-review vanta-review (delta) → commit LOCAL feat(iql) → campaign completed taskId 68
Contrato: ver ## Contrato
Invariantes: LIMIT reservada consumida; Limit{top_k} intacto; planner/executor sin edición semántica; sin push
Deuda: ninguna (FIND-312/313 diferidos)
Próxima tarea si completa: WIRE-13
last-synced: 2026-10-06
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** Sin deuda nueva — variante + campos aditivos; el código nuevo cierra un no-op silencioso (LIMIT lexado sin consumidor). Deuda diferida registrada como FIND-312 (trailing input general) y FIND-313 (ventana vectorial 5 hardcoded) — ninguna introducida por este PR.

## Definition of Done (3 niveles)

- **Task:** contrato verificado con tests parse+ejecución+interacciones (RED→GREEN) + `docs/api/IQL.md` sincronizado (Regla 3) + fmt/clippy scoped verdes
- **Commit:** atómico, `feat(iql): WIRE-12 — …`, solo paths del scope, verificación mecánica previa (nunca auto-reporte)
- **Release:** changelog minor — lo maneja release-plz (no manual); sin cambios de wire on-disk

## Herramientas necesarias
- `cargo check/nextest/clippy/fmt`, `campaign_verify_cmd`, `pwsh dev-tools/heavy-test-lock.ps1` (pruebas pesadas serializadas)
- codegraph / codebase-memory-mcp (blast radius), `pwsh dev-tools/ocr-review.ps1`
- **Skills cargadas (SDP):** campaign-executor + progreso (base auto) · documentation-and-adrs + api-and-interface-design (pins de policy "API docs") · test-driven-development + incremental-implementation (lifecycle BUILD) · rust-write-tests (dominio Rust tests) · source-driven-development (validación contra código real). `writing-guidelines`/`writing-plans` (base docs) evaluadas: subsumidas por `documentation-skill` para este update de API reference.

## Investigation Notes (DISCOVERY — evidencia)

- **Estado real verificado (2026-10-06):** `LIMIT` está en `RESERVED_KEYWORDS` (`lexer.rs:50`) pero **ningún** parser lo consume: `rg "LIMIT|OFFSET" src/parser/` → solo esa línea; `OFFSET` = 0 hits (ni reservada). Hoy `FROM Doc LIMIT 5` parsea como Query y **el tail se ignora silenciosamente** en `execute_hybrid` (`executor.rs:179` descarta el remainder) → devuelve TODO (no-op silencioso que esta tarea cierra).
- **Pipeline de paginación existente:** `LogicalOperator::Limit{top_k}` (`query.rs:403`) → planner lo colecta (`planner.rs:234`) y envuelve `PhysicalLimit` (`planner.rs:427`; impl `physical_plan/project.rs:53`) → cost (`cost_estimator.rs:243`) → registry name `"limit"` (`operator_registry.rs:49`). Sin productor IQL (solo tests).
- **Registry C2S6:** extensiones = variante + archivo físico + `register`; se compilan **post-chain** (`planner.rs:438-443`), o sea después de `Limit`; `Dedup` es el exemplar (`operator_registry.rs:134`). El cost arm del match exhaustivo delega al registry (`cost_estimator.rs:273-280`).
- **Vector search IQL:** `PhysicalVectorSearch::open` llama `search_nearest(..., 5, ...)` — ventana hardcoded 5 (`physical_plan/vector.rs:80`); `LIMIT` no puede excederla en consultas vector-only → documentado + FIND-313.
- **Filtros post-plan:** `AS OF` (`executor.rs:274-300`) y RBAC `ROLE` (`:557-568`) filtran **después** de ejecutar el plan → con LIMIT in-plan una página puede acortarse; `IQL.md:296-301` ya documenta "página corta = fin del walk" para filtros temporales.
- **Versionado:** `IQL_VERSION = 2` (`parser/mod.rs:18`), gates `iql_supports` por cláusula; consumidores feature-detectan (IQL.md:13-30).
- **Snapshot AST:** `tests/logic/snapshots/parser__dql_query_ast.snap` pinea el `Debug` completo de `Query` → agregar campos exige actualizarlo (mismo commit).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 (OFFSET mecanismo, orden, strictness, versión, vector, post-filtros → §Spec) |
| Pendientes | 5 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — N/A justificado: sin input externo nuevo en trust boundary (IQL ya valida en el borde parser); la strictness **aumenta** (no hay drop silencioso); sin auth/deps/FFI/red. Valores `usize` acotados por `parse_number` (u32) + `saturating_add`.
- [x] **PERFORMANCE** — Evaluado: LIMIT/OFFSET solo agregan operadores Volcano baratos (`skip`/`count` O(1)); no se toca hot path de búsqueda ni serialización. La ventana vectorial (5) se documenta, no se cambia (Regla 9: sin benchmark no hay "mejora").

## Steps

### Step 1: RED — tests de contrato (parse + ejecución + registry + plan)
- **Archivos:** `src/parser/mod.rs` (tests), `src/executor.rs` (tests), `src/operator_registry.rs` (tests), `src/cost_estimator.rs` (tests), `src/query.rs` (tests), `src/physical_plan/mod.rs` (tests)
- **Acción:** escribir tests que fallan por la razón correcta (API inexistente → compile-fail): parse LIMIT/OFFSET (valores, strictness, orden invertido, interacción RANK BY/AS OF en ambas posiciones, `SELECT`), ejecución (cap, skip, skip-then-take, RANK BY asc/desc antes del LIMIT, AS OF post-LIMIT página corta, LIMIT 0, OFFSET al final), registry (`"offset"` name + compile + cost), plan (ventana ensanchada `n+m`)
- **Verify:** `cargo check -p vantadb --tests` → **falló** con E0609 (`limit`/`offset` inexistentes), E0599 (`Offset` inexistente), E0425 (`IQL_VERSION_MIN_PAGINATION`, `OffsetCompiler`, `PhysicalOffset`) = RED probado ✅
- **Estado:** ✅ COMPLETED

### Step 2: GREEN — gramática + AST + versión
- **Archivos:** `src/parser/lexer.rs` (OFFSET reservada), `src/parser/grammar.rs` (`parse_pagination` + 3ª posición AS OF + strictness), `src/parser/mod.rs` (IQL_VERSION=3 + MIN_PAGINATION), `src/query.rs` (campos AST), `tests/logic/snapshots/parser__dql_query_ast.snap`
- **Acción:** cláusulas `LIMIT`/`OFFSET` estrictas (peek→número requerido; orden invertido = Failure) en `parse_query` y `parse_select`; campos `limit/offset` en `Query`/`SelectStatement`; snapshot actualizado; construction sites compilan (`tests/logic/joins.rs`, tests de `query.rs`/`executor.rs`)
- **Verify:** `cargo check -p vantadb --tests` ✅ exit 0; parse tests verdes en la corrida lib (Step 3)
- **Estado:** ✅ COMPLETED

### Step 3: GREEN — pipeline (variante + físico + registry + emisión de plan)
- **Archivos:** `src/query.rs` (`Offset{skip}` + `push_pagination`), `src/physical_plan/offset.rs` (nuevo), `src/physical_plan/mod.rs`, `src/operator_registry.rs`, `src/cost_estimator.rs`
- **Acción:** `PhysicalOffset` Volcano (skip counter), registro `"offset"` (compiler + cost `(rows-skip).max(0)`), arm forzado en cost (delega al registry, patrón `Dedup`), emisión `Limit{n+m}` + `Offset{skip}` cuando ambos; mensaje estable para paginación malformada (`iql_parse_error_message`)
- **Verify:** `cargo nextest run --profile audit -p vantadb --lib` → **2396/2396** (primer run; 1 fallo propio de expectativa de test corregido) → re-run final **2397/2397** ✅
- **Estado:** ✅ COMPLETED

### Step 4: Docs — `docs/api/IQL.md` + AST JSON + extensibilidad
- **Archivos:** `docs/api/IQL.md`, `src/parser/mod.rs` (test AST JSON extendido), `docs/index.md`/`docs/api/index.md`/`llms.txt` (regen)
- **Acción:** versión 3 en frontmatter/tabla; sintaxis `LIMIT`/`OFFSET` en ambos bloques; sección `Pagination` (post-fusión, skip-then-take, interacciones AS OF/ROLE/vector, página corta); ejemplo AST JSON con campos nuevos; nota registry (Offset como 2º exemplar)
- **Verify:** `node scripts/docs/check-links.mjs` ✅ · `check-docs.mjs` ✅ · `gen-index.mjs --check` ✅ (post-`--write`) · `scripts/validate-docs-coverage.ps1` ✅ 0 gaps
- **Estado:** ✅ COMPLETED

### Step 5: Cierre — verify contrato + OCR + review P2-01 + commit + campaña
- **Archivos:** `docs/dev/tasks/WIRE-12.md`, `docs/dev/Backlog.md` (FIND-312/313), `tests/api/public-api.txt` (refresh), `docs/api/VERSIONING.md` + `docs/api/openapi.yaml` (refs IQL_VERSION)
- **Acción:** suite scoped con LOCK + fmt + clippy; OCR delegation; review adversarial (vanta-review) → REQUEST CHANGES (snapshot public-api) → fixes → re-verify; commit LOCAL `feat(iql):`; cierre campaign taskId `68`; `skill progreso`
- **Verify:** fmt --all ✅ · check+clippy gate ✅ · clippy --lib ✅ · lib **2399/2399** ✅ · integración 19/19 ✅ · **public_api 1/1 ✅** (snapshot refrescado) · docs gates ✅
- **Estado:** ⏳ IN PROGRESS (re-review ronda 2 + commit pendientes)

## Dependencias
- Ninguna bloqueante. Dependiente: WIRE-13 (agregaciones — usa LIMIT).
- **Coordinación:** BENCH-02 en vuelo (benchmarks — área disjunta); lock de pruebas pesadas obligatorio.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (sesión `ses_eef820309ffefbuv7dC1mjVp9P`, 2026-10-06) — tier **adversarial** (el diff toca `src/parser/**`)
- **Enfoque:** mecanismo `Offset` (registry C2S6 + ventana `n+m`) en todos los casos; strictness; AS OF 3ª posición; versionado; docs sin deriva; invariantes
- **Cómo se probó:** re-ejecutó `nextest --lib` (2397/2397), integración sin `cli` (19/19), fmt, gates docs; verificó `planner.rs:427-443` + `push_pagination` adversarialmente. **Ronda 1 → 🔴 REQUEST CHANGES** (1 Critical + 2 Optional + 2 Nits)
- **Hallazgos y resolución:**
  - 🔴 **Critical — `tests/api/public-api.txt` desactualizado (gate HARD-01):** símbolos nuevos ausentes del golden. **Resuelto** con el mecanismo oficial: `VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api --run-ignored ignored-only` (en `CARGO_TARGET_DIR=target/wire12api` por el lock del exe del server MCP) + compare re-run **1/1 verde**. El refresh (+518 líneas, 0 borrados) contiene 55 líneas de WIRE-12 y 463 de drift **pre-existente de HEAD** (VER-10/attestation, sdk/entity/MergeOutcome de MEMG-17, config/ExportReport) — el golden ya estaba stale desde `97039053` (ver Notas).
  - 🟡 **Optional-1 — trailing clause tras paginación:** `FROM Doc LIMIT 5 WHERE n > 1` consumía el LIMIT y descartaba el resto (página sin filtro). **Resuelto:** `reject_trailing_after_pagination` en `parse_statement` (Failure si el remanente no está vacío y hubo paginación; subqueries intactas porque el caller consume `)`), con tests de parser + executor (`test_parse_statement_rejects_trailing_clause_after_pagination`, `pagination_rejects_trailing_clause_instead_of_silently_dropping_it`).
  - 🟡 **Optional-2 — refs stale de `IQL_VERSION`:** **Resuelto inline** en `docs/api/VERSIONING.md` (fila IQL: `= 1` → `= 3`) y `docs/api/openapi.yaml:152` (`= 1` → `= 3`). `UPGRADE.md:99` se deja sin tocar: es la fila histórica del upgrade 0.7→0.8.0 (correcta en su alcance; el upgrade a 0.9.0 se documenta en release).
  - 🟢 **Nit-1** (guard de duplicado sin boundary — solo input basura) y **Nit-2** (col cosmético, misma forma que `AS OF`): aceptados sin acción.
- **Checklist anti-hábitos tóxicos:** sin inventar salidas ✅ · sin saltar clarificación ✅ · fallo parcial reportado honestamente ✅ (el Critical propio) · evidencia re-ejecutada por el revisor ✅ · sin reintentos a ciegas ✅ · SDP ✅ · resto ✅
- **Veredicto:** 🔴 REQUEST CHANGES (ronda 1) → fixes aplicados → **✅ APPROVE (ronda 2, delta verificado por el revisor: snapshot compare 1/1 re-ejecutado, 11/11 tests del delta, refs de versión, premisa corregida aceptada; commit gate desbloqueado)**. Nit final del revisor (semicolon `;` tras paginación) resuelto: se tolera el terminador (misma lenidad que statements sin paginación) + assert en el test.

## Notas
- **Pre-mortem #1 resuelto:** OFFSET vía registry C2S6 (no extender `Limit`) — evidencia en §Spec #1.
- **Pre-mortem #2 resuelto:** semántica vectorial documentada (LIMIT post-fusión; ventana 5 hardcoded = FIND-313).
- **Pre-mortem #3 resuelto:** alias no se rompen (LIMIT/OFFSET reservadas; alias se consumen antes; patrón `non_keyword_ident` intacto; tests `LIMITED`/`OFFSETX`).
- **Hallazgos derivados (creados en DISCOVERY):** FIND-312 (trailing input general silencioso en `execute_hybrid`), FIND-313 (ventana vectorial hardcoded 5).
- **Scope extendido vs plan (justificado):** `src/parser/mod.rs` (version consts + tests del módulo parser — el plan listaba `lexer.rs`/`grammar.rs`), `src/physical_plan/offset.rs` (archivo nuevo del patrón C2S6), `tests/logic/snapshots/parser__dql_query_ast.snap` (fixture del AST), `docs/dev/tasks/WIRE-12.md` + `docs/dev/Backlog.md` (artefactos de tarea/FINDs).
- **Restricción de entorno (verificación):** el server MCP local corre desde `target/debug/vanta-cli.exe` (proceso vivo, lock Windows) → `cargo nextest` con default features no puede reconstruir ese bin. Mitigación: (a) suite **lib** con default features (2397/2397 — cubre parser/planner/executor/registry/cost in-crate); (b) targets de integración (`parser`/`joins`/`executor`/`operator_registry_extension`) con `--no-default-features` quitando solo `cli` (19/19) — el feature bajo prueba no está gateado por `cli`. `cargo clippy -p vantadb --tests` con ese combo tiene errores **pre-existentes** de dead_code en `wal.rs`/`wal_sharded.rs` (código consumido por `cli_handlers`, feature `cli` apagada) — ajenos a esta tarea; el clippy canónico del gate (`cli,fjall,memmap2,fs2,roaring`) pasa.
- **OCR delegation (advisory):** `pwsh dev-tools/ocr-review.ps1 -Format json` → rule groups 1-2 revisados contra el diff; sin Critical/High; Medium/Low: ninguno nuevo (FIND-312/313 ya ticketed). `dev-tools/heavy-test-lock.ps1` aparece en el spec OCR pero es untracked ajeno (BENCH-02) — no se toca ni commitea.
- **public-api snapshot — drift pre-existente detectado en REVIEW:** el golden `tests/api/public-api.txt` estaba **stale en HEAD** (último refresh `97039053`, 2026-10-05 15:40; después aterrizaron `7841879e` VER-10/attestation + cambios sdk/entity/MergeOutcome de MEMG-17 sin refrescar). El refresh de esta tarea (+518 líneas, 0 borrados) **acknowledgea mecánicamente** esas 463 líneas ajenas junto a las 55 de WIRE-12 — necesario porque el gate compara el archivo completo (dejarlo stale = CI `public-api-snapshot` rojo). Verificación: compare 1/1 verde + contenido idéntico al generado por `cargo public-api -s` (0 diferencias). **Acción para el orquestador:** avisar a los autores de VER-10/MEMG-17 que su superficie pública quedó acknowledgeada en este commit (HARD-01).
- **PROHIBIDO tocar (respetado):** `opencode.jsonc`, master plan, `docs/pipeline-state.json`; sin push (Regla 7).
- **Coordinación BENCH-02:** sus commits (`21e9ff0f`, `880611ff`) aterrizaron durante la tarea; `docs/index.md`/`llms.txt` quedaron limpios y la regeneración posterior solo contiene entradas de WIRE-12 (diff verificado).
- Si el linker falla en Windows → `pwsh dev-tools/target-cleanup.ps1 -Clean -Yes`.
