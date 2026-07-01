---
title: GraphRAG README
type: guide
status: active
tags: [vantadb, graphrag]
last_reviewed: 2026-07-01
---

# GraphRAG on VantaDB

> GraphRAG is a retrieval-augmented generation technique that uses a graph structure to represent entities, their relationships, and semantic context — enabling multi-hop reasoning, community-level summarization, and confidence-weighted retrieval that flat vector search cannot provide.

VantaDB was designed from day one as a **unified vector-graph database**. Unlike projects that bolt a graph abstraction on top of a vector store, VantaDB's core primitives — nodes with typed relational fields, labeled weighted edges, confidence scoring, semantic clustering, and an LLM-powered Semantic Compression Engine — form a complete substrate for production GraphRAG workloads.

---

## Why VantaDB for GraphRAG?

| Capability | VantaDB | Typical Vector DB | Microsoft GraphRAG |
|---|---|---|---|
| Native edges | Labeled, weighted edges on every node | Not available | Separate indexing pipeline |
| Graph traversal | BFS, DFS, topological sort, DAG detection | Not available | Python post-processing |
| Semantic clusters | `semantic_cluster` field for super-node routing | Not available | Community detection step |
| Confidence score | Per-node `f32` persisted in header, used in query routing | Not available | Not available |
| Semantic compression | Built-in LLM `summarize_context()` engine | Not available | Separate `compress` step |
| Vector search | HNSW (navigable small-world graph) | Required external index | Not native |
| Hybrid search | BM25 + HNSW vector | Some support | Not available |
| Multi-agent tracking | Collision metrics, exponential moving averages | Not available | Not available |
| Single binary | Yes, zero external dependencies | Usually | Python-only CLI |

---

## Core GraphRAG Primitives

### 1. Nodes as GraphRAG Entities

Every `UnifiedNode` in VantaDB carries the fields needed for GraphRAG:

```
UnifiedNode {
    id: u64                    — unique entity ID
    semantic_cluster: u32      — community / cluster assignment
    confidence_score: f32      — retrieval confidence (default 0.5)
    relational: BTreeMap<String, FieldValue>  — typed entity properties
    edges: Vec<Edge>           — outgoing labeled weighted edges
    importance: f64            — semantic priority for compression
    hits: u32                  — access frequency
    vector: Option<Vec<f32>>   — dense embedding
}
```

This maps directly to GraphRAG's entity model:
- `relational` fields = entity attributes (name, type, description, keywords)
- `edges` = entity relationships (label, target, weight)
- `semantic_cluster` = community ID for cluster-level summarization
- `confidence_score` = retrieval quality signal

### 2. Edges with Labels and Weights

Edges are first-class citizens, not afterthoughts:

```rust
struct Edge {
    target_id: u64,     // target node ID
    label: String,      // relationship type (e.g. "depends_on", "contains")
    weight: f64,        // relationship strength
}
```

The Python SDK exposes full graph operations:

```python
db.add_edge("namespace", source_id, target_id, "depends_on", weight=0.95)

# Graph traversals
bfs_results = db.graph_bfs("namespace", start_id, max_depth=3)
dfs_results = db.graph_dfs("namespace", start_id, max_depth=5)
is_dag = db.graph_is_dag("namespace")
sorted_nodes = db.graph_topological_sort("namespace")
```

### 3. Semantic Clusters

The `semantic_cluster: u32` field enables super-node routing: nodes in the same cluster share a semantic context. This is the foundation for:

- **Community detection** — assign cluster IDs via external algorithms or let VantaDB's engine group related nodes during compression
- **Cluster-level retrieval** — restrict search to relevant clusters instead of scanning all nodes
- **Hierarchical summarization** — compress each cluster independently, then compress cluster summaries

### 4. Confidence Score

Every node carries a `confidence_score: f32` (default 0.5) that:

- Is persisted in the 64-byte disk header alongside every node
- Is updated via the `AccessTracker` trait on every read/write access
- Filters low-confidence semantic summaries at query time (`confidence_score < 0.4` → skipped)
- Feeds into LLM prompts as a retrieval quality signal
- Supports multi-agent collision tracking via exponential moving average friction metrics

```python
record = db.get_memory("namespace", "entity-1")
print(record.confidence_score)  # 0.85
```

---

## Semantic Compression Engine

VantaDB includes a built-in LLM-powered compression engine at `src/llm.rs:summarize_context()`.

### How It Works

