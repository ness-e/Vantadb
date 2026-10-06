---
title: "TASK BENCH-02: BEIR/MTEB recall@k vs sqlite-vec (harness + número reproducible)"
kind: task
description: "Harness BEIR SciFact (test split) con embeddings MiniLM ONNX locales; número recall@10 vs sqlite-vec reproducible (dataset/hardware/seed declarados) + doc en BENCHMARKS.md"
---

# TASK BENCH-02: BEIR/MTEB recall@k vs pgvector/Chroma/sqlite-vec

- **Fecha:** 2026-10-06 · **Tipo:** feature-add (harness de benchmark, Python dev-only + docs) · **No toca código del producto** (cero cambios en `src/`, bindings, contratos)
- **Contrato (plan Task 67 L1923):** "número de recall@k (BEIR/MTEB — dataset/split declarado) vs ≥1 competidor (pgvector **o** sqlite-vec), reproducible con comando documentado (dataset, hardware, seed); doc en `docs/user/operations/BENCHMARKS.md`; sin claims sin comando."
- **Origen:** plan `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` Task 67 (L1914-1940, bloque F0 expandido) + Backlog `docs/dev/Backlog.md:140` (fila BENCH-02, se elimina al cierre — Trigger 1 progreso) + ADR-0006 `:36,53` (BEIR como precedente metodológico) + `docs/api/EMBEDDINGS.md:145` (MTEB en docs).
- **Alcance:** harness nuevo `benchmarks/beir_recall_bench.py` (dataset BEIR SciFact vía `mteb/scifact`, embeddings all-MiniLM-L6-v2 ONNX locales, recall@k vs qrels para VantaDB y sqlite-vec) + sección nueva en `BENCHMARKS.md` con el número y el comando. **No** se modifica `competitive_bench.py` (ver Spec D5). **No** pgvector en este run (ver Spec D1). **No** se toca `opencode.jsonc`/master plan/`docs/pipeline-state.json` (prohibidos).

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 67, bloque F0)
- **Fuente:** Backlog `:140` (fila BENCH-02) + plan Task 67 L1914-1940
- **Esfuerzo:** 🟠 3-5d (plan) | **Appetite:** max 1sem ✓
- **Prioridad:** 🟠 (plan) / 🟠 P1 (Backlog)
- **Tipo:** feature-add (harness Python dev-only) + docs
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-06 | **last-synced:** 2026-10-06
- **Estado:** ⏳ IN PROGRESS — steps 1-4 ✅; step 5 (cierre: review P2-01 + commit + campaign) en curso. **Números medidos 2026-10-06:** recall@10 = **0.7833** (VantaDB = exact-knn = sqlite-vec), nDCG@10 = **0.6451** (= referencia publicada MTEB 64.51), index-recall 1.0 (por construcción: ambos engines son exactos a este tamaño — VantaDB rutea al flat exact scan, `flat_threshold` 10.000 > 5.183 nodos; ver Review R1). JSON `benchmarks/beir_recall_report.json` (gitignored) regenerado 04:36:31 con el script final.
- **Incógnitas (uphill) — RESUELTAS en DISCOVERY (2026-10-06):**
  - **(a) Competidor (pre-mortem #1):** pgvector **no es viable en esta máquina** — Docker no instalado (`docker: NOT AVAILABLE`); pgvector requiere un servidor PostgreSQL + extensión compilada (instalación nativa Windows fuera de la filosofía "no-docker" del harness PERF-03). **sqlite-vec SÍ es viable**: `pip install sqlite-vec` (0.1.9, MIT/Apache-2.0) — extensión embebida, sin servidor, sin Docker. **Decisión: sqlite-vec como competidor del contrato.** pgvector queda como FIND (no bloquea; un competidor viable ya cumple el contrato).
  - **(b) Dataset BEIR real:** el host canónico UKP (`public.ukp.informatik.tu-darmstadt.de`) es **inalcanzable desde esta red** (timeout de conexión verificado). Alternativa viva: HuggingFace `mteb/scifact` (JSONL: corpus 5.183 docs, queries 1.109, qrels/test 339 filas → 300 queries únicas). Descargado y cacheado en `benchmarks/datasets/scifact/` (8 MB). Split declarado: **test** (el estándar BEIR/MTEB para SciFact).
  - **(c) Embeddings sin torch:** el `.venv` (Python 3.11.9) ya tiene `onnxruntime` 1.26.0 + `tokenizers` 0.23.1 y el modelo **all-MiniLM-L6-v2 ONNX local** (`embeddings/models/all-MiniLM-L6-v2/onnx/model.onnx`, rev manifest `1110a24`, 384d, max_seq 256). No se necesita `sentence-transformers`/torch. Smoke test ejecutado: ONNX→mean-pool→L2 norm OK; sqlite-vec OK; vantadb roundtrip OK.
  - **(d) Comparabilidad:** el número es recall@k **vs qrels** (semántica MTEB, TREC recall) — NO es comparable con el `recall_at_k` de `competitive_bench.py` (que mide fidelidad del índice vs kNN exacto). Se reportan AMBOS para separar calidad de recuperación (modelo+índice) de fidelidad del índice (ver Spec D4 y nota de metodología en el doc).
- **Pendientes (downhill):** 5 steps (2-5)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `67` (instrucción del orquestador — el server lo keyea numérico)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Lectores de docs/bench (sin código de producto):** master plan Task 67 (estado — **prohibido editar**), `Backlog.md:140` (fila BENCH-02 — se elimina al cierre, Trigger 1 progreso), `docs/user/operations/BENCHMARKS.md` (sección nueva §21), `benchmarks/README.md` (candidato a mención — evaluar), README del repo (sin cambios previstos). |
| Callees | Reusa (sin modificar): `embeddings/models/all-MiniLM-L6-v2/` (ONNX+tokenizer, artefacto local gitignored? verificado: `embeddings/models/` no está trackeado), `.venv` (onnxruntime/tokenizers/sqlite-vec/vantadb-py), dataset `mteb/scifact` (descarga cacheada). NO modifica `benchmarks/competitive_bench.py` ni `benchmarks/requirements.txt` más allá de un bloque opcional documentado. |
| Implicaciones | **Aditivo:** `benchmarks/beir_recall_bench.py` (nuevo) + `benchmarks/requirements.txt` (bloque opcional `sqlite-vec`/`onnxruntime`/`tokenizers` con nota) + `.gitignore` (`benchmarks/datasets/` + `benchmarks/beir_recall_report*.json` — artefactos regenerables) + `docs/user/operations/BENCHMARKS.md` (§21 nueva) + este task file + fila Backlog eliminada al cierre + registro `avance`. Sin cambios en `src/`, bindings, contratos, wire, deps del producto ni locks. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-06, HEAD `6029efd1`, branch `develop`; WIP ajeno: `docs/dev/plans/2026-10-04-master-plan-0.9.0.md`, `docs/index.md`, `opencode.jsonc` modificados y `dev-tools/heavy-test-lock.ps1`, `docs/dev/tasks/STRAT-04.md`, `benchmarks/wasm_threads_*`, `vantadb-wasm/threads-kernel/` untracked — **no se tocan ni se stagean**; commit con pathspec).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `benchmarks/competitive_bench.py` (header metodología L1-39, datasets L117-242, bench_vantadb L266-386, recall L360-367, registry L1041-1059; recall@k por query `:336-360` ✅ verificado).
  - `docs/user/operations/BENCHMARKS.md` (estructura §1-§20 + "Planificado"; §2 entorno/comando; §7 competitivo LanceDB/Chroma).
  - `benchmarks/requirements.txt` (deps actuales + estilo de comentarios `[cita NO VERIFICADA - sin red]`).
  - `embeddings/manifest.json` (all-MiniLM-L6-v2: repo `sentence-transformers/all-MiniLM-L6-v2`, rev `1110a24`, dim 384, ONNX 80MB, Apache-2.0) + `embeddings/models/all-MiniLM-L6-v2/` (artefactos ONNX/tokenizer) + `sentence_bert_config.json` (max_seq_length 256).
  - `docs/dev/Backlog.md:140` (fila BENCH-02: "un número de recall@k verificado vale más que 50 tareas de roadmap"; Dueño vanta-tuner + vanta-docs) y `:142` citado por el plan.
  - `docs/dev/tasks/STRAT-06.md` (formato canónico de task file F0 aprobado) + `.opencode/task-system/prompts/pipeline-full.md` + `prompts/task.md` (formato).
  - `.gitignore` (`datasets/*` es root-level; `benchmarks/datasets/` NO ignorado — verificado con `git check-ignore`; `benchmarks/competitive_data/` sí).
  - Metodología externa (fetch-verificada 2026-10-06): MTEB README + `mteb/_evaluators/retrieval_metrics.py` (recall@k = TREC recall vía pytrec_eval; k_values 1/10/100), MTEB retrieval docs (SciFact → `mteb/scifact`, score ndcg_at_10), HF `mteb/scifact` card/API (splits), sqlite-vec README + docs Python (API vec0), pgvector (requiere servidor PG).
