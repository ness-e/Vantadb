# WIRE-08: Range/group_by + cursor con resume + RRF en CBO + rewriting + MMR

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 22, F2 — bloque completo leído)
- **Fuente:** Backlog `:987` (fila WIRE-08) + plan `:574-597` + MGR-16 (`Backlog:856`, 🆕 Pendiente — **no existe task file ni spec en el árbol**)
- **Esfuerzo:** 🟡 3-5d (appetite max 5d) · **Prioridad:** 🟠 · **Tipo:** feature-add (`feat(search):`)
- **Turns estimados:** 8 steps · **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ IN PROGRESS
- **Branch:** develop · **Commit esperado:** `feat(search): range/group_by + cursor + MMR + RRF-CBO (WIRE-08)` (LEAD commitea; el worker NO commitea)
- **SDP (Paso 0b):** `campaign_discover_skills_v2 phase=BUILD` → base: campaign-executor, progreso; pinned: documentation-and-adrs (API docs), api-and-interface-design; lifecycle BUILD: incremental-implementation, test-driven-development; + source-driven-development (docs oficiales Milvus/Qdrant — obligatorio para la tabla) + rust-write-tests (suites Rust). Detección v2 marcó `taskType=Documentation` (falso — es feature-add; se corrige a mano por evidencia de contrato: "implementados con tests").
- **Incógnitas (uphill):** 1 (semántica exacta del cursor: offset vs keyset — resuelta en Spec) · **Pendientes (downhill):** 8 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `MemorySearchRequest` → 116 callers (`src/sdk/types/search.rs`, `src/sdk/search/*`, `src/graphrag/seed.rs`, `vantadb-mcp`, `vantadb-server`, `vantadb-python`, `vantadb-node`, `vantadb-wasm`, `providers/shared_py.rs`, `src/cli_handlers/search.rs`, tests/examples) — todos literales con `..Default::default()` o a corregir mecánicamente |
| Callees | `search_impl` (mod.rs) → `lexical_search` / `vector_memory_search` / `sparse_memory_search` / `fusion::*`; `planner::optimize_and_compile` → `PhysicalVectorSearch`/`PhysicalTextFilter`/`OperatorRegistry` |
| Implicaciones | Campos nuevos `Option<_>` + `#[serde(default)]` = aditivo (no rompe JSON de bindings); struct literals internos requieren fix mecánico; CBO intacto salvo rama nueva opt-in por `search_profile` (additive); `search()` sigue devolviendo `Vec<MemorySearchHit>` (delega en `search_page`) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/sdk/serialization/vector_types.rs` (10-38 + tests), `src/sdk/search/mod.rs` (445L), `src/sdk/search/fusion.rs` (1-437), `src/sdk/search/vector.rs`, `src/sdk/search/lexical.rs`, `src/sdk/search/hybrid.rs`, `src/planner.rs` (538L), `src/operator_registry.rs` (498L), `src/physical_plan/filter.rs`, `src/physical_plan/vector.rs` (1-120), `src/search_profile.rs`, `src/sdk/types/record.rs` (130-209), `src/sdk/api/namespaces.rs` (1-188), `src/sdk/types.rs` (95-174), `src/sdk/mod.rs` (1-34), `src/lib.rs` (180-199), `.opencode/rules/{core-engine,query-dsl,indexes,api-contract}.md`, `scripts/validate-docs-coverage.ps1` (60-99)
- **Referencias hacia dentro:** `MemoryListOptions.cursor` (record.rs:155) + `list()` (namespaces.rs:47-188) = precedente de paginación con `next_cursor`/page-full; `RRF_K=60` (search_profile.rs:20); `fuse_rrf_*` (fusion.rs); `HIGH_SELECTIVITY_THRESHOLD` (planner.rs:33); `OperatorRegistry` (planner.rs:169-173, 321-326)
- **Referencias entrantes:** `validate-docs-coverage.ps1:64` escanea `src/sdk/search/mod.rs` → `pub fn search_page` DEBE documentarse en `docs/api/EMBEDDED_SDK.md`; adapters MMR client-side (`integrations/langchain/.../vectorstore.py:149-244`, `integrations/llamaindex/.../vectorstore.py:238`) = semántica de referencia a centralizar (no se tocan en esta tarea)
- **Veredicto impacto:** alto — 4 símbolos públicos nuevos (`RangeFilter`, `GroupByConfig`, `MmrConfig`, `MemorySearchPage` + campo `cursor`/`range`/`group_by`/`mmr` en request) y 1 método (`search_page`); todos aditivos; CBO solo gana una rama opt-in

## Contrato (verbatim del plan)
"range/radius + group_by + cursor con resume en search implementados con tests (resume estable con writes intercalados y page-full/last-page) Y paridad Milvus/Qdrant documentada en tabla capacidad×capacidad Y RRF-CBO/rewriting/MMR implementadas o DEFER explícito por pata en el task file (motivo + dueño) Y suites search/planner verdes"

## Spec (SDD — feature-add; decisiones por evidencia, MGR-16 inexistente)

> Gate mecánico spec-first: MGR-16 (`Backlog:856`) está 🆕 Pendiente y **no existe en el árbol** (`docs/dev/tasks/MGR-16.md` ausente; `rg rewrit src/` = 0 código de query rewriting). Por instrucción del plan ("consumirla o DEFER anotado por pata"), las decisiones se toman contra **documentación oficial Milvus/Qdrant** (fuentes en §Paridad) y el patrón existente del repo (cursor de `list`). Sin spec upstream no se inventan semánticas: cada ítem cita su fuente.

| # | Decisión | Opciones (+tradeoff) | Resuelto | Evidencia |
|---|----------|----------------------|----------|-----------|
| 1 | Semántica de range | A) score-space `RangeFilter{min_score,max_score}` inclusivo (coherente con `MemorySearchHit.score` = similitud, higher=better) / B) distance-space espejo Milvus `radius`/`range_filter` (requiere invertir por métrica y duplicar convenciones) | **A** | Milvus docs: `radius`/`range_filter` son distance-space y **invierten según métrica** (COSINE: `radius < d <= range_filter`; L2: `range_filter <= d < radius`) — https://milvus.io/docs/range-search.md. Qdrant usa `score_threshold` score-space — `qdrant/qdrant/lib/api/src/rest/schema.rs` (`score_threshold: Option<ScoreType>`) |
| 2 | Semántica de group_by | A) `GroupByConfig{field,group_size}` con `top_k` = tope de **hits totales** (contrato propio claro) / B) espejo Milvus `limit` = nº de grupos | **A** | Milvus: `limit`=nº grupos + `group_size` por grupo + `strict_group_size` — https://milvus.io/docs/grouping-search.md. Qdrant: `group_by`/`group_size`(def 3)/`limit`(def 10, grupos) — schema.rs `QueryBaseGroupRequest`. Se documenta la diferencia en la tabla; sin `strict` (YAGNI) |
| 3 | Cursor: offset vs ancla | A) **ancla por identidad** `(key,node_id)` + `offset` consumido (presupuesto de ventana + fallback) + fingerprint de plan / B) offset puro (duplicados con writes intercalados) / C) snapshot server-side (memoria) | **A (best-effort, decisión LEAD post-review C1)** | Pre-mortem F1 del plan ("snapshot de plan + offset determinista"); el test de resume con writes intercalados demostró que el ancla **por score** es inestable (BM25/IDF se recalculan corpus-wide con cada escritura) → ancla por identidad + ventana con retry acotado (`MAX_PAGE_WINDOW`). Contrato explícito: **best-effort, no snapshot** — el reorder a través del ancla puede repetir un hit (test `test_search_page_resume_best_effort_when_writes_reorder_ranks`); cursor fuerte (snapshot/sesión) → **FIND-183** (dueño vanta-engine). Determinismo del orden: `fusion::sort_hits` + sorts de `lexical.rs`/`vector.rs` |
| 4 | Cursor × MMR/group_by | A) rechazo explícito (`ERR_CURSOR_UNSUPPORTED`) / B) permitir con semántica laxa (duplicados) | **A** | MMR reordena por diversidad dependiente del conjunto (Qdrant `candidates_limit` cap 16384, sin paginación MMR); group_by con conteos por página rompería el tope por grupo. Error estable documentado (evita corrupción silenciosa de página) |
| 5 | MMR λ y relevancia | A) `MmrConfig{lambda, fetch_k}` con relevancia min-max normalizada en la ventana / B) `score` crudo (escalas RRF≈0.03 vs coseno≈1 → λ sin sentido) | **A** | Fórmula clásica MMR `λ·rel − (1−λ)·max cos`; Qdrant `diversity∈[0,1]` (higher=diversidad) = espejo `lambda = 1−diversity` (schema.rs `Mmr`); fetch_k default 5× (`Backlog:987` "fetch_k 5-10×"), cap 16384 (Qdrant `candidates_limit` max) |
| 6 | RRF-CBO (planner) | A) operador de fusión **local al planner** (struct `PhysicalRrfFusion` en `planner.rs`) opt-in por `search_profile` / B) variante nueva en `LogicalOperator` + `OperatorRegistry` (requiere `query.rs`/`operator_registry.rs` fuera de scope) / C) reescribir el planner (prohibido por stop condition) | **A** | Stop condition del plan: "extensión por operador registrado, NO reescribir el planner". A mantiene CBO intacto sin profile (test de no-regresión) y cierra la divergencia IQL↔SDK cuando hay profile |
| 7 | Rewriting | A) DEFER explícito / B) implementar query rewriting sin spec | **A — DEFER** | MGR-16 inexistente; semántica ambigua (rewriting de usuario vs. plan lógico); `rg -in rewrit src/` = solo storage/migration. **Motivo:** sin spec MGR-16 no hay contrato de comportamiento. **Dueño:** vanta-engine (consume MGR-16 cuando el spec exista) |

## Steps (PLAN → ACT → VERIFY)

| # | Step | Estado | Verify |
|---|------|--------|--------|
| 1 | **Tipos + contrato** (`vector_types.rs`): `RangeFilter`, `GroupByConfig`, `MmrConfig`, `MemorySearchPage`, campos `range`/`group_by`/`mmr`/`cursor` en `MemorySearchRequest`; validación boundary; re-exports (`types/search.rs`, `sdk/mod.rs`, `lib.rs`); serde roundtrip/default tests | ✅ | `cargo check -p vantadb --all-targets` ✅ + `nextest -E 'test(search_request)'` 21/21 ✅ |
| 2 | **Range + group_by en el pipeline** (nuevo `sdk/search/page.rs` + wiring en `mod.rs`) con tests (filtro inclusivo, grupos, tope total, missing-field) | ✅ | `nextest -E 'test(range) or test(group)'` ✅ |
| 3 | **Cursor con resume** (`page.rs`: token JSON opaco con fingerprint + ancla de identidad + `search_page`; `search()` delega) con tests: page-full/last-page, resume con writes intercalados, fp mismatch, rechazo MMR/group_by | ✅ | `nextest -E 'test(cursor) or test(page)'` ✅ (fix real: ancla por identidad — los scores BM25 cambian con la escritura; ventana de fetch con retry acotado) |
| 4 | **MMR core** (`sdk/search/mmr.rs` + wiring) con tests: diversidad, λ=1 identidad, fetch_k, sin vectores, determinismo | ✅ | `nextest -E 'test(mmr)'` ✅ |
| 5 | **RRF-CBO** (`planner.rs`: `PhysicalRrfFusion` + routing opt-in por profile) con tests: unión IQL↔SDK (profile on/off), math RRF, modos Keyword/Vector intactos | ✅ | `nextest -E 'test(planner) or test(rrf)'` ✅ |
| 6 | **Docs**: `docs/api/SEARCH_PARITY.md` (tabla capacidad×capacidad con fuentes citadas) + `search_page`/campos nuevos en `docs/api/EMBEDDED_SDK.md` | ✅ | `pwsh -NoProfile scripts/validate-docs-coverage.ps1` → 0 gaps (30 items SDK) ✅ |
| 7 | **Rewriting: DEFER** documentado en §Pata 4 (motivo + dueño) — sin código | ✅ | evidencia `rg -in rewrit src/` (solo storage/migration) + MGR-16 ausente |
| 8 | **Cierre**: verify full (fmt/clippy/nextest/docs) + contrato + recitation | ✅ | ver §Evidencia |

## Evidencia (2026-09-28)

| Comando | Resultado |
|---------|-----------|
| `cargo fmt --all --check` | ✅ exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ 0 warnings (fix: `Box` en `ParsedSearchRequest::Ready` de vantadb-mcp por `large_enum_variant`) |
| `cargo nextest run --profile audit -p vantadb --build-jobs 2` | ✅ **2369/2369 passed** (2 skipped pre-existentes) — incluye los 4 tests nuevos del fix post-review |
| `cargo nextest run --profile audit -p vantadb -E 'test(search) or test(planner) or test(cursor) or test(group) or test(mmr) or test(rrf) or test(page)'` | ✅ **293/293 passed** (repro del reorder + banda de rango incluidos) |
| `cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2` | ✅ 136/136 |
| `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` | ✅ 549/549 |
| `cd vantadb-node; cargo check --all-targets` (standalone, no workspace) | ✅ exit 0 (O6) |
| `cargo test --doc -p vantadb` | ✅ 13/13 (incluye el doctest de `search_page`) |
| `pwsh -NoProfile scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| `tests/api/public_api.txt` | ✅ regenerado (+194 líneas: `search_page` + 4 tipos nuevos, aditivo) |
| `tests/snapshots/query_result_*__search_request*.snap` | ✅ regenerados (9 archivos, +4 líneas c/u) |
| `cargo nextest run --profile audit --workspace --build-jobs 2` | ⚠️ **pre-existente ajeno**: `vanta-memory/tests/smoke.rs:15` const-assert `cfg!(not(feature = "llm-driver"))` falla con la unificación de features del workspace (`vantadb-mcp`/`vanta-proxy` habilitan `llm-driver`). No relacionado con WIRE-08 (el diff no toca features); el gate canónico `dev-tools/verify.ps1:75` corre `-p vantadb` |