1. A group of related nodes is collected (by cluster, by traversal, or by query)
2. Each node's `content`, `keywords`, `type`, `importance`, `confidence_score`, and `hits` are structured into a prompt
3. The LLM (configurable via `VANTA_LLM_SUMMARIZE_MODEL`) generates a dense summary that preserves semantically important information
4. The summary is stored as a `SemanticSummary` node in the `CompressedArchive` partition
5. During query execution, low-confidence summaries are filtered out (`confidence_score < 0.4`)

### Configuration

```bash
# LLM endpoint (Ollama, OpenAI-compatible)
export VANTA_LLM_ENDPOINT=http://localhost:11434/v1
export VANTA_LLM_API_KEY=""  # or your API key
export VANTA_LLM_MODEL="llama3"
export VANTA_LLM_SUMMARIZE_MODEL="llama3"  # separate model for compression
```

### Prompt Architecture

The compression prompt includes semantic priority weighting:

```
You are VantaDB's Semantic Compression Engine.
Your task is to distill a group of related data fragments into a single,
dense summary that preserves the most semantically important information.
Pay special attention to fragments with high Semantic Priority.
Output ONLY the summary text, no preamble or formatting.

--- Node Fragment #1 ---
Type: conversation
Content: ...
Semantic Priority: 0.92
Confidence Score: 0.88
Keywords: architecture, caching
Access Count: 47
```

---

## Graph Traversal + Vector Search

VantaDB uniquely combines graph traversal with vector similarity. The experimental IQL query language supports graph-constrained vector search:

```lisp
(IQL
  (FROM namespace_id)
  (WITH DEPTH 3)
  (WHERE (REL edges.weight > 0.5))
  (VECTOR_SEARCH query_embedding top_k=10))
```

This enables queries like:
- "Find the top-5 most similar nodes reachable within 3 hops from entity X"
- "Retrieve all nodes in the dependency graph of component Y with confidence > 0.7"
- "Traverse from root Z along edges labeled 'contains', then semantically search within the result set"

The Python SDK exposes individual graph operations that can be composed with vector search:

```python
# Walk the graph, then search within results
traversed = db.graph_bfs("namespace", root_id, max_depth=3)
results = db.search_memory("namespace", query_vector, top_k=10)
# Intersect or filter by traversal results
```

---

## Multi-Agent GraphRAG

VantaDB's confidence metrics module (`src/utils/confidence_metrics.rs`) tracks origin-based confidence for multi-agent scenarios:

```rust
struct OriginConfidence {
    origin: String,       // agent identifier
    confidence: f64,      // smoothed confidence score
    session_count: u64,   // number of sessions observed
    friction: f64,        // collision rate (exponential moving average)
    updated_at: SystemTime,
}
```

This enables:
- **Per-agent confidence weighting** — agents with high collision rates get their contributions downweighted
- **Source-aware retrieval** — filter or boost results by originating agent
- **Session tracking** — detect agent behavioral shifts over time

---

## Getting Started: GraphRAG Workflow

### Minimal Example

```python
import vantadb_py as vanta

db = vanta.VantaDB("./graphrag_demo")

# 1. Insert entities with relational metadata
db.put("research", "paper-1", "Attention is All You Need",
       metadata={
           "type": "paper",
           "year": 2017,
           "keywords": "transformer, attention, NLP",
           "content": "We propose a new simple network architecture..."
       },
       vector=[0.1, 0.2, 0.3])  # embedding

db.put("research", "paper-2", "BERT: Pre-training of Deep Bidirectional Transformers",
       metadata={
           "type": "paper",
           "year": 2018,
           "keywords": "pretraining, bidirectional, NLP",
           "content": "We introduce a new language representation model..."
       },
       vector=[0.2, 0.1, 0.4])

# 2. Build the knowledge graph with weighted edges
db.add_edge("research", "paper-1", "paper-2", "cites", weight=0.9)
db.add_edge("research", "paper-2", "paper-1", "cited_by", weight=0.7)

# 3. Graph traversal
print("Citation chain:")
traversed = db.graph_bfs("research", "paper-1", max_depth=3)
for node_id in traversed:
    record = db.get_memory("research", str(node_id))
    print(f"  - {record.metadata.get('content', '')[:60]}...")
    print(f"    confidence: {record.confidence_score}")

# 4. Semantic search with confidence
results = db.search_memory("research", query_vector, top_k=5)
for r in results:
    if r.confidence_score < 0.4:
        continue  # skip low-confidence matches
    print(f"{r.id}: {r.text[:80]} (confidence: {r.confidence_score:.2f})")
```

### Full GraphRAG Pipeline

