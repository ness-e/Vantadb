---
title: "VER-08 — Harness propio: canonical_p99 + LoCoMo/LongMemEval-S/BEAM-subset + p99-CI"
kind: task
---

# VER-08: Harness propio: canonical_p99 + LoCoMo/LongMemEval-S/BEAM-subset + p99-CI

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 45, Fase F5 — wave F5.1, co-batch con ICP-01 ‖ ICP-02)
- **Fuente:** plan Task 45 · research `docs/dev/research/mgr-19-benchmarks-baseline-suites.md` + `docs/dev/research/mgr-12-confianza.md` §5
- **Esfuerzo:** 🔴 3-5d · **Prioridad:** 🔴 · **Tipo:** Mixto (Python evals + docs + CI-only-read)
- **Ruta:** vanta-tuner (+ vanta-audit: diseño del juez/protocolo — P2-01)
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Branch:** develop · **Commit:** lo hace el LEAD (wave F5.1: sub-agentes NO commitean)
- **Estado:** ⏳ IN PROGRESS — **trabajo ✅ completo (8/8 steps) + fixes review r2 aplicados (F1–F6)**; cierre = LEAD (commit local + re-review P2-01 vanta-audit). Este sub-agente no commitea ni se auto-revisa (mandato wave F5.1)
- **Incógnitas (uphill):** 0 abiertas (Q5 owner —decisión de enforcement— documentada como pendiente owner, no bloquea: el contrato admite la forma "informativo-que-abre-issue documentado")
- **Pendientes (downhill):** 0 steps ejecutables por este agente

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | VER-09 (Task 47, F6) consume todo el harness; DEF-06 (Task 46) usa protocolo/hardware del reporte; agentes/CI futuros (`evals/README.md`) |
| Callees | bindings `vantadb` Python 0.7.0 (`put`/`search`/`memory.get`); `evals/calibration.py`; datasets `evals/data/` |
| Implicaciones | 100% aditivo: no toca `src/`, ni benches, ni workflows, ni el enforcement de CI. El único archivo compartido editado es `docs/user/operations/BENCHMARKS.md` (§11 pointer, §17 DEFER→entregado, §19 nuevo) y `.gitignore` (reporte local) |

## Impacto mapeado (Regla 0)

> Gate cumplido antes de la primera edición (los archivos que se **editan** se
> leyeron completos; los que se **crean** no tienen referencias entrantes aún).

