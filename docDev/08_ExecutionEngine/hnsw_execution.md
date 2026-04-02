# Physical Plan Execution & Authentic HNSW
> **Status**: 🟡 In Progress — FASE 6

## 1. Executor Pipeline
The Executor takes a `LogicalPlan` from the Query Engine and physicalizes it:
1. Translates `Scan` to an iterator over `StorageEngine`.
2. Translates `FilterRelational` into an active evaluation over `UnifiedNode` properties.
3. Translates `VectorSearch` into calls against the `CPIndex` utilizing real Cosine Similarity and L2 distances.
4. Returns an aggregated `QueryResult`.

## 2. Real Cosine Similarity
To evaluate accuracy within the MVP, we compute Cosine Similarity utilizing Rust's standard FP32 operations. `VectorData(Vec<f32>)` represents our dimensions.

## 3. The Routing Algorithm
HNSW searches employ a greedy Best-First-Search navigating through `CPIndex.nodes`.
If `node.bitset & query_mask == query_mask`, we examine `CosineSimilarity(query_vec, node.vec_data)`.