```
┌────────────────────────────────────────────────────┐
│ 1. Document Ingestion                              │
│    Parse → chunk → embed → insert as UnifiedNode   │
│    with relational metadata and vector              │
├────────────────────────────────────────────────────┤
│ 2. Knowledge Graph Construction                    │
│    Add edges between related nodes:                │
│      - add_edge(namespace, src, tgt, label, weight) │
│      - Assign semantic_cluster IDs via:            │
│        external community detection or manual       │
├────────────────────────────────────────────────────┤
│ 3. Semantic Compression (optional)                  │
│    Group nodes by cluster → summarize_via_LLM()    │
│    → store as SemanticSummary in CompressedArchive │
├────────────────────────────────────────────────────┤
│ 4. Hybrid Retrieval                                │
│    vector search + BM25 text search +               │
│    graph traversal constraints +                    │
│    confidence score filtering                       │
├────────────────────────────────────────────────────┤
│ 5. Multi-Hop Reasoning                             │
│    graph_bfs/dfs → vector search within results    │
│    → LLM prompt with traversal context              │
├────────────────────────────────────────────────────┤
│ 6. Confidence-Aware Response                       │
│    LLM receives confidence scores per fragment,    │
│    can weigh evidence accordingly                  │
└────────────────────────────────────────────────────┘
```

---

## Comparison: VantaDB vs Microsoft GraphRAG

| Aspect | VantaDB GraphRAG | Microsoft GraphRAG |
|---|---|---|
| **Deployment** | Single binary, zero deps | Python CLI, multiple dependencies |
| **Indexing** | Automatic on insert | Separate pipeline (entities → communities → summaries) |
| **Storage** | Zero-copy MMap, no serialization overhead | Parquet files, separate storage per step |
| **Graph** | Native edges on every node, BFS/DFS/topo | Separate entity extraction graph |
| **Clustering** | `semantic_cluster` field + manual; Leiden via SDK | Leiden algorithm built-in |
| **Summarization** | Built-in LLM compression engine | NapkinCC / external LLM calls |
| **Vector search** | HNSW with SIMD acceleration | No native vector search |
| **Text search** | BM25 (Tantivy-based) | No native text search |
| **Hybrid search** | Vector + BM25 + graph traversal | Not available |
| **Streaming** | WAL + immutable segments | Not specified |
| **Confidence** | Per-node `confidence_score` | Not available |
| **Multi-agent** | Origin collision tracking + friction metrics | Not available |
| **Protocol** | MCP, HTTP API, native Rust SDK, Python SDK | CLI only |
| **Language** | Rust (fast, safe, zero-cost abstractions) | Python (slower, GIL-bound) |

### When to Use Microsoft GraphRAG

- You need Leiden community detection as a built-in step
- Your pipeline is batch-oriented (index once, query many)
- You are already in a Python-only ecosystem
- You need global query mode with map-reduce summarization

### When to Use VantaDB GraphRAG

- You need real-time inserts and queries (streaming ingestion)
- You want a single binary with no external dependencies
- You need hybrid search (vector + BM25 + graph traversal)
- You want per-node confidence scoring and multi-agent support
- You need MCP protocol for AI agent integration
- You want zero-copy memory-mapped storage for high performance

---

## Configuration Reference

| Env Variable | Default | Description |
|---|---|---|
| `VANTA_LLM_ENDPOINT` | `http://localhost:11434/v1` | LLM API endpoint |
| `VANTA_LLM_API_KEY` | `""` | LLM API key |
| `VANTA_LLM_MODEL` | `llama3` | Model for embeddings/generation |
| `VANTA_LLM_SUMMARIZE_MODEL` | `llama3` | Model for semantic compression |
| `VANTADB_LOG_FORMAT` | `compact` | Log format: `json`, `compact`, `full` |

---

## Related Documentation

- [Architecture Overview](../architecture/ARCHITECTURE.md) — Core design principles
- [How Hybrid Search Works](../articles/how_hybrid_search_works.md) — BM25 + HNSW deep dive
- [Python SDK Guide](../api/PYTHON_SDK.md) — Complete SDK reference
- [Model Context Protocol (MCP)](../api/MCP.md) — AI agent integration
- [Agent Local Memory with Ollama](../case_studies/agent_local_memory_ollama.md) — GraphRAG case study
- [RAG on Edge Devices](../case_studies/rag_edge_device.md) — Edge deployment patterns
- [Experimental IQL](../experimental/IQL.md) — Graph-constrained query language
- [Benchmarks & Performance](../operations/BENCHMARKS.md) — Performance comparisons
- [Configuration Schema](../operations/CONFIGURATION.md) — Full config reference
