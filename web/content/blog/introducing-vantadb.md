---
title: Introducing VantaDB — The Database That Thinks With You
date: 2026-06-15
description: VantaDB is an open-source embedded vector database that unifies vector search (HNSW), BM25 full-text, and hybrid search (RRF) in a single Rust binary.
author: VantaDB Team
tags: ["announcement", "embedded", "vector-database"]
---

We built VantaDB because the AI stack shouldn't need a database team.

Every AI agent, every RAG pipeline, every intelligent application deserves a database that embeds as easily as SQLite but understands vectors and text — without requiring a dedicated infrastructure team.

## What is VantaDB?

VantaDB is an **embedded vector database** that runs inside your application process. No server process, no cloud dependency, no per-vector pricing.

- **One binary** — `pip install vantadb-py` or `cargo add vantadb`
- **Three query engines** — vector search (HNSW), full-text search (BM25), hybrid search (RRF)
- **Zero ops** — no servers to maintain, no clusters to configure

## Key metrics

| Metric | Value |
|--------|-------|
| Recall@10 | 0.998 |
| Latency p50 | 1.2ms |
| Memory | 1172 bytes/vector |
| License | Apache 2.0 |

## Why embedded?

Most vector databases today require a separate server process. That works for large-scale deployments, but it's overkill for:

- **AI agents** running on a laptop
- **Local RAG pipelines** processing private documents
- **IDE tooling** analyzing a codebase
- **Mobile or IoT devices** with limited resources

VantaDB eliminates the network hop, the serialization overhead, and the infrastructure bill.

## What's next

We're in active development toward our community launch. The roadmap is public, the code is open, and contributions are welcome.

- [GitHub](https://github.com/ness-e/Vantadb)
- [Documentation](/docs)
- [Roadmap](/about/roadmap)
