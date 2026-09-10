# MEM-70 — Benchmarks LongMemEval-S + LoCoMo

- **Estado:** ✅ COMPLETED · **Ruta:** vanta-worker · **Wave:** Wave1 (disjunto con PRX-03, DESKTOP-42)
- **Plan:** docs/plans/2026-09-10-code.md (Task 5) · **Appetite:** max 2d
- **Contrato:** harness reproducible + comando documentado + tabla en BENCHMARKS.md + sin claims sin fuente
- **Stop conditions:** datasets inaccesibles → harness + metodología, DEFER números reales
- **SDP:** campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design (discover BUILD 2026-09-10; keywords: harness reproducible, LongMemEval, LoCoMo, benchmark)

## Spec (mini, slice con lógica nueva)

| Decisión | Opción elegida | Evidencia / por qué |
|---|---|---|
| Lenguaje harness | Python `evals/memory_bench.py` | Patrón `benchmarks/embed_bench.py` (argparse + `--no-vantadb` dummy + JSON gitignored + tabla markdown a stdout); `evals/` hoy solo `.mjs` de pipeline, sin harness de memoria |
| Datasets reales | NO descargar en este slice (DEFER) | Licencia/peso (pre-mortem plan); runner offline; stop condition explícita → submuestra sintética documentada |
| Submuestra sintética | Sesiones determinísticas seed 42: `--sessions` × `--turns`, hechos `fact-{s}-{t}`, queries sobre hechos + distractores | Emula shape LongMemEval-S (sesiones largas, QA sobre hechos) y LoCoMo (diálogo multi-turno con referencias) sin sus licencias |
| Métricas | recall@k (hecho gold en top-k por match textual) + latencia ingest/query p50/p99 | Suficiente para metodología Regla 11; sin vectores reales el recall es sobre store textual (techo documentado) |
| Backend | `vantadb_py` si importable, si no fallback in-memory dict (`--no-vantadb` fuerza fallback) | Igual que `embed_bench.py --no-vantadb`; harness corre en CI sin bindings compilados |
| Salida | JSON (`--output`, gitignored por defecto `evals/memory_bench_report.json`) + tabla markdown a stdout | Comando reproducible citable en BENCHMARKS.md (Regla 11) |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `docs/operations/BENCHMARKS.md` (§1-§16, último §16 A/B node; próximo §17 libre), `benchmarks/README.md` (tabla Scripts), `benchmarks/embed_bench.py:1-100` (patrón CLI/dummy/Regla 11), `evals/eval-metrics.mjs` (patrón evals existente, no tocado), `docs/plans/2026-09-10-code.md` (Task 5 + Wave1 + stop conditions)
- **Referencias hacia dentro:** nuevo `evals/memory_bench.py` solo usa stdlib (`argparse/json/random/statistics/time/pathlib`); import opcional `vantadb_py` en try/except; sin imports del repo
- **Referencias entrantes:** ninguna (archivo nuevo); `docs/operations/BENCHMARKS.md` §17 nuevo cita el comando; `benchmarks/README.md` +1 fila (docs, sin código)
- **Veredicto:** impacto mínimo — 1 archivo nuevo + 2 docs append-only; sin hot path, sin API pública, sin deuda P2; Gate D no disparado (downhill, Gate P del plan ya aprobó los 19 DO)

## Steps

- [x] **Step 1 — Slice 1 harness:** crear `evals/memory_bench.py` (CLI + dataset sintético + recall@k/p50/p99 + JSON + tabla md) → `python evals/memory_bench.py --help` + smoke `--sessions 4 --turns 8 --queries 8 --no-vantadb` ✅ (recall@5 1.0 techo sintético, q p50 0.01ms)
- [x] **Step 2 — Slice 2 docs + close:** §17 en BENCHMARKS.md (metodología + comando + tabla smoke 20×16×40 + DEFER reales) + fila README + `.gitignore` reporte + verify ✅ + commit solo-propios + completed + progreso
