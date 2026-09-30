# evals/ — harness de evals de VantaDB

| Archivo | Qué es | Estado |
|---|---|---|
| `memory_bench.py` | Harness sintético estilo LongMemEval/LoCoMo (MEM-70), seed 42 | ✅ (BENCHMARKS.md §17) |
| `memory_harness.py` | **Harness real (VER-08)**: schema LongMemEval-S contra bindings `vantadb` — recall_all@k + recall_any@k (session-level), p50/p99, ingest QPS, write-quality, abstención, token-economy, aislamiento per-user | ✅ |
| `calibration.py` | **ECE + temperature scaling** (spec MGR-12 §5.2): ECE antes/después, T*, reliability data | ✅ |
| `data/` | Sub-muestra LongMemEval-S commiteada (MIT) + procedencia/licencias | ✅ |
| `agent/` | Eval de agentes propios (FIND-151, judge determinista) | ✅ |

## Comandos (Regla 11)

```bash
# self-tests (offline, sin red, sin vantadb)
python evals/memory_harness.py --self-test
python evals/calibration.py --selftest

# smoke sobre la sub-muestra commiteada (5 preguntas)
python evals/memory_harness.py --label subset-5q

# corrida completa local (descarga 277 MB una vez; datasets/ está gitignoreado)
python evals/data/fetch_longmemeval_subset.py            # verifica sha256 del cache
python evals/memory_harness.py --data datasets/longmemeval/longmemeval_s_cleaned.json --label s-full-500q
```

## Protocolo (pinned)

- **Juez determinista**: corrección = evidencia en top-k (sin LLM, 0 tokens),
  en dos formas publicadas juntas: `recall_all@k` (todas las
  `answer_session_ids` en top-k — comparable con el retrieval eval de
  LongMemEval) y `recall_any@k` (hit-rate: al menos una; secundaria).
  El juez GPT-4o de LongMemEval queda para el reporte manual (owner Q1), nunca
  para un gate.
- Proxies declarados: token-economy = palabras; abstención = separación por
  umbral de score en `_abs`; write-quality = fidelidad get exacto. Ver docstring
  de `memory_harness.py` y `docs/user/operations/BENCHMARKS.md` §19.
- Licencias y procedencia de datasets: `evals/data/README.md`.
- VER-09 (head-to-head Mem0/Zep/Letta) reusa este harness — no duplicar.
