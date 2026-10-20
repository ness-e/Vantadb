---
title: "ADR-0058: Native sparse vectors — verified representation, metric, and coexistence with BM25"
kind: adr
status: proposed
description: "Sparse vectors are native in VantaDB — SparseVector(BTreeMap<u32, f32>) at src/node/vector_data.rs:30, scored with DistanceMetric::SparseDot, fused with BM25 and dense via RRF; restates ADR-0011 with the code references verified against the 0.9.0 tree"
tags: [vantadb, architecture, adr, vector, sparse, hybrid]
supersedes: docs/dev/architecture/adr/ADR-0011-sparse-vectors-native.md
created: "2026-10-20"
owner: vanta-arch
---

# ADR-0058: Native sparse vectors — verified representation, metric, and coexistence with BM25

## Status

Accepted. Supersedes [ADR-0011](./ADR-0011-sparse-vectors-native.md).

**No decision changes.** ADR-0011's outcome still holds verbatim: sparse
vectors are native, user-supplied, scored by plain dot product, and they
coexist with BM25 and dense retrieval through the same RRF helper. ADR-0011 is
superseded for two *documentation* defects, both verified against the 0.9.0
tree on **2026-10-20**:

1. its frontmatter `description` claimed *"VantaDB exposes dense-only vector
   search (VantaMemorySearchRequest.queryvector: Vec<f32>)"*. That was the
   **pre-decision** state, not the shipped one. Sparse vectors are native, and
   the request type is `MemorySearchRequest` (not `VantaMemorySearchRequest`).
2. §Decision #2 located `SparseVector` in `src/node.rs`, a file that no longer
   exists. The type lives in **`src/node/vector_data.rs:30`**, next to
   `DistanceMetric` (`src/node/vector_data.rs:11`).

Per [ADR-0000](./ADR-0000-adoptamos-adrs.md) ("a claim that is simply *wrong
about the code* is corrected in place with a dated verification note naming the
`file:line` that settles it"), ADR-0011's body text is left unedited and the
correction is recorded here, with the verified references below.

## Context and problem statement

ADR-0011 was written when VantaDB had exactly one vector channel: a dense
`Vec<f32>` query vector fused with BM25 text through Reciprocal Rank Fusion
(`fuse_rrf_with_report`). The backlog asked for **native sparse vectors** —
"hybrid search real" — meaning a second, user-supplied lexical channel with
its own similarity metric, not a re-skin of the dense path.

Two ambiguities had to be resolved before implementation:

1. **Input semantics** — is the sparse vector direct user input (a map
   `dim → value`), or is it derived internally from text (SPLADE-style learned
   sparsity)?
2. **Coexistence** — does sparse replace BM25 lexical text search, or
   complement it as a second lexical channel?

The tree at 0.9.0 answers both, and the answers are now load-bearing: the
`SparseVector` type, the `DistanceMetric::SparseDot` variant, the brute-force
sparse search path, the `SPARSE_VECTOR_EXT_KEY` persistence key, and the
`fuse_rrf_many` multi-channel fusion are all shipped and referenced by public
API contracts.

## Considered options

### Option A — Sparse vectors as direct user input (ADR-0011's choice)

- Pros: zero-training, drop-in addition; the user already owns the encoder
  (BM25-style, TF-IDF, or a learned sparse embedder). No change to ingestion
  semantics, no new model dependency.
- Cons: the caller must supply the sparse vector; VantaDB stores and scores
  it but does not derive it.

### Option B — Derive sparsity from text (SPLADE-style learned expansion)

- Pros: the caller only supplies text; the engine owns the whole channel.
- Cons: requires a trained model and a runtime dependency; changes ingestion
  semantics (the vector is no longer a function of the record alone, so
  re-embedding, export/import and snapshot replay all need a model-version
  contract). A SPLADE dependency was not justified for the 0.x surface.

### Option C — Drop BM25 and let sparse replace the lexical channel

- Pros: one lexical channel instead of two; simpler fusion.
- Cons: BM25 is the only channel that works without embeddings (BYO-vector is
  the default posture of the product). Removing it would regress the
  zero-embedding path.

## Decision outcome

**Option A — sparse vectors are direct user input, they coexist with BM25, and
they are native.** Concretely, in the 0.9.0 tree:

| Element | Verified location |
|---|---|
| `SparseVector(pub BTreeMap<u32, f32>)` — dimension → coefficient | `src/node/vector_data.rs:30` |
| `DistanceMetric::SparseDot` — sparse dot product, higher = more similar | `src/node/vector_data.rs:17-20` |
| `DistanceMetric` enum itself (Cosine default, Euclidean, SparseDot) | `src/node/vector_data.rs:11` |
| Persistence key `__vanta_sparse_vector` | `src/sdk/serialization/mod.rs:53` |
| Sparse → field / field → sparse conversion | `src/sdk/serialization/mod.rs:324,336` |
| Sparse search: dedicated brute-force path (never routed through the dense HNSW helpers) | `src/index/graph/core.rs:422,596`, `src/index/search/nearest.rs:139`, `src/index/distance/mapper.rs:146,191` |
| Multi-channel RRF fusion (`fuse_rrf_many` sums contributions across channels) | `src/sdk/search/fusion.rs:139` |
| Shared RRF contribution helper used by both the 2-way and N-way paths | `src/sdk/search/fusion.rs:177` |

Consequences of the choice, unchanged from ADR-0011:

1. **Representation** — `BTreeMap<u32, f32>` for deterministic serialization
   (stable byte order across runs) and an ordered dot-product walk.
2. **Metric** — plain dot product (higher = better). Sparse vectors are **not**
   indexed in the HNSW graph; they are searched by brute-force scan, which
   keeps the dense index untouched and makes the sparse search exact.
3. **Coexistence** — BM25 remains the text-token channel, sparse is the
   user-supplied dimension-vector channel, dense is the vector channel, and
   fusion stays RRF over the shared contribution helper.
4. **Persistence** — the sparse vector is stored under the
   `SPARSE_VECTOR_EXT_KEY` relational key, so no field is added to the node
   format and old databases load as-is (missing sparse = `None`).

### Positive consequences

- Hybrid search can combine three independent signals without a new index
  topology and without touching the dense path.
- The sparse channel works with BYO-vector (no embedding provider needed).
- No data migration: the sparse channel is additive and optional.

### Negative consequences

- Sparse search is O(n) per partition (no inverted index over dimensions).
  Acceptable for the native-sparse contract; a dimension-inverted index is a
  future optimisation, not part of this ADR.
- Export/import of sparse vectors needs the persistence format to be extended
  in lockstep (see [ADR-0019](./ADR-0019-sparse-vector-persisted-format.md) for
  the persisted format decision).
- No text→sparse derivation: the user must supply the sparse vector.
