---
title: "Head-to-Head: VantaDB vs Mem0 OSS"
kind: howto
status: active
description: An honest head-to-head of VantaDB vs Mem0 OSS on LongMemEval-S with the same harness, judge, and slice.
tags: [vantadb, benchmarks, memory]
---

# Head-to-Head: VantaDB vs Mem0 OSS

> Historical VantaDB 0.7.0 results are preserved below. This documentation update
> does not rerun the evaluation or establish v0.9.0 performance.

Measured with `evals/runners/head_to_head.py` on the **same** harness
(`evals/memory_harness.py`, unmodified), the same deterministic judge, the
same slice, the same machine. Full protocol: `evals/runners/README.md`.
Curated numbers: `docs/user/operations/BENCHMARKS.md` §19.

## Declared slice

First 10 questions of each of the 6 question types
(`--sample-stratified 10`) over the full LongMemEval-S split: n=60, no
`_abs` cells. Dataset sha256 `d6f21ea9…`.

| Metric | VantaDB 0.7.0 (text-only) | Mem0 2.2.1 (raw local) | Winner |
|---|---|---|---|
| **recall_all@5** | 0.7167 (43/60) | **0.9333** (56/60) | Mem0 |
| recall_any@5 | 0.8833 (53/60) | **0.9833** (59/60) | Mem0 |
| Latency p50 / p99 | **0.65 / 1.09 ms** | 550.0 / 663.7 ms | VantaDB (~850×) |
| Ingest QPS | **243.9 docs/s** | 1.3 docs/s | VantaDB (~187×) |
| Write fidelity | 1.0000 | 1.0000 | tie |
| Isolation | 0 violations | 0 violations | tie |

## Honest reading (losses included)

Mem0-raw — local embeddings plus hybrid BM25 — beats us on evidence recall
in **all 6 question-type cells**. The biggest gap is multi-session
(0.2 vs 0.8): multi-evidence questions require ALL sessions in the top-5,
and lexical-only retrieval misses paraphrases. That is the direct
consequence of running text-only against a semantic stack — declared
upfront in the protocol, not a dressed-up tie.

VantaDB wins the operational side by orders of magnitude on the same
machine and slice: ~0.65 ms vs ~550 ms per query and ~244 vs ~1.3 docs/s
ingest — Mem0's raw config pays for its local embedding on write and read.

## Limits (do not overread)

1. Mem0-raw ≠ Mem0's default pipeline (LLM extraction off); the native
   config gets its own row when commissioned.
2. Declared, not equalized stacks: these are the selected local configurations,
   not a demonstrated optimum for either system.
3. n=60, no `_abs` — not comparable with the full-500 headline in §19.
4. Zep and Letta **not run** (no API key / architectural mismatch).

## Reproduce

```bash
python evals/runners/head_to_head.py --systems vantadb,mem0 \
    --data datasets/longmemeval/longmemeval_s_cleaned.json \
    --sample-stratified 10 --label h2h-strat10
```
