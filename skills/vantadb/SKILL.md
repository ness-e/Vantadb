---
name: vantadb
description: Expert guide for VantaDB - Embedded persistent memory and hybrid retrieval engine for AI agents. Covers installation, core operations (put, get, search, list), hybrid search (BM25 + HNSW + RRF), Python SDK usage, LangChain/LlamaIndex integration, and best practices for local-first AI agent memory.
---

# VantaDB - Embedded Persistent Memory for AI Agents

VantaDB is an embedded, local-first hybrid database engine designed specifically to act as long-term memory for autonomous AI agents. Think of it as a specialized SQLite tailored for agent payloads, integrating BM25 lexical retrieval and HNSW vector indexing in a single engine.

## Core Philosophy

**Embedded-First & Local-First**
- Runs in-process (in-memory address space) like SQLite
- No network overhead, no external dependencies
- Data resides on the user's machine for privacy and sovereignty
- Zero-copy memory mapping for vector indexes

**Multi-Model Hybrid Retrieval**
- Vector search via HNSW (Hierarchical Navigable Small World)
- Lexical search via BM25 (text indexing)
- Hybrid fusion via Reciprocal Rank Fusion (RRF)
- Structured metadata filtering

**Durable by Design**
- Write-Ahead Log (WAL) with CRC32C checksums
- Auto-healing recovery from crashes
- Backend: Fjall (Rust LSM-tree) by default, RocksDB optional
- Atomic checkpoints and crash-safe operations

## Installation

### Python SDK (Recommended)

```bash
# From source (requires Rust toolchain)
pip install maturin
cd vantadb-python
maturin develop --release

# From pre-built wheel (when available)
pip install vantadb-py
```

### Rust SDK

```toml
[dependencies]
vantadb = "0.5.0"
```

## Quick Start

```python
import vantadb_py as vantadb

# Initialize embedded database (256MB memory limit)
db = vantadb.VantaDB("./agent_memory", memory_limit_bytes=256_000_000)

# Store memory with vector and metadata
db.put(
    namespace="agent/session-1",
    key="ctx-001",
    payload="User prefers concise technical answers",
    metadata={"type": "preference", "priority": 2},
    vector=[0.8, 0.1, 0.5]  # Your embedding
)

# Hybrid search (semantic + lexical)
results = db.search_memory(
    namespace="agent/session-1",
    query_vector=[0.75, 0.2, 0.6],
    text_query="technical answers",
    top_k=3
)

# Retrieve specific memory
memory = db.get_memory("agent/session-1", "ctx-001")

# List all memories in namespace
memories = db.list_memory("agent/session-1", limit=10)

# Cleanup
db.flush()
db.close()
```

## Core Operations

### put()

Store a memory record with vector, payload, and metadata.

```python
db.put(
    namespace="agent/session-1",
    key="unique-key",
    payload="The content to store",
    metadata={"type": "note", "priority": 1},
    vector=[0.1, 0.2, 0.3, ...]  # Embedding vector
)
```

**Parameters:**
- `namespace`: Logical namespace for isolation (e.g., "agent/session-1")
- `key`: Unique identifier within namespace
- `payload`: Text content or structured data
- `metadata`: Dictionary of scalar fields for filtering
- `vector`: Float32 embedding vector (dimensionality must be consistent)
- `ttl_ms`: Optional TTL in milliseconds — the record expires after this time and is excluded from search; expired records are removed by `purge_expired()`

**Best Practices:**
- Use hierarchical namespaces (e.g., "agent/user-123/session-456")
- Include relevant metadata for filtering (type, priority, timestamp)
- Ensure vector dimensions are consistent across all records in a namespace
- Use `ttl_ms` for ephemeral state (caches, session data); call `purge_expired()` periodically to reclaim storage

### get_memory()

Retrieve a specific memory by key.

```python
memory = db.get_memory("agent/session-1", "ctx-001")
print(memory["payload"])
print(memory["metadata"])
```

**Returns:** Dictionary with `payload`, `metadata`, `vector` (if available), or `None` if not found.

### search_memory()

Perform hybrid search combining vector similarity and lexical matching.

```python
# Vector-only search
results = db.search_memory(
    namespace="agent/session-1",
    query_vector=[0.75, 0.2, 0.6],
    top_k=10
)

# Text-only search (BM25)
results = db.search_memory(
    namespace="agent/session-1",
    text_query="rust database optimization",
    top_k=10
)

# Hybrid search (RRF fusion)
results = db.search_memory(
    namespace="agent/session-1",
    query_vector=[0.75, 0.2, 0.6],
    text_query="rust database",
    top_k=10
)

# With metadata filters
results = db.search_memory(
    namespace="agent/session-1",
    query_vector=[0.75, 0.2, 0.6],
    top_k=10,
    filters={"type": "preference"}  # Only match records with type=preference
)
```

