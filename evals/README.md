# evals/ — harness de evals de VantaDB

| Archivo | Qué es | Estado |
|---|---|---|
| `memory_bench.py` | Harness sintético estilo LongMemEval/LoCoMo (MEM-70), seed 42 | ✅ (BENCHMARKS.md §17) |
| `memory_harness.py` | **Harness real (VER-08)**: schema LongMemEval-S contra bindings `vantadb` — recall_all@k + recall_any@k (session-level), p50/p99, ingest QPS, write-quality, abstención, token-economy, aislamiento per-user | ✅ |
| `eval_longmemeval.py` | **Certificación LongMemEval-S (ACT-02 / D2)**: gate bloqueante `recall_all@5 >= 0.85` sobre 500 preguntas, validación `--strict-embeddings`, estudio de ablación (`vector_only`, `bm25_only`, `hybrid_rrf`, `hybrid_graphrag`) | ✅ (BENCHMARKS.md §23) |
| `eval_locomo.py` | **Arnés LoCoMo Benchmark (MEM0-05)**: 4 categorías (`single_hop`, `multi_hop`, `temporal_reasoning`, `open_domain`) contra cifras publicadas de Mem0 (arXiv:2504.19413), búsqueda híbrida + filtro bitemporal + deduplicación FastGate (0 tokens LLM) | ✅ (BENCHMARKS.md §23) |
| `calibration.py` | **ECE + temperature scaling** (spec MGR-12 §5.2): ECE antes/después, T*, reliability data | ✅ |
| `data/` | Sub-muestra LongMemEval-S commiteada (MIT) + procedencia/licencias | ✅ |
| `runners/` | **Head-to-head (VER-09)**: capa de runners Mem0/Zep/Letta sobre el MISMO harness — adapters + driver + protocolo/skips (ver `runners/README.md`) | ✅ |
| `agent/` | Eval de agentes propios (FIND-151, judge determinista) | ✅ |

## Comandos (Regla 11)

```bash
# self-tests (offline, sin red, sin vantadb)
python evals/memory_harness.py --self-test
python evals/calibration.py --selftest
python evals/eval_locomo.py --self-test
python evals/eval_longmemeval.py --self-test

# smoke sobre la sub-muestra commiteada (5 preguntas)
python evals/memory_harness.py --label subset-5q

# corrida completa local (descarga 277 MB una vez; datasets/ está gitignoreado)
python evals/data/fetch_longmemeval_subset.py            # verifica sha256 del cache
python evals/memory_harness.py --data datasets/longmemeval/longmemeval_s_cleaned.json --label s-full-500q

# Certificación de Gate LongMemEval-S (recall_all@5 >= 0.85 sobre 500 preguntas — ACT-02 / D2)
python evals/eval_longmemeval.py --questions 500 --top-k 5 --strict-embeddings --ablation hybrid_graphrag
# Estudio de ablación completo (vector_only, bm25_only, hybrid_rrf, hybrid_graphrag)
python evals/eval_longmemeval.py --ablation all --questions 500 --output evals/results/longmemeval_ablation.json

# LoCoMo Benchmark completo (MEM0-05 — arXiv:2504.19413 vs Mem0)
python evals/eval_locomo.py --smoke --output evals/results/locomo_v091.json
python evals/eval_locomo.py --dataset-path datasets/locomo/locomo_benchmark.json --output evals/results/locomo_v091.json

# head-to-head con competidores (VER-09 — mismo harness, capa de runners)
python evals/runners/head_to_head.py --check
python evals/runners/head_to_head.py --systems vantadb,mem0 --label h2h-5q
```

## Protocolo (pinned)

- **Juez determinista**: corrección = evidencia en top-k (sin LLM, 0 tokens),
  en dos formas publicadas juntas: `recall_all@k` (todas las
  `answer_session_ids` en top-k — comparable con el retrieval eval de
  LongMemEval) y `recall_any@k` (hit-rate: al menos una; secundaria).
  El juez GPT-4o de LongMemEval queda para el reporte manual (owner Q1), nunca
  para un gate.
- **Gate de Certificación LongMemEval-S**: `recall_all@5 >= 0.85` sobre 500
  preguntas del split `longmemeval_s_cleaned.json`. El harness `eval_longmemeval.py`
  retorna `GATE PASS` (exit code 0) o `GATE FAIL` (exit code 1).
- **LoCoMo Benchmark (arXiv:2504.19413)**: evalúa las 4 categorías canónicas
  (`single_hop`, `multi_hop`, `temporal_reasoning`, `open_domain`) calculando
  `recall@5`, `recall@10`, `mrr`, `ndcg@5`, `f1`, latencias p50/p95, QPS, RAM RSS
  pico y tokens LLM (0 en FastGate). Imprime la tabla comparativa directa contra Mem0.
- **F1/BLEU-1 léxicos** (secundarios, estilo LoCoMo/Mem0): `answer_f1.mean_max`
  y `answer_bleu1.mean_max` = mejor solape entre la respuesta gold y cualquier
  payload del top-k; a nivel retrieval con sesiones largas leen cerca de cero
  aun con recall 1.0 (ver nota en el JSON) — no son calidad de respuesta.
- **Costo** (estilo HELM/Mem0): `meta.llm_calls` + `est_cost_usd` (0 en todas
  las celdas actuales, sin LLM) + ingest QPS y latencias ya reportados.
- Proxies declarados: token-economy = palabras; abstención = separación por
  umbral de score en `_abs`; write-quality = fidelidad get exacto. Ver docstring
  de `memory_harness.py` y `docs/user/operations/BENCHMARKS.md` §19 y §23.
- Licencias y procedencia de datasets: `evals/data/README.md`.
- VER-09 (head-to-head Mem0/Zep/Letta) reusa este harness — no duplicar: la capa de runners vive en `runners/`.
