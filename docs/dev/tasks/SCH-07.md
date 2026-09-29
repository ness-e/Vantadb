---
title: "SCH-07: Superficies — bindings/server/MCP/IQL + docs/api mismo-PR"
kind: task
description: "Campos v2 (bitemporal + confianza + quarantined) y params de query (AS OF/valid_at + abstención) cruzan Py/TS/Node/WASM + HTTP + MCP + IQL con los mismos nombres de wire Y matriz de paridad verde Y docs/api mismo-PR."
---

# SCH-07: Superficies — bindings/server/MCP/IQL + docs/api mismo-PR

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 32 (F3) · **Origen:** plan L839-863; ADR-0046 §D2/§D3/§D5; SCH-03/04/05 §Pendientes
- **Fuente del prompt:** sub-agente vanta-worker (orquestador pipeline) — wave F3.5 (única en vuelo); branch `develop`
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠 · **Tipo:** feature-add (paridad cross-binding + docs mismo-PR)
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Estado:** ⏳ IN PROGRESS — implementación + verify ✅ + **review P2-01 ✅ APPROVE + batch de hallazgos aplicado** (este pase); pendiente **commit (LEAD)** · **Incógnitas (uphill):** 0 · **Pendientes (downhill):** §Pendientes
- **Gate D (question-gates):** pre-respondido por el orquestador — mandato explícito del contrato verbatim + stop conditions (WASM/desktop diferibles con FIND); sin question al usuario.

## Contrato (verbatim del plan, L849)
> "los campos v2 (bitemporal + confianza + quarantined) y los params de query (AS OF/valid_at + abstención) cruzan Py/TS/Node/WASM + HTTP (+`openapi.yaml`) + MCP + IQL con los MISMOS nombres de wire Y matriz de paridad verde (`openapi_yaml_parity` + `sdk_serialization` + stub drift `.pyi` + d.ts + meta-tests MCP de conteos) Y `docs/api/` actualizadas en el mismo PR (Regla 3: `validate-docs-coverage` exit 0) Y `public_api` snapshot regenerado deliberadamente (sin cambios Rust no intencionados)"

**Cláusulas a verificar (matriz de cierre):**

| # | Cláusula | Superficie | Evidencia esperada |
|---|----------|-----------|--------------------|
| C1 | Campos v2 (10: valid/invalid + 4 confianza + 4 cuarentena) visibles en records de las 4 bindings con los mismos nombres de wire | Py/TS/Node/WASM | getters py + pyi + drift; `MemoryRecord` d.ts node; `types.ts`; `memory_record_to_js` + d.ts wasm; tests por binding |
| C2 | Params de query `as_of_ms`/`valid_window`/`include_quarantined` (+ `min_confidence`) cruzan Py/TS/Node/WASM | 4 bindings | kwargs/structs/passthrough + tests por binding |
| C3 | HTTP: request (serde flatten ya) + respuesta `SearchPageV2` con `abstained`/`abstention_reason` | `handlers.rs` + `openapi.yaml` | e2e HTTP + `openapi_yaml_parity` |
| C4 | MCP: search args nuevos + abstención en el envelope + `memory_list` opt-ins; meta-tests de conteos intactos | `tools.rs` + `mcp_tests.rs` | tests MCP (abstención, args, conteos) |
| C5 | IQL: `AS OF`/v2/feature-detect documentados (`IQL_VERSION=2` ya en core) | `docs/api/IQL.md` | doc + gates |
| C6 | `docs/api/` actualizado mismo-PR: `validate-docs-coverage` exit 0 (2 gaps CONFIGURATION) + `openapi_yaml_parity` + stub drift + d.ts + docs de los SDK | docs | `validate-docs-coverage.ps1`, `check-links`, `check-docs`, `gen-index --check` |
| C7 | `public_api` snapshot regenerado deliberadamente (solo símbolos SCH-07) | `tests/api/public-api.txt` | `cargo test -p vantadb --test public_api` + diff auditado |

