---
title: How Hybrid Search Actually Works
kind: howto
status: active
description: How lexical and vector candidates contribute to native rank fusion, without confusing embeddings with index algorithms
tags: [hybrid-search, bm25, rrf]
---

# How Hybrid Search Actually Works

*By the VantaDB Team*

Lexical and vector retrieval provide different evidence. BM25 ranks token matches;
an embedding model maps text to vectors, and an index such as HNSW searches those
vectors. Neither method wins every query. Combining their candidate rankings can
help when an agent needs both exact terminology and paraphrase matches.

VantaDB runs these retrieval operations in-process. Vector-only, text-only, and
combined requests serve different needs; hybrid search does not remove the need
to evaluate retrieval quality on your own corpus.

In this article, we will dissect the engineering behind the hybrid search architecture of **VantaDB**—an embedded database engine written in Rust—detailing how BM25 lexical search and HNSW vector search are executed, planned, and fused at the lowest levels of the engine.

---

## 1. The Complementary Retrieval Models

Before diving into the code, let's look at the mathematical and logical differences between the two search paradigms:

### Lexical Search (BM25)
BM25 (Best Matching 25) is a sparse retrieval model based on term statistics. It scores documents based on the occurrence of query terms:

$$\text{Score}(D, Q) = \sum_{i=1}^{n} \text{IDF}(q_i) \cdot \frac{f(q_i, D) \cdot (k_1 + 1)}{f(q_i, D) + k_1 \cdot \left(1 - b + b \cdot \frac{|D|}{\text{avgdl}}\right)}$$

* **Strengths:** High precision for exact words, product codes, telephone numbers, code symbols (`Header`), and negative keyword filtering.
* **Weaknesses:** Cannot handle synonyms (e.g., matching "compute" to "processor") or conceptual similarity.

### Vector search (HNSW)

HNSW is an approximate nearest-neighbor index, not an embedding model. Semantic
and cross-lingual behavior depends on the encoder and corpus; the index returns
nearby vectors according to the requested distance metric.

Local configurations include `multilingual-e5-small` (384d) and
`EmbeddingGemma 2` (768d). Supply compatible document and query embeddings:
matching dimensions alone does not establish matching model spaces.

---

## 2. VantaDB's Lexical Subsystem (BM25 in LSM)

VantaDB provides native lexical indexing rather than requiring a separate
search service. Lexical indexing and storage recovery are different concerns;
the existence of a BM25 index does not establish an atomicity guarantee.

### Tokenization and Indexing
Payload text is tokenized for lexical retrieval. Tokenization and query syntax
depend on the configured tokenizer and API; do not assume ASCII-only behavior
or that every punctuation-bearing identifier is matched as one token.

### Quoted Phrase Search Support
Check the selected API's text-query syntax before relying on quoted-phrase
behavior. A token-ranking explanation is not a specification for every query
entry point.

---

## 3. VantaDB's Semantic Subsystem (SIMD HNSW)

For semantic search, VantaDB implements a Hierarchical Navigable Small World (HNSW) graph.

### Multi-Layer Traversal
The graph is organized into layers. The search begins at the top layer (`max_layer`), where nodes are sparse, allowing the engine to traverse large topological distances quickly. Once a local minimum is reached in a layer, the search drops to the next layer down, using the current minimum as the entry point, until reaching Layer 0, where the nearest neighbors are collected:

```
Layer 2:  [Entry] ───► [Node A]
                           │
                           ▼
Layer 1:               [Node A] ───────► [Node B]
                                             │
                                             ▼
Layer 0:                                 [Node B] ───► [Result (K=1)]
```

### SIMD Acceleration
Distance kernels live under `src/index/distance/`. Available instruction sets
and performance depend on the build and CPU. This article does not provide a
new SIMD benchmark or a standalone replacement for those implementations.

---

## 4. Query entry points and filtering

VantaDB has both SDK search operations and a query-language execution path.
An illustrative physical-plan diagram is not evidence that every Python
`Client.search()` request goes through the same planner.

Use the namespace, metadata filters, query vector, text query, and result limit
supported by your chosen SDK operation. Inspect the actual search implementation
under `src/sdk/search/` when diagnosing candidate selection; do not assume a
universal hard-coded selectivity threshold or identical behavior across query APIs.

---

## 5. Reciprocal Rank Fusion (RRF)

BM25 scores and vector distances are not interchangeable. In particular,
Euclidean distance is not generally bounded by 2. Rank fusion combines positions
in candidate lists rather than pretending these raw score scales are equal.

VantaDB solves this by performing **Reciprocal Rank Fusion (RRF)**. RRF merges candidate lists based purely on their *rank* (position in the result list) rather than their raw score. The fused score for document $d$ is computed as:

$$\text{RRF Score}(d) = \frac{1}{k + r_{\text{lexical}}(d)} + \frac{1}{k + r_{\text{semantic}}(d)}$$

Where:
* $r(d)$ is the 1-based rank of document $d$ in the respective retrieval list (set to $\infty$ if the document did not appear in that list).
* $k$ is a positive smoothing parameter. The expression illustrates two
  equally weighted lists; consult the implementation for the active parameters.

RRF is deterministic for fixed candidate lists and ordering. It can reward
agreement between lists, but does not guarantee relevance or make the retrieval
system parameter-free. The native fusion code is in
[`src/sdk/search/fusion.rs`](../../../src/sdk/search/fusion.rs).

---

## Conclusion

VantaDB combines native lexical and vector retrieval without a separate search
service. That is a deployment property, not proof of superior recall or latency.
Use [the benchmark reference](../operations/BENCHMARKS.md) for measured workloads.
Prebuilt wheels are available on supported platforms; source builds require the
Rust and C/C++ toolchains.

Want to see the query planner and RRF in action? Get the engine in one line:

```bash
pip install vantadb-py
```

Join the community on Discord and star the [VantaDB repository](https://github.com/ness-e/Vantadb). For a deeper look at why we chose LSM-trees and BFS compaction over a B-tree, read [SQLite for AI Agents: Benchmarks and Architecture Decisions](./sqlite_for_ai_agents.md).
