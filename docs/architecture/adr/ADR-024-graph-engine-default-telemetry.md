---
title: "ADR-024: Graph engine default-on until telemetry says otherwise (evidence-based opt-in decision)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, graph, features, telemetry, metrics, prometheus]
created: 2026-08-16
last_reviewed: 2026-08-16
related: [ADR-020-storage-backend-default.md, ADR-023-backend-compaction.md]
---

# ADR-024: Graph engine default-on until telemetry says otherwise

## Status

Accepted. Outcome of FND-23 (P20d): **the graph engine stays compiled by default
(no Cargo feature gate)**, and the decision to flip it to opt-in is **not made
by intuition** — it is conditioned on a defined telemetry signal that is not yet
instrumented. This ADR documents the decision, the required evidence
(metric + threshold + action), the current instrumentation state, and the
reopen signal.

## Context

FND-23 (backlog) asks: "Usar señales reales de adopción (métricas, feedback)
para decidir si el motor de grafos queda default-on o pasa a opt-in (Cargo
feature). No decidir por intuición." It complements FND-03 (isolate graph
features for a vector-only consumer) with the *default* decision.

Verified current state (file:line evidence):

- **No `graph` feature exists.** `Cargo.toml:96-139` defines features `cli`,
  `arrow`, `fjall`, `roaring`, `advanced-tokenizer`, `memmap2`, `fs2`,
  `sysinfo`, `rayon`, `rocksdb`, `server`, `prometheus`, `python_sdk`, etc. —
  none gates the graph engine. The graph engine is **always compiled**.
- The graph engine surface: `src/engine.rs:349` (`traverse`, BFS graph
  traversal), `src/edge_index.rs` (`EdgeIndex` adjacency tracking),
  `add_edge`/node-edge mutation paths in `src/engine.rs`. `tests/core/graph.rs`
  covers it.
- **Telemetry exists (FND-07 delivered `/metrics`):**
  - `src/cli_server.rs:147` — `/metrics` route (auth-protected), serves
    `export_metrics_text()` (`src/metrics/core/mod.rs:573`, Prometheus text
    format, `prometheus` feature).
  - `src/metrics/core/registry.rs` registers 20+ metrics: `vanta_query_latency_ms`,
    `vanta_http_requests_total` (labels method/route/status),
    `vanta_planner_hybrid_queries_total`, `vanta_planner_text_only_queries_total`,
    `vanta_planner_vector_only_queries_total`, HNSW/memory gauges, etc.
- **No graph-usage metric exists.** `grep GRAPH|EDGE|TRAVERSAL src/metrics/`
  → 0 matches. There is no counter for graph operations (`traverse`,
  `add_edge`, edge queries), so today there is **zero usage evidence** for the
  graph engine.

The decision under question: keep the graph engine default-on, or make it an
opt-in Cargo feature. Deciding this by intuition would violate the project's
own discipline (Regla 9: no optimization without measurement; FND-23: no
decision by intuition). At launch time we have no usage data, so the honest
decision is: **keep default-on, define the signal that will re-open the
decision, and do not instrument prematurely in this task** (instrumentation is
a separate task).

## Decision

**The graph engine remains default-on (always compiled, no feature gate) for
the post-launch observation window. The decision to flip it to opt-in
(behind a Cargo feature `graph`, mirroring the `rocksdb`/`prometheus`
pattern) is conditioned on a telemetry signal defined below — not on
intuition, roadmap aesthetics, or compile-time cost arguments alone.**

### Required evidence (metric + threshold + action)

