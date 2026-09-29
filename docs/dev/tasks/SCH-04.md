---
title: "SCH-04: Scores asserted/derived consumibles (slice 0.8.0)"
kind: task
description: Cláusulas a verificar (matriz de cierre)
---

# SCH-04: Scores asserted/derived consumibles (slice 0.8.0)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 29 (F3) · **Origen:** plan L761-785; ADR-0046 §D2/§D4b/§D4c/§D4d; MGR-12 §6.1
- **Fuente del prompt:** sub-agente vanta-engine (orquestador pipeline) — wave F3.3a, co-batch con SCH-03 (regiones disjuntas; `vector_types.rs`/`page.rs` compartidos al final)
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠 · **Tipo:** feature-add (wire cross-binding + filtro opt-in)
- **Branch:** develop · **Commit:** (LEAD) · **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ IN PROGRESS · **Incógnitas (uphill):** 0 · **Pendientes (downhill):** steps abajo

## Contrato (verbatim del prompt de tarea)
> "`confidence` por registro (valor + clase asserted|derived + `last_validated`, campos fijados por ADR SCH-01) visible en search/get/list del SDK core y propagado en HTTP/MCP vía serde `#[serde(default)]` (sin breaking) Y expuesto en los 4 bindings (Py/TS/Node/WASM con stubs/d.ts sincronizados) Y consultable (filtro opt-in por umbral/clase junto a `range`/`exclude_superseded`, hasheado en el fingerprint)"

**Cláusulas a verificar (matriz de cierre):**

| # | Cláusula | Superficie | Evidencia esperada |
|---|----------|-----------|--------------------|
| C1 | `confidence`/`confidence_class`/`last_validated_at_ms`/`derived_from` visibles en search/get/list SDK | Core Rust | test roundtrip + get/search/list (ya en `MemoryRecord`, SCH-02) |
| C2 | Propagado en HTTP/MCP por serde `#[serde(default)]`, sin breaking | `SearchPageV2`/MCP get/list/hits | test serde del hit/record + e2e si barato |
| C3 | Expuesto en Py/TS/Node/WASM con stubs/d.ts sincronizados | 4 bindings | getters py + pyi + drift test; d.ts node; `types.ts` TS; `memory_record_to_js` + d.ts wasm |
| C4 | Filtro opt-in `min_confidence` hasheado en fingerprint | `MemorySearchRequest` + `page.rs` | tests: filtra, default None no-op, fingerprint mismatch, boundary reject |
| C5 | Ranking/orden por defecto NO cambia | `fusion.rs`/assembly | diff nulo en ranking; tests existentes verdes |

**Alcance del filtro por superficie (precedente `range`/`exclude_superseded`):** core + HTTP (auto por `#[serde(flatten)]`) + Py kwarg + WASM `SearchRequest` + TS `SearchRequest` + MCP arg/schema **+ Node `parse_search_request`** (necesario para que el passthrough TS-native no fuera un drop silencioso; `range`/`exclude_superseded` siguen sin exponerse en Node). **`MemoryListOptions` NO** (región SCH-03; su `min_confidence` de list queda a SCH-05/07). **Filtro por clase: NO** (ADR-0046 solo fija `min_confidence`; "umbral/clase" del plan → umbral).

