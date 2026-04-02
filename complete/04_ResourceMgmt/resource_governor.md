# Resource Governor & Circuit Breaker
> **Status**: 🟡 In Progress — FASE 3A

## 1. OOM Protection (Out-Of-Memory Guard)
Queries such as unbounded graph traversals or massive brute-force vector scans can consume large amounts of memory.
The Resource Governor monitors active memory allocations. If the soft limit (e.g., 2GB overhead in our 16GB total budget) is crossed, it trips the Circuit Breaker.

## 2. TEMPERATURE Control Execution
The query syntax `WITH TEMPERATURE <0.0..1.0>` controls exhaustiveness versus resource usage:
- **T = 0.0 (Exhaustive)**: Follows all edges, executes full searches. Ignores soft resource constraints until a hard OOM is imminent.
- **T = 1.0 (Probabilistic/Aggressive)**: 
  - Restricts BFS traversal queue to max 100 paths.
  - Skips full vector distance computations if HNSW neighbors already yield a high score.
  - Aborts immediately if execution time > 50ms.

## 3. Circuit Breaker States
- **Closed**: Normal execution.
- **Half-Open**: Governor periodically checks if memory GC has freed enough RAM. Queries run with forced T=1.0.
- **Open**: Rejects all complex hybrid queries. Only allows simple lookups by ID or lightweight inserts.