## Re-baseline post-SCH-02..06 (verificado 2026-09-29, HEAD `791b661e`)
- **Ya existe en core/wire:** campos v2 en `MemoryRecord` (SCH-02); `as_of_ms`/`valid_window` en search/list (SCH-03); `min_confidence` en `MemorySearchRequest` + getters de confianza en Py/TS/Node/WASM (SCH-04); cuarentena operativa + `include_quarantined` en search/list + `abstained`/`abstention_reason` en `MemorySearchPage` (SCH-05).
- **Gap real de esta task:** (a) las 4 bindings NO conocen `as_of_ms`/`valid_window`/`include_quarantined` (placeholders `None`), ni los 6 campos de bitemporalidad/cuarentena en sus records (WASM no los emite; d.ts/types no los tipan; Py no los expone; Node d.ts no los tipa); (b) HTTP no propaga `abstained`/`abstention_reason` (`records_search` usa `search()` que los descarta); (c) MCP no expone los args nuevos ni propaga abstención; (d) `MemoryListOptions.min_confidence` (ADR-0046 §D2, diferido por SCH-04/05) no existe; (e) `docs/api/` completo (IQL.md, scores.md, CONFIGURATION.md ×2 gaps, EMBEDDED_SDK, MCP, HTTP, SDK docs, openapi.yaml, matriz).
- **Abstención (re-baseline del orquestador):** el ítem explícito es propagar `abstained`/`abstention_reason` a **HTTP `SearchPageV2` + MCP search** (page-shaped). Las APIs de binding devuelven arrays (sin shape de página) → paridad parcial **declarada** (matriz + docs), con `search_page` disponible en el SDK Rust.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Edita** | Core: `src/sdk/types/record.rs` (`MemoryListOptions.min_confidence`) · `src/sdk/api/namespaces.rs` (validación + retain) · `src/sdk/search/mod.rs` (`search_page_with_method`) · `tests/sdk_serialization.rs` (+abstención) · `tests/api/public-api.txt` (regen deliberada). HTTP: `src/server/handlers.rs` (`SearchPageV2` + `records_search`). MCP: `vantadb-mcp/src/handlers/tools.rs` (parse + dispatch + schemas + list) · `vantadb-mcp/src/validation.rs` (envelope con señal) · `vantadb-mcp/tests/mcp_tests.rs`. Py: `vantadb-python/src/lib.rs` (kwargs search/search_multi/list) + `src/types.rs` (getters + `__getitem__`) + `vantadb_py/vantadb_py.pyi` + `vantadb_py/__init__.py` (Async) + tests. Node: `vantadb-node/src/lib.rs` (parse) + `index.d.ts` + `dts-header.d.ts` + tests. TS: `vantadb-ts/src/{types,guards,native,vantadb}.ts` + tests. WASM: `vantadb-wasm/src/lib.rs` (structs + emit + mapping) + `vantadb_wasm.d.ts` + tests. Docs: `docs/api/{IQL,scores,EMBEDDED_SDK,MCP,HTTP_API,PYTHON_SDK,TS_SDK,NODE_SDK,WASM_API,BINDINGS_NAMESPACES}.md` + `docs/api/openapi.yaml` + `docs/user/operations/CONFIGURATION.md` + `dev-tools/../scripts/docs` gates |
| **Callers** | Literales de `MemoryListOptions` en todo el workspace (bindings/MCP/server/cli/tests) → fix mecánico guiado por `cargo check`; `SearchPageV2` (solo handlers.rs); `parse_search_request` (MCP + Node); d.ts/pyi consumidos por tests de drift |
| **Callees** | `run_search_page` (page.rs) · `Embedded::{search,search_page,search_with_method,list}` · `AbstentionReason` (serde snake_case) · `ValidWindow` (serde `{from_ms,to_ms}`) · `text_content_hits_with_budget`/`apply_output_budget` (MCP) |
| **Implicaciones** | Aditivo puro en wire (`#[serde(default)]`); sin cambio de ranking; `search_page_with_method` = nuevo símbolo público (snapshot deliberado, semver MINOR pre-launch); abstención page-shaped (HTTP/MCP) declarada; docs mismo-PR (Regla 3); `MemoryListOptions.min_confidence` cierra ADR §D2 diferido |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos / secciones):** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 32, L839-863) · `docs/dev/tasks/SCH-03.md` / `SCH-04.md` / `SCH-05.md` (completos) · `docs/dev/architecture/adr/ADR-0046-schema-v2-migracion-unica.md` (completo) · `.opencode/rules/api-contract.md` (pendiente re-lectura al editar core) · `src/sdk/types/record.rs` (grep campos) · `src/sdk/serialization/vector_types.rs` (:205-264) · `src/sdk/search/page.rs` (:53-200, :291-491) · `src/sdk/search/mod.rs` (:69-153) · `src/sdk/search/multi.rs` (completo) · `src/sdk/api/namespaces.rs` (:54-219) · `src/server/handlers.rs` (:385-577) · `src/parser/mod.rs` (:1-33) · `src/config.rs` (grep) · `vantadb-mcp/src/handlers/tools.rs` (:29-33, :203-448, :1040-1050, :1576-1643, :1690-1949, :3235-3456) · `vantadb-mcp/src/validation.rs` (:386-509) · `vantadb-mcp/tests/mcp_tests.rs` (:1-120, :370-409, :4592-4979, :5904-6053) · `vantadb-python/src/lib.rs` (:400-453, :1162-1230, :1755-1815) · `vantadb-python/src/types.rs` (:51-202) · `vantadb_py/vantadb_py.pyi` (:95-169) · `vantadb_py/__init__.py` (:250-329) · `vantadb-node/index.d.ts` (completo) · `vantadb-node/dts-header.d.ts` (completo) · `vantadb-node/src/lib.rs` (:204-217, :802-843) · `vantadb-ts/src/types.ts` (completo) · `guards.ts` (:180-286) · `native.ts` (:340-399) · `vantadb.ts` (:580-621) · `vantadb-wasm/src/lib.rs` (:100-200, :1166-1268, :2265-2377) · `docs/api/BINDINGS_NAMESPACES.md` (completo) · `docs/api/openapi.yaml` (:470-599, :1760-2039) · `tests/api/openapi_yaml_parity.rs` (completo) · `scripts/validate-docs-coverage.ps1` (completo)
- **Referencias hacia dentro:** ADR-0046 §D2 (wire `include_quarantined`/`min_confidence`/abstención), §D3 (semántica AS OF, default sin cambios), §D5/D5d/D5e, §D7 (predicados); plan Task 32; SCH-03 §Pendientes (docs IQL + garantía página); SCH-04 §Pendientes (docs scores/SDK/openapi/MCP); SCH-05 §Pendientes (CONFIGURATION ×2, EMBEDDED_SDK/MCP, abstención HTTP/MCP, `min_confidence` en list)
- **Referencias entrantes:** SCH-08 (corte 0.8.0 + UPGRADE sobre estas superficies); ICP-01..03 (paridad consumidores); VER-08 (calibración sobre `scores.md`)
- **Veredicto impacto:** alto — 8 superficies + docs mismo-PR; aditivo puro (sin breaking no marcado); sin cambios de índices/ranking/persistencia

