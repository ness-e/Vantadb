# SQLite for AI Agents: Benchmarks and Architecture Decisions

*By the VantaDB Team*

If you are building an application that runs locally, there is a $99\%$ chance you default to SQLite for storage. It is simple, single-file, highly optimized, and has decades of proven production reliability. 

But when we build **local-first AI agents**, the workload profile changes dramatically. Agents do not perform traditional relational transactions with tables of integers and strings. Instead, they continually stream text chat turns, write planning reflections, log execution steps, and insert high-dimensional dense vectors (often 100+ times per hour).

When designing **VantaDB**—an embedded hybrid retrieval engine written in Rust—we had to make fundamental database architecture decisions. SQLite was our baseline inspiration, but to optimize for agent memory, we chose a very different engineering path.

Here are the design trade-offs, architecture decisions, and benchmark insights behind VantaDB.

---

## 1. Storage Backend: LSM-Tree vs. B-Tree

SQLite uses a classic **B-Tree** structure. B-Trees are optimized for random read workloads, but they write by updating pages in-place, which requires heavy random disk write operations and page splits.

For AI agent memory, we chose a **Log-Structured Merge-Tree (LSM)** backend (Fjall by default, with RocksDB as an optional feature flag).

```
   [Write Path]               [Memory]                [Disk]
   
   Insert Record ───► [Write-Ahead Log]
         │
         ▼
    [MemTable] ──(Flush)──► [SSTable Layer 0] ──(Compaction)──► [Layer 1]
```

### Why LSM-Tree for Agents?
* **Write Throughput:** Agents write continuously (logs, tool reflections, intermediate thoughts). LSM-trees convert random writes into sequential writes in an append-only Write-Ahead Log (WAL) and memory table (`MemTable`). This results in orders of magnitude higher write throughput.
* **Derived Index Rebuilds:** HNSW vector graphs and BM25 postings lists are *derived indexes* in VantaDB. When we execute an index rebuild (`rebuild_index`), the engine performs a sequential scan of the LSM-tree canonical records. LSM-trees excel at sequential scans since data in SSTables is physically ordered by key on disk.

---

## 2. Memory-Mapped I/O and the BFS Layout Compaction

A major challenge for vector search on edge devices is RAM usage. A dataset of 100K 1536-dimensional vectors requires ~600MB of raw floats, plus the graph links of HNSW. Keeping this entirely in-memory limits execution on low-spec hardware.

VantaDB uses **Memory-Mapped Files (`memmap2`)** to back its HNSW graph. This lets the operating system manage which pages of the graph reside in RAM, loading pages on-demand.

### The Random Read amplification Problem
HNSW graph traversal is inherently graph-walk based. The search jumps from node to node, traversing links. If the nodes are written to disk in random order of insertion, each hop during a query traverses page boundaries, causing the OS to trigger a **page fault** and read from physical disk. Under search stress, this drops performance by $10x$.

### The Solution: BFS Layout Compaction
To restore cache locality, VantaDB implements a topological re-layout engine:
1. When the user compacts or rebuilds the index, VantaDB executes a Breadth-First Search (BFS) traversal of the HNSW graph starting from the Layer 0 entry point.
2. Nodos are written to the physical file *in the exact order they are visited during the BFS*.
3. Since HNSW queries naturally traverse nodes topolocically close to the entry point first, this BFS layout guarantees that nodes accessed together are co-located on the same physical $4\text{KB}$ page.

```
Logical Graph:    [Entry Point] ──► [Neighbor 1] ──► [Neighbor 2]
                         │
                         ▼
                    [Neighbor 3]

Physical Disk (BFS): [Entry | Neighbor 1 | Neighbor 3 | Neighbor 2]  ◄── Same Page!
```

**Benchmark Impact:** In our stress benchmarks, running search queries on an HNSW graph compacted with the BFS layout reduced physical OS major page faults by **59%**, increasing search throughput from 750 QPS to **1,195 QPS** under memory pressure.

---

## 3. PyO3 Boundary Optimization and GIL Safety

Most AI agent orchestration frameworks (like LangChain, LlamaIndex, or AutoGen) are written in Python. Consequently, VantaDB exposes a native Python package (`vantadb-py`) built using **PyO3** bindings to the core Rust library.

The PyO3 FFI (Foreign Function Interface) boundary introduces two major performance bottlenecks:
1. **FFI Call Overhead:** Crossing the boundary between Python's virtual machine and Rust's native machine code.
2. **The Python GIL (Global Interpreter Lock):** Python only allows one thread to execute at a time. If the database engine blocks the GIL while running searches, concurrent Python code stalls.

### Amortizing Boundary Costs with Batching
To solve FFI overhead, VantaDB implements `search_batch(queries, top_k)`. Instead of Python looping and making $N$ separate calls to Rust:
1. Python passes a list of search objects in a single FFI call.
2. Rust immediately converts the parameters to native types.
3. **GIL Release:** Rust releases the Python GIL using `py.allow_threads()`.
4. **Native Parallelism:** Rust runs the queries concurrently across all CPU cores using `Rayon`.
5. **Re-acquire GIL:** Rust re-acquires the GIL at the very end to build the final Python list wrapper.

**Benchmark Impact:** Our batch query benchmark shows a **4.01x speedup** when executing 10 concurrent queries via `search_batch` compared to sequential Python loops, dropping latency to **2.43ms** per query.

---

## 4. Durability & Parity Verification

To ensure VantaDB is reliable enough to serve as a production memory store, we maintain a strict **Reliability Gate**:

* **RSS Memory Stability:** Our hardware profile test suite inserts 100K high-dimensional vectors in a continuous loop for 30 minutes, validating that heap memory drift remains under **10%** and RSS remains stable, eliminating leaks.
* **Failpoint Chaos Loops:** Using Rust `fail` crate, we simulate system failures at key points (e.g. WAL append write fail, disk sync fail, index mmap write fail). The database engine is tested to guarantee that it recovers to a consistent state from the WAL and LSM files without panic.

---

## Conclusion: Engineering a Tailored Memory Engine

SQLite is the best database for embedded relational data. But for AI agent memory, VantaDB’s architectural deviations—LSM storage for append-heavy writes, topological BFS layouts on memory-mapped graphs, SIMD accelerations, and GIL-free batch query parallelization—provide the performance and safety edge AI agents deserve.

To run the hardware profiles and benchmarks on your own device, clone the repository at [GitHub: ness-e/Vantadb](https://github.com/ness-e/Vantadb).
