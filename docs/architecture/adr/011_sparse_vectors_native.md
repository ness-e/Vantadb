---
title: "ADR-011: Native sparse vectors — representation, metric, and coexistence with BM25"
type: adr
status: accepted
tags: [vantadb, architecture, adr, vector, sparse, hybrid]
created: 2026-08-02
last_reviewed: 2026-08-02
---

# ADR-011: Native sparse vectors — representation, metric, and coexistence with BM25

## Context

VantaDB exposes dense-only vector search (`VantaMemorySearchRequest.query_vector: Vec<f32>`)
fused with BM25 lexical text search via Reciprocal Rank Fusion (RRF, `fuse_rrf_with_report`).
The backlog asks for **native sparse vectors** ("Sparse vectors nativos - hybrid search real").
Today the only mention of "sparse" in the codebase is a terminal label string in a test, not a
real data type.

Two ambiguities must be resolved before implementation:

1. **Input semantics**: is the sparse vector direct user input (a map `dim → value`), or is it
   derived internally from text (SPLADE-style learned sparsity)?
2. **Coexistence**: does sparse replace BM25 lexical text search, or complement it as a second
   lexical channel?

## Decision

1. **Sparse vectors are direct user input.** The backlog says "sparse vectors nativos" — the
   user supplies `{dim: value}` pairs, e.g. `{3: 1.0, 7: 0.5}`. We do **not** derive sparsity
   from text (no SPLADE). Rationale: SPLADE requires a trained model and changes ingestion
   semantics; user-supplied sparse vectors (as produced by BM25-style encoders, TF-IDF vectors,
   or learned sparse embedders the user already has) are a zero-training, drop-in addition.
   Derivation can be added later as a separate feature behind a flag without changing this type.

2. **Representation is `SparseVector(BTreeMap<u32, f32>)`** (dimension → value), defined in
   `src/node.rs` next to `DistanceMetric`. `BTreeMap` is chosen over `HashMap` for
   deterministic serialization (serde roundtrips and persisted bytes are stable across runs),
   and dot-product iteration is ordered. The newtype keeps the serde surface explicit and
   extensible.

3. **New metric `DistanceMetric::SparseDot`.** Existing variants (`Cosine`, `Euclidean`) are
   untouched. Sparse vectors are scored by plain dot product (higher = better), which is the
   natural similarity for bag-of-dimension representations. Sparse vectors are **not** indexed
   in the HNSW graph — they are searched with a brute-force scan over the namespace's records
   (the same pattern the dense path already uses for its PreFilter / fallback paths). This
   keeps the dense HNSW index untouched and makes sparse an exact search.

4. **Sparse coexists with BM25 text, it does not replace it.** Sparse is a second lexical
   channel: BM25 remains the text-token lexical channel, sparse is the user-supplied
   dimension-vector channel. Fusion stays RRF; when both sparse and dense lists are present
   they fuse 2-way via the existing `apply_rrf_contributions` helper (no duplicated fusion
   logic). When text, dense, and sparse are all present the natural extension is a 3-way
   contribution pass over the same helper.

5. **Persistence**: the sparse vector lives on the node inside `UnifiedNode.ext_metadata`
   (a forward-compatible `HashMap<String, Vec<u8>>` explicitly designed for schema metadata
   that must not break Bincode). No new fields are added to `UnifiedNode`, so the persisted
   node format is unchanged and old databases load as-is (missing sparse = `None`).

## Consequences

- Pros:
  - Dense-only search, BM25 text search, and dense+BM25 hybrid keep their exact current
    behaviour (new request field is `Option`, default `None`).
  - No HNSW changes; sparse search is exact brute-force, deterministic and simple to verify.
  - No data migration: sparse is additive (`ext_metadata` is forward-compatible).
  - Sparse channel reuses the existing RRF helper — no duplicated fusion.
- Cons:
  - Sparse search is O(n) per namespace (no inverted index over dimensions). Acceptable for
    the native-sparse contract (records with sparse vectors are typically modest in number);
    a dimension-inverted index is a future optimisation, not part of this ADR.
  - Export/import of sparse vectors is deferred (export line schema is unchanged); a follow-up
    can extend `VantaMemoryExportLine` without breaking readers (missing field deserialises to
    `None`).
  - No SPLADE-style text→sparse derivation yet; user must supply sparse vectors explicitly.
