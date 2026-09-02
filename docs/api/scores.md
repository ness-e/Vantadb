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

| Metric | Core `VantaMemorySearchHit.score` (higher-is-better) | Wire semantics (`SearchHit.distance`) |
|--------|------------------------------------------------------|---------------------------------------|
| **Cosine** (default) | **similarity** ∈ [-1, 1] (parallel 1, orthogonal 0, opposite -1) via `cosine_sim_f32` (`src/index/distance/metrics.rs:47`) | **distance** `1 - similarity` ∈ [0, 2] — see MCP `search_semantic` conversion (`skills/vantadb-mcp/SKILL.md:236`). Rust core SDK keeps similarity; adapters convert via `similarity = 1 - distance/2`. |
| **Euclidean** | **negated distance** (higher = closer) — `-euclidean_distance` or `-sqrt(euclidean_sq)` | **distance** `sqrt(euclidean_sq)` (lower = closer) — direct L2. |

Helper centralization (this crate `src/api/scores.rs`): `cosine_distance_to_similarity`, `cosine_similarity_to_distance` — canonical `1 - d/2` and `2*(1-s)` mappings, avoiding duplication `1.0 - s/2.0` in adapters (`integrations/langchain/vectorstore.py:213`, `llamaindex:183`).

## Zero-Norm Contract (ERR-028)

- **Core:** `src/sdk/search/mod.rs:108-120` rejects **zero-norm cosine query vectors** (`f32_l2_norm < EPSILON`) with `VantaError::InvalidInput("zero-norm cosine query vector is undefined; use a non-zero vector or the euclidean distance metric (AUDREP-55, ERR-028)")`.
- **Rationale:** cosine `dot/(||a||·||b||)` is `0/0` undefined when `||query||=0`.
- **Drift documented (FND-06 H1):** `vantadb-ts/src/vantadb.ts:333-353` silently falls back to Euclidean on zero-norm; `vantadb-ts/src/native.ts:250-260` and `vantadb-python` correctly surface the core error. **Do not automate fallback in new bindings** — surface ERR-028 (R-8 boundary-violation).
- **`metrics.rs:22-27` internal:** `cosine_sim_*` returns `0.0` on zero-norm (safe kernel), but the search path **must** reject at request validation (sdk/search) to avoid silent empty results.

## Score Semantics by Binding

| SDK / Surface | Field | Direction | Conversion |
|---------------|-------|-----------|------------|
| `vantadb` (Rust core) `VantaMemorySearchHit` | `score` | higher-is-better | similarity (cosine) or negated Euclidean; pinned by `src/sdk/serialization/vector_types.rs::tests` |
| `vantadb-mcp` `search_memory`/`search_semantic` | `distance` | lower-is-better | `distance = 1 - similarity` (cosine), `sqrt(euclidean_sq)` |
| `vantadb-python` `hit.score` | `score` | higher-is-better | mirrors core |
| `vantadb-wasm` `VantaSearchHit` | `score` / `distance` | higher / lower | JS mapping; see `WASM_API.md` |
| `vantadb-ts` `SearchHit.distance` | `distance` | lower-is-better | `distance` = L2 or cosine distance (CODE-091 `docs/api/TS_SDK.md`) |
| HTTP `POST /api/v2/search` | `score` | higher-is-better | core score |

All hybrid results are **RRF-fused scores** (not raw BM25/cosine) — explanation ranks in `debug.rs` reconstruct per-arm contributions (`desktop/retrieval-core.ts:computeSegments`).

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
- `docs/research/archive/FND-06-core-bindings-boundaries.md` — H1/H3 drift
- `desktop/src/components/lens/retrieval/retrieval-core.ts` — RRF_K+contribution mirror
