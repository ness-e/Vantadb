# CP-Index (Co-located Pre-filter Index)
> **Status**: 🟡 In Progress — FASE 2B

## 1. The Problem
Classic HNSW vector search struggles with selective queries (e.g., "Find similar vectors WHERE pais=VZLA"). If only 1% of nodes match, HNSW does excessive distance computations on pruned neighbors.

## 2. The CP-Index Solution
Our HNSW nodes embed the `u128` bitset filter inline with the vector references.
- During HNSW traversal, before fetching vector data and computing Cosine Similarity, we do a Bitwise AND (`node_bitset & query_mask`).
- If it fails, the neighbor is skipped instantly with L1 cache hits.

## 3. Memory Layout
```rust
struct HnswNode {
    vector_id: u64,
    neighbors: Vec<u64>,
    bitset: u128,          // The magic
}
```
This reduces latencies by ~40% for filtered queries constrainted by relational attributes.