- **Archivos referenciados hacia dentro (imports/deps):** `beir_recall_bench.py` importará `numpy`, `onnxruntime`, `tokenizers`, `sqlite_vec` (+ `sqlite3` stdlib), `vantadb` — todos presentes en `.venv`. Sin imports desde el código del producto.
- **Referencias entrantes (grep `BENCH-02|beir|mteb` HEAD `6029efd1`):** `rg -i "beir|mteb" benchmarks/` = 0 hits (premisa del plan re-verificada); docs: `docs/dev/architecture/adr/ADR-0006-rrf-constant.md:36,53` (BEIR precedente — sin cambios), `docs/api/EMBEDDINGS.md:145` (MTEB — sin cambios), `embeddings/README.md` (MTEB/beir mención — sin cambios). Plan L1129 (MEMG-11 cita BENCH-02 como pata de calidad pendiente — sin cambios). Sin referencias de código.
- **Veredicto impacto:** **BAJO (aditivo, dev-tooling + docs)** — 1 script nuevo + 1 sección doc + 1 bloque opcional de requirements + 2 líneas .gitignore + 1 task file. Sin tocar producto, contratos, wire ni deps del core. Pre-mortem mitigado: (1) competidor → sqlite-vec viable (pgvector FIND); (2) dataset → split test declarado + cache local + descarga reproducible por comando; (3) incomparabilidad → nota de metodología explícita (qrels-recall ≠ índice-recall) + ambos números reportados.