## Re-baseline post-SCH-02 (re-verificado 2026-09-28, HEAD `aae4f7e5`)
- **Los campos YA EXISTEN** en `MemoryRecord` (`src/sdk/types/record.rs:225-235`: `confidence_class`, `confidence` con `default_confidence`, `last_validated_at_ms`, `derived_from`) y viajan por serde (snapshots `list_page_*`/`search_hit_basic` ya los muestran). `rg 'confidence'` en bindings = **0 hits de record** (solo `confidence_score` de grafo).
- **Gap real del slice:** (a) exposición en los 4 bindings (getters/objetos JS/d.ts/.pyi); (b) verificación HTTP/MCP (serde ya propaga; falta evidencia); (c) filtro opt-in `min_confidence` en `MemorySearchRequest` + fingerprint; (d) tests (roundtrip + getters + snapshot deliberado).
- El bloque del plan fue escrito pre-SCH-02 ("record sin confianza") — esta sección lo re-baselina.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Edita** | `vantadb-python/src/types.rs` (getters 2 clases) + `vantadb_py/*.pyi` · `vantadb-node/dts-header.d.ts` + `index.d.ts` (regen `napi build`) · `vantadb-ts/src/types.ts` (+ test) · `vantadb-wasm/src/lib.rs` (`memory_record_to_js`) + `vantadb_wasm.d.ts` · `src/sdk/serialization/vector_types.rs` (`MemorySearchRequest` +1 campo) · `src/sdk/search/page.rs` (validate/apply/fingerprint) · `vantadb-python/src/lib.rs` (kwarg search/search_multi) · `vantadb-wasm/src/lib.rs` (wire `SearchRequest`) · `vantadb-ts/src/{types,native,vantadb}.ts` · `vantadb-mcp/src/handlers/tools.rs` (arg+schema `memory_search`) · literales workspace (`MemorySearchRequest`) + snaps deliberados + `tests/api/public-api.txt` |
| **Callers** | `MemorySearchHit` = record+score+explanation (`vector_types.rs:195-202`); ranking no lo conoce (`fusion.rs:72-131`, `page.rs:317-330`) → sin cambios de orden. Hits consumidos por: Py `VantaPySearchHit`, Node wire napi serde_json, TS `SearchHit`, WASM `search_hit_to_js`, MCP `text_content_hits_with_budget`, HTTP `SearchPageV2` |
| **Callees** | `MemoryRecord` (campos SCH-02; solo lectura) · `plan_fingerprint` (`page.rs:137-166`) · `validate_search_options` (`page.rs:72`) · `Embedded::search` paths |
| **Implicaciones** | Aditivo puro en wire (`#[serde(default)]`, default `None` = comportamiento actual); cursor de otro umbral ⇒ `SEARCH_CURSOR_INVALID` (mejora, no breaking); snapshots `search_request_*` regenerados **deliberadamente**; `public-api.txt` re-snapshot; Node/TS WASM d.ts sincronizados; docs/api diferidas (WIP ajeno) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/architecture/adr/ADR-0046-schema-v2-migracion-unica.md` · `docs/dev/research/mgr-12-confianza.md` · `.opencode/rules/api-contract.md` · `.opencode/references/clean-code-clean-architecture.md` (Apéndice V) · `docs/dev/tasks/SCH-02.md` · `docs/dev/tasks/WIRE-03.md` · `vantadb-ts/src/guards.ts` · `vantadb-python/tests/test_stub_drift.py` · `vantadb-wasm/src/vantadb_wasm.d.ts` (:135-264).
- **Archivos leídos (rangos clave):** `src/sdk/types/record.rs` (:52-181, :225-235, :416-425) · `src/sdk/serialization/vector_types.rs` (:95-169) · `src/sdk/search/page.rs` (:40-119, :120-209, :248-363) · `src/sdk/search/mod.rs` (:60-144) · `vantadb-python/src/types.rs` (:52-181, :280-419) + `vantadb_py/vantadb_py.pyi` (:95-174) + `vantadb-python/src/lib.rs` (:1155-1204, :2300-2409) · `vantadb-node/src/lib.rs` (:80-149, :780-839) · `vantadb-ts/src/types.ts` (:40-199) · `vantadb-wasm/src/lib.rs` (:140-214, :1150-1358, :2235-2301) · `vantadb-mcp/src/handlers/tools.rs` (:313-412, :1504-1633, :1780-1859, :3200-3319) · `vantadb-mcp/src/validation.rs` (:380-450) · `src/server/handlers.rs` (:495-554) · `tests/sdk_serialization.rs` (:1-120) · `tests/query_result_advanced.rs` (:1-100) · `tests/snapshots/query_result_basic__search_hit_basic.snap` · `scripts/validate-docs-coverage.ps1` (:1-219).
- **Referencias hacia dentro:** ADR-0046 §D2 (wire `min_confidence` + fingerprint), §D4b (rechazo Derived), §D4c/D4d (backfill/tests ya en SCH-02), §D6 (record canónico); MGR-12 §6.1 (exposición + filtro); plan Task 29 (:761-785).
- **Referencias entrantes:** SCH-05 (consume `min_confidence`/abstención; `include_quarantined` en `MemoryListOptions`), SCH-06 (roundtrip/chaos), SCH-07 (superficies + docs/api), VER-08 (calibración), ICP-03 (paridad).
- **Veredicto impacto:** medio — aditivo en wire; sin cambio de ranking; blast radius mecánico por literales de `MemorySearchRequest` + snapshots deliberados. **Coexistencia SCH-03:** sus regiones (`parser/`, `executor.rs`, params temporales, `MemoryListOptions`, `memory.rs`) NO se tocan; edits a `vector_types.rs`/`page.rs` al FINAL, re-leídos justo antes.

## Spec (feature-add — decisiones por evidencia)

| # | Decisión | Alternativas | Elegido | Evidencia |
|---|----------|--------------|---------|-----------|
| 1 | Alcance real del slice | (a) re-implementar campos / (b) **exposición + filtro + tests** (campos ya en SCH-02) | (b) | `record.rs:225-235` + `rg` bindings 0 hits; re-baseline del prompt |
| 2 | Wire del filtro | (a) `MemorySearchRequest.min_confidence: Option<f32>` / (b) struct nuevo / (c) `MemoryListOptions` también | (a) | ADR-0046 §D2 (verbatim); list = región SCH-03 (reparto wave) |
| 3 | Semántica | (a) post-ranking junto a `exclude_superseded` / (b) pre-ranking en el pipeline | (a): `hit.record.confidence >= min` en assembly; **`selectors_can_shorten` SÍ incluye `min_confidence`** (`page.rs:344`) — puede acortar la página, espejo de los filtros temporales de SCH-03 | plan "sin index change" (:768); test `test_search_min_confidence_page_fills_when_enough_candidates_exist` |
| 4 | Validación boundary | (a) finito + `[0,1]` ⇒ `SEARCH_OPTIONS_INVALID` / (b) clamp silencioso | (a) | patrón `range`/`mmr.lambda` (`page.rs:72-117`); D4b rationale (nunca mudo) |
| 5 | Fingerprint | hashear `min_confidence.map(f32::to_bits)` | sí | ADR §D2 "hasheados en el fingerprint (`page.rs:157`)" |
| 6 | Exposición por binding | py getters+pyi / node d.ts / ts interface / wasm objeto JS+d.ts | los 4 (patrón `superseded_by`) | contrato C3; precedentes WIRE-03 |
| 7 | Filtro por binding | Py/WASM/TS/MCP/Node sí; list no; clase no | — | ADR solo fija `min_confidence`; Node suma este filtro para que el passthrough TS-native no sea drop silencioso (`range`/`exclude_superseded` siguen fuera de Node) |
| 8 | Snapshot | regen deliberada `search_request_*` (Debug +1 campo); resto intacto | sí | insta `assert_debug_snapshot`; plan "snapshot deliberado" |
| 9 | Docs | diferir (`docs/api/scores.md` etc. = WIP ajeno) | SCH-07/LEAD | instrucción del orquestador (NO editar docs/**) |

## Invariantes de dominio (handoff — MUST)
- **Aditivo puro:** `#[serde(default)]` en el campo nuevo; default `None` = comportamiento actual. Sin breaking.
- **No tocar `MemoryRecord`** (campos fijos por ADR-0046; solo lectura) **ni el ranking** (`fusion.rs`, orden del assembly, `mmr`, `group_by`).
- **Regiones SCH-03 intactas:** `src/parser/`, `src/executor.rs`, params temporales, `MemoryListOptions`, `memory.rs` histórico. Edits a `vector_types.rs`/`page.rs` mínimos y re-leídos justo antes.
- **PROHIBIDOS:** `docs/**`, `Backlog.md`, `perf-bench.yml`, `opencode.jsonc`, plan file. NO cuarentena/abstención (SCH-05).
- Stubs/d.ts sincronizados: `test_stub_drift` (py) / `tsc` (ts) / `napi build` (node) / d.ts wasm.
- Regla dura `-p` + `CARGO_BUILD_JOBS=2`; no commit (LEAD); no self-review (LEAD).

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: 0.** Sin `unsafe` nuevo, sin dependencias nuevas, sin hot-path nuevo (filtro O(n) sobre la página ya materializada, espejo `exclude_superseded`). Límites de calibración L1-L5 (MGR-12 §5.4) se documentan en `scores.md` (diferido SCH-07); grounding/jueces = v1.0 (FIND con dueño si no existe).