**Parameters:**
- `namespace`: Target namespace
- `query_vector`: Embedding for semantic search (optional)
- `text_query`: Text query for lexical search (optional)
- `top_k`: Number of results to return
- `filters`: Metadata key-value pairs for filtering
- `distance_metric`: `"cosine"` (default) or `"euclidean"`
- `method`: Optional index backend override: `"ivf"`, `"scann"`, `"hnsw"` or `"flat"`. Defaults to the engine's configured routing.
- `explain`: Include search explanations (default `False`)

**Returns:** List of `VantaSearchHit` with `key`, `payload`, `metadata`, `vector`, `score`, and timestamps.

**Performance Notes:**
- Hybrid search uses RRF (Reciprocal Rank Fusion) for deterministic ranking
- Vector search uses HNSW — p50 1.2ms at 10K vectors (128d, Cosine, AVX2), certified in `docs/operations/BENCHMARKS.md` (§1)
- Text search uses BM25 with persisted inverted index
- Use `search_batch()` for bulk queries to amortize FFI overhead (4.01x speedup over sequential, `docs/operations/BENCHMARKS.md` §6)

### list_memory()

List all memories in a namespace with optional filtering.

```python
# List all memories
result = db.list_memory("agent/session-1", limit=100)
memories = result.records  # list of records
total = result.total_count  # total matching count
next_cursor = result.next_cursor  # None when no more pages

# List with metadata filter
result = db.list_memory(
    "agent/session-1",
    filters={"type": "preference"},
    limit=50
)
```

### delete_memory()

Remove a specific memory.

```python
db.delete_memory("agent/session-1", "ctx-001")
```

### rebuild_index()

Manually rebuild the vector index from persisted data.

```python
db.rebuild_index()
```

**Use cases:**
- After bulk import operations
- If vector index becomes corrupted
- To optimize index layout after many updates

## Advanced Features

### Batch Queries

Amortize FFI overhead by querying multiple vectors at once.

```python
queries = [
    [0.1, 0.2, 0.3],
    [0.4, 0.5, 0.6],
    [0.7, 0.8, 0.9]
]

results = db.search_batch(queries, top_k=5)
```

**Performance:** 4.01x speedup over sequential queries, ~2.43ms per query average at 5K records/128d (sourced: `docs/operations/BENCHMARKS.md` §6).

### Operational Metrics

Monitor database health and performance.

```python
metrics = db.operational_metrics()
print(f"Process RSS: {metrics['process_rss_bytes'] / 1024 / 1024:.2f} MB")
print(f"HNSW Nodes: {metrics['hnsw_nodes_count']}")
print(f"HNSW Logical Memory: {metrics['hnsw_logical_bytes'] / 1024 / 1024:.2f} MB")
print(f"MMap Resident Pages: {metrics['mmap_resident_pages']}")
```

### Export/Import

Export and import memory as JSONL for backup or migration.

```python
# Export namespace
db.export_namespace("./backup.jsonl", "agent/session-1")

# Export entire database
db.export_all("./full_backup.jsonl")

# Import records
db.import_file("./backup.jsonl")
```

**Note:** JSONL export/import is for logical data movement, not physical backup. For physical backup, use backend-specific snapshots (Fjall volume snapshots or RocksDB checkpoints).

### Batch Inserts (`put_batch`)

Insert multiple records in one call. Two forms are supported:

```python
# Form 1: list of tuples (namespace, key, payload, metadata, vector, ttl_ms)
entries = [
    ("ns1", "k1", "payload 1", {"type": "note"}, [0.1, 0.2], None),
    ("ns1", "k2", "payload 2", {"type": "note"}, [0.3, 0.4], 60_000),
]
db.put_batch(entries=entries)

# Form 2: parallel kwargs (shared namespace + lists)
db.put_batch(
    namespace="ns1",
    keys=["k1", "k2"],
    payloads=["payload 1", "payload 2"],
    metadatas=[{"type": "note"}, {"type": "note"}],
    vectors=[[0.1, 0.2], [0.3, 0.4]],
    ttls=[None, 60_000],
)
```

For zero-copy bulk loads from a 2D numpy float32 array, use `put_batch_raw(vectors, keys, ...)`.

### TTL / Expiration

Records written with `ttl_ms` expire automatically and are excluded from search once past their deadline. Expired records are physically removed with `purge_expired()`:

