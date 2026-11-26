---
title: Search Capability Parity — VantaDB vs Milvus vs Qdrant
kind: reference
description: "1. topk semantics under groupby — VantaDB caps total hits; Milvus/Qdrant cap the number of groups. Choose topk = groups × groupsize to emulate the vendor behavior"
---
# Search Capability Parity — VantaDB vs Milvus vs Qdrant

> **Status:** ✅ WIRE-08 (2026-09-28) — capabilities implemented in `src/sdk/search/` and `src/planner.rs`.
> **Scope:** score-range filtering, group-by, cursor pagination, MMR diversity, RRF fusion, query rewriting.
> **Sources:** official vendor documentation, verified 2026-09-28 (see [Sources](#sources)).

## Capability × capability

| Capability | VantaDB (WIRE-08) | Milvus | Qdrant |
|------------|-------------------|--------|--------|
| **Range / score filter** | `MemorySearchRequest::range = RangeFilter { min_score, max_score }` — post-ranking, **inclusive** bounds, **score space** (higher = more relevant, same space as `MemorySearchHit::score`). Non-finite or `min > max` → `SEARCH_OPTIONS_INVALID`; the fetch window deepens (bounded by 10 000) until the page fills or the list is exhausted | `radius` + `range_filter` in `search_params` — **distance space**, sign flips per metric: COSINE/IP `radius < distance <= range_filter`; L2/JACCARD/HAMMING `range_filter <= distance < radius` [R1] | `score_threshold` (minimum score) on search/query/prefetch — score space, single lower bound [R2] |
| **Group-by** | `group_by = GroupByConfig { field, group_size }` — walk ranked hits, keep up to `group_size` per metadata value (default `1`); `top_k` caps **total hits**; records missing the field form their own group | `group_by_field` + `limit` (number of **groups**) + `group_size` (entities per group, default 1) + `strict_group_size`; works on indexed collections; groups can be ordered with `order_by_fields` [R3] | `group_by` (payload path) + `group_size` (default 3 in Query API) + `limit` (max **groups**, default 10) + `with_lookup`; one point can belong to multiple groups when the field is multi-valued [R2] |
| **Pagination / cursor** | `search_page(request) -> MemorySearchPage { hits, next_cursor }` — opaque token (plan fingerprint + consumed offset + identity anchor). Full page ⇒ `next_cursor = Some`; short page ⇒ last page. Resume is **best-effort, not a snapshot**: a hit returned in a previous page can be returned again (or skipped) when interleaved writes reorder its rank across the anchor (BM25/IDF are recalculated corpus-wide); writes that rank *before* the anchor are never duplicated, and deleting the anchor advances from its former offset. The fetch window grows (bounded by 10 000) to compensate. A strong cursor (snapshot / server-side session) is tracked as FIND-183. Rejected with `mmr`/`group_by` (`SEARCH_CURSOR_INVALID`) | `SearchIterator` (`batch_size` per call + total `limit`, session-scoped `next()`/`close()`); REST offset pagination is bounded by the server-side result window (16 384) [R4] | `offset` on search/query ("Offset of the first result to return. May be used to paginate results. Note: large offset values may cause performance issues") + `next_page_offset` scroll API [R2] |
| **MMR diversity** | `mmr = MmrConfig { lambda, fetch_k }` — classic `λ·rel − (1−λ)·max cos`; `lambda ∈ [0,1]` (default 0.5, 1.0 = identity order); relevance min-max normalized in-window; `fetch_k` default `top_k × 5`, cap 16 384. Core MMR available in `src/sdk/search/mmr.rs`; adapter migration (langchain/llamaindex drop their client-side copies) is deferred — out of WIRE-08 scope | Not native (client-side in application code) | `mmr = { diversity, candidates_limit }` on nearest queries — `diversity` higher = more diversity (mirror of `lambda = 1 − diversity`); `candidates_limit` capped at 16 384 [R2] |
| **RRF fusion** | SDK `search`/`search_page` fuse lexical+vector+sparse via `fuse_rrf*` (`RRF_K = 60`); **WIRE-08 adds the planner RRF operator**: an IQL plan carrying both a vector search and a text condition fuses both arms when a `search_profile` is present (opt-in; without a profile the previous vector-then-filter path is unchanged) | Not native (RRF is applied by client SDKs over multiple search calls) | `fusion: rrf` over `prefetch` sub-requests, with `Rrf { k, weights }`; also `dbsf` (distribution-based score fusion) [R2] |
| **Query rewriting** | **DEFER** — no spec (MGR-16 pending). Tracked in `docs/dev/tasks/WIRE-08.md` §Patas | — (rewriting is application-level) | — (rewriting is application-level) |

### Deliberate differences (documented, not bugs)

1. **`top_k` semantics under `group_by`** — VantaDB caps total hits; Milvus/Qdrant cap the number of groups. Choose `top_k = groups × group_size` to emulate the vendor behavior.
2. **Range bounds** — VantaDB bounds are inclusive on both ends and expressed in score space; Milvus bounds are exclusive at the outer circle and expressed in distance space (per-metric direction). Convert with the metric's score/distance relation before porting a query.
3. **Cursor stability model** — VantaDB tokens are *identity-anchored* and session-scoped (process-local fingerprint), like Milvus iterators; they are not durable across restarts and must not be persisted. Resume is best-effort on all three engines: VantaDB can repeat a hit whose rank is reordered across the anchor by interleaved writes (Qdrant offsets can likewise duplicate/skip under concurrent writes). A strong cursor (snapshot / server-side session) is tracked as FIND-183.
4. **RRF operator scope (IQL)** — the fused lexical arm ranks by scan order at the physical layer today (the SDK path remains the BM25 reference). Closing that gap is part of the MGR-16 spec (RRF-CBO design).

## Sources

| Ref | Source | URL |
|-----|--------|-----|
| R1 | Milvus — Range Search (`radius`, `range_filter`, per-metric requirements) | <https://milvus.io/docs/range-search.md> |
| R3 | Milvus — Grouping Search (`group_by_field`, `group_size`, `strict_group_size`, `limit` = groups) | <https://milvus.io/docs/grouping-search.md> |
| R4 | Milvus — Search Iterator (`batch_size`, `limit`, session iterator; REST result window 16 384) | <https://milvus.io/docs/with-iterators.md> |
| R2 | Qdrant — REST schema source of the served OpenAPI spec (`score_threshold`, `offset`, `group_by`/`group_size`/`limit`, `mmr { diversity, candidates_limit }`, `fusion: rrf`, `Rrf { k, weights }`) | <https://github.com/qdrant/qdrant/blob/master/lib/api/src/rest/schema.rs> |

> All vendor claims above were read from the pages/source above on 2026-09-28; no vendor behavior is asserted from memory.