## Spec (feature-add — decisiones por evidencia)

| # | Decisión | Alternativas | Elegido | Evidencia |
|---|----------|--------------|---------|-----------|
| 1 | Params temporales en bindings | (a) exponer `as_of_ms`/`valid_window` con los nombres de wire / (b) nombres locales | (a) | Contrato verbatim "MISMOS nombres de wire"; tipos ya serde (`ValidWindow{from_ms,to_ms}`, `as_of_ms: Option<u64>`) |
| 2 | `valid_window` en Py (kwargs) | (a) kwarg dict `{"from_ms","to_ms"}` / (b) dos kwargs planos | (a) | Mismo shape de wire que TS/WASM/MCP; precedente `query_sparse` dict |
| 3 | Abstención en las 4 bindings | (a) nuevo método page-shaped por binding / (b) HTTP+MCP ahora + paridad parcial declarada (arrays sin shape de página) | (b) | Re-baseline explícito del orquestador ("propagar a HTTP SearchPageV2 + MCP search"); FIND-able follow-up; declarado en matriz/docs (nunca silencioso) |
| 4 | Abstención en `search_with_method` (MCP) | (a) dropear (sin page API) / (b) `search_page_with_method` espejo de 6 líneas | (b) | "Nunca silencioso" (F5 SCH-05); espejo exacto de `search_page`/`search_with_method` |
| 5 | Abstención en `search_multi`/`search_all` (MCP/HTTP all-ns) | (a) forzar señal / (b) `abstained:false, reason:null` documentado como N/A (semántica multi no definida en core) | (b) | ADR §D2 define la señal por request de página única; multi no tiene page API; documentar en openapi/MCP |
| 6 | `MemoryListOptions.min_confidence` | (a) agregar (ADR §D2 diferido por SCH-04/05) / (b) seguir difiriendo | (a) | ADR §D2 verbatim (ambos structs); cierra deuda declarada; sin fingerprint en list (cursor = offset) |
| 7 | `include_quarantined`/`min_confidence` aplican post-scan en list | (a) retain final (espejo SCH-03/05) / (b) pre-filtro en índice | (a) | Patrón FIND-24/SCH-03/05; sin index change |
| 8 | Envelope MCP | (a) `structuredContent` gana `abstained`/`abstention_reason`; text payload sigue siendo el array plano / (b) cambiar el text payload | (a) | Back-compat MCP-39 (clientes parsean `text` como array); señal machine-readable en structuredContent |
| 9 | Schemas MCP nuevos args | `as_of_ms` (number), `valid_window` (object `{from_ms,to_ms}`), `include_quarantined` (boolean), `min_confidence` (number [0,1]) en `memory_search`/`search_with_method`/`search_multi`/`memory_list` | — | Paridad de shape con core; rechazo param-level de tipos (patrón MEM-32) |
| 10 | HTTP `SearchPageV2` | añadir 2 campos en single-ns; all-ns mantiene `false/null` (documentado) | — | Re-baseline orquestador; `records_search` usa `search_page` para single-ns |
| 11 | WASM record emit | 6 campos nuevos como decimal strings u64 (policy string-u64, espejo `expires_at_ms`/`last_validated_at_ms`); clase de cuarentena como string, reason como string | — | Contrato casing/policy (`BINDINGS_NAMESPACES.md`); precedente SCH-04 |
| 12 | Node d.ts | tipar los 6 campos (runtime ya los emite vía `serde_json::to_value`) + params en `SearchRequest`/`MemoryListOptions` | — | "solo tipado" (precedente SCH-04 step 2) |
| 13 | Docs | ediciones quirúrgicas en los docs listados + `openapi.yaml`; `gen-index --write` si stale | — | Regla 3 + `gate-api-docs.md`; `docs/**` limpio (sesión docs commiteada) |
| 14 | `public_api` snapshot | regen deliberada (nuevo `search_page_with_method` + campo `MemoryListOptions.min_confidence`) con diff auditado | — | Contrato C7; `VANTADB_PUBLIC_API_UPDATE=1` |