```python
# Write an ephemeral record (expires in 5 minutes)
db.put("cache", "key", "value", ttl_ms=300_000)

# Remove all expired records; returns the number removed
removed = db.purge_expired()
print(f"Purged {removed} expired records")
```

### WAL Compaction (`compact_wal`)

The write-ahead log is the durability frontier. Over time the WAL grows with update/delete history; `compact_wal()` compacts it to reclaim space while preserving durability:

```python
db.compact_wal()
```

Call it during maintenance windows or after large delete/purge workloads.

### IQL Queries

VantaDB exposes an IQL (query language) interface through `db.query()`. IQL returns results as a JSON string:

```python
result = db.query('SELECT * FROM memory WHERE namespace = "agent/session-1" LIMIT 10')
```

The MCP server exposes the same capability as the `query_iql` tool. (LISP is not supported.)

### In-Memory Mode

Pass `":memory:"` as the database path to run fully in-memory — no persistence, ideal for tests and ephemeral workloads:

```python
db = vantadb.VantaDB(":memory:")
```

### Async API (`AsyncVantaDB`)

`AsyncVantaDB` wraps the sync engine and runs queries in a thread pool (GIL released to Rust):

```python
import asyncio
from vantadb_py import AsyncVantaDB

async def main():
    async with AsyncVantaDB("./agent_memory") as db:
        await db.put("agent/session-1", "ctx-001", "User prefers concise answers",
                     vector=[0.8, 0.1, 0.5])
        results = await db.search_memory("agent/session-1", [0.75, 0.2, 0.6], top_k=3)

asyncio.run(main())
```

### Hardware Profile

`hardware_profile()` reports detected CPU/SIMD capabilities of the host:

```python
profile = db.hardware_profile()
print(profile)  # e.g. CPU model, core count, AVX2/NEON support
```

## Integration with AI Frameworks

### LangChain

Package: `vantadb-langchain` (source in `integrations/langchain/`).

```bash
pip install vantadb-langchain langchain-openai
```

```python
from langchain_openai import OpenAIEmbeddings
from vantadb_langchain import VantaDBVectorStore

# Initialize vector store
vector_store = VantaDBVectorStore(
    embedding=OpenAIEmbeddings(model="text-embedding-3-small"),
    db_path="./langchain_memory",
    namespace="documents",
)

# Add documents
vector_store.add_texts([
    "VantaDB is written in Rust",
    "It provides hybrid search capabilities"
])

# Similarity search
results = vector_store.similarity_search("rust database", k=3)
```

### LlamaIndex

Package: `vantadb-llamaindex` (source in `integrations/llamaindex/`).

```bash
pip install vantadb-llamaindex llama-index-core
```

```python
from llama_index.core import VectorStoreIndex, Document
from llama_index.embeddings.openai import OpenAIEmbedding
from vantadb_llamaindex import VantaDBVectorStore

# Initialize vector store
vector_store = VantaDBVectorStore(
    db_path="./llamaindex_memory",
    namespace="documents"
)

# Create index
documents = [Document(text="VantaDB provides persistent memory for AI agents")]
index = VectorStoreIndex.from_documents(
    documents,
    embed_model=OpenAIEmbedding(model="text-embedding-3-small"),
    vector_store=vector_store,
)

# Query
query_engine = index.as_query_engine()
response = query_engine.query("What is VantaDB?")
```

## Architecture Deep Dive

### Storage Engine

**Fjall (Default):**
- Pure Rust LSM-tree implementation
- High write throughput
- Automatic compaction
- Volume snapshots for backup

**RocksDB (Optional):**
- Enable with `--features rocksdb`
- Native checkpoint support
- Tunable compaction strategies

### Write-Ahead Log (WAL)

All mutations are written to WAL before being applied to storage:
- CRC32C checksums for integrity
- Auto-healing on crash recovery
- Truncation of corrupt tail
- Atomic checkpoints
- Compactable on demand via `db.compact_wal()` to reclaim space after heavy delete/update workloads

### HNSW Vector Index

- Multi-layer graph structure
- Topological BFS layout for cache locality
- SIMD-accelerated distance calculations (AVX2/NEON)
- Memory-mapped for zero-copy access
- Search latency p50 1.2ms at 10K vectors (sourced: `docs/operations/BENCHMARKS.md` §1)

### BM25 Text Index

- Persisted inverted index
- Advanced tokenization with stemming and stopwords (tantivy `advanced-tokenizer` feature, enabled by default)
- Token positions for phrase queries
- Reconstructible from canonical records
- Auto-repair on startup if stale

### Hybrid Retrieval (RRF)

