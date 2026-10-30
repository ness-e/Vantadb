---
title: Head-to-Head — VantaDB vs mem0 en memoria real
kind: reference
status: active
description: Comparativa honesta vantadb vs mem0 OSS sobre LongMemEval-S con el mismo harness, juez y slice
tags: [vantadb, benchmarks, memory, head-to-head]
---

# Head-to-Head: VantaDB vs mem0 OSS (VER-09)

Medido con `evals/runners/head_to_head.py` sobre el **mismo** harness
(`evals/memory_harness.py`, sin forks), mismo juez determinista, mismo slice,
misma máquina. Protocolo completo: `evals/runners/README.md`. Números
curados: `docs/user/operations/BENCHMARKS.md` §19.

## Slice declarado

Primeras 10 preguntas de cada uno de los 6 tipos (`--sample-stratified 10`)
sobre el split LongMemEval-S completo: n=60, sin celdas `_abs`.
Dataset sha256 `d6f21ea9…`.

| Métrica | vantadb 0.7.0 (text-only) | mem0 2.2.1 (raw local) | Ganador |
|---|---|---|---|
| **recall_all@5** | 0.7167 (43/60) | **0.9333** (56/60) | mem0 |
| recall_any@5 | 0.8833 (53/60) | **0.9833** (59/60) | mem0 |
| Latencia p50 / p99 | **0.65 / 1.09 ms** | 550.0 / 663.7 ms | vantadb (≈850×) |
| Ingest QPS | **243.9 docs/s** | 1.3 docs/s | vantadb (≈187×) |
| Fidelidad escritura | 1.0000 | 1.0000 | empate |
| Aislamiento | 0 violaciones | 0 violaciones | empate |

## Lectura honesta (derrotas incluidas)

mem0-raw — embeddings locales + BM25 híbrido — nos gana en recall de
evidencia en **las 6 celdas de tipo**. El hueco grande está en multi-sesión
(0.2 vs 0.8): las preguntas multi-evidencia exigen TODAS las sesiones en el
top-5 y la recuperación solo-léxica pierde paráfrasis. Es la consecuencia
directa de correr text-only contra un stack semántico, declarado desde el
protocolo, no un empate maquillado.

VantaDB gana lo operativo por órdenes de magnitud en la misma máquina y
slice: ~0.65 ms vs ~550 ms por query y ~244 vs ~1.3 docs/s de ingesta —
la config raw de mem0 paga su embedding local en escritura y lectura.

## Límites (no sobrestimar)

1. mem0-raw ≠ pipeline default de mem0 (extracción LLM apagada); native
   tendrá su propia fila cuando se comisione.
2. Stacks distintos declarados, no igualados: cada sistema corre su mejor
   config local.
3. n=60, sin `_abs` — no comparable con el headline full-500 de §19.
4. Zep y Letta **no corridos** (sin API key / mismatch arquitectónico).

## Reproduce

```bash
python evals/runners/head_to_head.py --systems vantadb,mem0 \
    --data datasets/longmemeval/longmemeval_s_cleaned.json \
    --sample-stratified 10 --label h2h-strat10
```
