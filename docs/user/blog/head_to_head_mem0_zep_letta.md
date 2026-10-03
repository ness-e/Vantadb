---
title: "VantaDB vs Mem0: a head-to-head on LongMemEval-S (protocol first, numbers second)"
kind: howto
status: draft
description: We ran Mem0 OSS and VantaDB through the same public harness on a declared LongMemEval-S slice, published the protocol, and report the result honestly — including the recall cells we lost
tags: [benchmarks, memory, mem0, longmemeval, evaluation]
---

# VantaDB vs Mem0: a head-to-head on LongMemEval-S (protocol first, numbers second)

*By the VantaDB team*

A vector-database QPS table says almost nothing about agent memory. What matters
is whether the right session comes back when the agent asks — and in this niche,
that question has become almost impossible to answer from vendor material. An
independent audit of LoCoMo found that 6.4% of the answer key itself was wrong
and that an LLM judge accepted incorrect answers up to 63% of the time. Vendors
publish numbers on different datasets, different judges, and different splits.

Our answer is not another marketing number. It is a protocol: one harness, one
judge, declared configurations — and a result table in which **we lose**.

---

## 1. What we ran

**The harness.** [`evals/memory_harness.py`](../../../evals/memory_harness.py)
(VER-08) is our public LongMemEval-S harness, reused unmodified for every system
in this comparison. Its judge is **deterministic — no LLM**. Correctness is
session-level evidence retrieval, reported in two forms:

- `recall_all@5` (headline): ALL `answer_session_ids` inside the top-5 hits —
  the form LongMemEval's own retrieval evaluation reports.
- `recall_any@5` (secondary): at least one evidence session in the top-5.

Every system also reports query p50/p99 latency, ingest QPS, write fidelity
(exact-key read-back), retrieved-token counts, and per-user isolation — all
computed by the same code.

**The runner layer.** [`evals/runners/`](../../../evals/runners/README.md) maps
third-party memory SDKs onto the same store contract the harness consumes, so
the metrics are identical by construction. If you want to check our work, the
protocol, pins, and skip policy are in that README, and every number below is
regenerable with one command.

**The slice (declared).** LongMemEval-S, full split
(sha256 `d6f21ea9…`), first **10 questions of each of the six question types**
(60 questions total). We stratify because the dataset file is ordered in
type blocks — a naive "first 50" slice would measure a single type and quietly
cherry-pick the comparison. The slice contains no abstention (`_abs`) cells;
the full-split run (n=470 non-abstention) is published separately in
[BENCHMARKS.md §19](../operations/BENCHMARKS.md).

**The configurations (declared, not equalized).**

- **Mem0 OSS 2.2.1 — raw mode** (`infer=False`): its LLM fact extraction is
  deliberately disabled, so the run makes **zero LLM calls**. Sessions are
  embedded locally with FastEmbed `bge-small-en-v1.5` (384d) and stored in an
  embedded Qdrant, searched as a semantic + BM25 hybrid. This is *not* Mem0's
  default product pipeline — and we say so everywhere this row appears.
- **VantaDB 0.7.0 — text-only path**: empty vector plus `text_query` (its
  lexical retrieval), in the in-memory backend, through the PyO3 bindings.
- One machine, single-threaded, one execution: Windows 11, Intel i5-1235U,
  32 GB RAM (the machine was under load — absolute latencies are conservative).

---

## 2. The table

| Metric | VantaDB 0.7.0 (text-only) | Mem0 2.2.1 (raw: semantic+BM25, local) |
|---|---|---|
| **recall_all@5** (headline) | 0.7167 (43/60) | **0.9333** (56/60) |
| recall_any@5 | 0.8833 (53/60) | **0.9833** (59/60) |
| multi-session (recall_all@5) | 0.2 | **0.8** |
| temporal-reasoning (recall_all@5) | 0.7 | **0.9** |
| knowledge-update / preference / assistant / single-user | 0.7 / 0.8 / 0.9 / 1.0 | **1.0 / 0.9 / 1.0 / 1.0** |
| Query latency p50 / p99 | **0.65 / 1.09 ms** | 550.0 / 663.7 ms |
| Ingest QPS | **243.9** docs/s | 1.3 docs/s |
| Write fidelity (exact-key) | 1.0000 | 1.0000 |
| Retrieved tokens p50 | 10,119 words | 10,019 words |
| Isolation violations | 0 / 300 checks | 0 / 300 checks |

---

## 3. Reading it honestly

**We lost the recall cells.** Mem0's raw configuration beats VantaDB's text-only
path in every question type on this slice. The sharpest gap is multi-session:
0.2 vs 0.8. The metric is strict — for multi-evidence questions, *all* evidence
sessions must land in the top-5 — and lexical-only retrieval misses paraphrases
that a semantic index catches. That is the honest reason: our text path finds
words, not meanings.

**We win the operational cells — by two orders of magnitude.** On the same
machine: ~0.65 ms vs ~550 ms per query (≈850×), and ~244 vs ~1.3 docs/s ingest
(≈187×). Mem0-raw pays for a local embedding at write time and query time;
VantaDB's text path answers from an in-process index. Retrieved token counts are
comparable, so neither system is winning by returning more context.

**What this does not say.** It is not a verdict on Mem0's product (extraction
disabled), not embeddings-vs-embeddings (VantaDB ran text-only — a declared
configuration, not its only one), and not a QA-accuracy comparison (the judge is
retrieval-level and deterministic). It is a declared slice, not the full split —
the full-split VantaDB number (0.7617) lives in BENCHMARKS §19 and is not
directly comparable with this table.

---

## 4. Zep and Letta: documented skips, not missing rows

A head-to-head that only runs the systems you beat is not a head-to-head. Here
is exactly what happened to the other two:

- **Zep** is cloud-first: running it requires `ZEP_API_KEY` and paid credits
  (the full split would consume roughly 790k credits against a 10k free tier).
  We implemented and verified the runner against the real `zep-cloud` 3.30.0
  SDK; the row will be published when the run is commissioned. We did not
  invent a number for it.
- **Letta** is not a put/search store — it is an agent harness whose memory
  (MemFS, dreaming) is managed by the agent inside its own loop. Our
  deterministic retrieval metrics do not map onto it without a different,
  LLM-in-the-loop protocol, which is deferred (tracked as a finding).
- **Mem0's default pipeline** (LLM extraction, `gpt-4o-mini`) is pending
  provider keys and budget; its cost estimate is declared as an estimate, not a
  measurement.

---

## 5. Reproduce it

```bash
# protocol + skip policy (no run)
python evals/runners/head_to_head.py --check

# the exact command behind the table (needs the 277 MB local cache,
# mem0ai==2.2.1 + fastembed==0.8.1 installed; raw mode = zero LLM calls)
python evals/runners/head_to_head.py --systems vantadb,mem0 \
    --data datasets/longmemeval/longmemeval_s_cleaned.json \
    --sample-stratified 10 --label h2h-strat10
```

Reports are written to `evals/runners/report_h2h_*.json` (gitignored). The
committed record, with the full protocol and limits, is
[BENCHMARKS.md §19](../operations/BENCHMARKS.md).

---

## 6. What's next

We will publish the native-pipeline rows (Mem0 with extraction, Zep, and a
Letta-appropriate protocol) as they are commissioned, and we will keep running
this harness on larger slices as hardware allows. If you find a configuration
where *we* come out ahead for the wrong reason — a broken baseline, a
cherry-picked slice, a judge that flatters us — we want the bug report. Trust in
a memory engine is earned with reproducible numbers and published losses.