## Contrato

"número de recall@k (BEIR/MTEB — dataset/split declarado) vs ≥1 competidor (pgvector **o** sqlite-vec), reproducible con comando documentado (dataset, hardware, seed); doc en `docs/user/operations/BENCHMARKS.md`; sin claims sin comando." (plan Task 67 L1923).

**Verificación del contrato (cierre):** (a) número recall@k medido en **BEIR SciFact split test** (300 queries; corpus 5.183) para **VantaDB vs sqlite-vec** (ambos con los mismos embeddings all-MiniLM-L6-v2 ONNX locales); (b) comando exacto documentado en `BENCHMARKS.md` §21 con **dataset/split + hardware (CPU/RAM/OS) + seed 42** (Regla 11); (c) harness con `--self-test` offline (provee la matemática de recall + parsing qrels sin red); (d) sin claims sin comando (el doc cita el comando que regenera el JSON); (e) gates docs 0 + `validate-docs-coverage.ps1`.

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado.** Blast radius ≤6 archivos propios, dev-tooling sin API pública del producto, contrato sancionado por el plan F0 (Gate Result ✅ DO). La incógnita real del pre-mortem #1 (competidor) se resolvió con evidencia y se documenta abajo (D1) — sin ambigüedad nueva que requiera GO del owner (la decisión `sqlite-vec` está dentro del contrato: "pgvector **o** sqlite-vec").

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Competidor | A) **sqlite-vec** (pip, embebido, sin Docker; 0.1.9 MIT/Apache; scan exacto SIMD) / B) pgvector (contra: Docker ausente en la máquina — verificado `docker: NOT AVAILABLE`; requiere servidor PG + extensión; instalación nativa fuera de la filosofía no-docker del harness) / C) competidor ya soportado (lance/chroma/qdrant — contra: el contrato lo prohíbe explícitamente) | ✅ **A** — viabilidad verificada en DISCOVERY (install + roundtrip OK); pgvector → **FIND** (deuda declarada, no bloquea) |
| 2 | Dataset + split | A) **BEIR SciFact, split test** vía `mteb/scifact` (HF; 5.183 corpus / 300 queries test / qrels) / B) UKP canónico (contra: host inalcanzable desde esta red — verificado) / C) BEIR sintético (contra: el contrato pide dataset BEIR real) | ✅ **A** — declarado: dataset `mteb/scifact`, split `test`, 300 queries, corpus completo 5.183 |
| 3 | Embedding model | A) **all-MiniLM-L6-v2 ONNX local** (repo `embeddings/models/`; rev manifest `1110a24`; 384d; mean-pool + L2; max_seq 256) / B) sentence-transformers torch (contra: torch no instalado; descarga pesada) / C) otro modelo del manifest (bge-small/multilingual-e5 — contra: no es el baseline BEIR estándar) | ✅ **A** — sin torch; modelo declarado con rev; el baseline histórico de BEIR (MiniLM es el modelo de referencia del paper BEIR) |
| 4 | Métrica primaria | A) **recall@k vs qrels (semántica MTEB/TREC: \|top-k ∩ relevantes\| / \|relevantes\|)**, k=1/10/100, headline **recall@10** / B) nDCG@10 (contra: el contrato pide recall@k) / C) recall del índice vs kNN exacto (contra: no es la métrica BEIR/MTEB) | ✅ **A** — verificada contra `mteb/_evaluators/retrieval_metrics.py` (pytrec_eval `recall_k`); **secundaria**: fidelidad del índice (recall@10 del índice vs kNN exacto numpy) + kNN exacto como techo — separa calidad de recuperación de fidelidad ANN (nota de metodología en el doc) |
| 5 | Hogar del harness | A) **script nuevo `benchmarks/beir_recall_bench.py`** / B) extender `competitive_bench.py` (contra: reshape de ground_truth fijo-k → qrels variable, imports ONNX/texto en el harness de velocidad, riesgo de romper flujos existentes; diff grande) | ✅ **A** — diff mínimo, cero riesgo al harness existente; `competitive_bench.py` queda intacto (sqlite-vec en su registry → FIND opcional) |
| 6 | Seed | A) **seed 42** con rol real: muestreo determinista de queries cuando `--limit-queries < total` (default: todas) + declarado en el comando del doc | ✅ **A** — el contrato exige seed declarado; el sampling determinista le da efecto verificable |
| 7 | Salida | A) **stdout markdown + JSON `benchmarks/beir_recall_report.json` (gitignored)** / B) solo stdout (contra: el doc necesita un artefacto regenerable citado por comando) | ✅ **A** — patrón `embed_bench_report.json` (gitignored, regenerable) |
| 8 | `--self-test` | A) **self-test offline**: recall math + parser qrels + sampler determinista con fixtures in-memory (sin red, sin modelos) / B) sin self-test (contra: la lógica de recall es no trivial y quedaría sin check) | ✅ **A** — patrón `competitive_bench.py --self-test`; RED antes del run real |

