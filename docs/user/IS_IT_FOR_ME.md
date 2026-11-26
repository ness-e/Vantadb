---
title: "Is VantaDB for your use case?"
kind: howto
status: active
description: "A use-case decision matrix: which shapes of problem VantaDB is built for, which it can serve with a declared limit, and which it is the wrong tool for — each verdict with its source"
tags: [vantadb, comparison, limits]
---

# Is VantaDB for your use case?

Decide by **what you are building**, not by how many vectors you think you will
have — you do not know that number yet. Each row below states a verdict and the
document that owns it; where a row says *not a fit*, believe it.

> Two rules for this page, inherited from
> [Comparison & practical limits](COMPARISON.md): qualitative statements are
> only made when publicly verifiable, and **no performance figure is quoted
> here** — numbers live in
> [BENCHMARKS.md](operations/BENCHMARKS.md) with their harness and command.

## The matrix

| What you are building | Verdict | What you get | What you must accept | Source |
|---|---|---|---|---|
| **Persistent memory for an AI IDE agent** (Claude Code, Cursor, OpenCode, Codex) | ✅ **Built for this** | One MCP server; the agent gets memory tools (store, search, recall) with no code from you | One local database per machine; the agent talks to a single node | [AI IDEs](AI_IDES.md) · [MCP server](../api/MCP.md) |
| **Local RAG over your own documents**, with nothing leaving the machine | ✅ **Built for this** | Hybrid BM25 + HNSW fused in one in-process call | You supply the vectors — the engine does not embed text by default | [Local RAG tutorial](tutorials/02-local-rag-pipeline.md) · [Embeddings](../api/EMBEDDINGS.md) |
| **A memory backend for an agent framework** (LangGraph, CrewAI, DSPy, Haystack, LlamaIndex, Mem0, Semantic Kernel, AutoGen) | ✅ **Fit** | Same memory code in dev and prod; namespaced isolation | Adapter coverage differs per framework — check the framework row before committing | [Frameworks](FRAMEWORKS.md) |
| **A Python service that must install with one `pip install`** | ✅ **Fit** | Pre-built wheels for Linux amd64/arm64, macOS x86_64/arm64, Windows amd64, Python 3.11+ | The Python surface is a binding, not the core: some core-only features have no Python entry point | [Python SDK](../api/PYTHON_SDK.md) |
| **A Node/TypeScript backend or CLI** | ✅ **Fit — if you pick the right binding** | Real on-disk persistence through the native binding | The WASM build in Node is **in-memory only**; choosing it silently loses persistence | [Choose a runtime](WHICH_RUNTIME.md) · [Node SDK](../api/NODE_SDK.md) |
| **A browser or edge application** | ⚠️ **Fit, with one limit** | WASM in the page, persistence via OPFS/IDB | Persistence outside the browser is not the shipped story; the cognitive layer is not exposed there | [WASM persistence](../api/WASM_PERSISTENCE.md) · [WASM API](../api/WASM_API.md) |
| **A multi-user service behind one server** (scopes, grants, revocation) | ⚠️ **Partial** | The HTTP server enforces namespace-scoped credentials (read/write grants per namespace) and exposes threads, skills and snapshots alongside records | Single node: no horizontal scale, no replication; RBAC/multi-tenancy as a platform feature is on the freeze list | [HTTP API §Multi-tenant](../api/HTTP_API.md) · [Sharing](../api/SHARING.md) · [Hardening](operations/hardening.md) |
| **Regulated deletion** ("right to be forgotten") or verifiable writes | ✅ **Fit, with declared limits** | Certified delete emits a purge certificate; writes can emit verifiable receipts | The limits are explicit — logical vs physical purge, backups and exports stay out of scope | [Certified delete](../api/CERTIFIED_DELETE.md) · [Write receipts](../api/WRITE_RECEIPTS.md) |
| **Encryption at rest** | ⚠️ **Opt-in build feature** | Per-artifact AEAD envelopes with a wrapped-DEK hierarchy and rotation | Existing databases are not migrated in place — encryption is a decision at creation time | [Storage formats](../api/STORAGE_FORMATS.md) · [Key runbook](operations/security/KEY_RUNBOOK.md) |
| **You want the embeddings computed for you, offline** | ⚠️ **Opt-in** | Local ONNX embeddings behind the `embed-local` feature | Extra build feature plus a model download; the default is still bring-your-own-vector | [Embeddings](../api/EMBEDDINGS.md) · [Quickstart §7](QUICKSTART.md) |
| **A distributed store, or >500k fragments across nodes** | ❌ **Not a fit** | — | The README's own threshold table hands this to Qdrant/Milvus/ClickHouse, and the recovery runbook assumes a single node | [README §When to Use](../../README.md) · [Disaster recovery](operations/DISASTER_RECOVERY_RUNBOOK.md) |
| **Analytics, SQL reporting, joins over business data** | ❌ **Not a fit** | — | IQL is a query DSL, not SQL; there is no general join engine | [IQL reference](../api/IQL.md) |
| **A store with well under ~1,000 records** | ❌ **Not a fit** | — | A dict, a JSON file or `grep` is faster to write and to debug than any database | [README §When to Use](../../README.md) |

## Three questions that settle it in a minute

1. **Does the data belong to one process on one machine?** If the answer is "a
   cluster", stop — this is an
   [embedded engine](concepts/what-is-vantadb.md), not a server product.
2. **Do you already produce vectors, or can you?** If you expect the database to
   embed your text, read [Embeddings](../api/EMBEDDINGS.md) first; it will not
   happen by default.
3. **Do you need a query you cannot express as filter + vector + text?** Joins
   across arbitrary entities and columnar analytics are out of scope; filtering,
   pagination and `COUNT`/`SUM`/`GROUP BY` are in.

Three yeses → continue to [Choose a runtime](WHICH_RUNTIME.md). Anything else →
[Comparison & practical limits](COMPARISON.md) tells you which alternative
fits the shape you actually have.

## Before you commit, measure on your own machine

Every number the project publishes is a measurement of a specific run on
specific hardware, reproducible from a command in this repo — including the
ones that are unflattering. The canonical source is
[BENCHMARKS.md](operations/BENCHMARKS.md), which also states which scopes must
never be compared with each other. Regenerate the SDK-scope numbers locally with
the command in [README §Benchmarks](../../README.md) before you size anything.

## Where to go next

| Next step | Page |
|---|---|
| Pick the language binding | [Choose a runtime](WHICH_RUNTIME.md) |
| Working code in five minutes | [Quickstart](QUICKSTART.md) |
| Coming from Chroma, LanceDB, Mem0, Zep or Letta | [Tutorials](tutorials/index.md) |
| What the engine does *not* claim to be | [What VantaDB is](concepts/what-is-vantadb.md) |
