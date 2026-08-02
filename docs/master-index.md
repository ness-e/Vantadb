---
title: VantaDB Master Index
type: master-index
status: active
last_reviewed: 2026-07-21
tags: [vantadb, documentation, index, master-index]
---

# VantaDB Master Index

> Global index of all documentation, architecture decisions, API references, and operational guides.

- **Project:** VantaDB — cross-platform memory layer for AI agents
- **Repository:** `https://github.com/vantadb/vantadb`
- **Owner:** Eros

---

## Navigation

- [Architecture Docs](#architecture-docs)
- [API Reference](#api-reference)
- [Architecture Decision Records (ADR)](#architecture-decision-records-adr)
- [Operations & Configuration](#operations--configuration)
- [Strategy & Vision](#strategy--vision)
- [Tutorials & Migration](#tutorials--migration)
- [Case Studies](#case-studies)
- [Glossary](#glossary)
- [Articles & Blog](#articles--blog)
- [GraphRAG](#graphrag)
- [Audit Reports](#audit-reports)
- [Plans](#plans)
- [Progress](#progress)
- [Other Documents](#other-documents)

---

## Architecture Docs

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](architecture/ARCHITECTURE.md) | High-level system architecture overview |
| [TEXT_INDEX_DESIGN.md](architecture/TEXT_INDEX_DESIGN.md) | Tantivy-based text index implementation |
| [MUTATION_RECOVERY_PROTOCOL.md](architecture/MUTATION_RECOVERY_PROTOCOL.md) | Mutation recovery and derived index rebuild protocol |
| [ADVANCED_TOKENIZER.md](architecture/ADVANCED_TOKENIZER.md) | Multilingual text tokenizer with stemming and stopwords |
| [STORAGE_VERSIONING.md](architecture/STORAGE_VERSIONING.md) | Storage versioning strategy |
| [EXPERIMENTAL_GOVERNANCE_DESIGN.md](architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md) | Experimental governance design |
| [LISP_ANALYSIS.md](architecture/LISP_ANALYSIS.md) | LISP query language analysis |
| [WASM_STORAGE_REVIEW.md](architecture/WASM_STORAGE_REVIEW.md) | WASM storage backends review and audit |
| [COMP-026: LSM Compaction](architecture/adr/COMP-026-lsm-compaction-design.md) | Multi-level LSM compaction design (proposed ADR) |

---

## API Reference

| Document | Description |
|----------|-------------|
| [Embedded SDK](api/EMBEDDED_SDK.md) | Core Rust SDK reference — `VantaEmbedded` (~45 public methods, all types) |
| [Python SDK](api/PYTHON_SDK.md) | Python bindings — `vantadb-py` |
| [HTTP API](api/HTTP_API.md) | REST / HTTP server specification |
| [MCP API](api/MCP.md) | MCP (Model Context Protocol) server specification |
| [TypeScript SDK](api/TS_SDK.md) | TypeScript / WASM bindings — `vantadb-ts` |
| [IQL](api/IQL.md) | Interactive Query Language reference |

---

## Architecture Decision Records (ADR)

| Document | Description |
|----------|-------------|
| [ADR-001: Configuración Unificada](architecture/adr/001_unified_config_readonly.md) | Unified config + read-only barrier architecture |
| [ADR-002: WAL CRC32C + Auto-Healing](architecture/adr/002_wal_crc32c_autohealing.md) | WAL physical resilience, CRC32C validation, self-healing |
| [ADR-003: Sync/Async Decoupling](architecture/adr/003_sync_async_decoupling.md) | Concurrent execution isolation architecture |
| [ADR-004: Storage Backend](architecture/adr/004_storage_backend.md) | Storage backend abstraction |
| [ADR-005: HNSW Parameters](architecture/adr/005_hnsw_parameters.md) | HNSW parameter configuration |
| [ADR-006: RRF Constant](architecture/adr/006_rrf_constant.md) | Reciprocal Rank Fusion constant decision |
| [ADR-007: PyO3 Binding Architecture](architecture/adr/007_pyo3_binding_architecture.md) | Python binding architecture |
| [ADR-008: WASM Support Strategy](architecture/adr/008_wasm_support_strategy.md) | WASM build and support strategy |
| [ADR-009: Community Governance Model](architecture/adr/009_community_governance_model.md) | Community governance model |
| [ADR-0001: Adoptamos ADRs](architecture/adr/ADR-0001-ADOPTAMOS-ADRS.md) | Decision to adopt ADR process |

---

## Operations & Configuration

Full listing in [Operations Master Index](operations/master-index.md).

Key documents:

| Document | Description |
|----------|-------------|
| [CONFIGURATION.md](operations/CONFIGURATION.md) | All runtime configuration knobs, env vars, CLI commands |
| [BENCHMARKS.md](operations/BENCHMARKS.md) | Benchmark results and methodology |
| [DURABILITY_GUARANTEES.md](operations/DURABILITY_GUARANTEES.md) | WAL durability and crash guarantees |
| [PERFORMANCE_GUIDE.md](operations/PERFORMANCE_GUIDE.md) | Performance optimization guide |
| [PERFORMANCE_TUNING.md](operations/PERFORMANCE_TUNING.md) | Performance tuning parameters |
| [SECURITY.md](operations/SECURITY.md) | Security policies and procedures |
| [RELIABILITY_GATE.md](operations/RELIABILITY_GATE.md) | Reliability gate criteria and sign-off |
| [CI_POLICY.md](operations/CI_POLICY.md) | CI pipeline configuration and policy |
| [FUZZING.md](operations/FUZZING.md) | Fuzzing strategy and results |
| [BACKUP_POLICY.md](operations/BACKUP_POLICY.md) | Backup and restore procedures |
| [DEPLOYMENT_GUIDE.md](operations/DEPLOYMENT_GUIDE.md) | Deployment procedures and checklist |
| [DISASTER_RECOVERY_RUNBOOK.md](operations/DISASTER_RECOVERY_RUNBOOK.md) | Disaster recovery runbook |
| [GRAFANA_SETUP.md](operations/GRAFANA_SETUP.md) | Grafana dashboard setup for metrics |
| [MEMORY_TELEMETRY.md](operations/MEMORY_TELEMETRY.md) | Memory footprint telemetry design |
| [PYTHON_RELEASE_POLICY.md](operations/PYTHON_RELEASE_POLICY.md) | Python SDK release and publishing policy |
| [SQLITE_MIGRATION_GUIDE.md](operations/SQLITE_MIGRATION_GUIDE.md) | SQLite migration guide |
| [GC_TTL.md](operations/GC_TTL.md) | Garbage collection TTL configuration |

---

## Strategy & Vision

| Document | Description |
|----------|-------------|
| [ROADMAP.md](strategy/ROADMAP.md) | Engineering roadmap, phases, and execution plan |
| [GO_TO_MARKET.md](strategy/GO_TO_MARKET.md) | Go-to-market and ecosystem strategy |
| [VISION.md](vision/VISION.md) | Product vision and strategic positioning |
| [SHOW_HN_PREP.md](strategy/SHOW_HN_PREP.md) | Hacker News launch preparation |

---

## Tutorials & Migration

| Document | Description |
|----------|-------------|
| [Tutorials index](tutorials/index.md) | Structured learning path (ordered by complexity) |
| [01: AI Agent Memory](tutorials/01-ai-agent-memory.md) | Building AI agent memory with VantaDB |
| [02: Local RAG Pipeline](tutorials/02-local-rag-pipeline.md) | Local RAG pipeline tutorial |
| [04: Hybrid Search](tutorials/04-hybrid-search-basics.md) | Vector, BM25, and hybrid search modes |
| [05: Embedding Providers](tutorials/05-embedding-integrations.md) | OpenAI, Ollama, LiteLLM embedding patterns |
| [03: Migrating from ChromaDB](tutorials/03-migrating-from-chromadb.md) | Migration guide from ChromaDB to VantaDB |
| [Migrating from LanceDB](tutorials/migration-from-lancedb.md) | Migration guide from LanceDB to VantaDB |

---

## Case Studies

| Document | Description |
|----------|-------------|
| [RAG on Edge Devices](case_studies/rag_edge_device.md) | Running VantaDB on resource-constrained hardware |
| [Agent Local Memory with Ollama](case_studies/agent_local_memory_ollama.md) | AI agent using VantaDB with local Ollama inference |

---

## Glossary

The glossary lives in two complementary locations:

| Location | Description |
|----------|-------------|
| [glosario/](glosario/) | 57 individual term files with detailed definitions (English) |
| [glosario/README.md](glosario/README.md) | Categorized index with quick descriptions |

---

## Articles & Blog

Published blog posts (in `web/content/blog/`):

| Article | Description |
|---------|-------------|
| [Why I Built a Local Memory Engine for AI Agents in Rust](../web/content/blog/why-i-built-vantadb-local-memory-engine.md) | Motivation and design philosophy |
| [How Hybrid Search Works: BM25 + HNSW + RRF](../web/content/blog/how-hybrid-search-works.md) | Technical deep-dive on hybrid search |
| [SQLite for AI Agents: Benchmarks and Architecture](../web/content/blog/sqlite-for-ai-agents.md) | Comparing embedded databases for agent memory |
| [Introducing VantaDB](../web/content/blog/introducing-vantadb.md) | Product announcement |

---

## GraphRAG

| Document | Description |
|----------|-------------|
| [GraphRAG README](graphrag/README.md) | Graph-based RAG integration research |

---

## Audit Reports

| Document | Description |
|----------|-------------|
| [Full Audit 2026-07-18](audit-reports/audit-full-2026-07-18.md) | Comprehensive codebase audit |

---

## Plans

| Document | Description |
|----------|-------------|
| [PROMPT-MAESTRO-FREEZE.md](plans/PROMPT-MAESTRO-FREEZE.md) | Prompt maestro freeze plan |
| [ACTION_PLAN.md](strategy/ACTION_PLAN.md) | Archived — superseded by ROADMAP.md v2.0 |

---

## Progress

| Document | Description |
|----------|-------------|
| [progreso/README.md](progreso/README.md) | Unified progress log and development history |
| [Backlog.md](Backlog.md) | Full project backlog and feature tracking |
| [backlog-guide.md](backlog-guide.md) | Backlog management guide |
| [CHANGELOG.md](CHANGELOG.md) | Release history and version changelog |
| [stabilization-report.md](stabilization-report.md) | Stability report |

---

## Other Documents

| Document | Description |
|----------|-------------|
| [QUICKSTART.md](QUICKSTART.md) | Quickstart guide for new users |
| [FAQ.md](FAQ.md) | Frequently asked questions |
| [ci-cd-guide.md](ci-cd-guide.md) | CI/CD setup and operations guide |
| [DESIGN_RULES.md](DESIGN_RULES.md) | Design rules and conventions |
| [README.md](README.md) | Documentation landing page and reading guide |

---

## See Also

- [Operations Master Index](operations/master-index.md) — Detailed operations document listing
- [GitHub Repository](https://github.com/vantadb/vantadb) — Source code and issues
- [CHANGELOG](CHANGELOG.md) — Version history
