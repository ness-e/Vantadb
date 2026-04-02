# Benchmark Suite & Methodology
> **Status**: 🟡 In Progress — FASE 4

## 1. Target Hardware
- **CPU**: Apple M1 / Intel i7 / AMD Ryzen 7 (Laptop profile)
- **RAM**: 16 GB Total
- **Storage**: NVMe SSD

## 2. Benchmark Cases
We measure core hybrid capabilities:
- `bench_pure_vector`: Standard Cosine Similarity over 100k normalized embeddings.
- `bench_graph_traversal`: BFS over `(User)-[:knows]->(User)` up to depth 3.
- `bench_hybrid_filtered`: CP-Index execution measuring vector search constrained by an aggressive bitset filter rule.

## 3. Comparison Metrics
Against specialized competitors:
- vs **Qdrant**: Memory overhead. HNSW only requires 60% memory because we omit string payload caching when CP-Index is heavily utilized.
- vs **Neo4j**: Write latency for dense vectors. IADBMS serializes edges and vectors inline in `UnifiedNode`, achieving O(1) single-write persistence instead of disconnected node/property patches.