## Invariantes de dominio (handoff — MUST)

1. **Cero cambios al producto:** sin `src/`, bindings, contratos, wire, deps del core. `competitive_bench.py` byte-idéntico.
2. **Cero claims sin comando:** el número de `BENCHMARKS.md` §21 cita dataset/split + hardware + seed + comando exacto (Regla 11).
3. **Métrica declarada:** el recall del doc es **vs qrels** (MTEB/TREC); la nota de metodología distingue explícitamente de índice-recall (no confundir con `competitive_bench.py`).
4. **No tocar:** master plan, `opencode.jsonc`, `docs/pipeline-state.json` (prohibidos). WIP ajeno no se stagea; commit con **pathspec**.
5. **Commit LOCAL** (`perf(bench):`/`docs:`), ⛔ nunca push (Regla 7 §Política de git).
6. **Datasets/reportes no se commitean:** `benchmarks/datasets/` + `benchmarks/beir_recall_report*.json` → `.gitignore` (regenerables por comando).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** ≤0 — sin código de producto, sin `unsafe`, sin deps del core. Deuda declarada:
- **pgvector no medido** (Docker ausente) → `FIND-314`.
- **El número de SciFact mide el flat exact scan, no el grafo HNSW** (5.183 < `flat_threshold` 10.000; R1 del review) → `FIND-315` (medición HNSW real con corpus >10K o `VANTADB_FLAT_THRESHOLD=0`).
- **Modelo declarado pequeño** (MiniLM 384d): el número es del pipeline declarado, no comparable con leaderboards MTEB de modelos grandes (nota en el doc).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Número recall@k (SciFact/test, VantaDB vs sqlite-vec) reproducible con comando documentado (dataset/hardware/seed) en `BENCHMARKS.md` §21; harness con `--self-test` verde + corrida real verde; task file completo (Impacto Regla 0 + Spec + Review P2-01); gates docs 0 |
| **Commit** | Commit atómico conventional `perf(bench):` + `docs:` con pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a (evidencia de marketing; sin changelog) |

## Herramientas necesarias