| Element | Definition |
|---|---|
| **Metric (pending)** | `vanta_graph_ops_total` — Prometheus counter, labels `op` ∈ {`traverse`, `add_edge`, `remove_edge`, `edge_query`}, incremented in `src/engine.rs` graph paths and exposed via `/metrics`. **Instrumented 2026-08-17 (FND-23-F1)** — see "Current instrumentation state" below. |
| **Metric (existing proxies)** | `vanta_http_requests_total` (labels method/route/status) and `vanta_planner_{hybrid,text_only,vector_only}_queries_total` from `src/metrics/core/registry.rs` — usable to correlate whether any HTTP/planner activity reaches graph paths, but they do NOT count graph ops directly. |
| **Observation window** | 90 days after first public release (post-Show HN), on deployments that expose `/metrics` (server + `prometheus` feature). |
| **Threshold** | **Adoption**: deployments with `vanta_graph_ops_total` > 0 (any op) count as "graph users". If **< 5%** of deployments reporting `/metrics` show graph usage over the window → graph engine shows no real adoption. |
| **Action on threshold met** | Move graph engine to **opt-in**: add `graph` Cargo feature (per FND-03's feature isolation work), remove graph modules from default features, add compile-matrix CI job (`--no-default-features --features graph`), update docs/api. Requires a new ADR superseding this one. |
| **Action on threshold not met** | Default-on **confirmed**; graph engine stays compiled by default. Close this ADR's reopen signal; document the measured evidence in `docs/operations/BENCHMARKS.md` or the follow-up ADR. |

### Current instrumentation state (honest)

**Instrumented as of 2026-08-17 (FND-23-F1, vanta-tuner).** `vanta_graph_ops_total`
is now a labelled Prometheus counter (`IntCounterVec`, label `op`) registered in
`src/metrics/core/registry.rs` and exposed via `/metrics` (`prometheus` feature),
with `record_graph_op(op)` helpers in `src/metrics/core/mod.rs` (cfg-guarded, no-op
when `prometheus` is off). Incremented at the real graph call sites:

- `op="traverse"` — `src/engine.rs:358` (`InMemoryEngine::traverse`, BFS)
- `op="add_edge"` — `src/sdk/api.rs:1051` (`VantaEmbedded::add_edge`)
- `op="remove_edge"` — `src/sdk/api.rs:1087` (`VantaEmbedded::remove_edge`)
- `op="edge_query"` — `src/sdk/graph.rs` (`graph_bfs`, `graph_dfs`,
  `graph_bfs_filtered`, `graph_dfs_filtered`, `graph_topological_sort`, `graph_is_dag`)

Before FND-23-F1 there was no graph-usage telemetry (`grep GRAPH|EDGE|TRAVERSAL
src/metrics/` → 0 matches). The decision itself (default-on) is unchanged — this
ADR's reopen signal is now measurable.

## Consequences

- **Positive:** no behavior change at launch; the graph engine (BFS traversal,
  edge index, `tests/core/graph.rs`) remains available to all users without a
  feature flag; the opt-in question is answered by data, not intuition; the
  ADR records the exact signal (metric name, window, threshold, action) so a
  future agent or human can execute it mechanically.
- **Negative:** the compile/CI cost of always-on graph code is unmeasured; if
  graph usage is genuinely ~0 post-launch, we carry that cost until the window
  closes (90 days). No metric counts graph ops until `vanta_graph_ops_total`
  is instrumented.
- **Deferred (reopen signal):** ~~instrument `vanta_graph_ops_total` (labels
  `op` ∈ {traverse, add_edge, remove_edge, edge_query}) in `src/engine.rs`
  graph paths, exposed via `/metrics` (server + `prometheus` feature).~~ ✅
  DONE (FND-23-F1, 2026-08-17). Then, at the 90-day post-launch mark, evaluate
  the <5% threshold above. A flip to opt-in requires a new ADR and FND-03's
  feature isolation to exist first.
- **Complementarity:** FND-03 isolates graph code behind a feature; this ADR
  fixes the *default*. FND-03's compile-matrix CI job becomes the verification
  harness for the opt-in action if the threshold is met.

## Related

- ADR-020 — storage backend default decision record (same "default vs opt-in"
  pattern: Fjall default, RocksDB opt-in).
- ADR-023 — backend config deferred until benchmark evidence exists (same
  "defer until measured" discipline, Regla 9).
- `src/metrics/core/registry.rs` — existing telemetry inventory.
- `src/cli_server.rs:147` — `/metrics` endpoint (FND-07 deliverable).
- FND-03 (`docs/Backlog.md:485`) — feature isolation for vector-only consumer.