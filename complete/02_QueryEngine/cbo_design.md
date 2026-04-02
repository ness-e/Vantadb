# Cost-Based Optimizer (CBO) Design — MVP
> **Status**: 🟡 In Progress — FASE 2A

## 1. Overview
The IADBMS CBO decides the execution order of cross-model operations. Classical CBOs focus on relational joins. Our CBO focuses on:
1. Bitset filtering (Fastest)
2. Graph Traversal (`SIGUE`)
3. Relational Filtering
4. Vector Similarity (`~`) (Slowest without CP-Index)

## 2. Execution Heuristics (Fase 2)
In the MVP, we use statically defined heuristics:
- **Condition Cost**: 
  - `Bitset Mask` = O(1)
  - `Relational Eq` = O(N) strings/ints
  - `Graph Traversal` = O(V + E) bounded by depth
  - `Vector Sim` = O(N * D) dot products
- **Rule**: Always push down Bitsets. Traverse graph before Vector Similarity unless Vector Similarity has a CP-Index.

## 3. Semantic Cost Estimator (SCE) (Fase 3)
Instead of pure heuristics, FASE 3 introduces SCE:
- **Density Metadata**: Tracks average out-degree for specific edge labels.
- **Radius Entropy**: Approximates the selectivity of vector searches.
- If a vector query `min` score > 0.95, it's highly selective. Vector search is executed *first* to seed the traversal.

## 4. Temperature Parameter
`WITH TEMPERATURE 0.0 - 1.0` controls the query planner's exhaustiveness.
- **T = 0.0**: `WITH EXHAUSTIVE`. Evaluates all paths, ensures 100% accurate recall.
- **T = 1.0**: Aggressive pruning. Uses HNSW approximate neighbor routing and limits graph BFS queue size for fast, probabilistic answers (e.g., for LLM top-k context).