## Steps

### Step 1: Python — getters de confianza (Record + SearchHit) + stubs + tests
- **Archivos:** `vantadb-python/src/types.rs`, `vantadb-python/vantadb_py/vantadb_py.pyi`, `vantadb-python/tests/test_sch04.py` (nuevo)
- **RED:** test que lee `rec.confidence`, `rec.confidence_class`, `rec.last_validated_at_ms`, `rec.derived_from` + `record["confidence"]` + `hit.confidence*` en search/get/list → falla (getters inexistentes)
- **GREEN:** 4 getters en `VantaPyMemoryRecord` + 4 en `VantaPySearchHit` + claves `__getitem__`; pyi (members en `SearchHit`/`Record`); sin tocar firmas de métodos
- **Verify:** `maturin develop` + `pytest test_sch04.py test_stub_drift.py` + `cargo check -p vantadb_py` + clippy
- **Estado:** ⬜ PENDING

### Step 2: Node — d.ts (MemoryRecord) sincronizado
- **Archivos:** `vantadb-node/dts-header.d.ts`, `vantadb-node/index.d.ts` (regen `napi build`)
- **Acción:** 4 campos en `MemoryRecord` (wire ya los trae via `serde_json::to_value`; solo tipado)
- **Verify:** `napi build` + `cargo check --manifest-path vantadb-node/Cargo.toml` + vitest node (suite existente)
- **Estado:** ⬜ PENDING