## Invariantes de dominio (handoff — MUST)
- **Aditivo puro:** todo campo nuevo `#[serde(default)]`; defaults = comportamiento actual (cero breaking silencioso, ADR §D3-5).
- **Wire canónico desde core:** los nombres son los de `MemoryRecord`/`MemorySearchRequest`/`MemoryListOptions` (serde); ninguna binding inventa forma.
- **No tocar:** `memory.rs` (sticky T1), ranking (`fusion.rs`), parser/executor (SCH-03), persistencia/migración (SCH-02), gates de inyección (SCH-05), ADR (no se edita).
- **`docs/**` limpio:** si aparece WIP ajeno in-flight → no pisarlo; anotar y esperar.
- **PROHIBIDOS:** `Backlog.md`, `perf-bench.yml`, `opencode.jsonc`, plan file (LEAD), `desktop/**`.
- Regla dura `-p` + `CARGO_BUILD_JOBS=2`; `git diff` antes de cerrar; no commit (LEAD), no self-review (LEAD).

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: 0.** Sin `unsafe` nuevo, sin dependencias nuevas, sin hot path nuevo (retains O(n) sobre páginas ya materializadas). Deudas declaradas (no nuevas): abstención no expuesta en APIs de binding array-shaped (paridad declarada; follow-up page-shaped si se pide); `search_all`/`search_multi` sin señal (semántica multi sin page en core); `quarantine_*` ops e `import` quarantine args en HTTP/MCP/CLI (fuera del contrato de esta task; SCH-05 las listó como resto de superficie — evaluar FIND con dueño).

## Definition of Done (3 niveles)
- **Task:** contrato verbatim (C1-C7) + `cargo fmt`/clippy/nextest (`-p vantadb`, `-p vantadb-mcp --ignore-default-filter`, `-p vanta-memory`) + gates por binding + docs gates (`validate-docs-coverage` exit 0, `check-links`, `check-docs`, `gen-index --check`) + `openapi_yaml_parity` + `public_api` deliberado.
- **Commit:** atómico — **lo ejecuta el LEAD** (sub-agente sin commit ni self-review).
- **Release:** N/A (corte 0.8.0 = SCH-08; gate F3).