Reciprocal Rank Fusion merges independent rankings:
```python
score = sum(1 / (k + rank_i) for each ranking)
```
- No parameter tuning required
- Deterministic results
- Balances semantic and lexical signals

## Performance Characteristics

### Benchmarks (10K vectors, 128d)

Source: `docs/operations/BENCHMARKS.md` — SDK operations (§2, `benchmarks/vantadb_local_bench.py`), engine certification (§1, `tests/certification/stress_protocol.rs`), batch search (§6).

| Operation | Latency p50 | Latency p99 | Throughput |
|-----------|-------------|-------------|------------|
| Put | 10.7ms | 19.0ms | 95 ops/sec |
| Vector Search | 62ms | 72ms | 16 qps |
| Text Search (BM25) | 115ms | 138ms | 9 qps |
| Hybrid Search | 180ms | 211ms | 6 qps |
| Batch Query (100) | 2.43ms/query | - | 4x speedup |

### Scalability

Source: `docs/operations/BENCHMARKS.md` §1 (Stress Protocol, AVX2 environment).

- **Recall@10:** 0.9980 at 10K, 1.0000 at 50K, 0.9980 at 100K
- **Scaling Factor:** 4.88x sub-linear (10K → 50K)
- **Memory:** ~1172 bytes per vector (HNSW overhead)
- **Construction:** 68.4s for 100K vectors (Balanced L2, SIFT1M — `docs/operations/BENCHMARKS.md` §5)

## Best Practices

### Memory Management

```python
# Set appropriate memory limit based on workload
db = vantadb.VantaDB("./db", memory_limit_bytes=512_000_000)  # 512MB

# Monitor RSS drift
metrics = db.operational_metrics()
if metrics['process_rss_bytes'] > threshold:
    # Consider rebuilding index or compacting WAL
    db.rebuild_index()
    db.compact_wal()
```

### Namespace Design

```python
# Good: Hierarchical, scoped
"agent/user-123/session-456"
"rag/documents/technical"
"cache/embeddings/nomic-embed-text"

# Avoid: Flat, monolithic
"all_memories"
"global_cache"
```

### Vector Dimensionality

- **Recommendation:** 384-768 dimensions for most use cases
- **Trade-off:** Higher dimensions = better accuracy but more memory
- **Consistency:** All vectors in a namespace must have same dimensionality

### Index Rebuild Strategy

```python
# After bulk import
for batch in large_dataset:
    db.put_batch(entries=batch)
db.rebuild_index()  # Optimize layout

# Periodic maintenance (weekly)
db.rebuild_index()
```

### Error Handling

```python
try:
    db.put(namespace, key, payload, vector=vec)
except Exception as e:
    # Handle WAL errors, disk full, etc.
    print(f"Storage error: {e}")
    # Retry or fallback
```

## Troubleshooting

### High Memory Usage

**Symptom:** RSS grows continuously over time

**Solutions:**
1. Check for memory leaks in application code
2. Rebuild index: `db.rebuild_index()`
3. Compact the WAL: `db.compact_wal()`
4. Reduce memory_limit_bytes
5. Monitor with `db.operational_metrics()`

### Slow Search Performance

**Symptom:** Search latency > 100ms

**Solutions:**
1. Use batch queries: `db.search_batch()`
2. Reduce top_k if not needed
3. Rebuild index for better layout
4. Check disk I/O performance

### Index Corruption

**Symptom:** Search returns unexpected results or errors

**Solutions:**
1. Rebuild index: `db.rebuild_index()`
2. Check WAL integrity in logs
3. Compact the WAL: `db.compact_wal()`
4. Restore from backup if needed
5. Run chaos tests to verify recovery

### Import/Export Issues

**Symptom:** Import fails or data missing

**Solutions:**
1. Verify JSONL format is correct
2. Check vector dimensions match
3. Ensure namespace exists
4. Use `db.import_file()` with error handling

## Security Considerations

### File Permissions

```bash
# Restrict database directory access
chmod 700 ./agent_memory
```

### Read-Only Mode

```python
# Open in read-only mode for safety
db = vantadb.VantaDB("./db", read_only=True)
```

### Backup Strategy

```python
# Logical backup (JSONL)
db.export_all("./backup.jsonl")

# Physical backup (Fjall)
# Copy entire database directory after flush
db.flush()
# Then: cp -r ./db ./backup
```

## Resources

### Documentation

- **Quickstart:** `docs/QUICKSTART.md`
- **Benchmarks:** `docs/operations/BENCHMARKS.md`
- **Architecture:** `docs/architecture/ARCHITECTURE.md`
- **ADRs:** `docs/architecture/adr/` (Architecture Decision Records)

