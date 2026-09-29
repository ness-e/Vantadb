---
title: "SCH-03: Queries AS OF / point-in-time + filtros valid_at + exclude_superseded"
kind: task
description: AS OF operable en IQL (con IQLVERSION bumpeado + feature-detect) y params equivalentes en search/list devuelven el estado histórico correcto con tests deterministas Y filtros por ventana de validez sobre v2 (validat/invalidat) Y...
---

# SCH-03: Queries AS OF / point-in-time + filtros valid_at + exclude_superseded

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 28 (F3) · **Origen:** `docs/dev/Backlog.md:940` (+ ADR-046 accepted 2026-09-28, §D3/§D7)
- **Fuente del prompt:** sub-agente vanta-engine (orquestador pipeline, wave F3.3a co-batch con SCH-04) — ejecución directa de SCH-03
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** feature-add (queries temporales valid-time)
- **Turns estimados:** — · **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ IN PROGRESS (implementación + verify ✅; review P2-01 + commit = LEAD)
- **Incógnitas (uphill):** 0 · **Pendientes (downhill):** review P2-01 + commit (LEAD) + docs/api/IQL.md (SCH-07) + compile-fix de literales en crates prohibidas (SCH-04)
- **Gate D (question-gates):** pre-respondido por el orquestador — mandato explícito de implementar SCH-03 con contrato verbatim, regiones asignadas y stop conditions (co-batch F3.3a). Sin question al usuario.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Alcance | `src/sdk/types/record.rs` (`ValidWindow` + helpers `MemoryRecord::is_valid_at`/`validity_overlaps` + `MemoryListOptions.as_of_ms`/`valid_window`) · `src/sdk/serialization/vector_types.rs` (`MemorySearchRequest.as_of_ms`/`valid_window`) · `src/sdk/search/page.rs` (validación de boundary + selectors_can_shorten + assembly temporal + fingerprint de cursor) · `src/sdk/api/namespaces.rs` (assembly temporal en `list`) · `src/parser/mod.rs` (`IQL_VERSION 1→2` + `IQL_VERSION_MIN_AS_OF` + `OF` reservada) · `src/parser/grammar.rs` (cláusula `AS OF` en `parse_query`/`parse_select` + rechazo en subqueries + autocomplete) · `src/parser/lexer.rs` (`parse_u64_id`) · `src/query.rs` (`Query.as_of_ms` + `SelectStatement.as_of_ms`) · `src/executor.rs` (filtro valid-time post-plan en `execute_query`/`execute_select`) |
| Callees | Literales de `MemorySearchRequest`/`MemoryListOptions`/`Query`/`SelectStatement` en `src/`, `benches/`, `tests/` (compile-fix `as_of_ms: None, valid_window: None`) — fuera de regiones del co-batch: bindings/MCP/server/`vanta-memory`/proxy/desktop/providers quedan para SCH-04/LEAD |
| Implicaciones | Wire aditivo (`#[serde(default)]`, default = comportamiento actual — cero breaking silencioso, ADR §D3-5); `exclude_superseded` extiende semántica (documentado, ADR §D3-6); IQL gana cláusula version-gateada (v1 sigue parseando); cursor fingerprint cubre los params nuevos; sin cambios de índices ("no index change", patrón ADR-028) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos / secciones):** `docs/dev/architecture/adr/ADR-046-schema-v2-migracion-unica.md` (§D0/D1d/D2/D3/D7/D8, completo) · `docs/dev/research/mgr-10-bitemporalidad.md` (§1.3-§4.5) · `src/parser/mod.rs` (1434L, completo) · `src/parser/grammar.rs` (505L, completo) · `src/parser/lexer.rs` (164L, completo) · `src/query.rs` (817L, completo) · `src/executor.rs` (1-640) · `src/sdk/search/page.rs` (1-363, completo) · `src/sdk/search/mod.rs` (1-499, completo) · `src/sdk/search/multi.rs` (92L, completo) · `src/sdk/serialization/vector_types.rs` (619L, completo) · `src/sdk/types/record.rs` (845L, completo) · `src/sdk/api/namespaces.rs` (368L, completo) · `src/sdk/api/memory.rs` (380-499, 660-789) · `src/sdk/types.rs` (1-80) · `src/lib.rs` (183-217) · `src/physical_plan/scan.rs` (210L, completo) · `src/planner.rs` (150-429) · `src/sdk/version_history.rs` (grep) · `.opencode/rules/{core-engine,query-dsl,api-contract}.md` · `.opencode/references/clean-code-clean-architecture.md` (Apéndice V) · `docs/dev/tasks/SCH-02.md`
- **Referencias hacia dentro:** ADR-046 §D3 (predicado canónico, ejes, default sin cambios), §D3-6 (extensión `exclude_superseded`), §D7 (normalización v1 `valid_at := created_at`), §D1d (alcance 0.8.0), §D8 (nombres); plan Task 28 (:735-759); Backlog:940; precedentes `exclude_superseded` (ADR-028, `page.rs:157,317`), `PROFILE` version-gateado (`parser/mod.rs:11-28`), cursor WIRE-08 (`page.rs:304-330`), MCP-29 (`physical_plan/scan.rs:81-92` — `SELECT * FROM <ns>` alcanza records de memoria)
- **Referencias entrantes:** SCH-04 (co-batch: `vector_types.rs`/`page.rs` — `min_confidence` + fingerprint), SCH-05 (cuarentena operativa sobre `MemorySearchRequest`/`page.rs`), SCH-06 (`tests/time_travel.rs` — time-travel determinista), SCH-07 (bindings/HTTP/MCP + `docs/api/IQL.md` AS OF + `IQL_VERSION 2`), SCH-08 (UPGRADE 0.8.0)
- **Veredicto impacto:** alto — API pública (structs + parser) + semántica de un flag existente; fan-out de literales en todo el workspace; sin cambios de persistencia ni de índices

