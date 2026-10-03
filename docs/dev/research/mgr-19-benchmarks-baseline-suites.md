---
title: "MGR-19 — Benchmarks propios y externos: baseline, reconciliación y suites (Cierre MGR)"
kind: research
description: "Causas: (1) README citaba un artefacto gitignored no versionado cuyos números ni siquiera coinciden con el artefacto actual (2.0 vs 2.78, 3.1 vs 5.70 — corrida de otra máquina/fecha nunca commiteada); (2) §2 es serie CI congelada..."
---

# MGR-19 — Benchmarks propios y externos: baseline, reconciliación y suites (Cierre MGR)

- **Fecha:** 2026-09-25 · **Tipo:** validación/research (cero implementación productiva)
- **Contrato:** "BENCHMARKS.md con baseline canónico + §2 reconciliada (0 claims sin comando reproducible); research-doc + Cierre MGR."
- **Plan:** programa P49 Track G · **Destraba:** EXE-01/02/03/05, MGR-20, VER-08 · **Next:** EXE-01

## §1. Baseline canónico `canonical_p99` + entorno (Regla 9)

- **Contrato del bench:** `benches/canonical_p99.rs` — insert 100k vectores × 1536d + search 1000 queries (p50/p95/p99), dataset determinístico seed 42, `CPIndex` in-memory puro (sin I/O storage), HNSW fijo (m=16, m_max0=32, ef_construction=100, ef_search=50, cosine).
- **Compile-guard (GOV-B3):** `cargo bench -p vantadb --bench canonical_p99 --no-run` ✅ 2026-09-25 (6m46s; 5 warnings pre-existentes en `src/sdk/search/debug_ops.rs`, no tocados — fuera de scope).
- **Timed run:** ✅ 2026-09-25 (`cargo bench -p vantadb --bench canonical_p99`, exe `canonical_p99-d623acf7db8342fa.exe`):
  - `insert_100k_1536d`: **644.23 s** por build completo 100k×1536d (criterion mean; rango [573.12, 709.29] s → ~155 vec/s @1536d, single-thread, seed 42).
  - `canonical_p99 search (1000 queries, 1536d, top_k=10)`: **p50=2.2388 ms, p95=4.248 ms, p99=4.9987 ms**.
  - `search_1000_queries_1536d` (batch 1000q): **2.8136 s** (~2.8 ms/query mean, coherente con p50).
  - Nota: insert con `sample_size(10)` × ~640 s ≈ ~1.9 h de wall-clock solo grupo insert (el wall total del bench es mayor: build previo de búsqueda + iters search); warnings de criterion ("increase target time to 6820.3s") esperados por diseño del bench, no un fallo.
- **Entorno de esta máquina (Regla 11):** Windows 11, i5-1235U 12 hilos, RAM 31.78 GiB, perfil `bench` (release + debuginfo), HEAD `b05f9d7f`. Los absolutos son locales; el valor versionable es el comando + la metodología, no el número.
- **Lectura anticipada:** este bench es scope Rust puro — no comparable con §2 (Python+PyO3+GIL) ni con `search_batch` (amortiza FFI). Ver §2.

## §2. Reconciliación README ↔ §1 ↔ §2 (triple divergencia verificada)

| Fuente | Vector search p50 | Hybrid p50 | Scope |
|---|---|---|---|
| README:356-357 (antes) | 2.0 ms | 3.1 ms | citaba artefacto gitignored |
| `benchmarks/vanta_benchmark_report.json` (local, 2026-08-25) | **2.78 ms** | **5.70 ms** | misma máquina, otra corrida |
| BENCHMARKS.md §2 (CI 2026-08-12) | **61.996 ms** | **179.810 ms** | Python+PyO3+GIL, pre-SIMD |
| BENCHMARKS.md §1 (Stress Protocol) | **1.2 ms** (@10K/128d) | n/a | Rust core, sin FFI |

Causas: (1) README citaba un artefacto gitignored no versionado cuyos números ni siquiera coinciden con el artefacto actual (2.0 vs 2.78, 3.1 vs 5.70 — corrida de otra máquina/fecha nunca commiteada); (2) §2 es serie CI congelada pre-SIMD con costo PyO3/GIL + single-thread; (3) §1 es Rust puro. Tres scopes, tres números — Regla 11 los declara no comparables.

**Cambios aplicados (docs-only):**
- `BENCHMARKS.md` §2: banner `FROZEN pre-SIMD (2026-08-12, regen pending DEF-06)` con scope explícito (Python SDK vía PyO3+GIL, single-thread, CI) + prohibición de restar/dividir entre scopes. Números intactos.
- `README.md:351-359`: tabla ahora cita §1 Rust canónico (p50 1.2 ms, scaling 4.88x, recall@10) con scope etiquetado; 0 claims SDK-scope hasta DEF-06; el comando de regeneración local queda documentado con la advertencia de que un run local no es claim citable.
- `validate-docs-coverage.ps1` ✅ 0 gaps (2026-09-25).

## §3. Suites externas: protocolo + subset mínimo viable (Step 3)

Digest vanta-research `ses_f265e1025ffeVk7IqdexX2sW1M` (≤500 palabras, 11 URLs verificadas vía webfetch). Resumen:

- **LongMemEval** (arXiv 2410.10813, ICLR 2025): 500 Q, splits **-S ~115K tokens/~40 sesiones** / -M / oracle; dataset MIT (`huggingface.co/datasets/xiaowu0162/longmemeval-cleaned`, 3.03 GB); juez **GPT-4o** (`evaluate_qa.py`); retrieval con recall por turno/sesión (30 `_abs` excluidas). Subset barato = **LongMemEval-S** (cabe en 128K).
- **LoCoMo** (arXiv 2402.17753): ~50 convs × ~300 turnos / ~9K tokens / 35 sesiones; `github.com/snap-research/LoCoMo`; jueces F1/FactScore. Licencia del dataset **[cita NO VERIFICADA]** — no commitear sin chequear.
- **BEAM** (arXiv 2510.27246, ICLR 2026): 100 convs 100K→10M tokens, 2.000 Q; tracks **BEAM-1M / BEAM-10M**; repo/licencia **[cita NO VERIFICADA]**. Costo 1M/10M → **subset propio obligatorio** (track 1M, pocas convs).
- **Disputa vendors:** números Mem0/Zep/ByteRover/Dakera no comparables (distinto juez/answering-model/rerank). La "auditoría 6.4%/63%" solo atestiguada en docs internos **[cita NO VERIFICADA — fuente primaria no localizada]**.
- **Repo hoy:** `evals/memory_bench.py` (MEM-70) = harness sintético seed 42 (`synthetic-longmem-style`, markers `M{s}-{t}`), recall@k + p50/p99 + ingest QPS, backend vantadb o fallback dict (techo recall 1.0 = gate del harness, no claim). §17 fija DEFER (loader versionado, corrida vantadb real, tabla propia); §Planificado define VER-08/VER-09.

**Subset mínimo viable (recomendación):** LongMemEval-S commiteado (MIT OK) + 1–2 convs LoCoMo como smoke (tras verificar licencia) + BEAM-subset propio track 1M. Protocolo obligatorio por corrida: backend vantadb real, **juez pineado (modelo+versión+prompt)**, reporte en pares **accuracy+tokens/query + p50/p99 + abstención**, hardware fijado, gate p99 en CI (VER-08 → VER-09).

**Comandos de reproducción (Step 3 verify):**
```bash
python evals/memory_bench.py --sessions 20 --turns 16 --queries 40 --top-k 5 --output evals/memory_bench_report.json
python evals/memory_bench.py --sessions 4 --turns 8 --queries 8 --no-vantadb   # smoke offline/CI
cargo bench -p vantadb --bench canonical_p99                                    # baseline propio Rust
python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000  # serie SDK (regen DEF-06)
```
Dataset commiteado LongMemEval-S/LoCoMo/BEAM-subset = **DEFER a VER-08** (3.03 GB + licencias LoCoMo/BEAM sin verificar; loader versionado con submuestra documentada). Ningún claim vs SuperMemory/Hindsight/Mem0/Zep hasta tabla propia reproducible (Regla 11).

## §4. Preguntas owner (Cierre MGR — requieren decisión, no bloquean EXE-01)

1. **Juez:** ¿GPT-4o pineado (costo por corrida) o juez local/determinista (F1/exact-match) para el gate CI? El juez LLM es la principal fuente de disputa entre vendors.
2. **BEAM-subset:** ¿track 1M con cuántas conversaciones fijas? (costo por corrida en CI).
3. **LoCoMo:** ¿se acepta commitear 1–2 convs como smoke antes de verificar su licencia, o solo tras verificación?
4. **DEF-06 (§2 regen):** ¿prioridad vs EXE-01? Sin regen, los SDK-claims públicos siguen congelados en 2026-08-12.
5. **Gate p99 CI:** ¿umbral de regresión (>10% como §11, u otro) para el harness de memoria VER-08?

## §5. Plan de implementación (para VER-08 / EXE-02 / DEF-06)

1. **DEF-06 (regen §2):** correr `benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000` con bindings release + hardware documentado; reemplazar tabla §2 + retirar banner FROZEN; README puede volver a citar SDK-scope.
2. **VER-08 (harness memoria):** loader versionado LongMemEval-S (+LoCoMo smoke si licencia OK) con submuestra documentada; corrida backend vantadb real (bindings compilados); métricas recall@k + p50/p99 + ingest QPS + **write-quality, abstención, aislamiento per-user, token-economy**; gate p99 en CI.
3. **BEAM-subset:** definir N convs track 1M + costo medido antes de escalar; nunca 10M en CI.
4. **VER-09 (head-to-head):** Mem0/Zep/Letta con MISMO harness + protocolo publicado completo (modelo-juez, stack, reranking; pares accuracy+tokens).
5. **Baseline canónico:** transcribir timed run §1 + registrar en BENCHMARKS §11 como punto de comparación Regla 9.

## Deuda y notas

- Deuda nueva: ninguna (research + docs-only; saldo neto Regla 6 = 0).
- NOTICED BUT NOT TOUCHING: warnings pre-existentes `src/sdk/search/debug_ops.rs` (5× unused imports, también citados en §13) → candidato a fila FIND (requiere decisión: ¿dead code o `cfg`?); WIP ajeno API-01 intacto; `src/` no tocado.
- Skills (SDP): writing-plans + writing-guidelines + test-driven-development (harness = tests: compile-guard + validate-docs-coverage como harness) + context-engineering.
