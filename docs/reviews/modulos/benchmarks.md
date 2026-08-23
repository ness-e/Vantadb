# Review de módulo: `benchmarks/`

> **Fecha:** 2026-08-23 · **Revisor:** ox-alpha (performance & observability)
> **Alcance:** directorio `benchmarks/` — scripts Python/Node/Rust, datasets, resultados, reportes.
> **Skills:** performance-optimization, code-review-and-quality, ponytail-audit.

## Score general: **6.5 / 10**

| Dimensión | Nota | Comentario |
|---|---|---|
| Honestidad de claims | 9 | PERF-03/MKT-18g: tabla honesta con derrotas publicadas; disclosure del "hidden rebuild"; Milvus/Qdrant medidos |
| Metodología | 8 | Warm-up, median-of-3, ground truth exacto, normalización cosine, mismo proceso para todos |
| Reproducibilidad | 5.5 | Seeds faltantes en 3 scripts; dependencia de red (~1 GB); baselines de regresión vacíos |
| Higiene del repositorio | 3 | **128 MB de artefactos binarios commiteados** en `data_comp_bench/` |
| Cobertura funcional | 7 | SDK Python / WASM / competitivo / batch / prefetch cubiertos; graph bloqueado |

---

## Inventario y veredictos

### Scripts

| Archivo | Qué es | Veredicto |
|---|---|---|
| `vantadb_local_bench.py` (BENCH-01) | Bench local vía SDK Python PyPI: ingesta PUT + rebuild + BM25/HNSW/híbrido con p50/p95/p99; exporta JSON | ✅ **Bueno.** Zero-dep, progreso con ETA, valida `rebuild_report`. ⚠️ `random.uniform` **sin seed** → dataset distinto por run; reportes no comparables run-a-run. Percentiles `int(n*0.95)` no nearest-rank (tolerable). |
| `competitive_bench.py` (PERF-03) | VantaDB vs LanceDB vs ChromaDB vs Qdrant vs Milvus (embedded, sin docker): ingest, QPS, p50/p99, recall@10, RSS | ✅ **El mejor asset del directorio.** Header documenta la metodología Y sus trampas (hidden rebuild por InsertMode::Auto dentro del timer de ingesta; baseline pre-regresión no comparable). Warm-up (D3), median-of-3 (D4), seed fija (D2), ground truth exacto recalculado en subsets, normalización cosine correcta. Motores sin cliente se marcan "not measured" en vez de inventar números. Sostiene la tabla honesta de `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` — VantaDB pierde en recall (59% vs Qdrant 100%/Chroma 98%) y se publica tal cual. Reproducible dados deps + red. |
| `batch_vs_sequential_bench.py` | `search_batch()` vs secuencial (amortización FFI + Rayon) + gate INV-008-B (batch de 10 < 3× single) con exit code CI | ✅ **Bueno.** Valida paridad de resultados con asserts antes de cronometrar. Gate CI real. ⚠️ Sin seed tampoco. |
| `prefetch_comparison.py` (SCALE-01c) | A/B de latencia con/sin prefetch predictivo; actualiza `docs/operations/BENCHMARKS.md` | ✅ OK. Mismo patrón que los anteriores. Sin seed. |
| `wasm_bench.mjs` (MCP-03) | Bench WASM multi-motor (ingesta, latencia, recall vs ground truth JS) | ⚠️ Funcional, pero su output (`wasm_benchmark_results.json`) **no está commiteado** → el reporte MD no es trazable a datos crudos. |
| `graphrag_bench.rs` (MKT-16) | GraphRAG: index time, latencia por hop, Token Reduction vs RAG plano | ⚠️ **Registrado como `[[example]]`, NO como bench** (`Cargo.toml` ~L570). Script completo y honesto (token counting = proxy whitespace documentado), pero fase de query **bloqueada por stack overflow reproducible** (STATUS_STACK_BUFFER_OVERRUN); métricas query/TR PENDING. Index timing sí es real. |
| `update_markdown.py` | Auto-actualiza tabla de métricas en `docs/operations/BENCHMARKS.md` | ✅ Utilitario simple. ⚠️ Ruta relativa al CWD — solo funciona desde la raíz o vía fallback. |

### Datos y resultados

| Elemento | Estado | Veredicto |
|---|---|---|
| `vanta_benchmark_report.json` | Resultado BENCH-01 real (10k): ingesta 74 rec/s, hybrid p50 3.1 ms | ⚠️ Número real expuesto tal cual (honesto), pero el JSON **sin metadatos** (fecha/hardware/versión) → imposible datarlo ni compararlo después. |
| `criterion_baseline.json` | Baseline del gate nocturno `heavy-bench-nightly-51` | 🔴 **VACÍO** (`"benchmarks": {}`). El workflow compara contra nada → gate criterion hoy es **no-op**. El metadata dice "update after first CI run" y nunca ocurrió. |
| `python_baseline.json` | Baseline del gate `perf-bench-40` (>15% falla) | 🔴 **VACÍO** (bootstrap documentado). Dos gates declarados, ambos inoperantes hasta un `workflow_dispatch` manual no ejecutado. |
| `data_comp_bench/` | DB VantaDB residual de una corrida competitiva | 🔴 **Commiteada en git**: 42 archivos, **128 MB** (`vector_store.vanta` y `0.jnl` de 64 MB c/u, `.vanta.lock`). Su hermana `data_bench_db/` sí está ignorada — inconsistencia de `.gitignore`. Los blobs persisten en el historial para siempre. |
| `__pycache__/` | Cache Python | Ignorada correctamente. ✅ |