## Contrato
"`AS OF` operable en IQL (con `IQL_VERSION` bumpeado + feature-detect) y params equivalentes en search/list devuelven el estado histórico correcto con tests deterministas Y filtros por ventana de validez sobre v2 (`valid_at`/`invalid_at`) Y `exclude_superseded` extendido a la semántica nueva Y cursor estable con resume sobre los params nuevos"

## Spec (SDD — Phase 1b; decisiones por evidencia, ADR-046 §D3 = spec)

| Decisión | Alternativas | Elegido | Evidencia / por qué |
|---|---|---|---|
| Eje de `AS OF T` | valid-time vs transaction-time | **valid-time** (predicado `valid <= T && invalid > T`) | ADR-046 §D3-4 verbatim ("el spec decía X en fecha T") — NO re-decidir |
| Sintaxis IQL | `AS OF <ms>` vs fecha ISO vs `FOR ... AS OF` | **`AS OF <u64 ms>`** tras el table-spec (`FROM e [alias] AS OF t [WHERE...]`; `SELECT ... FROM e [joins] AS OF t [WHERE...]`) | Convención `_ms` del proyecto; ms ya es el wire de `valid_at_ms`/`created_at_ms`; ISO exigiría parser de fechas (scope creep) |
| Versión IQL | mantener 1 vs bump | **bump `IQL_VERSION 1→2`** + `IQL_VERSION_MIN_AS_OF=2` + `iql_supports` | `parser/mod.rs:11-16` ("Bump when the grammar ... new clause"); pre-mortem F3 (feature-detect) |
| Scope AS OF | FROM/MATCH vs +SELECT vs subqueries | **FROM/MATCH + SELECT (top-level); subquery con AS OF ⇒ parse error** | MCP-29 usa `SELECT * FROM <ns>` para records; el silencio en subqueries violaría Hyrum → fail-fast |
| Punto de aplicación (IQL) | operador lógico nuevo vs filtro post-plan | **filtro post-plan en `execute_query`/`execute_select`** (helper puro) | Stop condition: "si exige rediseño del pipeline → recortar"; el predicado es por-nodo y determinista; sin cambios de planner/physical |
| Nodos sin metadata de validez | excluir vs incluir | **incluir** (no time-scoped) | Opt-in que solo estrecha records que declaran ventana; nodos de grafo intactos (default sin cambios, §D3-5) |
| Normalización v1 en IQL | sin `__vanta_valid_at_ms` ⇒ excluir vs fallback `created_at` | **fallback a `__vanta_created_at_ms`** (0 = unset) | ADR §D7 boundary #1 (`record_from_node`: `valid_at ⇒ created_at`) |
| Nombres wire search/list | `as_of`/`valid_from`/`valid_to` vs `as_of_ms`+`valid_window` | **`as_of_ms: Option<u64>` + `valid_window: Option<ValidWindow{from_ms,to_ms}>`** | ADR §D3-1 (intervalo `[start,end)`), §D2 (`_ms`), "filtros de ventana" (§D1d); la ventana usa AMBOS campos (overlap) |
| Semántica de ventana | cotas sobre `valid_at` vs solapamiento | **solapamiento** `[valid_at, invalid) ∩ [from, to) ≠ ∅` | "filtros por ventana de validez sobre v2 (`valid_at`/`invalid_at`)" — el contrato nombra ambos campos |
| Forma de la extensión `exclude_superseded` | flag aditivo nuevo vs extensión documentada | **extensión documentada**: drop si `superseded_by.is_some() \|\| invalid_at <= ref` | ADR §D3-6 ("excluir también `invalid_at_ms <= now`"; "el ADR fija el predicado, no la forma del flag"); contrato dice "extendido" |
| Referencia temporal de `exclude_superseded` | reloj por hit vs lectura única en boundary | **`now_ms()` UNA vez por request en el boundary**; predicados puros de `(record, ref)` | Determinismo: tests usan timestamps distantes fijos (pasado lejano/futuro lejano) o `as_of_ms`; "sin reloj en filtros" |
| Integración search | filtrado tras truncar vs assembly con growth | **assembly en `run_search_page`** junto a `exclude_superseded` + `selectors_can_shorten` (ventana crece) + fingerprint | Pre-mortem F2 ("rompen top_k/cursor"); patrón WIRE-08 `page.rs:304-341` |
| Integración list | cambio de índice vs assembly final | **assembly final en `namespaces.rs::list`** + validación de boundary | Patrón FIND-24 / `exclude_superseded:164-169`; sin index change |
| Validación de boundary | en filtro vs en entrada | **`valid_window.from_ms < to_ms`** validado al entrar (search: `SEARCH_OPTIONS_MARKER`; list: `InvalidInput`) | `api-contract` §validate at boundaries; regla L2 del agent file |