## Herramientas necesarias
- codegraph (index frozen → lectura directa documentada) · `cargo nextest run --profile audit -p vantadb --build-jobs 2` · `-p vantadb-mcp --ignore-default-filter` · `-p vanta-memory` · clippy `-D warnings` · `cargo fmt --all -- --check` · `pwsh scripts/validate-docs-coverage.ps1` · `node scripts/docs/{check-links,check-docs,gen-index}.mjs` · `cargo semver-checks --baseline-rev v0.7.0` (reportar) · pub: `maturin develop` + pytest · node: `npx vitest run` · ts: `npm run build` + vitest · wasm: `cargo test -p vantadb-wasm` + `wasm-pack build` + d.ts check · `CARGO_BUILD_JOBS=2`
- **SDP (v3, BUILD):** base `campaign-executor`+`progreso`+`ponytail` (auto) · pin `security-and-hardening` · pin `documentation-and-adrs` (→ `documentation-skill` cargada) · pin `api-and-interface-design` (cargada) · `source-driven-development` · `incremental-implementation` · `test-driven-development` · `rust-write-tests` (esperada por prompt) · `doubt-driven-development` (esperada por prompt)

## Steps

### Step 1: Core — `MemoryListOptions.min_confidence` + `search_page_with_method`
- **Archivos:** `src/sdk/types/record.rs`, `src/sdk/api/namespaces.rs`, `src/sdk/search/mod.rs`, literales workspace.
- **RED:** test list min_confidence filtra/no-op/rechaza NaN; test `search_page_with_method` devuelve page con abstención.
- **GREEN:** campo + validación boundary + retain; método espejo.
- **Estado:** ✅ DONE
- **Evidencia:** `search_page_with_method` + `MemoryListOptions.min_confidence` (validación + retain); tests core 2/2; literales mecánicos 21 archivos; `public-api.txt` +4 líneas auditadas.

### Step 2: HTTP — `SearchPageV2.abstained/abstention_reason` + `records_search`
- **Archivos:** `src/server/handlers.rs`.
- **RED:** e2e con `confidence_threshold` configurado → `abstained:true` + reason; default → `false/null`.
- **Estado:** ✅ DONE
- **Evidencia:** `test_e2e_search_abstention_signal_and_v2_fields` (single-ns abstained + all-ns N/A + campos v2 en el wire).