### Examples

- **Agent Memory:** `examples/python/agent_memory.py`
- **LangChain RAG (Ollama):** `examples/python/langchain_ollama_rag.py`
- **More integrations:** `examples/python/` (autogen, crewai, dspy, haystack, langgraph, mem0, semantic_kernel)
- Case Studies: archivados en `docs/archive/case-studies-unverified/` (internos, no verificados)

### Integration Packages

- **LangChain:** `integrations/langchain/` — `pip install vantadb-langchain`
- **LlamaIndex:** `integrations/llamaindex/` — `pip install vantadb-llamaindex`
- **MCP Server:** `vantadb-mcp/`

### Community

- **GitHub:** https://github.com/ness-e/Vantadb
- **Plan Maestro:** `VantaDB_Plan_Maestro_Unificado.md`
- **Contributing:** `CONTRIBUTING.md`

## Common Patterns

### RAG Pipeline

```python
def rag_pipeline(query, db, embedding_fn):
    # Generate query embedding
    query_vec = embedding_fn(query)
    
    # Retrieve relevant context
    results = db.search_memory(
        namespace="rag/documents",
        query_vector=query_vec,
        text_query=query,
        top_k=5
    )
    
    # Build context
    context = "\n".join([r['record']['payload'] for r in results])
    
    # Generate response (with your LLM)
    response = llm.generate(f"Context: {context}\n\nQuestion: {query}")
    return response
```

### Agent Memory Loop

```python
def agent_memory_loop(agent, db, embedding_fn):
    for interaction in agent.conversations:
        # Store interaction
        db.put(
            namespace=f"agent/{agent.id}",
            key=f"msg-{interaction.id}",
            payload=interaction.content,
            metadata={"role": interaction.role, "timestamp": interaction.time},
            vector=embedding_fn(interaction.content)
        )
    
    # Retrieve relevant context for new query
    context = db.search_memory(
        namespace=f"agent/{agent.id}",
        query_vector=embedding_fn(agent.current_query),
        top_k=10
    )
    return context
```

### Caching Layer

```python
def cache_with_vantadb(db, key, compute_fn, ttl_seconds=3600):
    # Try cache first
    cached = db.get_memory("cache", key)
    if cached and time.time() - cached['metadata']['timestamp'] < ttl_seconds:
        return cached['payload']
    
    # Compute and cache
    result = compute_fn()
    db.put(
        namespace="cache",
        key=key,
        payload=result,
        metadata={"timestamp": time.time()},
        vector=embedding_fn(str(result))
    )
    return result
```

## Limitations

### Current Boundaries (v0.5.x)

**Supported:**
- Embedded storage with WAL durability
- HNSW vector search (Cosine, Euclidean)
- BM25 text search with advanced tokenization (stemming, stopwords — tantivy `advanced-tokenizer` feature, enabled by default)
- Hybrid retrieval via RRF
- Namespace-scoped memory
- JSONL export/import
- IQL query language (`db.query()`)
- TTL-based expiration (`ttl_ms` + `purge_expired()`)
- Graph operations (BFS, DFS, topological sort, page rank, centrality)
- Python SDK with PyO3 bindings (sync + async)

**Not Supported (Deferred):**
- LISP query language
- Distributed replication (Pro feature)
- Disk encryption (Pro feature)
- Rich snippets/highlighting
- Multi-tenancy isolation (Pro feature)
- Server as primary product boundary

### Known Limitations

- Phrase queries are basic (no proximity search)
- Index rebuild is manual (not automatic)
- No native backup/restore beyond JSONL (use backend snapshots for physical backup)
- Ingestion is single-threaded in the SDK API (`docs/operations/BENCHMARKS.md` §2)

## Future Roadmap

### Near Term

- Rich snippets and highlighting
- Automatic index maintenance
- Improved phrase queries

### Long Term (Pro Features)

- P2P replication (WAL shipping)
- Disk encryption (AES-256-GCM)
- Multi-tenancy with isolation
- Quantization (SQ8/PQ) for compression
- Cloud managed service

## Version Compatibility

- **Current Version:** 0.5.0
- **Rust Minimum:** 1.94.1
- **Python Minimum:** >=3.11
- **Platforms:** Linux, macOS (Intel/ARM), Windows

## License

- **Core:** Apache 2.0 (Open Source)
- **Pro Features:** Commercial License (Future)

## Support

- **Issues:** GitHub Issues
- **Discussions:** GitHub Discussions
- **Documentation:** `docs/`
- **Quickstart:** `docs/QUICKSTART.md`