## Invariantes de dominio (handoff — MUST)
- Predicado `AS OF T` = ADR-046 §D3-1 verbatim: `valid_at_ms <= T && invalid_at_ms.map_or(true, |inv| inv > T)` — inclusive en T, exclusivo en invalid.
- **Default sin cambios** (ADR §D3-5): `as_of_ms=None` + `valid_window=None` + `exclude_superseded=false` ⇒ resultado byte-idéntico al de hoy.
- Determinismo de tests: reloj solo en `exclude_superseded`; tests pinnean con `as_of_ms`/timestamps distantes fijos — nunca dependen del valor exacto de `now`.
- Cursor: fingerprint hashea `as_of_ms`/`valid_window` (+ `exclude_superseded` ya hasheado); resume con params distintos ⇒ `SEARCH_CURSOR_INVALID` (mismatch), nunca mezcla de páginas.
- `exclude_superseded` extendido NO cambia para datos v1/legacy (`invalid_at=None`) ni para registros superseded alineados (`invalid_at == superseded_at`) — mismo resultado que hoy.
- IQL v1 sigue parseando (cláusula opt-in + version-gate); `PROFILE` sigue gateado en `IQL_VERSION_MIN_PROFILE=1`.
- Sin cambios de índices ni de persistencia; registro/nodo sin tocar (SCH-02 intacto).
- Prohibido: `AS OF` transaction cross-key (v1.0); parser de fechas ISO; MVCC; filtros fuera del assembly.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: 0.** No se introduce `unsafe`, ni clones nuevos en hot path, ni allocations adicionales en el caso default (los filtros se aplican solo si el request los pide). Deuda citada no nueva: docs/api/IQL.md sin `AS OF` hasta SCH-07 (dueño asignado por el plan); compile-fix de literales fuera de región (bindings/MCP/server/`vanta-memory`/proxy/desktop/providers) queda para SCH-04/LEAD (co-batch).

