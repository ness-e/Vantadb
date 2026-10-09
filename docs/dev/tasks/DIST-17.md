---
title: "TASK DIST-17: Test de paridad cross-language (Py/Node/WASM)"
kind: task
description: "Escenario canónico put/search/grafo/IQL → artefacto por binding + comparador canónico (hash/diff con normalización documentada) + job CI dedicado. Subconjunto común fijado con evidencia empírica; exclusión IQL de Node declarada + FIND."
---

# TASK DIST-17: Test de paridad cross-language (Py/Node/WASM)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 25, F1)
- **Fuente:** DELTA P0 — "con 3 conectores activos, la paridad es una promesa de producto que hoy solo se sostiene por espejos manuales; un comparador mecánico (mismo escenario → mismo hash) la convierte en verificable y atrapa drift entre bindings"
- **Esfuerzo:** 🟡 2-3d | **Appetite:** max 3d | **Prioridad:** 🟠
- **Tipo:** test-infra (harness cross-binding + CI; sin código de producción)
- **Creado:** 2026-10-04 (DISCOVERY) | **last-synced:** 2026-10-04
- **Estado:** ✅ COMPLETED (Steps 1-7; review P2-01 **APPROVE**; commit local `3c528df3` + cierre del task file)
- **Campaign ID:** master-plan-0.9.0-20261004 (taskId `25`)
- **Incógnitas (uphill):** 0 abiertas — resueltas en DISCOVERY con probes empíricos: **WASM corre el escenario completo** (put/search/grafo/IQL) y produce valores **idénticos** a Python; **Node no expone IQL** → exclusión declarada + FIND-268
- **Pendientes (downhill):** 7 steps (6 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (entrantes) | `vantadb-python/tests/` (suite pytest — el nuevo test corre en el gate por defecto), `vantadb-ts/src/__tests__/` + `vitest.config.ts` (include ya cubre `src/**/__tests__/**`), `vantadb-node/tests/` + `vitest.config`, CI (workflow nuevo + inventory docs `docs/dev/workflow/{README,TRIGGERS}.md`), `docs/api/BINDINGS_NAMESPACES.md` (§W1 parity matrix + evidencia same-PR) |
| Callees (salientes) | `tests/parity/scenario.json` (fixture compartido), APIs de los 3 bindings (`put`/`search`/`add_edge`/`graph_bfs`/`query`), `vantadb-wasm/pkg` (artefacto wasm), `vantadb-node` (napi build), extensión Python (maturin), `dev-tools/parity-compare.mjs` (comparador) |
| Implicaciones | Cero cambios de producción (el contrato lo exige salvo fixes justificados por FIND). Artefactos en `target/bindings-parity/` (gitignored). Job CI nuevo con cache (un solo job agregado). Docs de workflow re-baselineadas (38→39 workflows). Los tests productores corren también en las suites normales de cada binding (side-effect: escriben artefactos en `target/`, inocuo) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md:721-747` (Task 25), `docs/dev/tasks/DIST-15.md` (precedente inmediato — paridad puntual + wire canónico), `docs/dev/tasks/DIST-03.md` (precedente — scope WASM declarado con evidencia), `docs/api/BINDINGS_NAMESPACES.md` (409L — §W1 parity matrix, §v2 wire parity, tablas por binding), `vantadb-wasm/src/lib.rs:1092-1879` (`put`/`search`/`query`/`add_edge`/`graph_bfs`), `vantadb-wasm/tests/wasm_tests.rs:1-1628` (browser-only, `wasm_bindgen_test_configure!(run_in_browser)`), `vantadb-python/src/lib.rs:1066-1094,1297-1350,2012-2046,2148-2165,2348-2363` (firmas), `vantadb-python/src/types.rs:363-449` (`SearchHit`), `vantadb-python/src/convert.rs:326-397` (`format_query_result`/`query_result_to_pydict`), `vantadb-node/src/lib.rs` (superficie `#[napi]` — 35 métodos; **sin `query`/IQL**), `vantadb-node/index.d.ts:196-255` (`MemorySearchHit`/`GraphNodeRecord`), `vantadb-ts/src/vantadb.ts` (`create`/`search`/`graphBfs`/`query`), `vantadb-ts/src/types.ts:233-291` (`SearchHit`/`QueryResult`), `vantadb-ts/vitest.config.ts`, `vantadb-ts/package.json`, `vantadb-node/package.json`, `vantadb-python/pyproject.toml`, `vantadb-python/tests/conftest.py`, `.github/workflows/{release-npm-61,release-npm-node,ci-rust,gate-docs}.yml` (patrones de jobs), `.github/actions/rust-setup/action.yml`, `docs/dev/workflow/{RULES,README,TRIGGERS}.md`, `docs/dev/operations/CI_POLICY.md:1-120`, `.opencode/rules/{release-ci,js-ecosystem}.md`, `src/sdk/graph.rs:45-60` + `src/graph.rs:56-112` (BFS determinista — `next_level.sort()`), `.opencode/references/clean-code-clean-architecture.md` §Apéndice V
- **Referencias hacia dentro (imports/deps):** `vantadb-ts` ← `vantadb-wasm/pkg` (junction `file:`) + `vantadb-node` (devDep `file:`); `vantadb-node/tests/*` ← `../index.js` (napi build); `vantadb-python/tests/*` ← `vantadb` (venv local / wheel CI); `tests/parity/scenario.json` ← los 3 productores (path resuelto desde `__file__`/`import.meta.url`)
- **Referencias entrantes a los editados:** `BINDINGS_NAMESPACES.md` ← 20+ refs (docs/index, llms.txt, SDKs); `docs/dev/workflow/*` ← `docs/dev/operations/CI_POLICY.md` (inventory), `FIND-128`; el workflow nuevo será referenciado por `README.md`/`TRIGGERS.md` (mismo commit)
- **Veredicto impacto:** **BAJO-MEDIO** — 100% aditivo (archivos nuevos + 3 docs sincronizadas); sin cambios de producción; sin deps nuevas; gates de docs (check-links/check-docs) validan el cierre

## Contrato

> Del plan (Task 25): "un escenario canónico (put/search/grafo/IQL) corre en Py/Node/WASM y produce **resultados idénticos** (hash/diff canónico) en CI; job dedicado verde; divergencias encontradas → FINDs (no fixes silenciosos); sin cambios de código de producción salvo fixes que un FIND justifique."

**Subconjunto común (fijado en DISCOVERY con evidencia — pre-mortem #1):**

| Step | Python | WASM (pkg en Node) | Node (napi) | Evidencia DISCOVERY |
|---|---|---|---|---|
| `put` (3 records, vectores fijos) | ✅ | ✅ | ✅ | probes: 3× `node_id` idénticos |
| `search` (híbrido vector+texto) | ✅ | ✅ | ✅ | probes: mismos ids + scores (`0.032786883413791656`, `0.016129031777381897`) |
| `graph` (`add_edge` ×2 + `graph_bfs` depth 2) | ✅ | ✅ | ✅ | probes: BFS `[a, b]` idéntico; core ordena cada nivel (`src/graph.rs:106`) |
| `iql` (`INSERT NODE#…` + `FROM …`) | ✅ | ✅ | ❌ **excluido** | Node no expone método IQL (37 `#[napi]` sin `query`; `rg iql` = solo flag de capabilities) → FIND-268 |

**Criterios verificables:**

1. Los 3 productores emiten `target/bindings-parity/{python,wasm,node}.json` con la misma proyección canónica; `node dev-tools/parity-compare.mjs` imprime hash SHA-256 por binding y **exit 0 solo si los 3 coinciden** (tolerancia float `1e-6`, orden canónico documentado).
2. El comparador **detecta divergencia** (prueba negativa con artefacto manipulado → exit 1 + diff) — no es un gate vacuo.
3. Job CI dedicado `ci-bindings-parity.yml` (un solo job agregado con cache sccache/rust-cache/npm) que corre los 3 productores + comparador; verde en la simulación local end-to-end (mismos pasos, misma máquina).
4. Exclusiones declaradas por binding en el artefacto (`excluded: ["iql"]` en Node) + cobertura mínima por step (≥2 bindings) enforced por el comparador.
5. Divergencias encontradas → filas `FIND-*` (Backlog); sin fixes silenciosos; sin cambios de producción.
6. Docs sincronizadas: `BINDINGS_NAMESPACES.md` (subsección del runner + evidencia), `docs/dev/workflow/{README,TRIGGERS}.md` (inventory 38→39), gates docs verdes.
7. Sin breaking changes; sin deps nuevas de runtime (solo `pytest`/`vitest`/`maturin`/`wasm-pack` ya usados por CI).

## Spec (SDD — test-infra)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde corre el step WASM | A) `wasm_tests.rs` browser (`wasm-pack test --chrome`, job BEST-EFFORT no-blocking) / B) artefacto `vantadb-wasm/pkg` ejecutado en Node vía vitest (mismo artefacto wasm-bindgen que ya testea `vantadb-ts`) | B | ✅ decidido-por-evidencia: el job browser es `continue-on-error: true` (ci-rust.yml:445) — no puede sostener un gate "verde"; el pkg en Node es el MISMO binario wasm y ya es la superficie de test de `vantadb-ts` (release-npm-61). Browser queda como cobertura existente no-paridad |
| 2 | Subconjunto común + exclusiones | A) put/search/grafo en los 3 + IQL en Py+WASM con exclusión Node declarada / B) agregar IQL a Node (feature nueva — prohibida: "sin cambios de producción salvo fixes") / C) sacar IQL del escenario (pierde el 4º eje del contrato) | A | ✅ decidido-por-evidencia: probes muestran Node sin `query` (37 `#[napi]`, `rg` = 0); el contrato del plan exige "declarar exclusiones por binding" (pre-mortem #1) y el precedente DIST-03 sanciona declarar scope con evidencia |
| 3 | Dónde vive el escenario | A) fixture compartido `tests/parity/scenario.json` (una sola fuente de inputs; drift imposible) / B) constantes duplicadas en 3 tests (drift silencioso) | A | ✅ decidido-por-evidencia: "escenario canónico" implica UNA definición; cargo ignora subdirs sin `main.rs`; los 3 lenguajes leen JSON nativo |
| 4 | Dónde vive la normalización | A) comparador (`dev-tools/parity-compare.mjs`) — una sola implementación / B) cada productor normaliza (3× duplicación, drift de reglas) | A | ✅ decidido-por-evidencia: "no duplicar lógica" (constraint #6); los productores emiten la proyección cruda (ids string, scores number) y el comparador canonicaliza |
| 5 | Formato del artefacto | A) JSON por binding con `{binding, scenario, excluded, steps{put,search,graph_bfs,iql?}}` / B) golden file con valores esperados pineados | A | ✅ decidido-por-evidencia: el contrato pide paridad **cruzada** (mismo input → mismo output), no golden (que rompería ante cambios intencionales del core y duplicaría el pin que ya cubren los tests por binding). Hash canónico impreso = evidencia |
| 6 | Tolerancia/orden | A) scores `abs diff ≤ 1e-6` + orden canónico (search: score desc→id asc; put: key asc; BFS: secuencia del core; IQL: ids sorted) / B) byte-exact sin normalización | A | ✅ decidido-por-evidencia: los valores observados son idénticos (f32→f64 exacto en los 3), pero el orden de ties en search depende de iteración HashMap (`build_query_result_from_scored` sort estable sobre input no ordenado) → canonicalizar ties es obligatorio para no ser flaky; tolerancia documentada |
| 7 | Lenguaje del comparador | A) Node `.mjs` (`node:crypto` + JSON, patrón `dev-tools/*.mjs`) / B) Python | A | ✅ decidido-por-evidencia: patrón del repo (`build-wasm-types.mjs`, `fix-wasm-pkg-files.mjs`); Node ya está en el job (3 toolchains → no agregar una 4ª dependencia de script) |
| 8 | Job CI | A) workflow nuevo `ci-bindings-parity.yml` con UN job (checkout→rust-setup+cache→wasm-pack→node build→python wheel→3 productores→comparador), timeout 30 / B) matriz por binding / C) jobs secuenciales multi-workflow | A | ✅ decidido-por-evidencia: pre-mortem #3 "un job agregado con cache, no matriz completa"; comparte sccache/Swatinem entre los 3 builds Rust; triggers por paths de bindings (un cambio de core afecta a los 3 por igual — el drift vive en el glue) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) cero cambios en `src/`, `vantadb-python/src/`, `vantadb-wasm/src/`, `vantadb-node/src/`, `vantadb-ts/src/` de producción — solo tests/harness/CI/docs; (2) los artefactos viven en `target/` (gitignored) — nunca commiteados; (3) el comparador falla loud ante artefacto faltante o step con <2 bindings (nunca "pass" vacuo); (4) tests productores con sanity asserts propios (no dependen del comparador para fallar si el binding devuelve vacío); (5) no tocar `opencode.jsonc`, master plan, `docs/pipeline-state.json`, `docs/dev/plans/2026-09-28-docs-consolidation.md`, WIP ajeno (WSM-14 en vuelo); (6) docs con links relativos (documentation-skill), sin wikilinks; (7) workflows: SHA pins + timeout + permissions least-privilege + `push: [main]` sin develop (RULES.md §1-4).
- **Comandos de verificación:** por step (ver §Steps) + cierre: `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo nextest run --profile audit --workspace --build-jobs 2` · `pwsh scripts/validate-docs-coverage.ps1` · `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs` · simulación del job (3 productores + comparador).
- **Deuda pendiente:** ninguna al cierre esperada; divergencias → `FIND-*`; la exposición IQL de Node nace como `FIND-*` (no se arregla acá).