### Step 3: MCP — args + abstención + `memory_list` opt-ins + schemas
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/validation.rs`.
- **RED:** tests MCP: search con `as_of_ms`; `valid_window` inválido → param error; abstención en structuredContent; `memory_list` include_quarantined.
- **Estado:** ✅ DONE
- **Evidencia:** `test_mcp_search_and_list_temporal_quarantine_args` + `test_mcp_search_abstention_signal_in_structured_content` verdes; helper `search_page_envelope`; schemas ×4 + `validation.rs` signal.

### Step 4: Bindings — Py (kwargs + getters + pyi + Async) 
- **Archivos:** `vantadb-python/src/lib.rs`, `src/types.rs`, `vantadb_py/vantadb_py.pyi`, `vantadb_py/__init__.py`, `tests/test_sch07.py`.
- **Estado:** ✅ DONE
- **Evidencia:** `maturin develop` + pytest `test_sch07.py` 5/5 + drift 6/6 + `test_w1_surface` pin actualizado; pyi/`__init__.pyi` sincronizados.

### Step 5: Bindings — WASM + TS + Node
- **Archivos:** `vantadb-wasm/src/lib.rs` + `vantadb_wasm.d.ts` + `tests/wasm_tests.rs`; `vantadb-ts/src/{types,guards,native,vantadb}.ts` + `__tests__/sch07.test.ts`; `vantadb-node/src/lib.rs` + `index.d.ts` + `dts-header.d.ts` + `tests/sch07.test.ts`.
- **Estado:** ✅ DONE
- **Evidencia:** wasm32 check + `wasm-pack build --release` + `build-wasm-types` sync; TS 332/332 (sch07 4/4); Node 47/47 (sch07 2/2).

### Step 6: Docs — `docs/api/*` + `openapi.yaml` + CONFIGURATION + matriz
- **Archivos:** `docs/api/{IQL,scores,EMBEDDED_SDK,MCP,HTTP_API,PYTHON_SDK,TS_SDK,NODE_SDK,WASM_API,BINDINGS_NAMESPACES}.md`, `docs/api/openapi.yaml`, `docs/user/operations/CONFIGURATION.md`.
- **Estado:** ✅ DONE
- **Evidencia:** `validate-docs-coverage` 0 gaps · `openapi_yaml_parity` 11/11 · `check_openapi_parity.mjs` exit 0 · `gen-index --write` + `--check` exit 0.

### Step 7: Gates + cierre (sin commit — LEAD)
- **Verify:** fmt · clippy · nextest core/mcp/vanta-memory · gates por binding · `openapi_yaml_parity` · `sdk_serialization` + `public_api` · docs gates · semver-checks (reporte) · `git diff` audit.
- **Estado:** ⏳ EN CURSO (evidencia en §Verificación)

## Coordinación / WIP ajeno
- Working tree al abrir: `perf-bench.yml`, `CONSTRAINTS.md`, `desktop/README.md`, `opencode.jsonc` (+bak), `vantadb-ts/README.md` modificados por otras sesiones → NO tocar; anotado en RESULTADO si sigue in-flight al cierre.
- `docs/**` limpio (sesión docs commiteó su consolidación, `d23e1224` + `86c55e01` + `e23bc11e`).

## Verificación final (2026-09-29 — evidencia mecánica)

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb --build-jobs 2` | ✅ **2478/2478** (2 skipped; 320s) — incluye los 2 tests core SCH-07 + `sdk_serialization` 5/5 + `public_api` |
| `cargo nextest run -p vantadb-mcp --ignore-default-filter` | ✅ **248/248** (+3 SCH-07: args temporales/quarantine, abstención structuredContent, helper envelope) |
| `cargo nextest run -p vanta-memory` | ✅ **550/550** |
| `cargo nextest run -p vantadb-server -E 'binary(e2e)' --ignore-default-filter -j 2` | ✅ **20/20** (+1 SCH-07: abstención single-ns + all-ns N/A + campos v2) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ exit 0 (forma CI) |
| `cargo fmt --all -- --check` (+ node) | ✅ exit 0 |
| `cargo test --doc -p vantadb` | ✅ 13 passed · 1 ignored (pre-existente) |
| `cargo check -p vantadb-wasm --target wasm32-unknown-unknown --all-targets` | ✅ exit 0 |
| `wasm-pack build --release` + `node dev-tools/build-wasm-types.mjs --check` | ✅ pkg rebuildeado; d.ts en sync |
| `pytest vantadb-python/tests/` (venv) | ✅ **166 passed** (+5 `test_sch07.py`; 4 deselected) |
| `npx vitest run` (vantadb-node) | ✅ **47/47** (+2 `sch07.test.ts`) |
| `npm run build` (tsc) + `npx vitest run` (vantadb-ts) | ✅ tsc 0 errores · **332/332** (+4 `sch07.test.ts`) |
| `cargo nextest run -p vantadb --test openapi_yaml_parity --ignore-default-filter` | ✅ **11/11** |
| `node scripts/check_openapi_parity.mjs` | ✅ exit 0 (37 paths router ↔ yaml) |
| `tests/api/public-api.txt` | ✅ regen deliberada (+4 líneas: `search_page_with_method`, `MemoryListOptions::min_confidence`; diff auditado) |
| `pwsh scripts/validate-docs-coverage.ps1` | ✅ **0 gaps** (los 2 de `CONFIGURATION.md` cerrados) |
| `node scripts/docs/{check-docs,gen-index --check,check-links}.mjs` | ✅ exit 0 (links dentro de budget: 52/58, 0 nuevos) |
| `cargo semver-checks check-release --baseline-rev v0.7.0` | ⚠️ **reporte**: fallas = breakings acumulados del corte 0.8.0 (arity de `import_records`/`import_file` y CLI handlers; y la clase `constructible_struct_adds_field` de los structs del envelope v2). Todos intencionales del corte (ADR-0046 D1/D2, `feat!` a marcar en SCH-08). Sin fallas atribuibles a símbolos adicionales no previstos. Detalle: `.local-semver-sch07.txt` (artefacto local, no versionado) |

**Contrato (matriz):** C1 ✅ · C2 ✅ · C3 ✅ · C4 ✅ · C5 ✅ (IQL.md v2 + AS OF + feature-detect + nota de página) · C6 ✅ (docs mismo-PR + 4 gates exit 0) · C7 ✅ (snapshot deliberado auditado).

## Review P2-01

> **Revisor:** [LEAD: anteponer la línea del revisor — sesión fresca ≠ autor] — **✅ APPROVE** (0 Critical/Required; 4 Optional + 3 Nits). Batch de hallazgos aplicado en este pase; 2 hallazgos **NO aplicados por decisión del owner** (documentados en §Pendientes).

| # | Hallazgo → Fix | Evidencia (comando → resultado) |
|---|-----|---------------------------------|
| **Opt-1** | Tests WASM `list` endurecidos con guarda de discriminación: `as_of_ms: 0` debe devolver **0 records** (un param dropeado habría devuelto 1 y pasaba) — `vantadb-wasm/tests/wasm_tests.rs` (browser) + `vantadb-ts/src/__tests__/sch07.test.ts` (backend WASM) | `cargo check -p vantadb-wasm --target wasm32-unknown-unknown --all-targets` → exit 0 · `npx vitest run src/__tests__/sch07.test.ts` → **4/4** (la guarda corre contra el `pkg` real: el param cruza end-to-end) |
| **Opt-2** | `openapi.yaml` `SearchPageResponse`: `abstained` a `required` (siempre viaja); `abstention_reason` sin `enum` cerrado → nota "known values + code set open (`#[non_exhaustive]`): clients must tolerate unknown codes" | `cargo nextest -p vantadb --test openapi_yaml_parity` → **11/11** · `node scripts/check_openapi_parity.mjs` → exit 0 |
| **Opt-4** | Helper compartido `page::validate_min_confidence(f32) -> Result<()>` (marker estable `SEARCH_OPTIONS_INVALID`) usado por `validate_search_options` (search) y `namespaces.rs::list`; mismo comportamiento, sin clamp | `cargo nextest run -p vantadb -E 'test(min_confidence) or test(list)'` → **67/67** · clippy `-p vantadb --all-targets` → exit 0 · fmt exit 0 |
| **Nit** | `HTTP_API.md`: oxímoron "byte-identical apart from the two additive fields" → reformulado ("behaves exactly as before: `abstained` false / `abstention_reason` null, the two additive fields being the only difference") | `check-docs` exit 0 |
| **NO aplicado** | Canales de error MCP distintos (`memory_list` JSON-RPC vs `memory_search` envelope `Rejected`): ambos accionables y testeados; unificarlos toca compatibilidad MCP → se documenta en §Pendientes | Suite MCP no afectada (sin cambios de dispatch) |
| **NO aplicado** | TS `as_of_ms?: number \| null` sin `bigint` (WASM sí acepta `bigint`): sin impacto práctico (ms < 2^53); anotado en §Pendientes | — |

## Pendientes (§Pendientes)

- **LEAD:** commit local (nada de push) + línea de revisor en §Review P2-01 si aplica. Sin self-review (mandato).
- **Declarados, NO aplicados (decisión del owner):**
  - Canales de error MCP: `memory_list` responde JSON-RPC `invalid_params` mientras `memory_search` responde envelope `Rejected` (result accionable). Ambos accionables/testeados; unificar es cambio de compat MCP → no.
  - TS `SearchRequest.as_of_ms` tipa `number | null` (sin `bigint`; WASM/Node aceptan `bigint`). Sin impacto práctico en ms (< 2^53); unificar tipos si VER-08/ICP lo pide.
- **Follow-ups del contrato (declarados, con dueño):**
  - Abstención page-shaped por binding (Py/TS/Node/WASM devuelven arrays) — reabrir solo si se pide una API page-shaped en bindings.
  - `quarantine_*` ops (T2/T4) en HTTP/MCP y args de import quarantine en superficies MCP/HTTP/CLI (SCH-05 los dejó "resto de superficie"); candidato a fila FIND del corte.
  - `search_multi`/all-ns sin señal de abstención (N/A documentado).
- **SCH-08 (corte 0.8.0):** marcar los breakings del corte (`feat!`/`BREAKING CHANGE`) sobre esta base; semver-checks contra v0.7.0 reporta las 9 fallas del corte acumulado (esperadas).

## Recitation
```
=== RECITATION ===
Objetivo activo: SCH-07 — superficies (campos v2 + params + abstención) + docs/api mismo-PR
Estado: in-progress (implementación + docs + batch post-review ✅; commit = LEAD)
Última acción: batch post-review P2-01 aplicado — Opt-1 (tests WASM list endurecidos con as_of_ms:0 → 0 records), Opt-2 (openapi: abstained required + non_exhaustive documentado), Opt-4 (helper compartido validate_min_confidence con marker), Nit HTTP_API.md. No-aplicados documentados (canales MCP, TS bigint).
Resultado: PARTIAL
Próxima acción: LEAD — commit local del set SCH-07 (nada de push) + review ya ✅ APPROVE
Contrato: C1-C7 ✅ — campos v2 + params cruzan Py/TS/Node/WASM + HTTP + MCP + IQL con mismos nombres; matriz + docs mismo-PR; validación exit 0; public_api deliberado. Evidencia por cláusula en §Verificación final.
Invariantes: aditivo serde (cero breaking silencioso); wire canónico desde core; sin tocar ranking/parser/persistencia; docs gates verdes.
Deuda: follow-ups declarados en §Pendientes (abstención page-shaped por binding; quarantine ops HTTP/MCP; import args; MCP error channels; TS bigint) — ninguno bloquea el corte.
Próxima tarea si completa: SCH-08 (corte 0.8.0 — vanta-docs)
last-synced: 2026-09-29
=== END RECITATION ===
```

## Context Save Point
- **Discovery ✅ (2026-09-29):** re-baseline post-SCH-06; gap real = bindings params+fields, HTTP/MCP abstención, `MemoryListOptions.min_confidence`, docs completo; task file creado; Gate D pre-respondido; SDP registrado.
- **Implementación ✅ (2026-09-29):** Steps 1-6 completos con verify por crate/binding:
  - Core: `MemoryListOptions.min_confidence` (ADR §D2) + `search_page_with_method`; 30 literales mecánicos (21 archivos); `public-api.txt` +4 líneas auditadas.
  - HTTP: `SearchPageV2.abstained/abstention_reason` (single-ns `search_page`; all-ns N/A declarado).
  - MCP: args `as_of_ms`/`valid_window`/`include_quarantined` (+`min_confidence` en list) en search ×3 + `memory_list`; `search_page_envelope` con señal en structuredContent; schemas actualizados.
  - Py: kwargs + 6 getters (+`__getitem__`) + pyi/`__init__.py(i)`; `test_sch07.py` 5/5.
  - WASM/TS/Node: structs/passthrough/parse + d.ts + `memory_record_to_js` (6 campos) + tests.
  - Docs: `docs/api/` (IQL AS OF/v2 + scores L1-L5 + EMBEDDED_SDK + MCP abstention/quarantine + HTTP + 4 SDK + openapi + BINDINGS_NAMESPACES) + `CONFIGURATION.md` (2 gaps) + generados (`gen-index --write`).
- **Review P2-01 ✅ APPROVE + batch aplicado (2026-09-29):** Opt-1 (guarda de discriminación `as_of_ms: 0` en tests WASM `list`), Opt-2 (openapi: `abstained` required + `non_exhaustive` documentado), Opt-4 (helper `validate_min_confidence` con marker estable), Nit HTTP_API.md; 2 no-aplicados documentados en §Pendientes. Evidencia en §Review P2-01.
- **Pendiente (LEAD):** commit local (nada de push). **No commit hecho por este sub-agente** (mandato del orquestador). Sin self-review (LEAD).
- **WIP ajeno in-flight (no tocado):** `perf-bench.yml`, `CONSTRAINTS.md`, `desktop/README.md`, `opencode.jsonc` (+bak), `vantadb-ts/README.md`.
- **Set de archivos de la task (para el commit):** ver §Blast Radius + `docs/dev/tasks/SCH-07.md` + tests nuevos (`test_sch07.py`, `sch07.test.ts` ×2, wasm_tests extendido, e2e/mcp extendidos) + generados (`docs/index.md`, `docs/api/index.md`, `llms.txt`) + `tests/api/public-api.txt`. **NO incluir** WIP ajeno.
