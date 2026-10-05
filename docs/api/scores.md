---
title: Scoring Semantics — VantaDB Official Score Contract
kind: reference
status: active
description: VantaDB hybrid search combines two independent rankers
tags: [vantadb, api, scoring, search-scores]
---

# Scoring Semantics — VantaDB Official Score Contract

> **Source of truth:** this document is the canonical scoring contract for VantaDB search.
> Complements `FND-06-core-bindings-boundaries.md` (H3) and closes the gap `grep docs/api 0 hits score semantics`.
> Core owns all scoring logic (`src/planner.rs`, `src/index/distance/*`, `src/sdk/search/mod.rs:ERR-028`);
> bindings are thin glue (R-8).

## Overview

VantaDB hybrid search combines two independent rankers:

- **Lexical (BM25)** — sparse text relevance over the inverted index (`src/text_index.rs`).
- **Vector (HNSW)** — dense cosine or Euclidean ANN (`src/index/*`).

Rankings are fused with **Reciprocal Rank Fusion (RRF)** — scores are never compared directly,
only ranks. This avoids calibration between BM25 (unbounded) and vector (bounded [-1,1]) scales.

## RRF Fusion

- **Constant:** `RRF_K = 60.0` (`src/planner.rs:27`, `desktop/retrieval-core.ts:19`) — literature default, tunable per-request via `SearchProfileConfig { rrf_k }` (MEM-01).
- **Formula (0-based planner):** `contribution = 1 / (RRF_K + rank + 1)` where `rank` is 0-based position in each ranked list (`src/planner.rs:205`).
- **Wire rank (1-based):** `rank_map` in `src/sdk/search/debug.rs:23` exposes 1-based ranks → wire contribution `1 / (RRF_K + r_wire)` (see `desktop/retrieval-core.ts:52` `rrfContribution`).
- **Multi-channel:** `fuse_rrf_many` sums contributions across arbitrary channels (lexical, dense, sparse); hits in multiple lists accumulate.
- **Determinism:** fused list sorted descending by score, ties broken by `key` then `node_id` (`src/planner.rs:221 sort_hits`).
- **Candidate budget:** per-arm `hybrid_candidate_budget(top_k, candidate_k)` clamped `[32, 256]`, always `≥ top_k` (`src/planner.rs:96`).

Example: doc ranked #1 in BM25 and #3 in HNSW → `1/61 + 1/63 ≈ 0.0323`; #1 in both → `2/61 ≈ 0.0328`.

## BM25 Lexical Scoring

- Standard BM25 over tokenized text (tokenizer `src/tokenizer.rs`), per-term contributions in `VantaBm25TermContribution` (`docs/api/EMBEDDED_SDK.md`).
- Scores are higher-is-better but not comparable across namespaces/queries — RRF erases scale via ranks.
- Only `trimmed_text_query` non-empty enters the lexical arm (`src/planner.rs:129`).

## Vector Scoring — Cosine vs Euclidean

| Metric | Core `VantaMemorySearchHit.score` (higher-is-better) | Wire semantics (`SearchHit.distance`, raw ANN only) |
|--------|------------------------------------------------------|---------------------------------------|
| **Cosine** (default) | **similarity** ∈ [-1, 1] (parallel 1, orthogonal 0, opposite -1) via `cosine_sim_f32` (`src/index/distance/metrics.rs:47`) | **distance** `1 - similarity` ∈ [0, 2] — see MCP `search_semantic` conversion (`skills/vantadb-mcp/SKILL.md:236`). Rust core SDK keeps similarity; adapters convert via `similarity = 1 - distance/2`. |
| **Euclidean** | **negated distance** (higher = closer) — `-euclidean_distance` or `-sqrt(euclidean_sq)` | **distance** `sqrt(euclidean_sq)` (lower = closer) — direct L2. |

Helper centralization (this crate `src/api/scores.rs`): `cosine_distance_to_similarity`, `cosine_similarity_to_distance` — canonical `1 - d/2` and `2*(1-s)` mappings, avoiding duplication `1.0 - s/2.0` in adapters (`integrations/langchain/vectorstore.py:213`, `llamaindex:183`).

## Zero-Norm Contract (ERR-028)

- **Core:** `src/sdk/search/mod.rs:108-120` rejects **zero-norm cosine query vectors** (`f32_l2_norm < EPSILON`) with `Error::InvalidInput("zero-norm cosine query vector is undefined; use a non-zero vector or the euclidean distance metric (AUDREP-55, ERR-028)")`.
- **Rationale:** cosine `dot/(||a||·||b||)` is `0/0` undefined when `||query||=0`.
- **Drift documented (FND-06 H1):** `vantadb-ts/src/vantadb.ts:333-353` silently falls back to Euclidean on zero-norm; `vantadb-ts/src/native.ts:250-260` and `vantadb-python` correctly surface the core error. **Do not automate fallback in new bindings** — surface ERR-028 (R-8 boundary-violation).
- **`metrics.rs:22-27` internal:** `cosine_sim_*` returns `0.0` on zero-norm (safe kernel), but the search path **must** reject at request validation (sdk/search) to avoid silent empty results.

## Score Semantics by Binding