## Definition of Done (3 niveles)
- **Task:** contrato verbatim ✅ — AS OF IQL (version-gate) + params search/list + ventana + extensión + cursor, con tests deterministas + gates (fmt/clippy/nextest -p vantadb/docs-coverage).
- **Commit:** `feat(iql): AS OF / point-in-time + filtros valid_at + exclude_superseded extendido (SCH-03)` — **lo ejecuta el LEAD** (sub-agente sin commit ni self-review).
- **Release:** N/A (el corte 0.8.0 = SCH-08; gate F3).

## Herramientas necesarias
- `codegraph_codegraph_explore` (blast radius; index frozen ⇒ fallback Read directo) · `cargo nextest run --profile audit -p vantadb --build-jobs 2` (scoped `-E 'test(parser) or test(executor) or test(search) or test(cursor) or test(valid_at) or test(as_of)'` + full) · clippy `-D warnings` · `cargo fmt --all -- --check` · `pwsh -NoProfile scripts/validate-docs-coverage.ps1` · `CARGO_BUILD_JOBS=2` · regla dura `-p`
- **Skills cargadas (SDP v3, BUILD — `campaign_discover_skills_v2`):** base `campaign-executor`+`progreso`+`ponytail` (auto) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` (no cargada: contexto ya empaquetado por el orquestador) · + `rust-write-tests` y `api-and-interface-design` (esperadas por el prompt del orquestador) + `documentation-skill` (task file)

## Investigation Notes
- **Puntos de integración verificados:** search → `run_search_page` (`page.rs:257-363`; assembly `:317-330`, fingerprint `:137-166`); list → `Embedded::list` (`namespaces.rs:54-188`; retain superseded `:164-169`); IQL → `Executor::execute_query`/`execute_select` (`executor.rs:202-215`) sobre nodos con campos reservados `__vanta_valid_at_ms`/`__vanta_invalid_at_ms` (`serialization/mod.rs:20-49`; escritura `:547-556`).
- **`exclude_superseded` pre-existente:** search `vector_types.rs:120-123` + `page.rs:317-319`; list `record.rs:302-305` + `namespaces.rs:167-169`; hasheado `page.rs:157`; tests `sdk/api.rs:900-987` (hide + default keep). Extensión = predicado + docs, mismo call-site.
- **Determinismo:** `now_ms()` (`serialization/mod.rs:70`, pub(crate)) se lee una vez por request; los predicados son funciones puras `(record, ref)`. Tests con `put_record_exact` (pub(crate), `memory.rs:676`) para ventanas cerradas exactas + `MemoryInput.valid_at_ms` para bounds.
- **Subqueries:** `parse_subquery_condition_inner` (`grammar.rs:320-334`) usa `parse_select`; el AS OF de SELECT se parsea SOLO en el wrapper top-level (`parse_select_with_as_of`) ⇒ `AS OF` en subquery = leftover ⇒ parse error natural (sin silencio).
- **`AS` ya era reservada** (`lexer.rs:43`) pero el alias de traversal usaba `ident` ⇒ se cambia a `non_keyword_ident` + `OF` reservada para que `AS OF` no sea tragado como alias.
- **Fan-out de literales:** `MemorySearchRequest` (~8 literales plenos), `MemoryListOptions` (~40), `Query`/`SelectStatement` (~17) — compile-fix mecánico en `src/`, `benches/`, `tests/`.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 — eje (ADR), syntax, punto de aplicación, determinismo y cursor resueltos con evidencia antes de editar; review P2-01 ✅ APPROVE con Optional-2 aplicado |
| Pendientes | commit (LEAD, review ya ✅ APPROVE) · `docs/api/IQL.md` (§AS OF + v2) = SCH-07 · compile-fix de literales en crates prohibidas (vantadb-python/server/mcp/wasm) = SCH-04/LEAD · **diferidos review**: Optional-1 (garantía list vs truncado → SCH-07 + follow-up growth) y Nit-3 (`all_consuming` en boundary → FIND, registro Backlog bloqueado por WIP ajeno) |
| % completado | 100% contenido + verify + fixes review (cierre formal = LEAD) |

## Steps
### Step 1: Tipos + exports — `ValidWindow`, helpers, params en requests
- **Archivos:** `src/sdk/types/record.rs`, `src/sdk/serialization/vector_types.rs`, `src/sdk/types.rs`, `src/sdk/mod.rs`, `src/lib.rs`
- **Acción:** `ValidWindow{from_ms,to_ms}` (Copy/Eq/Hash/serde) + `MemoryRecord::{is_valid_at, validity_overlaps}` + `as_of_ms`/`valid_window` en `MemorySearchRequest`/`MemoryListOptions` (serde default, doc extendida de `exclude_superseded`) + re-exports (`types.rs`/`sdk/mod.rs`/`lib.rs`).
- **Verify:** tests serde roundtrip/defaults/helpers + `cargo check -p vantadb --all-targets` ✅
- **Estado:** ✅ DONE

### Step 2: Parser — cláusula `AS OF` + bump `IQL_VERSION`
- **Archivos:** `src/parser/mod.rs`, `src/parser/grammar.rs`, `src/parser/lexer.rs`, `src/query.rs`
- **Acción:** `parse_u64_id` (rechaza cola float); `OF` reservada; traversal alias `non_keyword_ident`; `AS OF <ms>` estricto en `parse_query` y `parse_select` (dos posiciones canónicas: tras el table-spec y al final; `opt` temprano + tardío); subquery con `AS OF` ⇒ `Err::Failure` propagado (`parse_where_item` no hace fallback sobre Failure); `Query.as_of_ms`/`SelectStatement.as_of_ms`; `IQL_VERSION=2` + `IQL_VERSION_MIN_AS_OF=2`; autocomplete `AS OF`.
- **Verify:** 163/163 scoped `test(parser) or test(executor)` ✅ (incluye AS OF posicional, estricto, subquery-reject, gate)
- **Estado:** ✅ DONE

### Step 3: Executor — filtro valid-time post-plan (IQL)
- **Archivos:** `src/executor.rs`
- **Acción:** `filter_valid_at`/`node_is_valid_at` (fallback `__vanta_created_at_ms`, `0`=unset; nodo sin metadata ⇒ incluir) aplicado en `execute_query`/`execute_select`.
- **Verify:** tests deterministas T conocido (inclusive/exclusive/open-ended/v1-fallback/nodo sin metadata/late-position) ✅
- **Estado:** ✅ DONE

### Step 4: Search — assembly temporal + cursor (page.rs)
- **Archivos:** `src/sdk/search/page.rs`
- **Acción:** validación `valid_window.from < to` (marker `SEARCH_OPTIONS_INVALID`); retains temporales pre-cursor/pre-truncado + `exclude_superseded` extendido (`invalid_at <= now`, `now` leído UNA vez en boundary); `selectors_can_shorten` incluye los params nuevos (growth no rompe top_k); fingerprint hashea `as_of_ms`/`valid_window`.
- **Verify:** tests time-travel deterministas + overlap + resume/mismatch de fingerprint + growth ✅ (286 scoped)
- **Estado:** ✅ DONE

### Step 5: List — filtros temporales en assembly (namespaces.rs)
- **Archivos:** `src/sdk/api/namespaces.rs`
- **Acción:** validación de ventana + retains temporales/extensión junto a `exclude_superseded`.
- **Verify:** tests list (as_of/ventana/extensión/cursor walk con params) ✅
- **Estado:** ✅ DONE

### Step 6: Compile-fan-out (literales) + snapshots + tests de integración
- **Archivos:** literales de `MemorySearchRequest`/`MemoryListOptions`/`Query`/`SelectStatement` en `src/`, `benches/`, `tests/`; snapshots insta; `tests/api/public-api.txt`.
- **Acción:** compile-fix mecánico; 9 snapshots insta regenerados (2 campos); `parser__dql_query_ast.snap` regenerado; `public-api.txt` regenerado (+67 líneas = símbolos SCH-03). Coexistencia: SCH-04 agregó `min_confidence` a `MemorySearchRequest` en paralelo → sync mecánico en literales core (ver RESULTADO).
- **Verify:** `cargo check -p vantadb --all-targets` ✅ (tras sync) · scoped query_result/dql ✅
- **Estado:** ✅ DONE (con nota de churn concurrente)

### Step 7: Verify full + cierre
- **Acción:** fmt + clippy + nextest scoped + full `-p vantadb` + docs-coverage + `git diff` coexistencia co-batch.
- **Estado:** ⏳ EN CURSO

## Review P2-01 — disposición de hallazgos (APPROVE 2026-09-29; 0 Critical/Required)
> **Revisor:** `ses_f145361ccffefBNeK4bRGif63v` (fresco ≠ autor `ses_f1535e878ffeHQ3jgv2R46vODx`) — **✅ APPROVE** (0 Critical/Required; 2 Optional + 1 Nit). Optional 2 (`AS OF` duplicado) aplicado + **delta ✅** confirmado por el mismo revisor (2/2 tests nuevos; 172/172 scoped). Optional 1 / Nit 3 diferidos (docs → SCH-07; `all_consuming` → FIND).

| # | Hallazgo | Disposición | Evidencia |
|---|----------|-------------|-----------|
| **Opt-2** | `as_of_early.or(as_of_late)` permite `AS OF` duplicado e ignora el segundo en silencio | ✅ **FIX APLICADO**: `merge_as_of_clauses` rechaza el duplicado con `Err::Failure` posicionado en la cláusula repetida; `iql_parse_error_message` mapea a mensaje estable `"AS OF specified more than once"` (executor) | `test_duplicate_as_of_is_a_parse_error` (query/where/select/subquery + controles) · `test_duplicate_as_of_reports_clear_message` (msg exacto + line/col = 17) |
| **Opt-1** | `list()` puede truncar estado histórico sin señal (página no llena = última, con filtros que dropean) | 📌 **DIFERIDO (por diseño ratificado)**: patrón pre-existente FIND-24/ADR-028 documentado en Spec; garantía "página no llena = última" para filtros temporales → **SCH-07** (docs) + follow-up growth tipo search con decisión de contrato | Spec "Integración list" + comentario `namespaces.rs` del retain |
| **Nit-3** | Tokens sobrantes ignorados (laxitud pre-existente del parser; p.ej. `FROM x AS OF 1 basura`) | 📌 **DIFERIDO**: follow-up `all_consuming` en el boundary de `execute_hybrid` con decisión de compat v1 → **FIND** (registro en Backlog bloqueado por WIP ajeno masivo de docs en la wave) | Comportamiento pre-existente (`test_parse_query_extra_tokens_remain`); no introducido por SCH-03 |

**Nota:** el `AS OF` duplicado en subqueries también queda rechazado (el `Failure` del `merge` propaga a través de `parse_where_item`, que ya no hace fallback sobre `Failure`).

## Verificación (evidencia 2026-09-28/29)

**Mapeo contrato → evidencia (una por cláusula):**

| Cláusula del contrato | Implementación | Test determinista |
|---|---|---|
| `AS OF` operable en IQL + `IQL_VERSION` bumpeado + feature-detect | `parser/mod.rs` (`IQL_VERSION=2`, `IQL_VERSION_MIN_AS_OF=2`), `grammar.rs` (`parse_as_of_clause`, dos posiciones, estricto), `query.rs` (`as_of_ms`), `executor.rs` (filtro) | `test_iql_version_defined_and_gated` · `test_parse_query_as_of_*` (5) · `test_parse_select_as_of_clause` · `as_of_filters_nodes_by_valid_window_with_known_t` · `as_of_after_where_also_applies_and_includes_unscoped_nodes` · `select_as_of_filters_namespace_scan_records` |
| Params equivalentes en search/list → estado histórico correcto | `MemorySearchRequest.as_of_ms` + `MemoryListOptions.as_of_ms` (assembly `page.rs` / `namespaces.rs`) | `test_search_as_of_returns_state_at_known_t` · `test_list_as_of_and_valid_window_filters` (T conocido: 999/1000/1999/2000/2999/3000) |
| Filtros por ventana de validez (`valid_at`/`invalid_at`) | `ValidWindow` + `MemoryRecord::{is_valid_at, validity_overlaps}` + assembly | `validity_overlaps_is_half_open_intersection` · `is_valid_at_*` (2) · `test_search_valid_window_overlap_filters` · `test_list_as_of_and_valid_window_filters` · `test_*_rejects_empty_valid_window` (2) |
| `exclude_superseded` extendido a la semántica nueva | `page.rs:retain` + `namespaces.rs:retain` (`superseded_by || invalid_at <= now`, `now` leído 1× en boundary) | `test_exclude_superseded_also_drops_ended_validity_windows` (search) · `test_list_exclude_superseded_also_drops_ended_validity_windows` |
| Cursor estable con resume sobre los params nuevos | `plan_fingerprint` hashea `as_of_ms`/`valid_window`; `selectors_can_shorten` los incluye (growth) | `test_search_page_cursor_is_bound_to_temporal_params` (resume + mismatch) · `test_search_as_of_page_fills_top_k_when_enough_valid_candidates_exist` · `test_list_cursor_walks_with_temporal_params_without_duplicates` |

**Gates mecánicos (final, árbol compartido con SCH-04):**
- `cargo check -p vantadb --all-targets` → ✅ exit 0
- `cargo nextest run --profile audit -p vantadb --build-jobs 2` → ✅ **2438/2438 passed** (502.8s, 2 slow; 2 skipped) — corrida final post-fix lazy-clock; retry tras 1 crash transitorio de `rustc` (STATUS_STACK_BUFFER_OVERRUN, patrón FIND-173: retry OK)
- Scoped `-E 'test(parser) or test(executor) or test(search) or test(cursor)'` → ✅ **433/433** · scoped temporal (`search|list|as_of|valid_window|exclude_superseded|cursor|validity`) → ✅ **330/330**
- `cargo clippy -p vantadb --all-targets -- -D warnings` → ✅ exit 0 · subset verify.ps1 (`--no-default-features --features "cli,fjall,memmap2,fs2,roaring"`) → ✅ exit 0
- `cargo fmt -p vantadb -- --check` → ✅ exit 0 (mis archivos); `cargo fmt --all -- --check` → solo diffs del WIP ajeno de SCH-04 (`vantadb-server/tests/e2e.rs`, `vantadb-wasm/*`) — NO tocados
- `pwsh -NoProfile scripts/validate-docs-coverage.ps1` → ✅ 0 gaps
- Snapshots deliberados: 9 insta (`+3` campos c/u), `parser__dql_query_ast.snap` (`+as_of_ms`), `tests/api/public-api.txt` (+67 líneas = símbolos SCH-03)

**Fixes review P2-01 (2026-09-29, Optional-2):**
- `cargo nextest run --profile audit -p vantadb -E 'test(parser) or test(executor)'` → ✅ **172/172** (incluye `test_duplicate_as_of_is_a_parse_error` + `test_duplicate_as_of_reports_clear_message`)
- Binarios de integración que consumen el parser: `--test parser --test joins --test openapi_yaml_parity` → ✅ 25/25 · `--test mutations --test structured_api_v2 --ignore-default-filter` → ✅ 2/2
- `cargo clippy -p vantadb --all-targets -- -D warnings` → ✅ exit 0 (fix `needless_lifetimes` en el helper nuevo)
- `cargo fmt -p vantadb -- --check` → ✅ exit 0 · `cargo fmt --all -- --check` → ✅ exit 0 (árbol completo, tras fixes de formato de SCH-04)
- `pwsh -NoProfile scripts/validate-docs-coverage.ps1` → ✅ 0 gaps
- Archivos del fix: `src/parser/grammar.rs` (`merge_as_of_clauses` + 2 call sites) · `src/executor.rs` (`iql_parse_error_message` + uso) · tests inline (`parser/mod.rs`, `executor.rs`)

**Coexistencia co-batch (verificado con `git diff` antes de cerrar):**
- `vector_types.rs`: hunks SCH-03 (`as_of_ms`/`valid_window` + docs + tests) y SCH-04 (`min_confidence` + docs) coexisten; docs fusionadas.
- `page.rs`: validación/boundary, fingerprint (`as_of_ms`/`valid_window` + `min_confidence`), selectors y retains de ambos coexisten; sin hunks perdidos.
- Churn concurrente absorbido: sync mecánico de `min_confidence: None` en literales core (6 sitios) + resolución de 3 duplicados transitorios; snapshots regenerados tras estabilizar el campo.
- Pendiente ajeno (NO repuesto): literales en `vantadb-python/src/lib.rs` (5), `vantadb-wasm/src/lib.rs` (5), `vantadb-mcp` (5) — regiones prohibidas para SCH-03; los cierra SCH-04/LEAD (su contrato toca esas crates).

**Deuda / handoff:** `docs/api/IQL.md` aún describe IQL v1 sin `AS OF` (SCH-07 lo cierra — prohibido tocar `docs/**` en esta wave). `MIN_COMPAT_VERSION` intacto; sin cambios de persistencia/índices.

## Recitation
```
=== RECITATION ===
Objetivo activo: SCH-03 — AS OF / point-in-time + filtros valid_at + exclude_superseded
Estado: in-progress (review P2-01 ✅ APPROVE; Optional-2 aplicado + verify ✅; commit = LEAD)
Última acción: Fix Optional-2 — `merge_as_of_clauses` rechaza `AS OF` duplicado (Failure posicionado en la cláusula repetida) + `iql_parse_error_message` → "AS OF specified more than once"; tests parser + executor. Diferidos anotados: Opt-1 (SCH-07/follow-up growth) y Nit-3 (`all_consuming` → FIND, Backlog bloqueado por WIP ajeno).
Resultado: PARTIAL
Próxima acción: LEAD — commit `feat(iql): AS OF / point-in-time + filtros valid_at + exclude_superseded extendido (SCH-03)` (review ya ✅) + plan/Backlog/progreso
Contrato: AS OF IQL (IQL_VERSION=2 + feature-detect) + params search/list + ventana valid_at/invalid_at + exclude_superseded extendido + cursor con resume → verificado (2438/2438 full; 172/172 scoped fix; evidencia por cláusula en §Verificación)
Invariantes: predicado D3 verbatim; default sin cambios; sin reloj en filtros (now 1× en boundary, lazy); fingerprint con params nuevos; subquery con AS OF y AS OF duplicado = parse error; IQL v1 sigue parseando
Deuda: ninguna nueva; diferidos review anotados con dueño (SCH-07 / FIND)
Próxima tarea si completa: SCH-04/SCH-05 (wave F3.3a) — luego SCH-06
last-synced: 2026-09-29
=== END RECITATION ===
```