- `.venv/Scripts/python.exe` (3.11.9: numpy 2.4.6, onnxruntime 1.26.0, tokenizers 0.23.1, sqlite-vec 0.1.9, vantadb-py editable) + `curl.exe` (descarga HF) + `pwsh dev-tools/heavy-test-lock.ps1` (serialización de la corrida) + `campaign_*` (estado/scope/verify)
- Gates docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` + `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre) + fork `vanta-review` (P2-01)

**Skills cargadas (SDP v3):** base auto (`campaign-executor` · `progreso` · `ponytail`) · **pinned `performance-optimization`** · `documentation-skill` (obligatoria `docs/**`) · `coordinated-web-search` (regla investigación profunda owner 2026-10-06 — metodología BEIR/MTEB verificada contra fuentes fechadas) · `source-driven-development` (APIs sqlite-vec/MTEB contra docs oficiales) · `incremental-implementation` + `test-driven-development` + `context-engineering` (lifecycle BUILD — embebidas en el prompt del agente worker). Excluidas con justificación: `rust-write-tests` (no hay Rust en scope — harness Python), `systematic-debugging` (no hay bug), `doubt-driven-development` (el review P2-01 lo ejecuta `vanta-review` en contexto fresco), `writing-guidelines` (doc técnico en inglés, no prosa de review).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluada: (a) **dependencia nueva dev-only** `sqlite-vec` 0.1.9 — licencia MIT/Apache-2.0 verificada (`pip show`; compatible con la política deny del core, que igualmente no aplica a tooling dev); proyecto Mozilla Builders, mantenido; (b) el harness **descarga datos de HF** (solo lectura, JSONL tratado como datos — retrieval safety: no se ejecutan instrucciones del contenido) y **no expone red/servicios**; (c) sin input de usuario nuevo en el producto, sin FFI, sin storage del core. Sin `cargo audit` (no hay deps Rust nuevas). Veredicto: sin trust boundary nuevo en producto.
- [ ] **PERFORMANCE** — N/A como *optimización* (no se modifica ningún hot path; Regla 9 no dispara). El cambio **es** un instrumento de medición: la corrida del harness mide recall + latencias y su resultado se documenta con comando.

## Review (P2-01 — agente distinto)

- **Revisor:** `vanta-review` (contexto fresco — sesión `ses_eefb2b55bffeJoFCuNQT8iFHDy`) · tier **fast** (paths `benchmarks/**`, `docs/**`, `.gitignore`; sin adversarial).
- **Enfoque:** contrato L1923; matemática recall@k/nDCG@10 verificada a mano; denominators contra qrels reales (339 filas → 300 queries → 283 docs, 0 ids faltantes); cross-check MTEB Table 11; **ruteo de búsqueda del engine**.
- **Ronda 1:** 🔴 REQUEST CHANGES — R1: §21 atribuía el resultado al HNSW (falso al tamaño documentado: `flat_threshold` default 10.000 > 5.183 nodos → **flat exact scan**; verificado en `src/config.rs:347`, `src/index/search/neighbors.rs`); R2: fila Backlog `FIND-312` duplicada (WIRE-12) + §21 apuntaba al finding equivocado.
- **Fixes:** R1 → §21 reescrita (modo flat-exact declarado + `engine_config` en el JSON + `FIND-315` para la medición HNSW real); R2 → renumber a `FIND-314`. Nits: guard `exact_knn` (`top_k >= len(corpus)`), `sample_queries` sin param muerto, sha256 del dataset en el JSON. Números/matemática/cita: sin cambios (verificados ✅ por el revisor).
- **Ronda 2 (fixes):** ✅ **approve** — ver §Review R2 en la recitation/commit.

## Steps

### Step 1 — DISCOVERY + task file (Regla 0 + Spec + contrato)

- **Archivos:** `docs/dev/tasks/BENCH-02.md` (nuevo, este archivo)
- **Acción:** evidencia anclada (plan Task 67, Backlog `:140`, código real de `competitive_bench.py`, manifest de embeddings); pre-mortem resuelto (sqlite-vec viable; UKP caído → HF `mteb/scifact`; sin torch → ONNX local); investigación profunda fetch-verificada (MTEB evaluator + sqlite-vec + SciFact); formato canónico con Impacto Regla 0 + Spec + DoD.
- **Verify:** task file existe + smoke test de las 3 patas (ONNX/sqlite-vec/vantadb) OK
- **Evidencia:** ✅ este archivo; smoke test ejecutado (onnx OK (2,384) norm 1.0 · sqlite-vec OK · vantadb OK)
- **Estado:** ✅ COMPLETED