### Step 3: TS — `MemoryRecord` interface + test de tipado
- **Archivos:** `vantadb-ts/src/types.ts`, `vantadb-ts/src/__tests__/sch04.test.ts` (nuevo)
- **Acción:** 4 campos opcionales con doc (semántica asserted/derived); runtime ya passthrough (`_mapRecord`); test con record sintético + guard
- **Verify:** `npm run build` (tsc) + vitest
- **Estado:** ⬜ PENDING

### Step 4: WASM — `memory_record_to_js` + d.ts + test
- **Archivos:** `vantadb-wasm/src/lib.rs`, `vantadb-wasm/src/vantadb_wasm.d.ts`, `vantadb-wasm/tests/wasm_tests.rs`
- **RED:** test de `memory_record_to_js` (via helper, patrón tests existentes) verifica claves `confidence_class`/`confidence`/`last_validated_at_ms`/`derived_from`
- **GREEN:** emitir los 4 (clase como string `"Asserted"|"Derived"` externally-tagged? → usar `serde_wasm_bindgen::to_value(&rec.confidence_class)` para paridad serde; `last_validated_at_ms` como decimal string (policy string-u64); `derived_from` array)
- **Verify:** `cargo test -p vantadb-wasm` (target host) o check wasm32 + d.ts sync
- **Estado:** ⬜ PENDING

### Step 5: HTTP/MCP — verificación de propagación serde (tests)
- **Archivos:** `tests/sdk_serialization.rs` (roundtrip hit/record con campos), `vantadb-mcp/tests/mcp_tests.rs` (get/list/search con campos), `vantadb-server/tests/e2e.rs` (search response con campos) — solo si el harness existente lo permite barato
- **Acción:** sin cambios de código en superficies (serde ya propaga); tests de contrato C2
- **Verify:** tests scoped
- **Estado:** ⬜ PENDING

