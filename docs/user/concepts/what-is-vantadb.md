---
title: "What VantaDB is — and what it is not"
kind: concept
status: active
description: "The product in one page: an embedded memory and retrieval engine that runs inside your process, plus the closed list of things it deliberately is not, each with the document that owns the boundary"
tags: [architecture, memory, engine]
---

# What VantaDB is — and what it is not

This page is a **map, not a manual**. Every technical statement here ends at the
document that owns it; if this page and that document disagree, that document
wins. Read this to decide whether to keep reading.

## The short answer

VantaDB is an **embedded persistent memory engine**: a Rust library that stores
records — a payload, scalar metadata, an optional vector, graph edges — behind a
`namespace + key` address, and answers three kinds of retrieval over them:
dense vector search (HNSW), lexical search (BM25), and both fused in one query
via Reciprocal Rank Fusion. See
[the architecture overview](../../dev/architecture/ARCHITECTURE.md#1-product-boundary)
for the authoritative boundary statement and
[the scoring contract](../../api/scores.md) for what a returned score means.

It runs **inside your process**. There is no cluster, no daemon and no external
service to operate — the sentence is the README's, and it is the single most
consequential property of the product:
[no separate cluster, daemon, or external service is required](../../../README.md).

> "SQLite for agents" is the analogy the project uses, not a capability claim.
> The reasoning behind it — and where the analogy breaks — is in
> [SQLite for AI Agents](../blog/sqlite_for_ai_agents.md).

## Three properties that decide everything downstream

| Property | What it buys you | Where it is specified |
|---|---|---|
| **In-process execution** | No network hop on the write or the read path; no service to deploy, version or secure. Your process's lifetime is the engine's lifetime. | [Embedded SDK boundary](../../api/EMBEDDED_SDK.md) |
| **Canonical records + derived indexes** | Indexes can be rebuilt from the canonical records, so a corrupt or stale index is a repair, not data loss. | [Design principles](../../dev/architecture/ARCHITECTURE.md#design-principles) |
| **Durability as the default, not a mode** | Every mutation goes through a Write-Ahead Log with CRC32C validation; recovery after a crash is the normal path, not a special one. | [Durability guarantees](../operations/DURABILITY_GUARANTEES.md) · [WAL integrity](../../api/WAL_INTEGRITY.md) |

## What it is not

Each row is a boundary someone has already written down. If your plan needs one
of these, that is a signal to stop — not a gap to work around.

| It is **not** | In practice, that means | The boundary is owned by |
|---|---|---|
| **A managed service, cloud platform or control plane** | You self-host the library or the optional local server. Nothing in this repo runs infrastructure for you. | [Freeze list — Experimental Features and Product Boundary](../operations/EXPERIMENTAL_FEATURES.md) |
| **A distributed database** | Single node. No replication, HA or clustering; the recovery runbook assumes one node. | [Disaster recovery runbook](../operations/DISASTER_RECOVERY_RUNBOOK.md) |
| **A SQL / OLTP / warehouse / time-series database** | IQL is a query DSL with filtering, pagination and `COUNT`/`SUM`/`GROUP BY` — it is not SQL and there is no join engine across arbitrary tables. | [IQL reference](../../api/IQL.md) |
| **A graph database** | Edges and traversals exist (BFS/DFS, topological sort, PageRank) inside the record model. The README says so explicitly: *not a graph database claim*. | [GraphRAG API](../../api/GRAPH_RAG.md) |
| **A full-text search platform** | BM25 is one leg of hybrid retrieval, tuned for agent memory — not a competitive replacement for a search engine. | [Architecture §1](../../dev/architecture/ARCHITECTURE.md#1-product-boundary) |
| **An embedding service** | Bring your own vectors by default. Local ONNX embeddings exist, but they are an opt-in build feature. | [Embeddings — BYO-vector + `embed-local`](../../api/EMBEDDINGS.md) |
| **A universal multimodel store** | No multimodal columns, no alternative ANN families (IVF/DiskANN/ScaNN) in the shipped surface. | [Architecture §1](../../dev/architecture/ARCHITECTURE.md#1-product-boundary) |
| **An RBAC / multi-tenancy platform** | Isolation today is namespace-scoped: the HTTP surface enforces read/write grants per namespace and sharing exposes scopes and grants. Broader RBAC and multi-tenancy as a platform feature are on the freeze list, and it is all single-node. | [HTTP API §Multi-tenant](../../api/HTTP_API.md) · [Shared memory: scopes and grants](../../api/SHARING.md) |
| **One product with a frozen UI** | Of the four surfaces, two are frozen and one is unpublished: the desktop studio is a **viewer**, the lab experiments are **frozen**, and `vanta-proxy` is **not published until 1.0.0**. | [README §Product Boundary](../../../README.md) |

## The surfaces you will actually touch

| Surface | Reach for it when | Documentation |
|---|---|---|
| **Engine** (Rust core + bindings) | You are writing the application. This is where integration work happens. | [Runtimes](../WHICH_RUNTIME.md) |
| **CLI** (`vanta-cli`) | You need to put/read/export, audit an index, or run the server. | [README §Embedded CLI](../../../README.md#embedded-cli) |
| **MCP server** | Your client is an AI IDE or agent that speaks MCP — no code at all. | [MCP server reference](../../api/MCP.md) |
| **HTTP server** | Something that is not Rust/Python/JS needs to talk to it over the network. | [HTTP API](../../api/HTTP_API.md) |

## Licensing shape

The core crate is Apache-2.0 and stays that way. A separate proprietary edition
exists outside this repository; it is not a workspace member and ships only as
compiled artifacts. The decision and its consequences are recorded in
[ADR-0013](../../dev/architecture/adr/ADR-0013-oc-licensing-vantadb-pro.md).

## Where to go next

| Your question | Page |
|---|---|
| Does this fit what I am building? | [Is VantaDB for your use case?](../IS_IT_FOR_ME.md) |
| Which language binding should I use? | [Choose a runtime](../WHICH_RUNTIME.md) |
| Show me it working in five minutes | [Quickstart](../QUICKSTART.md) |
| How does it compare to the alternative I already use? | [Comparison & practical limits](../COMPARISON.md) |