### Step 2 — Harness `benchmarks/beir_recall_bench.py` + self-test

- **Archivos:** `benchmarks/beir_recall_bench.py` (nuevo)
- **Acción:** CLI (`--dataset scifact --split test --model all-MiniLM-L6-v2 --k 10 --top-k 100 --engines vanta,sqlite-vec --limit-queries N --seed 42 --self-test --json-output`); descarga/cache HF; embeddings ONNX batch (mean-pool+L2); qrels → relevantes por query; recall@k (MTEB/TREC) para kNN exacto (techo) / VantaDB / sqlite-vec; **nDCG@10** (métrica principal MTEB — cross-check publicable); recall del índice vs exacto; latencias p50/p99; JSON + tabla markdown; `--self-test` offline (recall math + nDCG + parser qrels + sampler).
- **Verify (RED→GREEN):** `--self-test` 16/16 PASS (2 expectativas propias corregidas en RED — la matemática del test fallaba, no el código); smoke `--limit-queries 5` verde end-to-end; corrida real verde.
- **Evidencia:** `--self-test` 16/16 ✅ · smoke (5 queries: recall@10 1.0, 3 engines) ✅ · corrida final (300 queries) ✅ — JSON `benchmarks/beir_recall_report.json`
- **Estado:** ✅ COMPLETED

### Step 3 — Corrida real (heavy lock) → número

- **Archivos:** `benchmarks/beir_recall_report.json` (artefacto gitignored)
- **Acción:** `heavy-test-lock acquire` → corrida completa (300 queries, corpus 5.183, top-k 100, seed 42) → `release` (2 corridas full: la primera pre-fix de latencias; la final con el script commiteado).
- **Verify:** JSON con recall@1/10/100 + nDCG@10 + index-recall de vanta y sqlite-vec + exact; exit 0.
- **Evidencia:** **VantaDB recall@10 = 0.7833 / nDCG@10 = 0.6451 / index-recall = 1.0000** (idéntico a exact-knn y sqlite-vec — ambos engines son exactos a este tamaño: VantaDB rutea al flat exact scan, `flat_threshold` default 10.000 > 5.183 nodos, `VANTADB_FLAT_THRESHOLD` sin setear; corregido por Review R1); ingest 26.268 s, q p50 7.088 ms, q p99 19.413 ms; sqlite-vec p50 8.489 ms. Cross-check externo: nDCG@10 = 0.6451 == MTEB paper Table 11 (MiniLM-L6 SciFact = 64.51). JSON final (04:36:31) incluye `engine_config` (`flat-exact`) + sha256 de los 3 archivos del dataset.
- **Estado:** ✅ COMPLETED

### Step 4 — Doc: `BENCHMARKS.md` §21 + requirements + .gitignore

- **Archivos:** `docs/user/operations/BENCHMARKS.md` (§21 nueva + pointer en §7), `benchmarks/requirements.txt` (bloque opcional), `.gitignore` (datasets + report), `docs/dev/Backlog.md` (FIND-314 + FIND-315)
- **Acción:** sección con metodología (dataset/split/modelo/métrica), tabla de resultados, entorno (Regla 11), comando exacto, notas de comparabilidad (qrels vs índice; no comparable con §2/§7/§8) y elección de competidor (pgvector→FIND-314; flat-scan/HNSW→FIND-315).
- **Verify:** gates docs 0 — `check-links` ✅ · `check-docs` ✅ · `gen-index --write` + `--check` ✅ (index regenerado incluye BENCH-02; también paga el atraso de STRAT-04/06 ya commiteados) · `validate-docs-coverage.ps1` 0 gaps ✅ · markdownlint 0 issues ✅
- **Estado:** ✅ COMPLETED

### Step 5 — Cierre: verify full + OCR + review P2-01 + commit + campaign

- **Acción:** OCR delegation (advisory; reglas recibidas para el .py — 3 fixes aplicados: dead code `eprint` eliminado, guard `limit<=0`, `dim` explícito en `embed_texts`); fork `vanta-review` (P2-01 — tier **fast**: paths `benchmarks/**`, `docs/**`, `.gitignore`; sin adversarial); commit LOCAL `perf(bench):`/`docs:` con pathspec; campaign `completed` taskId `67` + `skill progreso` (Trigger 1: eliminar fila Backlog `:140` + registro avance).
- **Verify:** contrato 1:1 (número + comando + harness + doc) + review veredicto registrado
- **Estado:** ⬜ PENDING