## Fixes post-review P2-01 (ronda 1 — dictamen del LEAD: ❌ CHANGES REQUIRED)

> El dictamen completo vive en la sección Review que llena el LEAD. Acá solo las acciones aplicadas.

| ID | Acción | Evidencia |
|----|--------|-----------|
| **C1** | Contrato best-effort del cursor corregido en 5 lugares (`page.rs` rustdoc del módulo, rustdoc de `search_page`, `EMBEDDED_SDK.md` fila `search_page`, `SEARCH_PARITY.md` fila cursor + §Deliberate differences #3, `WIRE-08.md` invariantes) + nota en el campo `cursor` del request. **No** se implementó token de identidades ni sesión server-side (fuera de scope) → **FIND-183** propuesto (dueño vanta-engine) | test `test_search_page_resume_best_effort_when_writes_reorder_ranks` (repro del revisor: page1 [P,Q] ancla=Q → 20 writes invierten P/Q → page2 [P,W0] con P duplicado, assert del comportamiento CONOCIDO) |
| **R1** | La ventana de fetch se profundiza (doubling acotado por `MAX_PAGE_WINDOW`) cuando un selector puede acortar la página y la lista no está agotada, usando el largo **PRE-selector** (`ranked_len < window` = agotada). Aplica a `range.max_score`, cursor y `group_by` | test `test_range_max_score_deepens_the_window_until_the_band_fills` (banda bajo la ventana: top_k=2 con max_score = score del 3º → devuelve 3º+4º, no vacío) |
| **R2.1** | `SEARCH_PARITY.md` MMR: "Core-owned (adapters no longer reimplement it)" → "Core MMR available; adapter migration deferred — out of WIRE-08 scope" | `docs/api/SEARCH_PARITY.md` fila MMR |
| **R2.2** | Comentario stale en `planner.rs` ("el CBO no fusiona RRF") → actualizado: `rrf_k` del profile SÍ alimenta `PhysicalRrfFusion` en modo Hybrid; `candidate_k` sigue siendo solo-SDK | `src/planner.rs` (comentario MEM-01/WIRE-08) |
| **O3** | Test de rango vacío | `test_range_matching_nothing_returns_empty_page_without_cursor` |
| **O4** | `top_k == 0` decidido y documentado: se mantiene el early-return (ERR-033: limit 0 = sin registros), página vacía sin cursor | test `test_search_page_top_k_zero_is_an_empty_page` + rustdoc de `search_page` |
| **O5** | Cursor no aplica a multi-ns, anotado en `search_multi` rustdoc + rustdoc de `search_page` | `src/sdk/search/multi.rs` |
| **O6** | `vantadb-node` compila standalone ✅ + indentación rota corregida en `providers/shared_py.rs:238-241` y `vantadb-node/src/lib.rs:813-816` (escaparon al `cargo fmt` por no ser workspace members) | `cd vantadb-node; cargo check --all-targets` exit 0; `cargo fmt --all --check` verde |
| **O1/O2** | Aceptados y anotados (deuda §Invariantes): O1 = selectores post-fetch con ventana acotada + retry; O2 = brazo léxico del CBO por scan order | §Invariantes + `SEARCH_PARITY.md` §4 |

## Paridad Milvus/Qdrant (borrador → `docs/api/SEARCH_PARITY.md`)

| Capacidad | VantaDB (WIRE-08) | Milvus | Qdrant |
|-----------|-------------------|--------|--------|
| Filtro por rango de score | `MemorySearchRequest.range = {min_score, max_score}` (inclusivo, score-space) | `radius` + `range_filter` (distance-space; signo por métrica) [R1] | `score_threshold` (min score) [R2] |
| Agrupación | `group_by = {field, group_size}`; `top_k` = tope de hits | `group_by_field`, `group_size`, `strict_group_size`; `limit` = nº grupos [R3] | `group_by`, `group_size`, `limit`(grupos) [R2] |
| Paginación | `search_page` + `cursor` (keyset ancla + fingerprint; page-full→`next_cursor`) | SearchIterator (sesión) / REST `offset` (ventana 16 384) [R4] | `offset` (nota: offsets grandes degradan) [R2] |
| MMR | `mmr = {lambda, fetch_k}` (λ clásico; default 5×, cap 16 384) | — (no nativo) | `mmr = {diversity, candidates_limit}` [R2] |
| Fusión RRF | SDK (`search`) + CBO opt-in por `search_profile` (IQL) | — (RRF en SDKs cliente) | `fusion: rrf` con `k`/`weights` (prefetch) [R2] |
| Rewriting | DEFER (sin spec MGR-16) | — | — |

**Fuentes:** [R1] https://milvus.io/docs/range-search.md · [R3] https://milvus.io/docs/grouping-search.md · [R4] https://milvus.io/docs/with-iterators.md · [R2] https://github.com/qdrant/qdrant/blob/master/lib/api/src/rest/schema.rs (fuente del OpenAPI servido en api.qdrant.tech; verificado 2026-09-28).

## Patas — estado final

| Pata | Estado | Detalle |
|------|--------|---------|
| 1. range/radius + group_by | ✅ implementada | `RangeFilter` (bounds inclusivos, score-space) + `GroupByConfig` (field/group_size; `top_k` = tope de hits) en `MemorySearchRequest`; post-proceso en `page.rs`; validación boundary con marcador `SEARCH_OPTIONS_INVALID` |
| 2. cursor con resume | ✅ implementada | `search_page` + token opaco (fingerprint de plan + offset consumido + ancla de **identidad**); page-full ⇒ `next_cursor`; resume estable ante writes intercalados (test) y ante shifts de score BM25; rechazo con `mmr`/`group_by` |
| 3. MMR centralizado | ✅ implementada | `sdk/search/mmr.rs` (core): λ clásico, relevancia min-max normalizada, coseno entre vectores densos, `fetch_k` default 5× cap 16 384; opt-in por request. Adapters (langchain/llamaindex) quedan como consumidores — su migración no está en el alcance de WIRE-08 (fuera de `src/`) |
| 4. rewriting | **DEFER** | Motivo: MGR-16 no existe (spec ausente; `docs/dev/tasks/MGR-16.md` no está en el árbol; `rg -in rewrit src/` = solo storage/migration). Dueño: vanta-engine (consumir MGR-16 cuando exista). Sin spec no se inventa semántica (Gate mecánico spec-first) |
| 5. RRF como operador del CBO | ✅ implementada | `PhysicalRrfFusion` local en `planner.rs` + routing opt-in por `search_profile` (modo Hybrid con brazos vector+texto). Sin profile = path previo byte-idéntico (test de no-regresión). Limitación documentada: el brazo léxico del CBO rankea por scan order (el ranking BM25 físico queda para el spec MGR-16) |

## Invariantes de dominio (handoff — MUST)
- `search()` conserva firma y semántica (sin campos nuevos = mismo resultado; test `test_search_without_cursor_returns_the_same_hits_as_search_page`).
- Cursor: **best-effort, no snapshot** — un hit puede repetirse (o saltearse) si writes intercalados reordenan su rank a través del ancla (BM25/IDF se recalculan corpus-wide); garantizado: writes que rankean *antes* del ancla nunca duplican (el ancla se saltea por identidad donde esté). Página corta ⇒ `next_cursor = None`; `top_k == 0` ⇒ página vacía sin cursor (ERR-033); token solo válido para el mismo plan (fingerprint) y mismo proceso; fallback por offset solo si el ancla fue borrada; no aplica a `search_multi`/`search_all` (single-namespace).
- CBO sin `search_profile` = comportamiento previo exacto (test `rrf_cbo_fuses_arms_when_profile_is_present` rama sin profile).
- Serialización: solo campos `Option` con `#[serde(default)]` (JSON viejo sigue deserializando; test `test_search_request_without_new_fields_deserializes`).
- **Deuda pendiente:** rewriting (DEFER MGR-16), paginación con MMR/group_by (rechazo documentado), techo de ventana `MAX_PAGE_WINDOW=10_000`, ranking BM25 del brazo léxico en el CBO (scan order), migración de adapters a MMR core (fuera de alcance), `vanta-memory/tests/smoke.rs` pre-existente vs unificación `--workspace`.
- **FIND-183 propuesto (cursor fuerte):** cursor con snapshot/sesión server-side (o token de identidades) que elimine el best-effort del resume bajo reorder — dueño: vanta-engine; ref: `src/sdk/search/page.rs` (ancla de identidad + `MAX_PAGE_WINDOW`).
- **O1/O2 (review, aceptados):** O1 = `group_by`/`range` post-fetch con ventana acotada (documentado en `SEARCH_PARITY` §Deliberate differences + retry de ventana para `max_score`/cursor/group_by); O2 = el brazo léxico del CBO rankea por scan order (documentado en `SEARCH_PARITY` §4).

## Risk Register
| Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger |
|--------------|--------|------------------------|---------|
| 🟡×🟠 | Cursor no resumible | ancla de identidad + fp + test con writes intercalados (detectó el fallo real del ancla por score) | test resume rojo |
| 🟡×🔴 | RRF-CBO rompe IQL | opt-in por profile + test de no-regresión (sin profile = 0 cambio) | suite planner roja |
| 🟡×🟡 | Scope creep 5 patas | orden por valor + DEFER rewriting documentado | review P2-01 |

## Herramientas necesarias
- codegraph/CBM antes de grep · `cargo nextest run --profile audit -p vantadb -E '...'` · clippy `-D warnings` · `CARGO_BUILD_JOBS=2` · `pwsh -NoProfile scripts/validate-docs-coverage.ps1` · `campaign_verify_cmd` por step
- **Skills cargadas (SDP):** campaign-executor + progreso (base) · documentation-and-adrs + api-and-interface-design (pinned API docs) · incremental-implementation + test-driven-development (BUILD) · source-driven-development (docs oficiales Milvus/Qdrant) · rust-write-tests (suites Rust)

## Review (GATE P2-01 — vanta-review adversarial, sesión distinta)

- **Revisor:** `vanta-review` fresco adversarial (sesión `ses_f170d176effe5yfV43A1GAKF7w`; ≠ autor `ses_f1773274dffeM5h9he06nqF88y`) — 2 rondas (❌ C1/R1/R2 → fixes → ✅ delta APPROVE).
- **Ronda 1 (❌):** C1 — el cursor duplicaba con reorder BM25 (repro: page1 [P,Q] → 20 writes → page2 [P,W0]); R1 — `range.max_score` truncaba en silencio (banda bajo la ventana → 0 hits); R2 — 2 claims de docs falsos.
- **Fixes (decisión LEAD):** C1 → contrato best-effort en 5 lugares + test del reorder + FIND-183 (cursor fuerte, futuro); R1 → ventana profundizable (`MAX_PAGE_WINDOW=10_000`, largo pre-selector) + test; R2 → claims corregidos; O3/O4/O5/O6 aplicados.
- **Ronda 2 (✅ APPROVE):** repros re-corridos — C1 documentado/testeado, R1 corregido (banda → 2 hits, `next_cursor=true`); gates 2369/2369 + 303/303 + clippy/fmt/coverage/doctest verdes; 0 debilitamiento (+4 tests, 0 eliminados).
- **Veredicto:** ✅ **APPROVE** (2026-09-28) — contrato completo; ACCEPT habilitado (payload review fresh HARD-07).

## Context Save Point
- **Estado:** ✅ COMPLETO + fixes post-review P2-01 ronda 1 aplicados (2026-09-28). 8/8 steps ✅; verify mecánico verde (fmt/clippy workspace/docs/nextest scoped 2369+293+136+549 + node standalone + doctests); pendiente: re-review del LEAD + commit local.
- **Commit esperado:** `feat(search): range/group_by + cursor + MMR + RRF-CBO (WIRE-08)` — NO ejecutado por el worker (política owner: el lead commitea).
- **Siguiente tarea:** MGR-10 (F3).
