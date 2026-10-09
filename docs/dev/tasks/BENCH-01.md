---
title: "TASK BENCH-01: competitive_bench — aislar la región medida del Ingest + eliminar el doble rebuild"
kind: task
description: "El timer de Ingest envuelve init/prep del harness y el modo single-call duplica el rebuild HNSW (hidden rebuild dentro del Ingest + rebuild_index() en Index); fix: región medida = put calls + flush, chunks siempre <1000 por construcción, self-test de las regiones"
---

# TASK BENCH-01: `competitive_bench` — fix del doble-conteo (mide mal lo que dice medir)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 13, Wave F0)
- **Fuente:** `docs/dev/Backlog.md` (BENCH-01) + plan Task 13
- **Esfuerzo:** 🟢 2-3h | **Appetite:** 1d
- **Prioridad:** 🟡
- **Tipo:** Python (dev-tools/bench harness) + Docs
- **Creado:** 2026-10-04T10:13Z | **last-synced:** 2026-10-04T10:26Z
- **Estado:** ⏳ IN PROGRESS (steps locales ✅; ACCEPT/Review P2-01 del orquestador delegado)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY (§Investigation Notes)
- **Pendientes (downhill):** 0 locales (ACCEPT P2-01 delegado al orquestador)
- **Campaign ID:** master-plan-0.9.0-20261004

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/user/operations/BENCHMARKS.md` §18 (puntero operativo al run), `docs/user/benchmarks/COMPETITIVE_SDK_BENCH.md` (doc curado), `docs/user/benchmarks/COMPETITIVE_ANALYSIS.md` (metodología), `docs/user/blog/benchmarks_vs_lancedb_chroma.md` (blog histórico), `docs/dev/archive/_run_stdout.md` (log histórico). Ningún consumidor programático del script. |
| Callees | `vantadb.Client` / `put_batch_raw` / `flush` / `rebuild_index` / `search` (PyO3, `vantadb-python/src/lib.rs:736,1377`), numpy, psutil, tabulate. |
| Implicaciones | Solo dev-tools + docs: no toca motor, API pública, storage, WAL ni bindings. Los números publicados del run 2026-08-12 NO se regeneran (fuera de appetite; el run sigue citado con su entorno). El delta del fix es de **región medida** (init/prep fuera) y de **modo** (single-call eliminado). |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, WIP de otros workers (DUR-02: `tests/dur02_encryption_probe.rs` + `src/crypto/vfile.rs`; DUR-03: `src/sdk/**` + `tests/edge_cases.rs`; FIND-233 ya tocó `perf-bench.yml`/`compare_baseline.py` — no re-tocar). `docs/dev/Backlog.md` tiene WIP de otros workers → no se edita en este commit (FIND candidato se documenta acá y lo rutea el orquestador).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `benchmarks/competitive_bench.py` (1079L), `benchmarks/README.md` (89L), `docs/user/benchmarks/COMPETITIVE_SDK_BENCH.md` (66L), `docs/user/benchmarks/COMPETITIVE_ANALYSIS.md` (L20-154 + grep), `docs/user/operations/BENCHMARKS.md` (§18 L1095-1136 + grep), `docs/dev/tasks/FIND-232.md` (182L), `docs/dev/tasks/FIND-233.md` (248L), `vantadb-python/src/lib.rs` (L700-769, L1360-1409), `src/sdk/api/memory.rs` (L590-828), `src/storage/engine/ops.rs` (L1-70), `src/storage/engine/insert.rs` (grep), `benchmarks/vantadb_local_bench.py` (257L), `.opencode/references/clean-code-clean-architecture.md` (Apéndice V), `.opencode/references/definition-of-done.md`, `.opencode/references/performance-checklist.md`, `.opencode/task-system/prompts/task.md`, `pipeline-full.md`.
- **Referencias hacia dentro:** grep `competitive_bench` → docs §18/COMPETITIVE_SDK_BENCH/COMPETITIVE_ANALYSIS/blog/archive (todos citas de runs, no consumidores); `--batch-size` → docs listadas en Blast Radius; CI: `docs/dev/operations/CI_POLICY.md` lista competitive_bench como Heavy Certification (no invocado en workflows del repo — verificado por grep, sin `run:` que lo llame).
- **Referencias hacia salientes:** stdlib + numpy/psutil/tabulate/h5py/lancedb/chromadb/vantadb (PyO3). Sin imports del repo.
- **Veredicto impacto:** **LOCALIZADO / dev-tools+docs.** No hay migración de datos, no cambia API pública, no toca el motor. Riesgo de regresión: solo el harness y las notas de metodología.

## Contrato

**Plan (Task 13):** "el bench no duplica el rebuild (o el workaround deja de ser necesario) + comparación before/after del número afectado documentada + docstring actualizado."
**Prompt del worker:** "el timer aísla **solo el ingest** (excluye init/teardown) → número real medido; README/BENCHMARKS actualizados si el número cambia; self-test/fixture verde; sin tocar el engine."

Verificación del contrato:

1. `benchmarks/competitive_bench.py` — región medida del Ingest = `put_batch_raw` calls + `flush()` (Client init + prep de payloads FUERA); `effective_chunk_size()` garantiza chunks < 1000 por construcción (ningún modo puede disparar el hidden rebuild + repetirlo con `rebuild_index()`).
2. `python benchmarks/competitive_bench.py --self-test` → PASS (fixture de región con stub engine: init excluido, calls < 1000, exactamente 1 rebuild).
3. Fixture RED→GREEN capturado (pre-fix FALLA, post-fix PASA) — §Fase 1.
4. Before/after del número afectado documentado (batch-0 vs chunked + micro-medición del trabajo excluido) — §Diagnóstico.
5. Docstring del harness + `benchmarks/README.md` + `docs/user/operations/BENCHMARKS.md` §18 + `COMPETITIVE_SDK_BENCH.md` + `COMPETITIVE_ANALYSIS.md` actualizados (metodología; nota de comparabilidad histórica).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Esquema JSON INV-007-B intacto** (`schema_version 1`, claves de `results[]`, `ingest_mode` string) — consumido por web.
  2. **Formato de tabla/CLI intacto** (columnas, flags existentes; `--batch-size` sigue aceptándose; 0/negativo = default).
  3. **Sin tocar el engine** (`src/**`, `vantadb-python/**`): el fix es 100% harness + docs.
  4. **Corrida real del harness sigue funcionando end-to-end** con el `.venv` del repo (vanta/lance/chroma) — evidencia post-fix.
  5. Números publicados del run 2026-08-12: **no se re-publican** (fuera de appetite); se agrega nota de comparabilidad, no se borra historia.
- **Comandos de verificación:** `.venv/Scripts/python.exe benchmarks/competitive_bench.py --self-test` → `9/9 PASS`; `.venv/Scripts/python.exe -m py_compile benchmarks/competitive_bench.py` → exit 0; corrida real `--engines vanta --size 2000 --queries 50` → tabla + JSON emitidos.
- **Deuda pendiente:** (a) asimetría cross-engine de la región de Ingest (Lance/Chroma siguen incluyendo connect/collection dentro del timer; Vanta ya no) — FIND candidato, fuera de scope; (b) el SDK re-construye índices derived/text/sparse en cada `put_batch_raw` (memory.rs:776-778) → el camino chunked paga n/999 de esos rebuilds dentro del Ingest (costo real del API path, no duplicado — documentado).

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** `.venv/Scripts/python.exe benchmarks/competitive_bench.py --dataset synthetic --size 2000 --queries 50 --engines vanta --batch-size 0` (pre-fix) → Ingest 856.4 QPS con Index 1512.4 ms: el timer Ingest incluye un rebuild completo del vector index (`src/sdk/api/memory.rs:721-723,766-768`, disparado por chunks ≥1000 — `src/storage/engine/ops.rs:50-59`, umbral default 1000) y `db.rebuild_index()` (`vantadb-python/src/lib.rs:1377`) lo repite en Index → **2 builds del mismo índice por run**. En el camino chunked (default 999), el timer Ingest además incluye `vantadb.Client(db_path)` + preparación de keys/payloads/metadatas (`competitive_bench.py:250-288` pre-fix).
- **Hipótesis:** la región medida está mal delimitada en dos ejes: (a) envuelve setup (init + prep) que no es ingest; (b) el modo single-call duplica el rebuild (hidden dentro de Ingest + explícito en Index). Fix = delimitar la región (`_put` loop + `flush`) y hacer imposible el chunk ≥1000 por construcción.
- **1 variable controlada:** por corrida se cambia UNA cosa: (1.ª) región medida (init/prep fuera); (2.ª) clamp de chunk. El fixture con stub engine aísla ambas con sleeps controlados (init 300 ms vs put 2 ms/chunk).
- **Test RED:** fixture standalone `$TEMP\opencode\bench01\bench01_fixture.py` (mismas aserciones que el `--self-test` a integrar) contra el harness **pre-fix** → **FALLA (exit 1)**:
  ```
  [FAIL] effective_chunk_size exists -- helper not present (pre-fix)
  [FAIL] no put_batch_raw call >= 1000 nodes -- calls=[2000]
  [PASS] exactly one rebuild_index() per run -- rebuilds=1
  [FAIL] Ingest timer excludes client init -- throughput=6568 rec/s, floor=13333 rec/s
  ```
  (transcripción literal; 3 failure(s), exit 1). GREEN se verifica post-fix con el mismo fixture + `--self-test` integrado.

## Spec (SDD — no aplica, justificación)

No es feature-add: no agrega símbolos públicos, endpoints, bindings ni capabilities de usuario final. `effective_chunk_size` y `--self-test` viven en un harness dev-only sin consumidores programáticos (web consume solo el JSON emitido, cuyo esquema no cambia). Tipo mecánico = bug-fix de harness (`fix(bench):`).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin input de usuario, auth, red, ni dependencias nuevas (Python stdlib). Sin trust boundaries nuevos.
- [x] **PERFORMANCE** — aplica (es el objeto de la tarea): baseline = corridas pre-fix en este box (§Diagnóstico) + evidencia publicada; impacto esperado = número Ingest más limpio (región aislada) y fin del doble build; verificación = fixture determinista + corrida real + micro-medición. Regla 9/11: todo claim con comando y fuente.

## Steps

### Step 0 — DISCOVERY (task file + Regla 0 + SDP) ✅
- **Archivos:** `docs/dev/tasks/BENCH-01.md` (este archivo)
- **Acción:** blast radius + Regla 0 + SDP v3 + evidencia de debugging + design freeze (región medida; clamp <1000; self-test).
- **Verify:** task file creado con §Regla 0 poblada antes de editar.
- **Estado:** ✅ COMPLETED

### Step 1 — Test RED (fixture standalone, pre-fix) ✅
- **Archivos:** `$TEMP\opencode\bench01\bench01_fixture.py` (no versionado)
- **Acción:** escribir el fixture con las aserciones del futuro `--self-test` y correrlo contra el harness pre-fix.
- **Verify:** exit 1 con 3 FAIL (calls=[2000]; init dentro del timer; helper ausente).
- **Estado:** ✅ COMPLETED

### Step 2 — Fix en `bench_vantadb`: región medida + clamp de chunks ✅
- **Archivos:** `benchmarks/competitive_bench.py`
- **Acción:** (a) `INCREMENTAL_THRESHOLD=1000` + `effective_chunk_size()`; (b) mover `vantadb.Client(db_path)` + prep de keys/payloads/metadatas FUERA del timer; región = `_put` loop + `flush`; (c) chunk loop único (sin branch single-call); (d) docstring ítem 1 reescrito (región + clamp + refs de código vigentes + nota histórica); (e) argparse help + `ingest_mode` JSON + prints finales.
- **Verify:** `.venv/Scripts/python.exe -m py_compile benchmarks/competitive_bench.py` → exit 0 (vía `campaign_verify_cmd`; 1.º intento exit -1 por spawn transitorio sin output, reintentado y verde — mismo patrón FIND-233).
- **Estado:** ✅ COMPLETED

### Step 3 — Self-test integrado `--self-test` + GREEN ✅
- **Archivos:** `benchmarks/competitive_bench.py`
- **Acción:** integrar `run_self_test()` (9 checks: 6 de chunk planning + 3 del fixture de región) + flag `--self-test` + early-exit en `main()`.
- **Verify:** `--self-test` → `9/9 PASS` (exit 0, `campaign_verify_cmd`); fixture standalone post-fix → `PASS -- 0 failure(s)` (exit 0); `calls=[999, 999, 2]`, `rebuilds=1`, throughput 257k rec/s > floor 13.3k.
- **Estado:** ✅ COMPLETED

### Step 4 — Corrida real post-fix + delta documentado ✅
- **Archivos:** `$TEMP\opencode\bench01\{after.json,after_batch0.json,bench01_init_prep.py}` (no versionados)
- **Acción:** corrida real `--engines vanta --size 2000 --queries 50` post-fix (mismo comando que el before) + `--batch-size 0` (clamp) + micro-medición del trabajo excluido (init+prep, 10 reps) → delta documentado en §Diagnóstico.
- **Verify:** tabla + JSON emitidos; `ingest_mode` = `chunked (--batch-size 999)` en ambos (999 y 0); micro: init 317.28 ms / prep 0.74 ms (medianas).
- **Estado:** ✅ COMPLETED

### Step 5 — Docs (metodología + comparabilidad) ✅
- **Archivos:** `benchmarks/README.md`, `docs/user/operations/BENCHMARKS.md` §18, `docs/user/benchmarks/COMPETITIVE_SDK_BENCH.md`, `docs/user/benchmarks/COMPETITIVE_ANALYSIS.md`, `docs/index.md` + `llms.txt` (generados)
- **Acción:** quitar la necesidad del flag, documentar región medida + clamp + nota de comparabilidad histórica. NO se re-publican números.
- **Verify:** `check-links` exit 0 · `check-docs` exit 0 (GATING all clear) · `gen-index --check` exit 0 tras `--write` (diff esperado: BENCH-01 + DIST-01 + counts) · `validate-docs-coverage.ps1` exit 0 (0 gaps) · markdownlint: 0 issues nuevos (MD028 `benchmarks/README.md:44` preexistente — HEAD:39).
- **Estado:** ✅ COMPLETED

### Step 6 — Verify full + Review P2-01 + commit LOCAL ✅
- **Archivos:** `docs/dev/tasks/BENCH-01.md` (Review + RESULTADO)
- **Acción:** verify mecánico + §Review con evidencia (tier Fast; sin fork a `vanta-review` — worker leaf; ACCEPT delegado al orquestador) + commit **LOCAL** `fix(bench): BENCH-01 — ...` (NUNCA push).
- **Verify:** `dev-tools/verify_changed.ps1` → ALL 4 PASS; OCR delegation exit 0; `git show --stat HEAD` = solo archivos propios.
- **Estado:** ✅ COMPLETED

## Dependencias

- F0 — sin dependencias. nextTask: DIST-01.
- FIND-232/FIND-233 ✅ (contexto del área de benches; no se re-tocan sus archivos).

## Review (GATE — agente distinto, P2-01)

- **Tier:** **Fast** (paths: `benchmarks/**`, `docs/**` — sin paths adversariales; regla mecánica de `pipeline-full.md` §Cierre).
- **Revisor:** ✅ `vanta-review` (contexto fresco — `ses_ef9876a58ffeI1MCNxwT28HJk2`) — **APPROVE** (2026-10-04) — este worker es leaf (sin fork a `vanta-review`); evidencia mecánica completa registrada acá para spot-check Fast + ACCEPT con reviewer fresco (precedente FIND-233). No se usa `mode:degraded` sin waiver (prohibido el bypass silencioso).
- **Enfoque:** ¿la región medida quedó correctamente aislada? ¿el clamp hace imposible el doble build por construcción? ¿el self-test prueba el harness real (no una reimplementación)? ¿los docs no sobre-claimen?
- **Evidencia de verificación (real, no auto-reporte):**
  1. `.venv/Scripts/python.exe benchmarks/competitive_bench.py --self-test` → **9/9 PASS** (`campaign_verify_cmd`, exit 0) — incluye `calls=[999, 999, 2]` y `rebuilds=1`.
  2. Fixture standalone `$TEMP\opencode\bench01\bench01_fixture.py`: **RED pre-fix (3 FAIL, exit 1) → GREEN post-fix (0 failure, exit 0)** — mismas aserciones que el `--self-test`.
  3. `.venv/Scripts/python.exe -m py_compile benchmarks/competitive_bench.py` → exit 0 (`campaign_verify_cmd`; 1.er intento -1 transitorio, reintento verde — documentado).
  4. Corridas reales post-fix (`after.json`, `after_batch0.json`): tabla + JSON emitidos; `ingest_mode` clampeado correcto en ambos.
  5. `node scripts/docs/check-links.mjs` → exit 0; `check-docs.mjs` → exit 0 (GATING all clear); `gen-index.mjs --check` → exit 0 tras `--write`; `scripts/validate-docs-coverage.ps1` → exit 0 (0 gaps).
  6. `dev-tools/verify_changed.ps1` → **ALL 4 PASS** (fmt/check/clippy/docs-coverage; 163 archivos afectados — el árbol incluye WIP Rust ajeno de DIST-01).
  7. markdownlint-cli2 sobre los 5 archivos tocados: **0 issues nuevos** (MD028 `benchmarks/README.md:44` preexistente — verificado contra `HEAD:benchmarks/README.md:39`).
  8. `nextest`: **n/a** — el diff no toca Rust (Python+docs); el fast gate Rust del árbol pasó vía verify_changed.
- **OCR delegation (advisory, exit 0):** preview → mi reviewable propio es `benchmarks/competitive_bench.py` (Rule Group 1 — Python). Reglas aplicadas manualmente: sin dead code (helper/flags usados), sin mutable-defaults (`batch_size=0` int; stubs con `None`), edge cases de `effective_chunk_size` cubiertos (0/negativo/≥1000), `finally` restaura el stub y limpia tmpdir, sin bare-except nuevo, sin patrones de seguridad (eval/subprocess/pickle: ausentes) → **0 Critical / 0 High**. Los demás reviewables del preview (release-plz.toml, vanta-memory/*) son WIP de DIST-01 — fuera de mi scope, no se tocan.
- **Veredicto:** ✅ **APPROVE** (2026-10-04) — contrato verificado con re-ejecución (self-test 9/9 · RED→GREEN externo reproducido · trazado del clamp sin caminos ≥1000 · rebuilds=1 no circular); Optionals: umbral duplicado del engine (drift) + priorizar FIND-253 antes de la próxima corrida publicada; cierre mecánico por el orquestador. **ACCEPT delegado** (evidencia lista para spot-check Fast del orquestador; sin blockers mecánicos).

## Notas

- La evidencia del prompt ("Ingest 25.0ms > P50 19ms") **no se pudo localizar** en ningún artefacto del repo (grep + corridas reales); la evidencia CÓDIGO-REAL y reproducida es la estructural (§Fase 1 + §Diagnóstico). Se documenta como tal, sin fabricar el par 25/19.
- Decisión de diseño: el modo single-call (`--batch-size 0`) se **elimina por clamp** (0 → 999) en vez de mantenerlo con skip de `rebuild_index()`: (a) mantenerlo exigía una tercera semántica en la columna Index ("build incluido en Ingest"), rompiendo el formato de salida; (b) el delta `0 − 999` que aislaba el hidden rebuild era un diagnóstico one-off (2026-07-31) cuyo costo era un footgun activo. Pre-mortem #3 respondido: el doble rebuild no era intencional — era un artefacto del `InsertMode::Auto` del SDK + el rebuild explícito del harness.
- Ponytail: el fix es ~40 líneas netas (helper + reordenamiento + self-test); sin abstracciones nuevas fuera del helper necesario en 3 call-sites.

## Deuda técnica (Regla 6)

- **Introducida:** ninguna (Python stdlib; sin dependencias nuevas; sin unsafe).
- **Pagada:** elimina la clase "el harness puede medir el mismo build dos veces" (footgun documentado desde 2026-07-31) y la región Ingest contaminada con setup.
- **Diferida (FIND candidato, lo rutea el orquestador):** asimetría cross-engine de la región de timer (Lance/Chroma incluyen connect/collection+prep en su Ingest; Qdrant/Milvus ya no; Vanta ahora tampoco) — alinear las 5 regiones es un cambio de metodología con re-medición completa.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificado: región aislada + clamp + self-test verde (RED→GREEN capturado) + before/after documentado + docs actualizados. |
| **Commit** | Commit atómico conventional `fix(bench):` + `git diff` limpio (solo archivos propios) + verificación mecánica transcripta. |
| **Release** | n/a (dev-tools; sin cambio de release). Push diferido por instrucción del owner (commit LOCAL). |

## Herramientas necesarias

- `.venv/Scripts/python.exe` (Python 3.11.9 del repo: vantadb 0.7.0 + numpy/h5py/lancedb/chromadb/psutil/tabulate) — corridas del harness.
- `python -m py_compile` — sintaxis.
- `node scripts/docs/{check-links,check-docs,gen-index}.mjs` — gates de docs.
- `pwsh dev-tools/ocr-review.ps1 -Format json` — OCR delegation (advisory).
- `campaign_verify_cmd` — verificación mecánica por comando.
- `codegraph_codegraph_explore` + `codebase-memory-mcp_check_index_coverage` — blast radius (usados).

**Skills cargadas (SDP v3):** `performance-optimization` (pinned policy: performance) · `systematic-debugging` (bug-fix: Iron Law) · `test-driven-development` (fixture RED→GREEN) · `documentation-skill` (docs/** obligatoria) · `incremental-implementation` (slices) · `writing-guidelines` · `writing-plans` · `context-engineering` · (+ `campaign-executor`/`progreso` base, auto). SDP: base+lifecycle+keywords, 8 cargadas de 8.

## Investigation Notes

- **Mecanismo real del "hidden rebuild" (código vigente, verificado):** `put_batch_raw` (PyO3) → `put_batch_inner` (SDK) chunkea por `config.batch_size` (default 1000); `BatchInsertOptions::needs_rebuild` con `InsertMode::Auto` → `batch_size >= 1000` → tras el batch: `engine.rebuild_vector_index()` **dentro del call** (`src/sdk/api/memory.rs:721-723,766-768`; `src/storage/engine/ops.rs:50-59`). El header del harness citaba `ops.rs:957-964` — **stale** (el archivo tiene 353L hoy); corregido en el fix.
- **El SDK además reconstruye derived/text/sparse en cada call** (`memory.rs:776-778`) → el camino chunked (n/999 calls) paga esos rebuilds dentro del Ingest. Es el costo real del API path (no duplicado); documentado en el docstring.
- **`put_batch_raw` no expone `insert_mode`** (`vantadb-python/src/lib.rs:736` — firma sin modo) → el único lever del harness es el tamaño de chunk. Confirma que el clamp es la solución correcta del lado harness.
- **`db.rebuild_index()`** (`vantadb-python/src/lib.rs:1377-1382`) → `engine.rebuild_index()` (índice vectorial) — es el build canónico que la columna Index mide exactamente una vez.
- **Evidencia publicada del síntoma:** `COMPETITIVE_ANALYSIS.md:122` (doble build + fix 2026-07-31) y `BENCHMARK_OPTIMIZATION_2026.md:857` (el gap Ingest/Index medía la misma regresión dos veces). El workaround (`--batch-size 999`) quedó como default pero el footgun seguía vivo para `--batch-size 0`/≥1000.
- **Cobertura de índice:** `codebase-memory-mcp_check_index_coverage` → los 4 paths clave `no_recorded_issue` (best-effort); CodeGraph sí indexa `benchmarks/competitive_bench.py` (blast radius verificado).
- **Incógnitas resueltas:** (1) ¿el doble build era intencional? → No: artefacto SDK+harness, documentado como bug desde 2026-07-31; (2) ¿se puede forzar incremental desde Python? → No (sin param de modo); (3) ¿el fix altera el esquema JSON? → No.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas en DISCOVERY (§Investigation Notes) |
| Pendientes de ejecución (downhill) | 5 — Steps 2–6 |
| % completado | 20% (Steps 0–1 ✅ de 7) |

## Diagnóstico (evidencia dura — Regla 9/11)

### Pre-fix (harness actual) — corridas reales en este box

| Run | Comando | Ingest QPS | Index (ms) | Query QPS | p50 (ms) | Fuente |
|---|---|---|---|---|---|---|
| batch-999 (default) | `--engines vanta --size 2000 --queries 50` | 137.5 | 12022.4 | 152.8 | 6.116 | `$TEMP\opencode\bench01\before.json` (caja contaminada: CPU 100%, 6× VS Code) |
| **batch-0 (single call)** | idem + `--batch-size 0` | **856.4** | **1512.4** | 588.0 | 1.736 | `$TEMP\opencode\bench01\before_batch0.json` |
| publicados (2026-08-12) | idem + `--batch-size 999` | 520.3 | 1695.7 | 635.6 | 1.510 | `docs/user/benchmarks/competitive_sdk_bench.json` |

Lectura: en batch-0 el Ingest (856 QPS ≈ 2.34 s) incluye un rebuild completo (~1.5 s) que el Index timer **repite** (1512 ms) → 2 builds del mismo índice por run. En chunked, el Ingest paga 11× los rebuilds derived/text/sparse del SDK y el Index mide exactamente 1 build vectorial.

### RED fixture (pre-fix)

```
[FAIL] effective_chunk_size exists -- helper not present (pre-fix)
[FAIL] no put_batch_raw call >= 1000 nodes -- calls=[2000]
[PASS] exactly one rebuild_index() per run -- rebuilds=1
[FAIL] Ingest timer excludes client init -- throughput=6568 rec/s, floor=13333 rec/s
Fixture: FAIL (RED expected pre-fix) -- 3 failure(s)   (exit 1)
```

### Post-fix — corridas reales + micro-medición + delta

| Run (post-fix) | Comando | Ingest QPS | Index (ms) | Query QPS | p50 (ms) | Fuente |
|---|---|---|---|---|---|---|
| default (chunk 999) | `--engines vanta --size 2000 --queries 50` | 692.3 | 1016.6 | 752.6 | 1.287 | `after.json` |
| `--batch-size 0` (clampeado → 999) | idem + `--batch-size 0` | 835.9 | 932.7 | 887.2 | 1.058 | `after_batch0.json` |
| self-test (stub, determinista) | `--self-test` | — | — | — | — | `9/9 PASS` |

**Delta de la región medida (determinista, atribuible):** micro-medición de 10 reps con el engine real → `Client()` init mediana **317.28 ms** (min 292.47 / max 377.79) + prep de payloads **0.74 ms** = **≈318 ms por run** que antes entraban al timer Ingest y ahora no. Sobre el ingest del run publicado (2000/520.3 ≈ 3.84 s) ≈ **8.3%**; sobre el after-run (2000/692.3 ≈ 2.89 s) ≈ **11%**. Ese es el sesgo que el número viejo arrastraba.

**Delta del doble build (modo eliminado):** pre-fix `--batch-size 0` → el Ingest contenía 1 rebuild vectorial completo y el Index timer lo repetía (1512.4 ms) = 2 builds del mismo índice. Post-fix `--batch-size 0` → clampeado (JSON `ingest_mode: chunked (--batch-size 999)`), el Ingest no contiene rebuild y el Index mide 1 build (932.7 ms). Prueba estructural determinista: fixture con stub → `calls=[999, 999, 2]` (ningún call ≥1000) + `rebuilds=1`.

**Honestidad de la comparación before/after de QPS:** el before-run quedó contaminado (CPU 100%, 6× VS Code: Ingest 137.5 QPS) y el after-run corrió con la caja más descargada (692.3 QPS) — la diferencia NO es atribuible al fix (varianza cross-VM documentada en FIND-232: 1.7–2.4×). Los deltas atribuibles son los dos de arriba (micro-medición + fixture), no el QPS crudo. Los números publicados del run 2026-08-12 no se regeneran (fuera de appetite).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7 (Steps 0–6)
PROXIMO_STEP: ninguno local — ACCEPT/Review P2-01 (orquestador, reviewer fresco) + plan Task 13 ✅ + push diferido al final del plan (instrucción owner)
COMMIT_HASH: 860340b9 (fix) + commit de bookkeeping de este archivo (RESULTADO §7)
ARCHIVOS: benchmarks/competitive_bench.py · benchmarks/README.md · docs/user/operations/BENCHMARKS.md · docs/user/benchmarks/COMPETITIVE_SDK_BENCH.md · docs/user/benchmarks/COMPETITIVE_ANALYSIS.md · docs/dev/tasks/BENCH-01.md · docs/index.md · llms.txt (generados)
VERIFY_CONTRATO: pasa (self-test 9/9 + fixture RED→GREEN + corridas reales + docs gates 0 + verify_changed ALL 4 PASS + OCR 0 Critical/High)
BLOQUEO: ninguno (push NO ejecutado — diferido por instrucción del owner; ACCEPT delegado al orquestador)
GATES_EVALUADOS: P:no D:no V:no C:no | contrato del plan explícito; sin ambigüedad residual; verify sin fallas; sin colaterales fuera de scope (asimetría cross-engine → deuda documentada)
SKILLS_CARGADAS: performance-optimization (pinned) · systematic-debugging · test-driven-development · documentation-skill · incremental-implementation · writing-guidelines · writing-plans · context-engineering (+ campaign-executor/progreso base)
```
