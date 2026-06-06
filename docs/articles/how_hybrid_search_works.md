# How Hybrid Search Works: BM25 + HNSW + RRF in Practice

*By the VantaDB Team*

Modern search systems are increasingly hybrid. Semantic search using dense vector embeddings (like HNSW) has revolutionized how applications find conceptually related information. However, semantic search has a famous blind spot: it is terrible at matching exact keywords, unique identifiers, serial numbers, or specific terminology. For that, classic lexical retrieval (like BM25) remains undefeated.

To give autonomous AI agents the best possible context window payload, we cannot choose between the two. We must run both. 

In this article, we will dissect the engineering behind the hybrid search architecture of **VantaDB**—an embedded database engine written in Rust—detailing how BM25 lexical search and HNSW vector search are executed, planned, and fused at the lowest levels of the engine.

---

## 1. The Complementary Retrieval Models

Before diving into the code, let's look at the mathematical and logical differences between the two search paradigms:

### Lexical Search (BM25)
BM25 (Best Matching 25) is a sparse retrieval model based on term statistics. It scores documents based on the occurrence of query terms:

$$\text{Score}(D, Q) = \sum_{i=1}^{n} \text{IDF}(q_i) \cdot \frac{f(q_i, D) \cdot (k_1 + 1)}{f(q_i, D) + k_1 \cdot \left(1 - b + b \cdot \frac{|D|}{\text{avgdl}}\right)}$$

* **Strengths:** High precision for exact words, product codes, telephone numbers, code symbols (`VantaHeader`), and negative keyword filtering.
* **Weaknesses:** Cannot handle synonyms (e.g., matching "compute" to "processor") or conceptual similarity.

### Semantic Search (HNSW)
HNSW (Hierarchical Navigable Small World) is a dense retrieval model that structures high-dimensional vectors (e.g., 1536-dimensional OpenAI embeddings or 384-dimensional local embeddings) into a hierarchical graph of links.
* **Strengths:** Captures conceptual relationships, semantics, intent, and cross-lingual matching.
* **Weaknesses:** High compute requirements, prone to "hallucinating" proximity (returning words that sound conceptually similar but miss the exact target key), and struggles with OOV (Out-Of-Vocabulary) terms.

---

## 2. VantaDB’s Lexical Subsystem (BM25 in LSM)

To avoid external dependencies and guarantee atomic persistence, VantaDB implements a custom BM25 index stored directly inside its Log-Structured Merge-tree (LSM) storage backend.

### Tokenization and Indexing
When a text payload is inserted into VantaDB:
1. The text is passed through our standard tokenizer, which strips punctuation, converts to ASCII lowercase, and outputs individual tokens.
2. The engine writes **postings lists** to the LSM storage. A posting list maps a token to a list of matching document keys, the frequency of the token in that document, and the exact token offsets (positions) for phrase matching.
3. Statistics (total document count, document lengths, and term document frequencies) are updated in a derived index namespace.

### Quoted Phrase Search Support
Because VantaDB stores token positions within the posting lists, it supports quoted phrase queries (e.g., `"MMap BFS layout"`). During query execution, the engine retrieves posting lists for each term and filters out documents where the tokens do not appear consecutively, ensuring lexical precision matching.

---

## 3. VantaDB’s Semantic Subsystem (SIMD HNSW)

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
To make graph traversal fast on local CPUs, VantaDB utilizes SIMD (Single Instruction, Multiple Data) intrinsics for distance calculations. Using the `wide::f32x8` crate, we compute Euclidean ($L_2$) and Cosine distances by processing 8 floating-point coordinates in a single CPU clock cycle:

```rust
// A snippet of VantaDB's SIMD L2 distance calculations
use wide::f32x8;

pub fn SIMD_l2_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut sum_sq = f32x8::ZERO;
    let chunks = a.len() / 8;
    
    for i in 0..chunks {
        let va = f32x8::from(&a[i * 8..]);
        let vb = f32x8::from(&b[i * 8..]);
        let diff = va - vb;
        sum_sq += diff * diff;
    }
    
    let mut total = sum_sq.reduce_add();
    // process remaining elements (modulo 8)...
    total
}
```

---

## 4. The Volcano Physical Planner & Cost-Based Optimizer (CBO)

Unlike primitive vector stores that perform vector search first and filter results post-hoc (often returning zero results if filters are strict), VantaDB uses a structured query planner based on the **Volcano Iterator Model**.

When a hybrid query arrives, it is parsed and compiled into a physical execution tree where each node implements an iterator trait (`open`, `next`, `close`):

```
       [LimitOperator]
              │
      [FuseRRFOperator]
         /         \
        /           \
  [BM25Operator]   [VectorSearchOperator]
        │                     │
  [FilterOperator]     [RefineOperator]
        │                     │
  [ScanOperator]       [FilterOperator]
```

### Cost-Based Optimizer Routing
Before execution, our CBO evaluates the selectivity of relational filters. It uses the following heuristics:
1. **High Selectivity ($<10\%$):** If a metadata query filters out more than $90\%$ of the collection, traversing the HNSW graph is inefficient (and can easily get stuck in isolated graph sub-graphs). The CBO compiles the query to a **Scan + Relational Filter + Vector Refine** plan.
2. **Low Selectivity ($>10\%$):** If the filter matches almost all documents, the CBO compiles the query to a **Vector Search + Relational Post-Filter** plan, utilizing the HNSW graph fully.

---

## 5. Reciprocal Rank Fusion (RRF)

Once the lexical operator and semantic operator produce their top candidates, their results must be merged. Score scales between BM25 (arbitrary positive floats) and HNSW (Cosine/L2 distances between 0 and 2) are incompatible. Normalizing them linearly requires knowing the minimum and maximum possible scores, which change per query.

VantaDB solves this by performing **Reciprocal Rank Fusion (RRF)**. RRF merges candidate lists based purely on their *rank* (position in the result list) rather than their raw score. The fused score for document $d$ is computed as:

$$\text{RRF Score}(d) = \frac{1}{k + r_{\text{lexical}}(d)} + \frac{1}{k + r_{\text{semantic}}(d)}$$

Where:
* $r(d)$ is the 1-based rank of document $d$ in the respective retrieval list (set to $\infty$ if the document did not appear in that list).
* $k$ is a constant smoothing factor (defaulting to $60$), which prevents high ranks from dominating the score disproportionately.

RRF is deterministic, computationally trivial, and guarantees that documents appearing high in both lists are promoted to the very top, providing a highly relevant context window payload for the AI agent.

---

## Conclusion

By executing lexical and semantic searches natively, coordinating them with a Volcano-style physical execution engine, and fusing results with Reciprocal Rank Fusion, VantaDB achieves a level of query performance and relevance typically reserved for complex, enterprise cloud setups—all inside a lightweight, zero-dependency embedded database.

To explore the query planner and run your own hybrid benchmarks, visit [GitHub: ness-e/Vantadb](https://github.com/ness-e/Vantadb).
