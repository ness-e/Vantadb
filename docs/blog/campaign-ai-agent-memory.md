---
title: "AI Agent Memory Campaign — Measurable Deliverables"
type: strategy
status: active
tags: [vantadb, marketing, campaign, ai-agents, memory, agent-memory]
last_reviewed: 2026-08-05
aliases: [MKT-10, campaign-ai-agent-memory, AI Agent Memory]
---

# AI Agent Memory Campaign

> **Domain:** Marketing & Content
> **Purpose:** Turn the "AI Agent Memory" campaign (backlog `MKT-10`) from a vague claim into a set of **measurable deliverables**, each with a concrete asset path and a verifiable criterion. Nothing in this checklist is vague.
> **Tracking task:** `MKT-10` (rescued under Task 53, `docs/plans/2026-08-05-backlog-validation-actions.md`).
> **Narrative:** VantaDB is "the SQLite of agent memory" — durable, embedded, zero-dependency persistent memory so agents stop re-embedding and stop re-ingesting full context every turn.

---

## Status Summary

Prior validation (`docs/audit-reports/backlog-validation-2026-07-28.md:118`) marked `MKT-10` ❌ ("sin materiales de campaña"). On review **2026-08-05**, the campaign's landing and demo surfaces already exist in the product/web; **1 real gap remains** (a benchmark post of memory-vs-full-context with real run data). Each deliverable below states its asset path and a **verifiable criterion** — no item is a vague "write a post about X".

High-level status: **2 of 3 deliverables covered. 1 deliverable (D2) is an open task requiring a real run.**

---

## Deliverables checklist

### D1 — Landing "AI Agent Memory" — ✅ COVERED

**Narrative asset** proving "VantaDB = persistent memory for agents".

- **Path:** `web/src/app/solutions/ai-agents/page.tsx` — route **`/solutions/ai-agents`**.
- **Copy:** localized ES/EN in `web/src/lib/dictionaries.ts` (`solutionsAgents.*`): pain ("Memory that forgets"), solution, 4-step flow (observe → put → flush → reopen intact), metric labels (recall latency 1.2ms, network hops 0, Recall@10), CTA (`pip install vantadb-py`).
- **Tutorial complement:** `docs/tutorials/01-ai-agent-memory.md` — full "Building AI Agent Memory" REPL build (`search_memory`, `text_query`, metadata filters).
- **Criterion (verifiable):** `cd web && npm run dev` → `http://localhost:3000/solutions/ai-agents` renders the "Give your agent a memory" section with the 4-step flow and CTA. **Status: ✅ covered.**

### D2 — Blog benchmark: memory vs full-context — ❌ OPEN (real gap)

**The one real gap.** Two existing posts show engine performance but **not** the memory-vs-full-context trade the campaign's "SQLite of memory" narrative hinges on (token reduction, cheaper recall):

- **Existing (partial, NOT this deliverable):** `docs/blog/sqlite_for_ai_agents.md` (QPS 750→1,195, 4.01x batch, 2.43ms) and `docs/blog/benchmarks_vs_lancedb_chroma.md` (recall/QPS vs LanceDB & Chroma). Harness: `benchmarks/competitive_bench.py`.
- **Deliverable:** draft post `docs/blog/benchmark_agent_memory_vs_full_context.md` (slug `benchmark-agent-memory-vs-full-context`) that runs a reproducible script and reports **real numbers**. The script measures: (1) end-to-end token cost per turn of a memory-backed agent vs a full-context-rewriting agent across N=100 turns; (2) retrieval cost and recall as history size K grows.
- **Rule (MANDATORY):** publish only numbers from a real run; cite the exact script, dataset, and hardware in the post frontmatter. Follow the MKT-05 precedent (`benchmarks_vs_lancedb_chroma.md`). **Prohibido inventar cifras** — the plan's own caveat says the GraphRAG "40-60% token reduction" metrics "parecen claims, no runs" and require a reproducible benchmark.
- **Criterion (verifiable):** `docs/blog/benchmark_agent_memory_vs_full_context.md` exists **and** its frontmatter contains `run_date`, the script path, and numbers from an actual run. **Status: PENDING** — owner: downstream content task (recommend the MKT-05 pattern: script in `benchmarks/`, numbers in `docs/blog/`).

### D3 — Interactive demo — ✅ COVERED

- **Path:** `web/src/app/demo/page.tsx` + `web/src/app/playground/page.tsx` + `web/src/components/vanta/code-playground.tsx` — interactive API runnable in the browser via the WASM runtime; each run opens a fresh in-memory instance (see `code-playground.tsx` line ~620: "Each Run opens a fresh in-memory VantaDB instance (wasm32 engine)").
- REPL illustration in `docs/tutorials/01-ai-agent-memory.md`.
- **Criterion (verifiable):** `cd web && npm run dev` → `/playground` runs `put` → `get` → `search_memory` and prints expected values without a server. **Status: ✅ covered.**

---

## Open work / next actions

| # | Deliverable | Owner | Next concrete step | Verifiable when |
|---|-----------|-------|--------------------|-----------------|
| 1 | D2 blog "memory vs full-context" | MKT (content) | build the benchmark script + draft the post | `docs/blog/benchmark_agent_memory_vs_full_context.md` exists with a `run_date` and numbers from an actual run |

---

## See Also

- `docs/strategy/BLOG_SERIES_PLAN.md` — series plan; tracks the planned GraphRAG token-reduction post (not drafted)
- `docs/blog/` — drafts
- `docs/Backlog.md` → `MKT-10`