## Deuda técnica (Regla 6)

Saldo esperado **cero o negativo**: archivos nuevos de test/harness/CI sin `unwrap`/`expect`/`unsafe`/deps nuevas; reutiliza infraestructura existente (vitest/pytest/maturin/wasm-pack/sccache); el comparador usa solo stdlib de Node (`node:crypto`, `node:fs`). Los 3 docs editados son aditivos.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 1-7 ✅: 3 artefactos + comparador verde + prueba negativa de detección + job CI (simulación local end-to-end) + exclusiones declaradas + FINDs registrados + docs sincronizadas |
| **Commit** | Commit único `test(bindings): DIST-17 — cross-language conformance harness + CI job` (local, sin push), verificación mecánica previa, solo archivos del blast radius |
| **Release** | n/a — test-infra sin cambio user-visible (justificado en Notas); release-plz no aplica |

## Herramientas necesarias

- `codegraph_codegraph_explore` + `codebase-memory-mcp_*` (blast radius/cobertura) — ejecutados en DISCOVERY
- `vantadb-python/.venv` (pytest, extensión ya construida) · `npx vitest` (vantadb-ts, pkg wasm ya construido) · `npx vitest` (vantadb-node, napi ya construido)
- `node dev-tools/parity-compare.mjs` (comparador — el gate)
- `campaign_verify_cmd` (checks mecánicos por step)
- `pwsh dev-tools/ocr-review.ps1` (cierre)
- `node scripts/docs/*.mjs` + `pwsh scripts/validate-docs-coverage.ps1` (gates docs)

