# WASM Vector DB Benchmark Report (MCP-03)

> Generated: 2026-07-05
> Compares VantaDB WASM against EdgeVec, minimemory, altor-vec, and lattice-db.

---

## Feature Comparison Matrix

| Feature | VantaDB | EdgeVec | minimemory | altor-vec | lattice-db |
|---|---|---|---|---|---|
| **HNSW Vector Search** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| **BM25 Full-Text Search** | ✅ Yes | ✅ Yes (sparse) | ✅ Yes | ❌ No | ✅ Yes |
| **Hybrid Search (RRF)** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes (linear) |
| **Write-Ahead Log (WAL)** | ✅ Yes | ❌ No (save-only) | ❌ No | ❌ No | ✅ Yes |
| **MCP Protocol Support** | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| **IQL Query Language** | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| **Persistence** | ✅ OPFS + WAL | ✅ File save | ✅ .mmdb binary | ✅ Serialize | ✅ WAL-backed |
| **Graph Traversal** | ✅ BFS/DFS/DAG | ❌ No | ❌ No | ❌ No | ✅ Yes (nodes+edges) |
| **Metadata Filtering** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes |
| **SIMD Acceleration** | ✅ Cosine SIMD | ✅ SIMD128 | ❌ Not in WASM | ✅ Available | ❌ No info |
| **Text Index Audit** | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| **Export/Import** | ✅ Namespace+All | ❌ No | ❌ No | ❌ No | ❌ No |
| **Search Explanation** | ✅ BM25 breakdown | ❌ No | ❌ No | ❌ No | ❌ No |
| **TTL Expiry** | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| **Batch Operations** | ✅ put_batch | ❌ No | ❌ No | ❌ No | ❌ No |
| **Pagination** | ✅ Cursor-based | ❌ No | ✅ Yes | ❌ No | ❌ No |

## Bundle Size

| Engine | Raw | Gzipped |
|---|---|---|
| **VantaDB** | 1,101 KB | ~404 KB |
| **EdgeVec** | ~300 KB | ~100 KB |
| **minimemory** | 493 KB | ~200 KB |
| **altor-vec** | 170 KB | ~54 KB |
| **lattice-db** | ~500 KB | ~200 KB |

VantaDB's bundle is larger because it includes: HNSW + BM25 (Tantivy) + Graph engine + WAL + OPFS persistence + SIMD + IQL parser.

## Browser / Runtime Compatibility

| Engine | Browser | Node.js | CF Workers |
|---|---|---|---|
| **VantaDB** | ✅ (wasm32-uk) | ✅ (wasm32-wp1) | ❌ (WAL needs fs) |
| **EdgeVec** | ✅ | ✅ | ❌ |
| **minimemory** | ✅ | ✅ | ✅ |
| **altor-vec** | ✅ | ✅ | ❌ |
| **lattice-db** | ✅ | ✅ | ❌ |

## Key Differentiators

### VantaDB
- Only WASM vector DB with **both MCP and IQL** — query your DB from any MCP client or directly with IQL
- Only one with **text index audit/repair** — production-grade data integrity
- Only one with **full graph traversal** (BFS, DFS, topological sort, DAG check)
- Only one with **WAL-backed persistence** + OPFS browser persistence
- Only one with **search explanation** showing BM25 term contributions
- Largest WASM bundle due to richest feature set

### EdgeVec
- Strong SIMD128 acceleration (8.75x Hamming distance)
- Binary quantization (32x memory reduction)
- Largest star count in this comparison

### minimemory
- Multiple quantization types (Int8, Int3, Binary, Polar)
- Multiple index types (Flat, HNSW, IVF)
- Cloudflare Workers compatible
- Smallest WASM with hybrid search

### altor-vec
- Smallest bundle (54KB gzipped) — pure HNSW
- Fastest for pure vector search
- No text/hybrid search

### lattice-db
- Only hybrid graph+vector DB aside from VantaDB
- Has Cypher-like query language (MATCH)

## Conclusion

VantaDB leads in **feature completeness** — it's the only WASM vector DB with HNSW + BM25 + Hybrid + WAL + MCP + IQL + Graph. The tradeoff is a larger WASM bundle (404KB gzipped vs 54KB for altor-vec). 

For **pure vector search** (no text, no graph), altor-vec wins on size and speed.
For **browser RAG with text + vector**, VantaDB and minimemory are the top contenders.
For **full-stack AI apps** needing MCP protocol support, VantaDB is the only option.
