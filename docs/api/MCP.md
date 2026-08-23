---
title: VantaDB Model Context Protocol (MCP) Server
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-08-22
aliases: []
---

# VantaDB Model Context Protocol (MCP) Server

The VantaDB MCP server (`vantadb-mcp`) exposes the database to LLM agents over the Model Context Protocol. Tool definitions live in `vantadb-mcp/src/handlers/tools.rs` (`handle_tools_list`); per-IDE setup lives in [`skills/vantadb-mcp/SKILL.md`](../../skills/vantadb-mcp/SKILL.md).

**56 tools in 4 families:**

| Family | Count | Source module |
|--------|-------|---------------|
| Core | 36 | `handlers/tools.rs` — listed in `tools/list` |
| `code_*` | 8 | `code.rs` |
| `skill_*` | 6 | `skills.rs` |
| `wiki_*` | 6 | `wiki.rs` |

## Core Tools (36)

### Memory CRUD (7)

| Tool | Description |
|------|-------------|
| `memory_put` | Inserts or updates a memory record in a namespace with payload, vector, optional sparse vector, metadata, and TTL. |
| `memory_put_batch` | Stores multiple records in a single all-or-nothing batch; duplicate keys are upserts, vector dimensions must match the live index. |
| `memory_get` | Retrieves a memory record by namespace and key. |
| `memory_delete` | Deletes a memory record by namespace and key. |
| `memory_delete_by_filter` | Batch-deletes every record in a namespace whose metadata matches the given filters (AND semantics). |
| `memory_list` | Lists memory records in a namespace with optional pagination and metadata filters. |
| `memory_list_namespaces` | Lists all available namespaces in the database. |

### Search & Query (3)

| Tool | Description |
|------|-------------|
| `search_memory` | Hybrid memory search in a namespace: text/vector/hybrid modes, filters, distance metric, RRF tuning, and explain output. |
| `search_semantic` | Raw semantic vector search directly in the HNSW index. |
| `query_iql` | Executes an IQL statement against typed graph nodes and memory namespaces (each namespace is queryable as a table named by its sanitized form: `/` and `-` → `_`, leading digit/`.` gets a `_` prefix). LISP not supported. |

### Graph (6)

| Tool | Description |
|------|-------------|
| `get_node_neighbors` | Inspects neighbors or lineage of a node. |
| `graph_page_rank` | Computes PageRank over the subgraph reachable from the given root nodes. |
| `graph_degree_centrality` | Incoming/outgoing edge counts for every node reachable from the given roots. |
| `graph_traverse` | Multi-hop BFS/DFS traversal from start nodes with optional edge-label and temporal filters. |
| `graph_topological_sort` | Topological sort of the subgraph reachable from the given roots; errors on cycles. |
| `graph_is_dag` | Returns whether the subgraph reachable from the given roots is a directed acyclic graph. |

### Context & Axioms (2)

| Tool | Description |
|------|-------------|
| `inject_context` | Injects external state or context connected to a specific thread for subsequent consolidation. |
| `read_axioms` | Returns the active Devil's Advocate Axioms (Iron Axioms) in the database. |

### Collections (3)

| Tool | Description |
|------|-------------|
| `collection_list` | Lists all collections with record count, vector index status, and creation time. |
| `collection_stats` | Statistics for one namespace/collection: count, byte size, index info, creation time. |
| `collection_delete` | Deletes an entire namespace/collection and all its records (requires `confirm: "yes"`). |

### Maintenance, Indexes & Snapshots (9)

| Tool | Description |
|------|-------------|
| `flush` | Flushes WAL and memory-mapped files to disk as a manual durability checkpoint. |
| `compact_wal` | Archives the current WAL file and starts a fresh one to reclaim WAL space. |
| `compact_layout` | Compacts the vector store file grouping nodes in BFS order from the HNSW entry point; returns bytes reclaimed. |
| `rebuild_index` | Rebuilds the HNSW vector index, derived indexes, and text index from scratch (recovery primitive). |
| `audit_text_index` | Read-only integrity audit of the derived persistent text index vs canonical records; `deep=true` verifies postings/tf/stats. |
| `repair_text_index` | Repairs the text index by rebuilding it from canonical storage (use when `audit_text_index` reports drift). |
| `list_snapshots` | Lists physical snapshot names under `<data_dir>/snapshots`. |
| `purge_expired` | Scans all records and physically deletes those whose TTL expiry has passed. |
| `rehydrate` | Recovers shadow-archived nodes that belonged to a summary node from TombstoneStorage. |

### Introspection & Utility (2)

| Tool | Description |
|------|-------------|
| `capabilities` | Introspects supported engine features: runtime profile, persistence, vector search, IQL queries, read-only mode. |
| `generate_snippet` | Generates a text snippet from a payload, optionally highlighting matched query terms. |

### Backup & Bulk Import (4)

| Tool | Description |
|------|-------------|
| `export` | Exports memory records as JSONL (max 10 MB per call); pair with `import` for backup/restore. |
| `import` | Imports records from JSONL produced by `export`; malformed lines are counted as errors, not fatal. |
| `bulk_import_file` | Bulk-imports from a binary `.vdbdump` file on the host filesystem, bypassing per-record validation for throughput. |
| `bulk_import_stream` | Bulk-imports inline NDJSON or raw `.vdbdump` content (max 10 MB); imported entries are raw engine nodes. |

## Extended Tool Families (20)

Dispatched via `tools/call`, defined outside `handlers/tools.rs`:

### Code Intelligence — `code.rs` (8)

| Tool | Description |
|------|-------------|
| `code_search` | Searches indexed code symbols. |
| `code_explore` | Explores symbols with call paths and blast radius. |
| `code_callers` | Lists callers of a symbol. |
| `code_callees` | Lists callees of a symbol. |
| `code_impact` | Impact analysis for a change target. |
| `code_node` | Fetches a single code-graph node. |
| `code_files` | Lists indexed files. |
| `code_status` | Index health/status of the code graph. |

### Skills Management — `skills.rs` (6)

| Tool | Description |
|------|-------------|
| `skill_list` | Lists installed agent skills. |
| `skill_view` | Views a skill's content. |
| `skill_create` | Creates a new skill. |
| `skill_update` | Updates an existing skill. |
| `skill_patch` | Applies targeted patches to a skill. |
| `skill_files_write` | Writes supporting files for a skill. |

### Wiki — `wiki.rs` (6)

| Tool | Description |
|------|-------------|
| `wiki_ingest` | Ingests documents into the wiki knowledge base. |
| `wiki_ingest_status` | Reports status of a wiki ingest run. |
| `wiki_read` | Reads a wiki page/node. |
| `wiki_search` | Searches the wiki corpus. |
| `wiki_graph` | Queries the wiki knowledge graph. |
| `wiki_list` | Lists wiki pages/nodes. |

## Parity

Tool coverage on this page is enforced mechanically by `scripts/validate-docs-coverage.ps1` against `handle_tools_list()` in `vantadb-mcp/src/handlers/tools.rs`. Last sync: **2026-08-22**.