**Skills cargadas (SDP):** `test-driven-development` (pinned) · `ci-cd-and-automation` (pinned) · `api-and-interface-design` (pinned) · `git-workflow-and-versioning` (pinned) · `security-and-hardening` (pinned) · `documentation-and-adrs` (pinned) · `systematic-debugging` (pinned) · `source-driven-development` (base) · `rust-write-tests` · `incremental-implementation` · `documentation-skill`. Base auto (campaign-executor, progreso, ponytail).

## Investigation Notes

### DISCOVERY — viabilidad por binding (probes empíricos, 2026-10-04)

Escenario mínimo ejecutado contra los 3 artefactos ya construidos (mismo dataset: `put` a/b con vectores `[0.1,0.2,0.3]`/`[0.2,0.3,0.4]`, search híbrido, `add_edge` a→b, `graph_bfs`, IQL):

| Salida | Python (`vantadb-python/.venv`) | WASM (`vantadb-wasm/pkg` en Node) | Node (`vantadb-node` napi) |
|---|---|---|---|
| `put(a).node_id` | `50177344762235134037546702795750552960` | `50177344762235134037546702795750552960` | `50177344762235134037546702795750552960` |
| `put(b).node_id` | `177459167670083881481397560394709165110` | `177459167670083881481397560394709165110` | `177459167670083881481397560394709165110` |
| hits `(id, score)` | `(50177…, 0.032786883413791656)`, `(177459…, 0.016129031777381897)` | idénticos | idénticos |
| `graph_bfs` | `[50177…, 177459…]` | idéntico (bigint→string) | idéntico |
| IQL `INSERT`+`FROM` | `{"kind":"write","node_id":"42424242"}` / `{"kind":"read","nodes":[{"id":"42424242",…}]}` | `{"Write":{"node_id":"42424242"}}` / `{"Read":[{"id":"42424242",…}]}` | **no existe método IQL** (`typeof db.query === "undefined"`) |

