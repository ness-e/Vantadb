# Inference Query Language (IQL) Specification

NexusDB abandons the complexity of standard SQL JOINs and Graph query languages (like Cypher) by combining traversing arrays and geometric similarities into a unified functional grammar. We call this the **Inference Query Language (IQL)**.

## 1. Core Grammar & Philosophy

IQL is designed explicitly so that **Machine Learning queries (Vectors)** and **Deterministic attributes (Graphs/Relational)** can be executed simultaneously during the same abstract syntax tree traversal, maximizing speed.

### Standard Pipeline Syntax
When utilized via the raw engine text-parser:

```sql
VECTOR search ~ [0.12, 0.44, ...] 
WHERE category == "technology" AND confidence > 0.8
WITH DEPTH 2 
LIMIT 5
```

### Deconstructing the Operands:
*   `VECTOR search ~ [n]`: The tilde (`~`) operator triggers native HNSW Cosine Similarity traversal using the provided dimensional slice. Mapped to physical space instantly.
*   `WHERE`: Standard BTreeMap filtering. The engine evaluates equality (`==`), comparators (`<, >, >=`), and booleans. If pre-filtering is faster (via cardinality limits), NexusDB applies it *before* the HNSW execution.
*   `WITH DEPTH`: Graph traversal initiator. Dictates the max recursion of adjacency list jumps from candidate nodes.
*   `LIMIT`: The HNSW `top_k` threshold.

---

## 2. Production Examples

### A. Complex RAG System (Retrieval-Augmented Gen)
Filter documents that belong exclusively to the `company_internal` tag, while finding the closest vector distance, ignoring stale documents.

```python
# In standard PyO3 SDK SDK
results = db.search(
    vector=query_embedding,
    top_k=5,
    filter_expr="category == 'company_internal' AND is_stale == false"
)
```

### B. Graph Recommendations (E-Commerce)
Find elements semantically similar to this `product_vector`, but constrain the search exclusively to nodes connected via edge relation (a verified `PURCHASED_WITH` chemical connection).

```python
results = db.search(
    vector=product_vector,
    top_k=10,
    graph_constraint="EDGE_TYPE == 'PURCHASED_WITH'",
    depth=1
)
```

### C. Fraud Analysis Ring
Check geographical IP locations attached as attributes, combined with a behavioral embedding space, allowing the engine to traverse 2 degrees of connections (finding connected fraudulent wallets/users).

```python
# Uncovering a ring by going multiple hops into the metadata
results = db.search(
    vector=behavior_embedding,
    top_k=3,
    filter_expr="geo_risk_score >= 80",
    depth=2 
)
```

## 3. Weight Management & Operability

Every edge connecting two nodes inside NexusDB operates natively with an intrinsic `weight` (f32).

When chaining graph searches with vector searches, if an edge weight degrades drastically (e.g., `weight < 0.2`), NexusDB's executor interprets it as an asynchronous disconnection and will halt traversal early, effectively self-pruning noisy pathways in memory.
