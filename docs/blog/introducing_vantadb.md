---
title: "Introducing VantaDB"
version: 0.5.0
slug: introducing-vantadb
date: 2026-06-06
author: "VantaDB Team"
tags: ["announcement", "local-first", "embedded-database", "hybrid-search", "rust", "pyo3"]
description: "Why we built an embedded Rust engine for local-first hybrid retrieval — and why \"local-first\" matters more than ever."
tag: Announcement
readTime: "6 min"
canonical: https://vantadb.dev/blog/introducing-vantadb
draft: true
---

# Introducing VantaDB

*By the VantaDB Team*

Every vector database I've used in the last two years has the same shape: a server. You spin up a container, connect over the network, serialize your embeddings, send them across, and wait. The wait is never long in absolute terms — 20ms, 50ms, 150ms — but it is always there, always a hop, always a billable event.

## The local-first thesis

VantaDB starts from a different premise: the fastest network hop is no network hop. If your application needs hybrid retrieval — BM25 for keywords, HNSW for vectors, RRF to fuse them — that retrieval should happen inside the process that already has the embeddings. No serialization. No connection pool. No API key. No per-query cost.

This is not a rejection of cloud databases. Pinecone and Weaviate are excellent products for teams that need managed scale, multi-region replication, and someone else to carry the pager. VantaDB is for the other cases: agents that need durable memory, RAG pipelines that can't leak data, edge devices with no cloud access, and developers who want to ship without a credit card on file.

## Why Rust, why PyO3

We chose Rust for the core because memory safety is not optional in a database, and because deterministic performance is not optional in a retrieval engine. There is no garbage collector pause to explain away in a p99 latency chart. The PyO3 bindings expose a stable `src/sdk.rs` boundary — Python callers never touch raw pointers, and the FFI surface is narrow enough to audit by hand.

The result is an engine that runs in-process, serves hybrid queries in 1.2ms, recovers from crashes via a CRC32C-checksummed WAL, and costs nothing to operate beyond the hardware you already own. It is Apache 2.0, it is on PyPI, and it is ready for you to try today.

```bash
pip install vantadb-py
```

Join the community on Discord and star the [VantaDB repository](https://github.com/ness-e/Vantadb). If you want the story behind the engine, read [Why I Built a Local Memory Engine for AI Agents in Rust](/blog/why-i-built-vantadb-local-memory-engine).