### Step 6: Core — `min_confidence` en `MemorySearchRequest` + fingerprint (RE-LEER archivos antes)
- **Archivos:** `src/sdk/serialization/vector_types.rs`, `src/sdk/search/page.rs`, literales workspace, `tests/api/public-api.txt`, snaps `search_request_*` (regen deliberada), tests core
- **RED:** tests: (a) filtra hits `< min`, default `None` no-op; (b) boundary: NaN/∞/fuera de `[0,1]` ⇒ `SEARCH_OPTIONS_INVALID`; (c) cursor con umbral distinto ⇒ `SEARCH_CURSOR_INVALID`; (d) serde roundtrip + default
- **GREEN:** campo `#[serde(default)]` + `Default::default()` + validación + `retain` en assembly (junto a exclude_superseded) + hash en fingerprint
- **Verify:** `cargo nextest run -p vantadb -E 'test(sch04) or test(search) or test(cursor)'` + literales workspace + fmt/clippy
- **Estado:** ⬜ PENDING

### Step 7: Filtro por superficie — Py kwarg + WASM + TS + MCP
- **Archivos:** `vantadb-python/src/lib.rs` (search/search_multi kwarg `min_confidence`; pyi/`__init__` si aplica), `vantadb-wasm/src/lib.rs` (`SearchRequest`), `vantadb-ts/src/{types,native,vantadb}.ts`, `vantadb-mcp/src/handlers/tools.rs` (arg + schema `memory_search`)
- **Acción:** aditivo al final de firmas; WASM→core; TS passthrough native+wasm; MCP parse `min_confidence` con validación de tipo (número)
- **Verify:** por binding (pytest, vitest node, vitest ts, cargo test mcp) + tests del filtro end-to-end por superficie
- **Estado:** ⬜ PENDING

### Step 8: Verify full + cierre (sin commit — LEAD)
- **Verify:** `cargo fmt --check` · clippy `-D warnings` (workspace de la task) · `cargo nextest run --profile audit -p vantadb --build-jobs 2` (full, timeout 900) · gates de los 4 bindings · `validate-docs-coverage.ps1` (baseline vs ahora) · `cargo test -p vantadb --test public_api` (re-snapshot) · `git diff` coexistencia SCH-03
- **Estado:** ⬜ PENDING

## Pendientes (§Pendientes)
- **Review P2-01 ✅ APPROVE (2026-09-29) — hallazgos aplicados en este pase** (Optional 1: `minimum`/`maximum` en el schema MCP; Optional 2 + Nits: citas/spec del task file corregidas; FYI `-0.0` sin acción por instrucción del owner). **Único pendiente de la task: commit local (LEAD).**
- **docs/api diferidas por WIP ajeno — retomar en SCH-07/LEAD:** `docs/api/scores.md` (límites L1-L5 + `min_confidence`), `EMBEDDED_SDK.md`/`HTTP_API.md`/`MCP.md`/`openapi.yaml`/`PYTHON_SDK.md`/`TS_SDK.md`/`NODE_SDK.md`/`WASM_API.md` (campos + kwarg). Regla 3 exige doc en el mismo PR del corte → SCH-07 (contrato propio) o LEAD. **NO editar ahora (WIP ajeno sin commitear).**
- `min_confidence` en `MemoryListOptions` (ADR-0046 §D2) → SCH-05/SCH-07 (región SCH-03 hoy).
- Filtro por `confidence_class` (si el owner lo pide) → extender `MemorySearchRequest` con enum (no en ADR-0046).
- Grounding con jueces = v1.0 (FIND con dueño; ya citado en plan/MGR-12 §6.3 — no duplicar).

## Dependencias
- **Consume:** SCH-02 ✅ (`7af34366`; campos + serde defaults + snapshots base) · ADR-0046 `accepted` ✅ · MGR-12 ✅.
- **Destraba:** SCH-05 (abstención consume `min_confidence`), SCH-06 (roundtrip/chaos), SCH-07 (superficies + docs), VER-08 (calibración).
- **nextTask:** SCH-05 · **Co-batch:** SCH-03 (regiones disjuntas; verificar coexistencia al cierre).