- **Archivos leídos (completos):** `benches/canonical_p99.rs` · `.github/workflows/bench-canonical-p99-informational.yml` · `evals/memory_bench.py` · `evals/agent/run_eval.py` · `.gitignore` · `docs/dev/research/mgr-19-benchmarks-baseline-suites.md` · `docs/dev/research/mgr-12-confianza.md` · `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 45/46/47 + recitations F5) · `docs/user/operations/BENCHMARKS.md` (§1/§11/§17/§Planificado; resto por secciones)
- **Referencias hacia dentro (imports):** `evals/memory_harness.py` importa `evals/calibration.py` (path local) y el paquete `vantadb` (opcional: `--self-test` corre sin él). `fetch_longmemeval_subset.py` solo stdlib.
- **Referencias entrantes (grep):** `rg "memory_bench|memory_harness" .github/` → solo `lurkr-informational.yml` referencia `evals/agent/**` (no se toca). `BENCHMARKS.md §17` cita `evals/memory_bench.py` (intacto). Plan Tasks 45/46/47 citan `evals/data/` y `evals/memory_bench.py` → el destino `evals/data/` queda declarado acá (resuelve la duplicidad `datasets/` gitignored).
- **Veredicto impacto:** **bajo** — aditivo puro; nada de `src/`, `vanta-proxy/`, workflows, plan file ni Backlog se modifica. Riesgo único: BENCHMARKS.md es fuente citada por README (DEF-06 lo reconcilia después; mis ediciones no tocan §2/§5/§8).

## Contrato

> **Verbatim del plan (Task 45, F5):**

"harness publicado con dataset commiteado y comandos de reproducción (Regla 11): canonical_p99 (ya) + LongMemEval-S con submuestra versionada (licencia MIT verificada) + LoCoMo smoke y BEAM-subset SOLO con licencia verificada o disposición documentada Y corrida con backend vantadb real (bindings release) reportando recall@k + p50/p99 + ingest QPS + write-quality + abstención (LongMemEval `_abs`, N-14) + token-economy + aislamiento per-user Y reporte en `BENCHMARKS.md` con juez/protocolo pineados y pares accuracy+tokens Y calibración MGR-12 §5 medida: ECE + temperature scaling (antes/después) sobre el ground truth del harness Y gate p99 en CI: política §11 decidida con owner (enforced con umbral, o informativo-que-abre-issue documentado — Q5)"

**Contexto verificado del bloque (no re-derivado):** canonical_p99 ya existe
(`benches/canonical_p99.rs:1-15`, medido 2026-09-25 → p99 4.9987 ms, MGR-19 §1)
y su CI es informativo por diseño (`bench-canonical-p99-informational.yml:121-128`,
**abre issue en regresión**; upgrade a enforcement = decisión owner Q5 — NO se
cambia sin OK). Los datasets no estaban commiteados (`BENCHMARKS.md:1012-1064`
DEFER); destino declarado: **`evals/data/`** (`datasets/` es gitignored).

## Spec (SDD — decisiones)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Destino de la sub-muestra | `evals/data/` vs `datasets/` | ✅ `evals/data/` — `datasets/*` está gitignored (`.gitignore`: "Datasets / local assets"); `evals/data/` es versionable y ya estaba declarado por MGR-19 §3 y Task 45 |
| 2 | Sub-muestra LongMemEval-S | N preguntas full-fidelity vs truncar sesiones | ✅ **4 no-abs + 1 abs**, primeras en orden del dataset, objetos sin modificar. Repo: artefactos ≤2.4MB (`coupling-baseline.json`); 0.5MB/pregunta → 5 q ≈ 2.7MB. Truncar sesiones habría roto fidelidad del benchmark |
| 3 | Licencia LongMemEval | commit / no commit | ✅ **MIT** (HF `license:mit` + tag) → commiteable; revisión pinneada `98d7416c…` + sha256 del archivo fuente |
| 4 | LoCoMo | commit / runner / no tocar | ✅ **Disposición documentada**: `CC BY-NC 4.0` (LICENSE.txt del repo) = incompatible con repo de producto → **no se commitea ni corre en CI**; runner local opcional (usuario acepta licencia). Fila FIND propuesta (§Findings) |
| 5 | BEAM | commit subset / runner / no tocar | ✅ Licencia **MIT verificada** (repo `mohammadtavakoli78/BEAM`); commit descartado por **tamaño** (`chats/1M/<n>/chat.json` ≈ 4.6MB + `chat.pickle` 4.3MB por conv; evaluador upstream pickle/torch) → runner local opcional documentado. Fila FIND propuesta (§Findings) |
| 6 | Juez del gate | LLM GPT-4o vs determinista | ✅ **Determinista** (0 tokens): corrección = evidencia session-level en dos formas publicadas (`recall_all` headline = `answer_session_ids ⊆ top-k`; `recall_any` secundaria = intersección ≠ ∅). Pre-mortem F2 del plan + Q1 owner abierta: GPT-4o queda SOLO para reporte manual. Declarado como proxy en el reporte (fix review F1) |
| 7 | Métricas | definición de write-quality/abstención/token-economy/isolation | ✅ definidas y documentadas en docstring + §19: write-quality = fidelidad get exacto (byte-igual); abstención = proxy de umbral de score en `_abs` (τ = p5 de scores de evidencia); token-economy = palabras (sin tokenizer); isolation = hits fuera de namespace/key-set = violación |
| 8 | ECE (§5.2) | módulo nuevo / dentro del harness | ✅ `evals/calibration.py` (stdlib): ECE B=10, T* por NLL con clamp numérico `[1e-6, 1-1e-6]`, reliability antes/después, selftest de 6 checks. Límite declarado: sin reader LLM, correctitud = proxy de retrieval; D_a=1.0 en el clamp superior |
| 9 | Gate p99 CI (Q5) | enforcement con umbral / informativo-que-abre-issue | ✅ **Estado documentado**: el workflow vigente ya implementa "informativo + abre issue" (`bench-canonical-p99-informational.yml:121-128`, 176-228) → la cláusula se cumple en su forma permitida; el upgrade a enforcement sigue siendo decisión owner Q5 y **no se toca** (mandato explícito del bloque F5.1) |
| 10 | CI del harness de memoria | workflow nuevo / comandos documentados | ✅ Sin workflow nuevo (scope: "dataset commiteado y comandos de reproducción"; wiring CI = decisión futura con costo medido — VER-09/owner). Smoke offline = `--self-test` de ambos módulos |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `benches/canonical_p99.rs` y su contrato (100k×1536d seed 42 + p99) NO se tocan — es la fuente de la cláusula "canonical_p99 (ya)".
  2. El enforcement del workflow informativo NO se cambia sin OK del owner (Q5 abierta).
  3. `vanta-proxy/**` y docs de tracks ICP-01/ICP-02 son región de co-batch → solo lectura.
  4. Los datos commiteados son redistribuibles: solo LongMemEval-S (MIT) entren a `evals/data/`; jamás LoCoMo/BEAM.
  5. Reportería honesta (Regla 11): todo número citable cita comando + dataset sha256; proxies etiquetados como proxies.
- **Comandos de verificación:**
  - `python evals/calibration.py --selftest` → `OK (6 checks)` ✅
  - `python evals/memory_harness.py --self-test` → `OK (8 checks)` ✅
  - `python evals/memory_harness.py --label subset-5q` → tabla §19 ✅
  - `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs` → 0 errores nuevos ✅
- **Deuda pendiente:** ninguna de código. Pendiente owner: Q5 (enforcement p99). Pendientes de proceso: filas FIND (LoCoMo runner, BEAM runner, Python SDK sin `confidence` declarable) — ver §Findings; las inserta el LEAD (Backlog prohibido para este agente) + **re-review P2-01 round 2 (vanta-audit)** del batch F1–F6.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR: sin deuda** — aditivo de evaluación + docs; no toca código de producto ni `src/`. Los proxies declarados son límites de medición documentados, no deuda funcional.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §Contrato verificado por cláusula (ver §Verificación por cláusula) |
| **Commit** | Lo ejecuta el LEAD (wave F5.1: commits = LEAD). Changeset atómico listo con conventional commit sugerido: `feat(VER-08): memory eval harness + LongMemEval-S subset + ECE calibration` |
| **Release** | No aplica (evals/docs, sin release ni semver surface) |

## Herramientas necesarias → Skills (SDP v3, phase=BUILD)

| Skill | Fase | Justificación | Score |
|-------|------|---------------|-------|
| campaign-executor | TASK | base type CI/CD-DevOps | 1.0 |
| progreso | TASK | base | 0.9 |
| ci-cd-and-automation | BUILD | **pin** CI/release (contrato toca política CI §11/Q5) | 1.0 |
| git-workflow-and-versioning | BUILD | **pin** CI/release (close = commit LEAD) | 1.0 |
| performance-optimization | BUILD | **pin** performance (harness = benchmarking) | 1.0 |
| doubt-driven-development | BUILD | base type + verificación adversarial de mediciones | 1.0 |
| incremental-implementation | BUILD | lifecycle BUILD | 1.0 |
| test-driven-development | BUILD | lifecycle BUILD (selftests de ambos módulos) | 1.0 |
| source-driven-development | BUILD | verificación de datasets/licencias contra fuentes oficiales (URLs en §Investigation) | (judgment) |
| documentation-skill | BUILD | **obligatoria** (BENCHMARKS.md + READMEs + task file) | (judgment) |

**SKILLS_CARGADAS:** campaign-executor, progreso (auto vía MCP) · ci-cd-and-automation, git-workflow-and-versioning, performance-optimization, doubt-driven-development, incremental-implementation, test-driven-development, source-driven-development, documentation-skill.

## Investigation Notes

### Licencias (verificadas 2026-09-29, fuentes oficiales)

| Dataset | Licencia | Fuente | Veredicto commit |
|---|---|---|---|
| LongMemEval-cleaned (`xiaowu0162/longmemeval-cleaned`) | **MIT** | HF API: `tags:["license:mit"]`, `cardData.license="mit"` · https://huggingface.co/api/datasets/xiaowu0162/longmemeval-cleaned | ✅ commiteable |
| LoCoMo (`snap-research/locomo`) | **CC BY-NC 4.0** | `LICENSE.txt` = "Attribution-NonCommercial 4.0 International" (GitHub `spdx: NOASSERTION`) · https://api.github.com/repos/snap-research/locomo/license | ❌ NonCommercial → no commit |
| BEAM (`mohammadtavakoli78/BEAM`) | **MIT** | GitHub API `license.spdx_id="MIT"` · https://api.github.com/repos/mohammadtavakoli78/BEAM/license | ⚠️ licencia OK, tamaño no (4.6MB/conv) → runner opcional |

### LongMemEval — formato y protocolo (fuentes)

- Campos: `question_id` (sufijo `_abs` = abstención; 30/500), `question_type`, `question`, `answer`, `question_date`, `haystack_session_ids/dates/sessions` (sesiones de turnos con `has_answer: true` en evidencia), `answer_session_ids` (ground truth session-level). Fuente: README oficial https://github.com/xiaowu0162/LongMemEval (fetch 2026-09-29).
- Protocolo upstream: retrieval eval **salta las 30 `_abs`** (sin ground truth de location); juez QA = GPT-4o (`evaluate_qa.py`); granularidad turn/session.
- S = ~115K tokens/~40 sesiones por pregunta; archivo 277,383,467 B, sha256 `d6f21ea9…` (HF LFS oid).
- Dataset real: 500 preguntas; en la práctica 38–62 sesiones, ~500KB JSON por pregunta (medido sobre el archivo).

### API de bindings (probada localmente)

- `Client.put(ns, key, payload:str)`; `Client.search(ns, query_vector, text_query=…, top_k=…)` acepta `[]` como vector para búsqueda textual; `Client.memory.get(ns,key)` → Record con `payload` (para write-quality).
- `SearchHit` expone `key/score/confidence/confidence_class/namespace/payload/…` (`confidence` = campo MGR-12, default `D_a=1.0`).
- `put` del binding 0.7.0 **no acepta `confidence`** (kwargs faltante) → los registros del harness quedan en D_a; ECE mide la política declarada (Q1), con el límite del clamp documentado. Candidata a fila FIND (SCH-04: declarar confianza desde Python).

### Q5 (gate p99) — estado a documentar

- Workflow `bench-canonical-p99-informational.yml` ya: evalúa regresión (criterio >10%), **abre issue** con labels `benchmark, regression` y NO bloquea (header :1-22 + steps 176-228). → la forma "informativo-que-abre-issue documentado" del contrato ESTÁ viva.
- §11 de BENCHMARKS.md citaba como baseline "p99 57 ms @10k" (serie Stress Protocol) — ambiguo frente al bench canónico (100k×1536d → p99 4.9987 ms). Se corrige el puntero (mi edición, §11) sin cambiar enforcement.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 (Q5 es decisión owner, no incógnita técnica; disposición admitida por contrato) |
| Pendientes de ejecución (downhill) | 0 al cierre |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** no aplica en producto (no toca trust boundaries ni deps). Sí aplica higiene de datos: (a) solo datos redistribuibles commiteados (MIT), (b) descarga con **sha256 verificado** contra pin, (c) `--self-test` offline para CI sin red. Sin secretos, sin red en CI.
- **PERFORMANCE:** aplica como *medición*, no como optimización: el harness ES el instrumento (Regla 9/11). Baseline citado: `benches/canonical_p99.rs` + `criterion_baseline.json` (p99 4.9987 ms, MGR-19 §1). No se modifica ningún hot path.

## Steps

### Step 1: DISCOVERY (contrato + licencias + formato + API + SDP) ✅
- **Archivos:** research MGR-19/MGR-12, plan Task 45, bench + workflow, evals existentes
- **Acción:** verificar contrato verbatim, resolver licencias con fuentes, probar API real de bindings, definir destino y regla de submuestra
- **Verify:** licencias con URL (tabla §Investigation); API probada (SearchHit fields); SDP v3 ejecutado ✅

### Step 2: Sub-muestra commiteada + script de procedencia ✅
- **Archivos:** `evals/data/fetch_longmemeval_subset.py` (nuevo), `evals/data/longmemeval_s_subset.json` + `.meta.json` (nuevos), `evals/data/README.md` (nuevo)
- **Acción:** descarga cache 277MB con sha256 verificado → selección determinista 4+1 → artefacto 2.70MB sha `b8a994ba…`
- **Verify:** `python evals/data/fetch_longmemeval_subset.py` → `cache sha256 OK` + write ✅

### Step 3: Módulo de calibración ECE ✅
- **Archivos:** `evals/calibration.py` (nuevo)
- **Verify:** `python evals/calibration.py --selftest` → `OK (6 checks)` ✅

### Step 4: Harness real + métricas + self-test ✅
- **Archivos:** `evals/memory_harness.py` (nuevo)
- **Verify:** `python evals/memory_harness.py --self-test` → `OK (8 checks)` ✅

### Step 5: Corrida sub-muestra (vantadb real) ✅
- **Verify:** `python evals/memory_harness.py --label subset-5q` → recall_all@5 1.0000 / recall_any@5 1.0000, WQ 1.0, p50 0.774 ms/p99 0.984 ms (smoke; latencia varía con carga), ECE degenerado (20 pares, todo correcto) ✅

### Step 6: Corrida full-500 local (reporte) ✅
- **Verify:** `python evals/memory_harness.py --data datasets/longmemeval/longmemeval_s_cleaned.json --label s-full-500q` (re-run 2026-09-30 post-review) → **recall_all@5 0.7617 (358/470) · recall_any@5 0.9213 (433/470) · p50 1.459 ms / p99 2.446 ms · ingest 110.0 QPS (23,867 sesiones) · write-quality 1.0000 · isolation 0 violaciones · ECE 0.0787→0.0003 (T\*=5.63)** ✅

### Step 7: Publicación (BENCHMARKS.md §19 + §17 + §11 + READMEs) ✅
- **Archivos:** `docs/user/operations/BENCHMARKS.md` (§11 pointer/Q5, §17 DEFER→entregado, §19 nueva), `evals/README.md`, `evals/data/README.md`, `.gitignore`
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs` ✅

### Step 8: Validación final + RESULTADO ✅
- **Verify (todas mecánicas):**
  - `python evals/calibration.py --selftest` → `OK (6 checks)` ✅
  - `python evals/memory_harness.py --self-test` → `OK (13 checks)` ✅ (incluye: separación any=1.0/all=0.5 en fixture multi-evidencia; percentile nearest-rank; `search.latencies_ms` presente)
  - `python -m py_compile` ×3 → exit 0 ✅
  - `python evals/data/fetch_longmemeval_subset.py --check` → sha256 OK ✅
  - `python evals/data/fetch_longmemeval_subset.py --check --cache .tmp_no_such_cache.json` → exit≠0 sin descarga (fix F5) ✅
  - re-run full-500 (fix F1) → all 0.7617 / any 0.9213 ✅
  - `node scripts/docs/check-docs.mjs` → `GATING (fail -> 1): all clear` ✅
  - `node scripts/docs/check-links.mjs` → exit 0 (0 links nuevos rotos; se reparó 1 gating pre-existente de BENCHMARKS §3) ✅
  - `node scripts/docs/gen-index.mjs --check` → exit 0 (índices regenerados) ✅
  - `pwsh scripts/validate-docs-coverage.ps1` → `0 gaps` ✅
  - `npx markdownlint-cli2` sobre los 4 .md tocados → 0 issues ✅
  - OCR delegation (advisory, sin API key) → reglas revisadas; limpieza aplicada (asserts → ValueError en API pública, walrus muerto) ✅

## Context Save Point (para LEAD — changeset del commit)

**Archivos nuevos:**

- `evals/memory_harness.py` · `evals/calibration.py` · `evals/README.md`
- `evals/data/fetch_longmemeval_subset.py` · `evals/data/longmemeval_s_subset.json` (+ `.meta.json`) · `evals/data/README.md`
- `docs/dev/tasks/VER-08.md` (este archivo)

**Archivos editados:**

- `docs/user/operations/BENCHMARKS.md` (§11 pointer + Q5, §17 DEFER→entregado, §19 nueva, fix de link machine-local en §3)
- `.gitignore` (patrón `evals/memory_harness_report*.json`)

**Generados (regeneración obligatoria al agregar docs):**

- `docs/index.md` · `docs/user/index.md` · `llms.txt` — regenerados con `gen-index.mjs --write`; incluyen también archivos del co-batch F5.1 (p.ej. `docs/user/PRIVACY.md` de ICP-02). Si el co-batch agrega más docs, re-ejecutar `--write` en el cierre.

**Cache local (gitignored, NO commit):** `datasets/longmemeval/longmemeval_s_cleaned.json` (277 MB).

**Commit sugerido (LEAD):** `feat(VER-08): memory eval harness + LongMemEval-S subset + ECE calibration`

## Verificación por cláusula (contrato verbatim → evidencia)

**Resultados (re-run 2026-09-30 post-review F1, bindings vantadb 0.7.0, memory backend):**

| Corrida | n | recall_all@5 | recall_any@5 | p50/p99 (ms) | ingest QPS | write-quality | isolation | ECE | tokens p50 |
|---|---|---|---|---|---|---|---|---|---|
| `s-full-500q` (split completo) | 470+30 | **0.7617** (358/470) | 0.9213 (433/470) | 1.459 / 2.446 | 110.0 | 1.0000 | 0/2500 | 0.0787→0.0003 (T\*=5.63) | 10,483 |
| `subset-5q` (commiteada) | 4+1 | 1.0000 | 1.0000 | 0.774 / 0.984 † | 245.3 | 1.0000 | 0/30 | degenerado (20 pares, todo correcto) | — |

† smoke sobre máquina cargada: la latencia varía entre corridas; la referencia es la fila full (300/470 preguntas son multi-evidencia → `any` sobreestima; `all` es el headline).

Abstención proxy (full): 3/30 `_abs` bajo τ=5.67 (0.10) · false-alarm 14/470 (2.98%).
Reportes JSON regenerables: `evals/memory_harness_report*.json` (gitignored).

| # | Cláusula | Evidencia | Estado |
|---|----------|-----------|--------|
| 1 | canonical_p99 (ya) | `benches/canonical_p99.rs:1-15` intacto; baseline citado en §19 | ✅ |
| 2 | LongMemEval-S submuestra versionada (MIT verificada) | `evals/data/longmemeval_s_subset.json` (2.70MB, sha en `.meta.json`) + licencia MIT (HF API) | ✅ |
| 3 | LoCoMo / BEAM con licencia verificada o disposición documentada | LoCoMo CC BY-NC → disposición §Spec/README; BEAM MIT verificada → disposición por tamaño + runner opcional | ✅ |
| 4 | Corrida backend vantadb real reportando recall@k + p50/p99 + ingest QPS + write-quality + abstención + token-economy + aislamiento per-user | reporte JSON (gitignored, regenerable) + tabla §19 con todos los campos; recall en dos formas `recall_all@5` (headline) / `recall_any@5` (secundaria) — fix review F1 | ✅ |
| 5 | Reporte en BENCHMARKS.md con juez/protocolo pineados y pares accuracy+tokens | §19 (nueva) con protocolo + pares en el JSON | ✅ |
| 6 | Calibración MGR-12 §5: ECE + temperature scaling (antes/después) | `evals/calibration.py` + campo `calibration` del reporte (§19 resume) | ✅ |
| 7 | Gate p99 en CI: política §11 decidida con owner (enforced o informativo-que-abre-issue) | §11 actualizado: estado informativo+issue documentado; Q5 abierta = upgrade pendiente owner | ✅ (forma informativo) |

## Diseño para review P2-01 (vanta-audit) — juez y protocolo

> Punto focal del review externo (mandato del bloque F5.1). Decisiones a auditar:

1. **Juez determinista (sin LLM):** dos formas session-level, publicadas juntas:
   `recall_all@k` (headline) = `answer_session_ids ⊆ top-k` (todas las sesiones de evidencia recuperadas — la forma que reporta el retrieval eval de LongMemEval, comparable upstream) y `recall_any@k` (secundaria) = `answer_session_ids ∩ top-k ≠ ∅` (hit-rate any-evidence).
   - *Por qué `all` es headline:* ~300/470 preguntas de LongMemEval-S tienen >1 sesión de evidencia; `any` infla (encontrar 1 de 3 cuenta igual que las 3). `all` es la lectura conservadora y la comparable con `recall_all@k`/`ndcg_any@k` upstream.
   - *Riesgo auditado:* presentar `any` como el número principal — corregido en §19 (`all` headline + `any` etiquetada "hit-rate any-evidence").
   - *Calibración:* los pares usan `correct` = forma `any` (question-level) — declarado en §19; con `confidence` constante (D_a) el ECE no depende de esa elección en esta corrida.
2. **Abstención:** proxy por umbral τ = p5 de los scores de evidencia (positivos), aplicado a top-1 de `_abs`.
   - *Límite:* sin reader no hay "respuesta correcta"; τ es operativo y documentado. La medición QA-level (`_abs`) queda para el reporte manual con juez LLM pineado (VER-09).
3. **ECE (§5.2):** pares `(confidence del hit, correctitud de la pregunta)`; T* = argmin NLL con clamp `[1e-6, 1-1e-6]`.
   - *Límite:* todos los registros en `D_a=1.0` → pre-T bin único en el clamp; la corrección por temperatura mueve el clamp y muestra la sobre-confianza; interpretación: revisa Q1 (`D_a` política) — no es una probabilidad aplicable en runtime (v1.0).
   - *Consumidor alternativo:* pares declarables si SCH-04 expone `confidence` en el binding Python (hoy no; FIND propuesta).
4. **write-quality = get exacto byte-igual:** mide fidelidad de escritura (no "calidad" semántica). Nombre honesto: Write Fidelity. Se mantiene el nombre de contrato (write-quality) con definición explícita.
5. **Aislamiento per-user = namespace por pregunta:** el harness ingiere cada pregunta en su namespace y verifica que ningún hit salga del namespace/`key set`. Es una verificación de *containment* (0 fugas) — no un test de multi-usuario concurrente.

## Findings / disposiciones (para LEAD — Backlog prohibido en este agente)

| ID propuesto | Descripción | Dueño sugerido |
|---|---|---|
| FIND-* (a crear) | LoCoMo (CC BY-NC 4.0) sin runner: implementar loader loco-only cuando el uso interno esté aprobado (licencia no permite redistribuir; ver `evals/data/README.md`) | vanta-tuner |
| FIND-* (a crear) | BEAM-1M sin runner committeado: wire de subconjunto local (1 conv) + costo medido antes de escalar; nunca 10M en CI | vanta-tuner / VER-09 |
| FIND-* (a crear) | Python SDK 0.7.0 no expone `confidence` en `put` (declarar confianza imposible desde el binding) → bloquea calibración con pares declarados | vanta-worker (SCH-04 follow-up) |
| FIND-* (a crear) | `BENCHMARKS.md` §11 citaba baseline ambiguo ("§1: p99 57 ms @10k") → corregido a `criterion_baseline.json`+§1 canónico en este task; si aparece en otros docs, rastrillar | vanta-docs / DEF-06 |

## Dependencias

- **Consumidores:** VER-09 (Task 47, F6 — head-to-head Mem0/Zep/Letta con este harness); DEF-06 (Task 46 — regen §2 con protocolo/hardware normalizado).
- **Prerrequisitos:** MGR-19 ✅, MGR-12 §5 ✅, bindings `vantadb` 0.7.0 ✅ (importable local), red para el fetch del dataset (solo regeneración; el artefacto va commiteado).
- **nextTask:** DEF-06.

## Review (GATE — agente distinto, P2-01)

- **Ronda 1 (vanta-audit, 2026-09-30):** ❌ **CHANGES REQUIRED** — 1 High metodológico (F1: la métrica publicada como `recall@5` era `recall_any`; upstream LongMemEval reporta `recall_all@k`) + 5 menores (F2–F6, ver §Fixes).
- **Ronda 2 (fixes aplicados 2026-09-30):** ver §Fixes review round 2. Pendiente: re-review LEAD (vanta-audit). Este sub-agente no se auto-revisa (mandato del bloque).

## Fixes review round 2 (2026-09-30)

| # | Fix | Evidencia (comando → resultado) |
|---|-----|-------------------------------|
| **F1** (High) | `recall_at_k` ahora expone `{all, any, all_by_type, any_by_type}`; `per_question` con `correct_any`/`correct_all`; `_markdown` con ambas columnas; §19 headline `recall_all@5` + `recall_any@5` secundaria; calibración declara `correct` = forma any | `python evals/memory_harness.py --self-test` → OK (13 checks, incluye separación any=1.0/all=0.5 en fixture multi-evidencia) · re-run full-500 → **all 0.7617 / any 0.9213** (ver §Resultados) |
| **F2** (Low) | `percentile()` nearest-rank: `idx = max(0, min(math.ceil(p*n)-1, n-1))` | selftest: `percentile([1,2,3,4],0.5) == 2` ✅ (el viejo daba 3) |
| **F3** (Info) | `TOKEN_ECONOMY_CUT` → `ABSTENTION_TAU_PCTL` | `rg TOKEN_ECONOMY_CUT evals/` → 0 matches ✅ |
| **F4** (Info) | `search.latencies_ms` crudas en el reporte JSON (p50/p99 recomputables) | re-run full-500 → 500 valores en `search.latencies_ms` ✅ |
| **F5** (Info) | `--check` sin cache → `SystemExit` sin descargar 277 MB | `python evals/data/fetch_longmemeval_subset.py --check --cache .tmp_no_such_cache.json` → exit 1 + "cache missing … run without --check to download it first" ✅ |
| **F6** (Info) | meta note + README: "object-identical (deep-equal, no field edits)" | `evals/data/longmemeval_s_subset.json.meta.json` + `evals/data/README.md` actualizados; subset sha256 sin cambio (`b8a994ba…`) ✅ |

## Notas

- Wave F5.1 co-batch: no se tocan `vanta-proxy/**` (ICP-02, solo lectura) ni docs de tracks (ICP-01). `docs/dev/Backlog.md` y plan file: prohibidos para este agente.
- Regla 11: los reportes JSON van gitignoreados (`evals/memory_harness_report*.json`); la fuente citable es la tabla §19 + comando + sha256 del dataset.
- `datasets/longmemeval/longmemeval_s_cleaned.json` (277MB) queda como cache local gitignored para corridas grandes.