**Conclusión:** WASM **sí** corre el escenario completo (la hipótesis del pre-mortem #1 "WASM sin IQL completo" queda refutada con evidencia); la única exclusión real es **Node × IQL**.

### Determinismo (verificado en código core)

- **Node ids:** derivados de `XxHash3_128(namespace\0key)` → deterministas por construcción (los 3 probes coinciden).
- **BFS:** `src/graph.rs::bfs_traverse` ordena `next_level.sort()` por nivel → orden determinista; roots en orden de entrada.
- **Search:** `build_query_result_from_scored` ordena por score desc con sort estable sobre iteración HashMap → **ties no deterministas** dentro de un mismo binding; el comparador canonicaliza (score desc → id asc) y aplica tolerancia `1e-6` (los scores f32→f64 observados son exactamente idénticos entre bindings).
- **Campos wall-clock** (`created_at_ms`, `updated_at_ms`, `valid_at_ms`, `last_accessed`) quedan **fuera** de la proyección canónica (no deterministas por diseño).

### Precedentes usados

- **DIST-15** (misma train): paridad puntual WASM↔Node byte-exacta + wire canónico; este harness la generaliza a Py+WASM+Node con hash mecánico.
- **DIST-03:** declaración de scope por binding con evidencia (compile checks); acá la evidencia es empírica (probes).
- **WIRE-03:** espejos manuales por binding (`wire03.test.ts` ×2 + Python) — el problema que este harness convierte en comparación mecánica.
- **SCH-07:** `wasm_tests.rs` es browser CI no-blocking → WASM paridad corre sobre el pkg en Node (decisión Spec #1).

### Evidencia de verificación (post-implementación, 2026-10-04)

| Gate | Comando | Resultado |
|---|---|---|
| Comparador end-to-end | `node dev-tools/parity-compare.mjs` | **PARITY OK** — 3/3 bindings; hashes por step idénticos (`put 6d43a8e5…`, `search 0fb2bacd…`, `graph_bfs a3f0b45c…`, `iql 99218974…` en Py+WASM) |
| Prueba negativa (fault injection) | tamper `node.json` score +0.001 → comparador | **exit 1** + `python vs node diverge at search[0].score: 0.032787 != 0.033787`; restaurado → exit 0 |
| Python (productor) | `.venv/Scripts/python -m pytest tests/test_cross_language_parity.py -q` | **1 passed** |
| Python (suite completa) | `pytest -q` | **182 passed** (4 deselected slow) |
| WASM/TS (productor) | `npx vitest run src/__tests__/parity.test.ts` | **1 passed** |
| TS (suite completa) | `npx vitest run` | **343 passed** (20 files) |
| Node (productor) | `npx vitest run tests/parity.test.ts` | **1 passed** |
| Node (suite completa) | `npx vitest run` | 49 passed / **2 failed — pre-existentes/ambientales** (`api.test.ts::searchWithMethod`, `ENOSPC os error 112`; pasan en aislamiento; la suite falla también SIN los archivos de DIST-17 → `FIND-269`) |
| Workflow | `actionlint .github/workflows/ci-bindings-parity.yml` | **exit 0**; YAML parse OK; 39 workflows en el tree (inventory actualizado) |
| Rust core (no tocado) | `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo nextest run --profile audit --workspace --build-jobs 2` | exit 0 · exit 0 · **3745/3745 passed** (7 skipped) |
| Docs gates | `validate-docs-coverage` · `check-links` · `check-docs` · `gen-index --check` | 0 gaps · 0 broken · all clear · exit 0 |

**Nota sobre el "job verde":** la simulación local corre la secuencia funcional del job (3 productores + comparador) con los artefactos construidos; los build steps del workflow son los comandos ya usados por workflows existentes (`wasm-pack build --release` + `build-wasm-types.mjs` de `release-npm-61.yml`; `npm ci && npm run build` de `release-npm-node.yml`; maturin de `release-wheels.yml`) — validados por `actionlint`. El job real corre en el próximo PR que toque paths de bindings.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — viabilidad WASM/IQL resuelta con probes; subconjunto común y exclusiones fijados; normalización fijada por código del core |
| Pendientes de ejecución (downhill) | 7 steps (6 ✅; Step 7 = cierre en curso) |
| % completado | 100% implementación; cierre (review + commit + campaña) en curso |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** aplica en modo test-infra (trust boundary = CI/artefactos): sin secretos en diff; el workflow nuevo usa `permissions: contents: read` + SHA pins; el fixture y los artefactos no contienen datos de usuario; el comparador no ejecuta input externo (JSON propio). Checklist `security-and-hardening` acotada: sin deps nuevas, sin red en tests, sin secretos. Gate: revisión del diff + OCR sin Critical/High.
- **PERFORMANCE:** no aplica (no toca hot paths ni código de producción; los tests corren una vez por job). Justificado — sin benchmark canónico.

## Steps

### Step 1 — Fixture + comparador (RED) — ✅
- **Archivos:** `tests/parity/scenario.json`, `dev-tools/parity-compare.mjs`
- **Acción:** fixture canónico (dataset/query/grafo/IQL pineados); comparador con canonicalización + tolerancia + hash SHA-256 por step + diff; RED = corre sin artefactos → falla loud
- **Verify:** `node dev-tools/parity-compare.mjs` → `PARITY FAIL: missing artifact target\bindings-parity\python.json…` exit 1 (RED ✅)
- **Estado:** ✅

### Step 2 — Python: productor + artefacto — ✅
- **Archivos:** `vantadb-python/tests/test_cross_language_parity.py`
- **Acción:** escenario completo + sanity asserts + escritura de `target/bindings-parity/python.json` (3 records, 3 hits con scores distintos, BFS 3 nodos, IQL `42424242`)
- **Verify:** `.venv/Scripts/python -m pytest tests/test_cross_language_parity.py -q` → **1 passed**; artefacto emitido
- **Estado:** ✅

### Step 3 — WASM: productor + artefacto — ✅
- **Archivos:** `vantadb-ts/src/__tests__/parity.test.ts`
- **Acción:** escenario sobre `Client` (pkg wasm real) + artefacto `wasm.json`
- **Verify:** `npx vitest run src/__tests__/parity.test.ts` → **1 passed**; artefacto emitido
- **Estado:** ✅

### Step 4 — Node: productor + artefacto (IQL excluido) — ✅
- **Archivos:** `vantadb-node/tests/parity.test.ts`
- **Acción:** escenario sobre napi + `excluded: ["iql"]` + artefacto `node.json`
- **Verify:** `npx vitest run tests/parity.test.ts` → **1 passed**; artefacto emitido
- **Estado:** ✅

### Step 5 — VERIFY local end-to-end + prueba negativa — ✅
- **Archivos:** (sin cambios; evidencia)
- **Acción:** comparador con 3 artefactos → `PARITY OK` + hashes por step idénticos; **prueba negativa**: score de `node.json` +0.001 → `PARITY FAIL: python vs node diverge at search[0].score: 0.032787 != 0.033787` exit 1; restaurado → exit 0
- **Verify:** `node dev-tools/parity-compare.mjs` → exit 0 (3/3) · tamper → exit 1 + diff (fault injection real)
- **Estado:** ✅

### Step 6 — CI job + docs (BINDINGS_NAMESPACES, workflow inventory, FINDs) — ✅
- **Archivos:** `.github/workflows/ci-bindings-parity.yml`, `docs/api/BINDINGS_NAMESPACES.md`, `docs/dev/workflow/README.md`, `docs/dev/workflow/TRIGGERS.md`, `docs/dev/Backlog.md`, `docs/index.md`, `llms.txt`
- **Acción:** job dedicado (rust-setup+cache → wasm-pack → node build → maturin wheel → 3 productores → comparador); sección del runner en BINDINGS_NAMESPACES; inventory 38→39; `FIND-268` (Node sin IQL) + `FIND-269` (flakiness ambiental Node/ENOSPC); `gen-index --write`
- **Verify:** `actionlint` **exit 0**; YAML parse OK (PyYAML); conteo de workflows **39**; `check-links`/`check-docs`/`gen-index --check`/`validate-docs-coverage` exit 0
- **Estado:** ✅

### Step 7 — Cierre: verify full + OCR + review P2-01 + commit + campaña — ✅
- **Archivos:** los anteriores + task file
- **Acción:** verify full (fmt/clippy/nextest audit/validate-docs-coverage) → OCR delegation (Rule Groups revisados; sin Critical/High) → review P2-01 `vanta-review` (**APPROVE**, 5/5 adversariales del comparador) → commit local `test(bindings):` → campaña `completed` (taskId 25)
- **Verify:** todos los gates verdes + verdict registrado + commit local `3c528df3` (13 archivos, sin WIP ajeno) + pre-commit hook ✅
- **Estado:** ✅

## Dependencias

- DIST-15 ✅ (cerrada) — aporta `graphrag_search` + la evidencia de paridad puntual que este harness generaliza.
- DIST-03 ✅ (cerrada) — precedente de declaración de scope por binding.
- WSM-14 en vuelo (`vantadb-wasm/README.md`) — **no tocar** (WIP ajeno).

## Notas

- (DISCOVERY) Gate D evaluado: **no disparado** — sin símbolos públicos nuevos (test-infra + CI + docs); contrato del plan (Task 25, Gate Result ✅ DO) sanciona el harness; sin ambigüedad que requiera question (el pre-mortem del plan ya define la respuesta al subconjunto/exclusiones y la evidencia la resolvió).
- (DISCOVERY) El plan lista `vantadb-wasm/tests/wasm_tests.rs` como archivo clave; la decisión Spec #1 lo deja fuera del harness (browser no-blocking) — el step WASM corre sobre el mismo artefacto wasm en Node. Declarado acá y en BINDINGS_NAMESPACES.
- (DISCOVERY) Los tests productores escriben artefactos en `target/bindings-parity/` al correr en cualquier suite — side-effect inocuo (target/ gitignored), documentado en cada test.
- (CIERRE) `FIND-268` (Node sin superficie IQL — exclusión declarada) y `FIND-269` (suite Node con ENOSPC ambiental bajo carga completa; reproducido sin los archivos de DIST-17) registrados en Backlog.
- (CIERRE) WIP ajeno excluido de commits: `opencode.jsonc`, master plan, `vantadb-ts/{README.md,package.json}`, `vantadb-wasm/{README.md,Cargo.toml}`, `docs/dev/tasks/WSM-14.md` (sesiones paralelas WSM-14/TS-10).
- (CIERRE) `docs/index.md` + `llms.txt` regenerados con `gen-index --write` — incluyen las filas de task files en vuelo (WSM-14 untracked, TS-10 tracked) además de DIST-17; si WSM-14 no aterriza, regenerar antes del merge.
- (CIERRE) El step WASM del job usa el artefacto `vantadb-wasm/pkg` en Node (mismo wasm-bindgen que la suite TS); los browser tests de `wasm_tests.rs` siguen en `ci-rust.yml` como BEST-EFFORT (fuera de este gate).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — subagente P2-01 en sesión fresca (contexto distinto al implementador; sin participación en el changeset).
- **Enfoque:** evidencia del subconjunto/exclusiones (Node sin IQL, WASM con IQL); normalización anti-drift; fallos loud del comparador (tamper/artefacto faltante/cobertura <2); workflow vs RULES.md §1-4/§6; scope discipline (cero cambios de producción); coherencia docs.
- **Cómo se probó (re-ejecutado):** `node dev-tools/parity-compare.mjs` → PARITY OK 3/3 (hashes `put 6d43a8e5…`, `search 0fb2bacd…`, `graph_bfs a3f0b45c…`, `iql 99218974…`); tamper `node.json` +0.001 → exit 1 (`python vs node diverge at search[0].score: 0.032787 != 0.033787`) + 3 adversariales (artefacto faltante · step sin exclusión · iql con cobertura 1) → exit 1, restores verificados por SHA-256; productores re-corridos (pytest 1 passed · TS vitest 1 passed · Node vitest 1 passed); `actionlint` exit 0; `git diff` en `src/` de producción vacío; artefactos gitignored (`**/target/`); `check-docs` + `check-links` exit 0. `gen-index --check` rojo local solo por `docs/dev/tasks/TS-11.md` (untracked de sesión paralela, posterior al regen) — no atribuible a DIST-17: el index commiteado (1516) coincide con el árbol del commit (1513 base + DIST-17 + TS-10 + WSM-14); regenerar si TS-11 aterriza antes.
- **Veredicto:** ✅ **APPROVE** — contrato 1-7 verificado; solo hallazgos Optional/Nit (conteo `#[napi]` 35→37 en task file/test/FIND-268; hardening opcional del comparador: validar `scenario` contra el fixture). Sin Critical/Required.
- **Findings aplicados (post-review):** 🟢 `#[napi]` 35→**37** (task file, `vantadb-node/tests/parity.test.ts`, FIND-268) · 🟢 guard del fixture implementado en `dev-tools/parity-compare.mjs` (valida `scenario` de los artefactos contra `tests/parity/scenario.json`) + verificado (`artifacts use scenario "canonical-v0" but the fixture declares "canonical-v1"` → exit 1; restore → exit 0) · 🟢 regla de cobertura por-step sin cambio (es la del contrato).

## RESULTADO (§7 — contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 3c528df3 (LOCAL, sin push) — changeset de 13 archivos; cierre del task file en el commit `docs(task):` siguiente
ARCHIVOS: tests/parity/scenario.json · dev-tools/parity-compare.mjs · vantadb-python/tests/test_cross_language_parity.py · vantadb-ts/src/__tests__/parity.test.ts · vantadb-node/tests/parity.test.ts · .github/workflows/ci-bindings-parity.yml · docs/api/BINDINGS_NAMESPACES.md · docs/dev/workflow/{README,TRIGGERS}.md · docs/dev/Backlog.md (FIND-268/269) · docs/index.md · llms.txt · docs/dev/tasks/DIST-17.md
VERIFY_CONTRATO: pasa — PARITY OK 3/3 (hash por step idénticos) + detección demostrada (tamper +0.001, artefacto faltante, step sin exclusión, cobertura <2, scenario stale — todos exit 1 con diff); suites Py 182 ✅ / TS 343 ✅ / Node productor ✅ (suite con FIND-269 ambiental pre-existente); Rust 3745/3745 ✅; actionlint ✅; docs gates ✅
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P: no aplica (sin símbolos públicos nuevos — test-infra) · D: no disparado (contrato del plan sanciona; sin ambigüedad) · V: no disparado (todo verde al primer intento; excepción ambiental FIND-269 documentada) · C: no disparado (FIND-268/269 registrados; WIP ajeno intacto; commit sin sweep)
SKILLS_CARGADAS: test-driven-development (pinned), ci-cd-and-automation (pinned), api-and-interface-design (pinned), git-workflow-and-versioning (pinned), security-and-hardening (pinned), documentation-and-adrs (pinned), systematic-debugging (pinned), source-driven-development (base), rust-write-tests, incremental-implementation, documentation-skill
```