| SDK / Surface | Field | Direction | Conversion |
|---------------|-------|-----------|------------|
| `vantadb` (Rust core) `VantaMemorySearchHit` | `score` | higher-is-better | similarity (cosine) or negated Euclidean; pinned by `src/sdk/serialization/vector_types.rs::tests` |
| `vantadb-mcp` `memory_search` (hybrid, ex-`search_memory`; API-04 canonical) | `score` | higher-is-better | mirrors core (`MemorySearchHit.score`); legacy alias still dispatchable |
| `vantadb-mcp` `search_semantic` (raw ANN) | `distance` | lower-is-better | `distance = 1 - similarity` (cosine), `sqrt(euclidean_sq)` |
| `vantadb-python` `hit.score` | `score` | higher-is-better | mirrors core |
| `vantadb-wasm` `SearchHit` | `score` / `distance` | higher / lower | JS mapping; see `WASM_API.md` |
| `vantadb-ts` `SearchHit.score` | `score` | higher-is-better | mirrors core (W1/API-02: the pre-W1 `distance` rename was removed) |
| HTTP `POST /api/v2/search` | `score` | higher-is-better | core score |

All hybrid results are **RRF-fused scores** (not raw BM25/cosine) — explanation ranks in `debug.rs` reconstruct per-arm contributions (`desktop/retrieval-core.ts:computeSegments`).

## Record Confidence (`MemoryRecord.confidence`)

Every memory record carries a declared/computed confidence in `[0, 1]` plus its
provenance class (ADR-0046 §D2/§D4, SCH-04):

- `confidence_class: "Asserted"` — a direct writer claim; absent score defaults
  to `D_a = 1.0` ("trust the writer" policy).
- `confidence_class: "Derived"` — computed by the engine from `derived_from`
  parents: `score = clamp(min(parents) × 0.9, 0, 1)` (`DERIVATION_DISCOUNT`).
  Declaring a score on a derived record is rejected at the boundary.
- Filters: `min_confidence` (per request, opt-in) and
  `confidence_threshold` (config, opt-in — also triggers the explicit
  `abstained` signal when it empties the page, ADR-0046 §D2).

### Reinforcement (outcome loop, MEMG-02)

`Embedded::reinforce(namespace, key, outcome)` (SDK) / `memory_reinforce` (MCP)
let the host declare what happened with a recalled memory — the engine never
infers the outcome (no silent feedback). Declared policy (not calibrated —
VER-08):

| Outcome | Effect |
|---------|--------|
| `used` | `confidence = min(1.0, confidence + 0.05)` and `last_validated_at_ms = now` (successful re-validation). Rate-limited: at most one bump per record per 5-minute window anchored on the previous stamp; inside the window the call is a no-op (audited as `used_rate_limited`, never silent). |
| `corrected` | `confidence = max(0.0, confidence - 0.10)`. `last_validated_at_ms` is **not** touched (success-only stamping, MGR-12 §3.3). |
| `unused` | Neutral: no score change, no stamp; the audit event records the explicit declaration. |

- Applies to `asserted` records only — a `derived` score is computed from its
  parents (`min × 0.9`) and mutating it would break determinism (V4); derived
  records are rejected explicitly.
- Interaction notes: a `corrected` does **not** reset the positive rate-limit
  anchor — a `used` shortly after a correction is still limited by the previous
  validation stamp (the window counts successful validations). Quarantine state
  is not inspected: quarantined records are excluded from default recall/list,
  so only an explicit-key call can reach one.
- State-only change (same class as `quarantine_apply`): `version` does not
  change and no version-history snapshot is written; `updated_at_ms` is
  refreshed and the operation is audited (`memory_reinforce`). Callers that
  never invoke the op observe no change (`put` still leaves
  `last_validated_at_ms = None`).
- No ranking change: default search ordering is untouched (weighted ranking is
  post-calibration); the consumer-visible effect is threshold selection
  (`min_confidence` / `confidence_threshold`) before/after reinforcement.
- L1–L5 calibration limits below apply: the bump/decay are declared policy,
  not measurement.

### Calibration limits (L1–L5) — declared ranges, not probabilities

| # | Limit |
|---|-------|
| **L1** | `D_a = 1.0` and the `0.9` discount are **declared policy**, not measurement. Empirical calibration (ECE/temperature) is scheduled for VER-08. |
| **L2** | Scores are **not calibrated probabilities**. Consumers must treat them as ranges and pick configurable thresholds — never statistical significance. |
| **L3** | No temporal decay: freshness is tracked separately by `last_validated_at_ms`. |
| **L4** | `derived_from` is same-namespace in 0.8.0 (cross-namespace parents are v1.0). |
| **L5** | No reactive recomputation: a derived record keeps its stored score until an explicit re-consolidation. |

## Verification

```powershell
Select-String -Path "docs/api/scores.md" -Pattern "RRF|BM25|cosine|zero-norm" | Measure-Object Count  # >=1
Select-String -Path "src/api/scores.rs" -Pattern "cosine_distance|rrf" | Measure-Object Count  # >=1
cargo check -p vantadb  # exit 0
cargo test -p vantadb --lib -- scores  # helpers pinned
```

## References

- `src/planner.rs` — RRF constants, fusion, candidate budget
- `src/index/distance/metrics.rs` — cosine/Euclidean kernels
- `src/index/distance/mapper.rs` — metric dispatch
- `src/sdk/search/mod.rs` — ERR-028 guard
- `src/sdk/search/debug.rs` — rank_map wire
- `docs/dev/research/archive/FND-06-core-bindings-boundaries.md` — H1/H3 drift
- `desktop/src/components/lens/retrieval/retrieval-core.ts` — RRF_K+contribution mirror