## Stop conditions / Pre-mortem / Risk
- **Stop conditions:** calibración empírica → defaults documentados + VER-08 ✅ (no dispara) · filtro complica cursor/fingerprint → exponer sin filtro + FIND (fingerprint = 1 línea; NO dispara salvo bloqueo de SCH-03 en `page.rs`) · jueces/FACTS → NO (v1.0).
- **Pre-mortem:** F1 "scores decorativos" → doc de límites L1-L5 (diferida SCH-07) + test de que el filtro discrimina de verdad (derived 0.45 vs asserted 1.0); F2 record↔nodo divergente → ya cubierto SCH-02 (D6); acá solo lectura; F3 wire/snapshot roto → snapshots deliberados + stub drift + tsc + napi/e2e por binding.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Colisión de edits con SCH-03 en `vector_types.rs`/`page.rs`/literales | editar al final + re-leer justo antes + `git diff` de coexistencia | cierre de wave |
  | 🟡×🟡 | Literales de `MemorySearchRequest` en todo el workspace | `..Default::default()` donde exista; fix mecánico guiado por `cargo check` | step 6 |
  | 🟢×🔴 | Wire cross-binding roto | stubs/d.ts + snapshots deliberados + gates por binding | step 1-7 |

## Herramientas
- `codegraph_codegraph_explore` (bloqueado/stale: lock de otro proceso — se usó lectura directa) · CBM `detect_changes` ruidoso por WIP docs (3656 archivos) → greps dirigidos.
- Rust: `cargo nextest run --profile audit -p vantadb --build-jobs 2` · clippy/fmt scoped · `CARGO_BUILD_JOBS=2` · regla `-p`.
- Py: `maturin develop` (venv `vantadb-python/.venv`) + pytest (`test_sch04.py` + `test_stub_drift.py`). Node: `npm run build` + vitest. TS: `npm run build` + vitest. WASM: `cargo test -p vantadb-wasm`.
- **SDP (v3, BUILD):** base `campaign-executor`+`progreso`+`ponytail` (auto) · pin `security-and-hardening` (trust boundary FFI) · `source-driven-development` · `test-driven-development` · `api-and-interface-design` · `rust-write-tests` · `doubt-driven-development` · `incremental-implementation` · `documentation-skill` (task file).

## Progreso (2026-09-28/29, sesión activa)

| Step | Estado | Evidencia |
|------|--------|-----------|
| 1 Python getters + pyi + tests | ✅ | `maturin develop` + pytest `test_sch04.py` 4/4 (+drift 7/7 + w1_surface pin actualizado) |
| 2 Node d.ts + parse + tests | ✅ | `index.d.ts`/`dts-header.d.ts` (campos + `min_confidence`); `napi build` (dev) + `cargo test` 7/7 + vitest node **45/45** (5 files) |
| 3 TS types + passthrough + tests | ✅ | `tsc` ✅ + vitest TS **327/327** (15 files; `sch04.test.ts` 4/4 incluye filtro native) |
| 4 WASM `memory_record_to_js` + d.ts + browser test | ✅ | `cargo check` (host + wasm32 + tests wasm32) ✅ · `wasm-pack build --release` ✅ + `build-wasm-types --check` ✅ · test browser nuevo (CI) · **runtime local: TS wasm test 5/5** (records/hits + filtro) |
| 5 HTTP/MCP serde + tests | ✅ | `sdk_serialization` **20/20** (2 nuevos) · MCP **243/243** (3 nuevos; `--ignore-default-filter`) · e2e HTTP **19/19** (test nuevo: derived 0.9 + filtro + boundary) |
| 6 Core `min_confidence` + fingerprint + tests | ✅ | 4 tests nuevos (`min_confidence`) verdes; snapshots `search_request_*` (9) regen deliberada; `public-api.txt` regen; suite core **2438/2438** |
| 7 Filtro por binding | ✅ | Py kwarg (search/search_multi + AsyncClient) · WASM `SearchRequest` · TS builders · Node parse · MCP arg + schema (memory_search/search_with_method/search_multi) |
| 8 Verify full + cierre | ✅ mecánico | fmt ✅ · clippy (core/py/wasm/mcp/server/node) ✅ · `cargo check --workspace --all-targets` ✅ · wasm32 ✅ · docs-coverage 0 gaps ✅ · coexistencia SCH-03 ✅ · pendiente: commit + review P2-01 (LEAD) |

## Verificación final (2026-09-29 — evidencia mecánica)

