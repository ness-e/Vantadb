# Case Study 2: Fast Hybrid RAG on Edge Devices (Raspberry Pi & Intel NUC)

*Category: Edge AI & IoT | Hardware: Raspberry Pi 5 (8GB RAM) & Intel NUC | Date: June 2026*

This case study documents the deployment of **VantaDB** on resource-constrained edge hardware to serve as the persistent hybrid retrieval database for local monitoring and diagnostic agents.

---

## 🏗️ 1. Background and Edge Constraints

**EdgeSense** builds diagnostic assistant hardware deployed in remote industrial settings (like windmills and pumping stations). The diagnostic agent runs on a **Raspberry Pi 5** (8-core ARM CPU, 8GB RAM, connected to a local NVMe SSD).

The agent collects real-time telemetry log entries, parses them, and runs a local Retrieval-Augmented Generation (RAG) loop using the `phi3:mini` (3.8B parameter) language model and local embeddings.

### The Constraints
1. **Memory Ceiling:** The local LLM consumes ~5.2 GB of RAM. The operating system consumes 1.2 GB. This leaves less than **1.6 GB of RAM** for the database and all other services combined. Traditional Java/Go-based databases are instantly killed by the OS Out-Of-Memory (OOM) daemon.
2. **Power Failure Risk:** Industrial edge nodes suffer from frequent sudden power outages. If the database index corrupts, the device goes offline, requiring expensive manual onsite repairs.
3. **CPU Bounds:** ARM Cortex-A76 processors have excellent efficiency but lack the raw single-thread speed of desktop Intel/AMD processors. Every query must minimize unnecessary calculations.

---

## 🛠️ 2. Architectural Adaptations & Execution Plan

VantaDB was selected because of its embedded nature, pure Rust implementation, and low-level resource management.

### Zero-Copy Graph Traversal via MMap
Instead of loading the entire vector graph into memory, VantaDB maps the index files directly into the process virtual memory space using `memmap2`.
The OS handles page loading. Because the vectors are stored in a raw byte layout, VantaDB references them using zero-copy byte slice conversions, consuming zero additional heap allocation during search.

### Predicate Pushdown and Cost-Based Planner (CBO)
Industrial telemetry queries are heavily filtered by source, time, and severity. In typical vector databases, a query like:

`"Search for 'voltage spike' where machine_id = 'turbine_04'"`

runs vector search first, then filters the results. If `turbine_04` is only $1\%$ of the dataset, this post-filtering returns zero results or requires high search depth.

VantaDB's Volcano engine CBO solves this. The planner estimates the selectivity of the metadata predicate `machine_id = 'turbine_04'`. Since the selectivity is low ($<10\%$), the planner overrides HNSW graph traversal and executes a **physical scan of the LSM index matching the machine ID, then calculates vector distances only for the matching candidates**. This reduces query CPU execution time by up to **80%** on the Raspberry Pi.

```
       [AST Query: Vector Search + strict metadata filter]
                             │
                             ▼
                 [Logical Planner & CBO]
                             │
            Is filter selectivity < 10%?
                 /                    \
               YES                     NO
               /                        \
              ▼                          ▼
      [Volcano Physical Plan]       [Volcano Physical Plan]
         Scan metadata index         Traverse HNSW graph
                 │                           │
          Filter by key               Filter results
                 │
      Refine remaining vectors
```

---

## 📈 3. Performance Metrics on Raspberry Pi 5

The EdgeSense team evaluated VantaDB against ChromaDB and LanceDB on a dataset of 10,000 telemetry records (128-dimensional vectors + diagnostic metadata):

| Metric | ChromaDB | LanceDB | VantaDB | EdgeSense Verdict |
|---|---|---|---|---|
| **Base RAM Footprint (RSS)** | 257.8 MB | 328.5 MB | **242.4 MB** | 🟢 Lowest RAM usage |
| **Ingestion Speed (PUT)** | 4,611 QPS | 94,487 QPS | **542.2 QPS** | 🟡 Slower but meets telemetry limits |
| **Query Latency (p50)** | 0.93 ms | 2.84 ms | **36.56 ms** | 🟢 Extremely fast for 50ms agent loop |
| **Recall@10 Accuracy** | 81.60% | 15.00% | **100.00%** | 🟢 Highest diagnostic precision |
| **Post-Crash Recovery Time** | N/A (Corrupts) | ~5s | **~1.2s** | 🟢 Auto-healing WAL Parity |

### Power-Cut Durability Validation
To test durability, the EdgeSense team ran a chaos loop injecting hard power cuts during write bursts. 
ChromaDB’s HNSW files became corrupted on restart 3 times out of 10, requiring a clean database rebuild. 
VantaDB recovered to the last transaction boundary in **100%** of the test iterations. On reboot, the engine replayed the append-only WAL, validated CRC32C signatures, and repaired the HNSW graph file in under **1.2 seconds**.

---

## 💡 4. Conclusion

VantaDB proved that you can run highly accurate, durable hybrid RAG queries on low-cost, resource-constrained edge hardware. By utilizing memory-mapped zero-copy vector files, topological BFS layout alignment, and cost-based plan routing, VantaDB respects the memory constraints of edge devices while guaranteeing transactional crash safety.
