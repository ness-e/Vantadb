---
title: Experimental Features and Product Boundary
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-08-25
aliases: []
---

# Experimental Features and Product Boundary

This document classifies the current v0.5.0 repository surface. It is the operational
reference for what is production-facing, optional, new, experimental, or deferred.
It mirrors the boundary table in the root `README.md` — when they disagree, this file wins
for operational detail and the README wins for positioning.

## Production-Facing MVP

Embedded local-first persistent memory engine:

| Area | Status |
| --- | --- |
| Embedded Rust SDK and CLI | Production-facing |
| Memory `put/get/delete/list/search` (+ versions, batch, delete-by-filter) | Production-facing |
| WAL-backed recovery (ShardedWAL) | Production-facing |
| Namespaces and scalar metadata filters | Production-facing |
| Derived namespace and metadata indexes | Production-facing |
| HNSW / Flat vector retrieval (+ IVF/SCANN opt-in) | Production-facing |
| BM25 lexical retrieval | Production-facing |
| Hybrid Retrieval v1 with deterministic RRF | Production-facing |
| Phrase filtering (`Condition::TextMatch`) | Production-facing |
| Manual rebuild and structural audit flows | Production-facing |
| JSONL export/import + `.vdbdump` bulk import | Production-facing |
| Python SDK (PyPI) / TypeScript SDKs (npm: Node + WASM) | Production-facing |

## Optional Wrapper

| Area | Status |
| --- | --- |
| Local `vantadb-server` binary (HTTP REST `/api/v2/*`, ~27 endpoints + MCP stdio) | Optional wrapper around the embedded core |

Promoted from experimental to **stable** on 2026-08-25: it is the base of the Docker deploy,
the Vanta Studio admin surface (ADR-026/ADR-027) and exposes the full SDK surface over REST.

## New (stable but recent)

| Area | Boundary |
| --- | --- |
| MCP server for AI agents (`vanta-cli server --mcp`) | Stable stdio surface; tool parity tracked in `docs/api/MCP.md`. Known limitation: single instance per data dir (single-writer lock) — multi-instance fallback tracked as backlog MCP-35 |
| Web Console (`/dashboard`) | Promoted from experimental to stable 2026-08-25; e2e-tested |

## Experimental

These surfaces exist and work, but are not stable product claims:

| Area | Boundary |
| --- | --- |
| ~~PITR (`pitr` feature)~~ | **Removed 2026-08-25 (FIND-26)**: `wal_archiver.rs` was dead code (zero engine call sites, RES-02) and was deleted; code preserved in git history. See superseded ADR-014. Re-integration would require base snapshot + log replay first (backlog CORE-02, needs re-evaluation) |
| `POST /conversation/add`, `GET /skill/listing`, skills CRUD endpoints | Marked `x-experimental: true` in `openapi.yaml`; may change without notice |
| LLM/Ollama integration | External optional integration (`llm.rs`, feature-gated), not a core dependency |
| Graph traversal beyond stored local edges | Experimental; not a graph database claim |

## Archived (2024-06)

The runtime governance/LISP experiment was archived. What remains is intentional:

| Remnant | Where |
| --- | --- |
| IQL parser/executor paths (IQL itself is production-facing; legacy LISP entrypoints error clearly) | `src/parser/`, `src/query.rs`, `src/executor.rs` |
| Extracted utilities (production-ready): `DuplicatePreventionFilter`, `OriginCollisionTracker`, `compute_confidence_friction` | `src/utils/` |
| Design record of the abandoned framework | `docs/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md` |

Note: the original quarantine directory referenced by earlier versions of this document
(`archive/experimental-quarantine-2024-06/`) no longer exists in the tree.

## Deferred

Explicitly outside the current MVP (see root `README.md` product boundary):

Cloud/enterprise platform · HA/replication · distributed clustering · SQL/OLTP/warehouse/time-series ·
advanced ranking/snippets/tokenization beyond current scope · RBAC promotion beyond HTTP middleware ·
multi-tenancy.