### Reportes publicados

| Archivo | Veredicto |
|---|---|
| `WASM_BENCHMARK_REPORT.md` | ⚠️ Matriz de features y bundle sizes útiles, pero **sin números crudos vinculados** (el JSON del bench no está versionado). Claims ("SIMD128 8.75x", "fastest pure vector search") sin trazabilidad desde este repo. Conclusión honesta (reconoce que altor-vec gana en tamaño/velocidad pura). Dato verificable: bundle 1,101 KB raw / ~404 KB gzipped. |
| Ref. cruzada `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` | ✅ Existe y es coherente: contrato de honestidad explícito, Chroma con solo 1 iteración válida anotada (WinError 32), Milvus medido. Ejemplar post-MKT-18g. |

---

## Evaluación

### ¿Son reproducibles?

- **competitive_bench.py: sí** (dados deps + descarga ann-benchmarks). Seed fija, metodología documentada, median-of-3.
- **vantadb_local_bench / batch_vs_sequential / prefetch_comparison: parcialmente.** Sin seed → vectores distintos por run. Latencias representativas, pero cualquier claim "X% mejor que ayer" sobre estos scripts no es defendible.
- **wasm_bench.mjs: sí**, pero sin resultados commiteados no hay evidencia.
- **graphrag_bench.rs: sí** (LCG determinista, tempdir fresco) — bloqueado por el bug del engine.

### ¿Hay claims asociados y son honestos?

1. **✅ PERF-03 / MKT-18g:** la cultura de honestidad es verificable en código + docs. Puntos débiles expuestos (recall bajo, hidden rebuild, ingesta lenta 74 rec/s en BENCH-01).
2. **⚠️ WASM_BENCHMARK_REPORT.md:** claims comparativos sin datos crudos versionados. Matriz probablemente correcta pero no auditable desde este repo.
3. **⚠️ GraphRAG:** el blog documenta celdas PENDING — correcto, pero **ningún claim de Token Reduction está respaldado por medición completa** hasta arreglar el stack overflow.

### ¿Falta algo crítico?

1. **Gates de regresión muertos:** ambos baselines vacíos. El trabajo pesado (harness, workflows, scripts) existe; falta una sola corrida bootstrap + commit del baseline para activarlos.
2. **Metadatos en resultados:** los JSON de resultados no llevan fecha/commit/hardware → cualquier número publicado es indatable a futuro.
3. **Seeds en los benches Python locales:** una línea (`random.seed(42)` / equivalente numpy ya presente solo en competitive).
4. **Latency percentiles / RSS:** ✅ bien cubiertos aquí (p50/p95/p99 + RSS por script). Sin gap.
5. **Memory profiling fino:** no hay dhat/heap-profile en este dir ni en `benches/` — gap compartido, no bloqueante.

---

## Ponytail-audit (higiene)

- `delete:` 128 MB de artefactos DB commiteados en `benchmarks/data_comp_bench/`. Reemplazo: `git rm -r --cached benchmarks/data_comp_bench` + entrada en `.gitignore` junto a `data_bench_db`. [benchmarks/data_comp_bench/]
- `shrink:` seeds faltantes → 1 línea por script. [vantadb_local_bench.py, batch_vs_sequential_bench.py, prefetch_comparison.py]
- `yagni:` `graphrag_bench.rs` registrado como example dentro de un directorio de benchmarks — mover a `examples/` o registrar como bench real cuando el stack overflow esté arreglado.
- `stdlib:` `update_markdown.py` reinventa extracción de tabla con marcadores; suficiente como está — no tocar.

Net posible: −128 MB repo, ~−3 líneas.

---

## Acciones recomendadas (priorizadas)

1. **[Crítico]** Activar gates: correr `heavy-bench-nightly-51` + `perf-bench-40` con `update_baseline=true` y commitear ambos baselines.
2. **[Crítico]** `git rm -r --cached benchmarks/data_comp_bench` + `.gitignore`.
3. **[Alto]** Añadir metadatos (fecha/commit/HW) a `vanta_benchmark_report.json` y al reporte WASM; commitear el output JSON del wasm bench junto al MD.
4. **[Alto]** Seed determinista en los 3 scripts Python sin seed.
5. **[Medio]** Arreglar el stack overflow de GraphRAG (AUDIT-04) y re-correr `graphrag_bench` para llenar las celdas PENDING.

