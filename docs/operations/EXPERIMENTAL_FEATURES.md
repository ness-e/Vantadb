# Experimental Features and Product Boundary

This document classifies the current v0.1.x repository surface. It is the operational reference for
what is production-facing, optional, experimental, or deferred.

## Production-Facing MVP

The v0.1.x product boundary is an embedded local-first persistent memory engine:

| Area | Status |
| --- | --- |
| Embedded Rust SDK and CLI | Production-facing |
| Memory `put/get/delete/list/search` | Production-facing |
| WAL-backed recovery | Production-facing |
| Namespaces and scalar metadata filters | Production-facing |
| Derived namespace and metadata indexes | Production-facing |
| HNSW vector retrieval | Production-facing |
| BM25 lexical retrieval | Production-facing |
| Hybrid Retrieval v1 with deterministic RRF | Production-facing |
| Basic phrase filtering | Production-facing |
| Manual rebuild and structural audit flows | Production-facing |
| JSONL export/import | Production-facing |
| Source-installed Python SDK | Production-facing |

## Optional Wrapper

| Area | Status |
| --- | --- |
| Local `vanta-server` binary | Optional wrapper around the embedded core |

The server exists for local development and network exposure around the same embedded engine. It is
not the primary product identity for this release.

## Experimental or Not MVP

These surfaces may exist in the repository, but they are not stable product claims for v0.1.x:

| Area | Boundary |
| --- | --- |
| IQL/LISP/DQL parser, evaluator, and executor paths | Historical or experimental |
| MCP API | Experimental integration surface |
| LLM/Ollama integration | External optional integration, not core dependency |
| Governance and maintenance semantics | Internal or future-facing infrastructure |
| Graph traversal beyond stored local edges | Experimental, not a graph database claim |
| Docker/Ollama examples | Experimental development examples |

## Deferred

The following are explicitly outside the v0.1.x MVP:

| Area | Boundary |
| --- | --- |
| Agent metacognition and automatic memory consolidation | Deferred |
| Plugins and marketplace | Deferred |
| RBAC, true multi-tenancy, quotas, and enterprise audit | Deferred |
| HA, replication, clustering, and cloud managed service | Deferred |
| Production PyPI publication and signed installers | Deferred |
| Advanced ranking, snippets, highlighting, Unicode folding, stopwords, stemming | Deferred |
| SQL, general OLTP, warehouse, and time-series workloads | Deferred |