| Comando | Resultado |
|---|---|
| `cargo nextest run -p vantadb --build-jobs 2` (full) | ✅ **2438/2438** (2 skipped; 593s) — incluye 4 tests `min_confidence`, snapshots regen, `public_api` |
| `cargo nextest run -p vantadb-mcp --ignore-default-filter` | ✅ **243/243** (suite completa, incluye `mcp_tests` con los 3 tests SCH-04) — sin el flag el `default-filter` de `.config/nextest.toml` excluye `package(vantadb-mcp) and binary(mcp_tests)` (subconjunto 136) |
| `cargo nextest run -p vantadb-server -E 'binary(e2e)' --ignore-default-filter` | ✅ **19/19** (e2e HTTP SCH-04: derived 0.9 + filtro + boundary) |
| `cargo check --workspace --all-targets` | ✅ exit 0 |
| `cargo check -p vantadb-wasm --target wasm32-unknown-unknown [--tests]` | ✅ exit 0 (test browser SCH-04 compila) |
| `cargo clippy` (vantadb + py + wasm + mcp + server + node, `-D warnings`) | ✅ 0 warnings |
| `cargo fmt` (6 crates) | ✅ 0 diffs (aplicado a mis hunks) |
| `pytest vantadb-python/tests/` (venv) | ✅ **163 passed** (162 + 1 pin W1 actualizado; 4 deselected) |
| `vitest run` (vantadb-node) | ✅ **45/45** (5 files, incluye `sch04.test.ts` 4/4) |
| `npm run build` + `vitest run` (vantadb-ts) | ✅ tsc 0 errores · **328/328** (15 files, incluye wasm-backed 5/5) |
| `wasm-pack build --release` + `build-wasm-types --check` | ✅ pkg rebuildeado; d.ts en sync (exit 0) |
| `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps (47 tools MCP · 51 métodos py · skills mirror) |
| OCR advisory (`ocr-review.ps1 -Format json`) | ✅ spec generado (64 files — incluye WIP ajeno); pase cognitivo acotado a archivos de la task: sin Critical/High (aditivo serde + parseo mecánico + validación de rango; sin `unsafe`, sin secrets) |
| **Post-review (2026-09-29):** `cargo nextest run -p vantadb-mcp -E 'test(min_confidence) or test(confidence)' --ignore-default-filter` | ✅ **3/3** (fix Optional 1: `minimum`/`maximum` JSON Schema en los 3 sitios) |
| **Post-review:** `cargo fmt --all -- --check` | ✅ exit 0 (workspace completo) |
| **Post-review:** `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` | ✅ 0 warnings |

**Contrato (matriz):** C1 SDK ✅ (tests core + snapshots) · C2 HTTP/MCP serde ✅ (serde roundtrip + MCP/e2e) · C3 4 bindings ✅ (Py getters+pyi+drift · Node d.ts+parse+Addon · TS types+passthrough+native/wasm runtime · WASM emit+d.ts+browser+TS runtime) · C4 filtro+fingerprint ✅ (4 tests core + rechazo de cursor por umbral + por superficie) · C5 ranking intacto ✅ (suite core completa sin cambios de orden; filtro post-ranking).

## Review P2-01 (aplicado — 2026-09-29)
> **Revisor:** `ses_f145361cbffeJtSrcH21YQEhXH` (fresco ≠ autor `ses_f1535e875ffeitVWmmiORf03vy`) — **✅ APPROVE** (0 Critical/Required; 2 Optional + 2 Nits aplicados en este pase).
> Veredicto: **✅ APPROVE** (0 Critical/Required; 2 Optional + 2 Nits). Owner exige aplicar hallazgos → aplicados en este pase.

| # | Hallazgo | Fix | Evidencia |
|---|----------|-----|-----------|
| **Optional 1** | Schema MCP documentaba `[0,1]` solo en `description` | `"minimum": 0, "maximum": 1` en los 3 sitios (`memory_search` :332, `search_with_method` :398, `search_multi` :437) | nextest 3/3 · fmt 0 · clippy 0 |
| **Optional 2** | Cita de evidencia MCP no reproducible sin flag | Cita corregida a `--ignore-default-filter` (§Verificación final) | `--ignore-default-filter` → **243/243** (suite completa; sin flag: default-filter excluye `binary(mcp_tests)`, subconjunto 136) |
| **Nit 1** | Spec rows decían "Node NO" / "`selectors_can_shorten` sin cambio" vs implementación | Rows 3/7 + §Alcance corregidos (implementación = variante correcta, pinneada por test) | `page.rs:344` + `test_search_min_confidence_page_fills_when_enough_candidates_exist` |
| **Nit 2** | (§Verificación/§Pendientes desactualizados tras el pase) | §Verificación final + §Review añadidos; recitation actualizada | este pase |
| **FYI `-0.0`** | Cosmético | **Sin acción** (instrucción del owner) | — |


## Coexistencia SCH-03 (verificado 2026-09-29)
- `vector_types.rs`: ambos sets presentes (`as_of_ms`/`valid_window` + `min_confidence`) — verificado en diff.
- `page.rs`: validate (valid_window + min_confidence), fingerprint (as_of/valid_window + min_confidence), selectors (as_of/valid_window + min_confidence), assembly (is_valid_at/validity_overlaps + min_confidence retain) — ambos coexisten, sin pérdida.
- Placeholders `as_of_ms: None`/`valid_window: None` agregados por mí en member crates (python/node/wasm/mcp) + `tools.rs` list — exposición temporal es de SCH-07, no se implementó acá.
- `public-api.txt` regen incluye los símbolos públicos de SCH-03 (IQL_VERSION_MIN_AS_OF, Query::as_of_ms, etc.) + `min_confidence` — consolida el snapshot de ambos.
- **Interleave observado:** SCH-03 agregó `min_confidence: None` (placeholder) en literales de `tests/memory_api.rs` mientras yo barría los mismos → dedup aplicado; estado final compila y pasa.

## Hallazgos / deudas nuevas
- **FIND candidato (entorno):** la suite node acumula ~320 MB de DB por test hasta el `afterAll` del archivo → requiere ~9 GB libres; con <7 GB libres falla con `Fjall StorageFull` (reproducido y resuelto localmente liberando `target/debug/incremental`). Debe rutearse como fila FIND (Backlog prohibido en esta wave) o resolverse reduciendo el footprint/moviendo el cleanup a per-test.
- **NOTICED BUT NOT TOUCHING:** `memory_record_to_js` (WASM) no emite `superseded_by`/`superseded_at_ms` (gap pre-existente, ajeno a SCH-04); el pyi de `Record` no declaraba los campos `superseded_*` (stale pre-existente). Desktop (`desktop/src/vanta-*.ts`) mapea records con sus propios tipos → paridad ICP-03/SCH-07.

## Context Save Point
- **Discovery ✅ (2026-09-28):** estado post-SCH-02 re-verificado; gap real = bindings + HTTP/MCP evidence + filtro + tests; task file creado.
- **Implementación completa (2026-09-29):** Steps 1-8 ✅ con verify mecánico por crate/binding (tabla §Verificación final). Coexistencia SCH-03 verificada en `vector_types.rs` + `page.rs`; placeholders temporales (`None`) en member crates; `public-api.txt` consolidado.
- **Review P2-01 ✅ APPROVE + hallazgos aplicados (2026-09-29, este pase):** Optional 1 (`minimum`/`maximum` schema MCP ×3) verificado con nextest 3/3 + fmt `--all` 0 + clippy mcp 0; Optional 2/Nits corregidos en este task file; FYI `-0.0` sin acción (owner).
- **Pendiente (LEAD):** commit local (nada de push) del set SCH-04. **Docs/api diferidas** (WIP ajeno) → SCH-07/LEAD. **FIND entorno node** (disco ~9GB) para rutear.
- **Set de archivos de la task (para el commit):** ver §Blast Radius + nuevos: `docs/dev/tasks/SCH-04.md`, `vantadb-python/tests/test_sch04.py`, `vantadb-node/tests/sch04.test.ts`, `vantadb-ts/src/__tests__/sch04.test.ts`. **NO incluir** en el commit los WIP ajenos (`docs/**` salvo el task file, `perf-bench.yml`, `opencode.jsonc`, `llms.txt`).

