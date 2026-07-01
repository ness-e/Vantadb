---
title: VantaDB Master Index
type: master-index
status: active
last_reviewed: 2026-06-22
review_interval_days: 90
language: en
aliases: [Index, Documentation Index, Master Index]
tags: [vantadb, documentation, index, master-index]
---

# VantaDB Master Index

> Index of all documentation, architecture decisions, protocol definitions, and operational references for the VantaDB project.

- **Project**: VantaDB — cross-platform memory layer for AI agents
- **Version**: 0.1.5
- **Last Updated**: 2026-06-22
- **Repository**: `https://github.com/vantadb/vantadb`
- **Owner**: Eros

---

## Navigation

- [Architecture Docs](#architecture-docs)
- [API Reference](#api-reference)
- [Operations & Configuration](#operations--configuration)
- [Architecture Decision Records (ADR)](#architecture-decision-records-adr)
- [Architecture Audits](#architecture-audits)
- [Glossary (`glosario`)](#glossary-glosario)
- [Articles & Publications](#articles--publications)
- [Case Studies](#case-studies)
- [Migration Guides](#migration-guides)
- [Experimental / Research](#experimental--research)
- [Other Documents](#other-documents)
- [MPTS Vault](#mpts-vault)
- [Progress](#progress)
- [MPTS Checkpoints](#mpts-checkpoints)
- [Meta / Configuration](#meta--configuration)

---

## Architecture Docs

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) | High-level system architecture overview | Done |
| 2 | [TEXT_INDEX_DESIGN.md](../architecture/TEXT_INDEX_DESIGN.md) | Tantivy-based text index implementation | Done |
| 3 | [MUTATION_RECOVERY_PROTOCOL.md](../architecture/MUTATION_RECOVERY_PROTOCOL.md) | Mutation recovery and derived index rebuild protocol | Done |
| 4 | [ADVANCED_TOKENIZER.md](../architecture/ADVANCED_TOKENIZER.md) | Multilingual text tokenizer with stemming and stopwords | Done |

---

## API Reference

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [Embedded SDK](../api/EMBEDDED_SDK.md) | Core Rust SDK reference — `VantaEmbedded` (~45 public methods, all types) | Done |
| 2 | [Python SDK](../api/PYTHON_SDK.md) | Python bindings — `vantadb-py` | Done |
| 3 | [HTTP API](../api/HTTP_API.md) | REST / HTTP server specification — `GET /health`, `GET /metrics`, `POST /api/v2/query` | Done |
| 4 | [MCP API](../api/MCP.md) | MCP (Model Context Protocol) server specification | Done |

---

## Operations & Configuration

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [CONFIGURATION.md](../operations/CONFIGURATION.md) | All runtime configuration knobs, env vars, CLI commands | Done |
| 2 | [MEMORY_TELEMETRY.md](../operations/MEMORY_TELEMETRY.md) | Memory footprint telemetry design | Done |
| 3 | [BENCHMARKS.md](../operations/BENCHMARKS.md) | Benchmark results and methodology | Done |
| 4 | [AGENT_INSTRUCTIONS.md](../operations/AGENT_INSTRUCTIONS.md) | Instructions for AI coding agents working with VantaDB | Done |
| 5 | [BACKUP_POLICY.md](../operations/BACKUP_POLICY.md) | Backup and restore procedures | Done |
| 6 | [CI_POLICY.md](../operations/CI_POLICY.md) | CI pipeline configuration and policy | Done |
| 7 | [COMMUNITY_GOVERNANCE.md](../operations/COMMUNITY_GOVERNANCE.md) | Community guidelines and governance model | Done |
| 8 | [DURABILITY_GUARANTEES.md](../operations/DURABILITY_GUARANTEES.md) | WAL durability and crash guarantees | Done |
| 9 | [EDITOR_INTEGRATIONS.md](../operations/EDITOR_INTEGRATIONS.md) | IDE / editor integration notes | Done |
| 10 | [EXECUTIVE_TECHNICAL_AUDIT.md](../operations/EXECUTIVE_TECHNICAL_AUDIT.md) | Full technical audit report | Done |
| 11 | [EXPERIMENTAL_FEATURES.md](../operations/EXPERIMENTAL_FEATURES.md) | Feature flags and experimental functionality | Done |
| 12 | [FUZZING.md](../operations/FUZZING.md) | Fuzzing strategy and results | Done |
| 13 | [GRAFANA_SETUP.md](../operations/GRAFANA_SETUP.md) | Grafana dashboard setup for metrics | Done |
| 14 | [MILESTONE_V0.2.0.md](../operations/MILESTONE_V0.2.0.md) | V0.2.0 milestone plan and tracking | Done |
| 15 | [PILOT_PROGRAM.md](../operations/PILOT_PROGRAM.md) | Early access pilot program docs | Done |
| 16 | [PUBLIC_ISSUE_DRAFTS.md](../operations/PUBLIC_ISSUE_DRAFTS.md) | Public issue templates and drafts | Done |
| 17 | [PYTHON_RELEASE_POLICY.md](../operations/PYTHON_RELEASE_POLICY.md) | Python SDK release and publishing policy | Done |
| 18 | [RELEASE_V0.1.1.md](../operations/RELEASE_V0.1.1.md) | V0.1.1 release notes and tracking | Done |
| 19 | [RELIABILITY_GATE.md](../operations/RELIABILITY_GATE.md) | Reliability gate criteria and sign-off | Done |
| 20 | [REPO_CHECKLIST.md](../operations/REPO_CHECKLIST.md) | Repository setup and maintenance checklist | Done |
| 21 | [SHOW_HN_PREP.md](../operations/SHOW_HN_PREP.md) | Hacker News launch preparation | Done |
| 22 | [TEXT_INDEX_PHASE_1_CLOSEOUT.md](../operations/TEXT_INDEX_PHASE_1_CLOSEOUT.md) | Text index phase 1 closeout report | Done |

---

## Architecture Decision Records (ADR)

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [ADR-001: Configuración Unificada](../architecture/adr/001_unified_config_readonly.md) | Unified config + read-only barrier architecture | Done |
| 2 | [ADR-002: WAL CRC32C + Auto-Healing](../architecture/adr/002_wal_crc32c_autohealing.md) | WAL physical resilience, CRC32C validation, self-healing | Done |
| 3 | [ADR-003: Sync/Async Decoupling](../architecture/adr/003_sync_async_decoupling.md) | Concurrent execution isolation architecture | Done |

---

## Architecture Audits

*(Historical audit reports moved to [`docs/archive/audits/`](../archive/audits/))*

---

## Glossary (`glosario`)

The glossary lives in two complementary locations:

| Location | Description |
|----------|-------------|
| [Glosario.md](Glosario.md) | Categorized index with quick descriptions and cross-concept relationships |
| [Glosario/](Glosario/) | Individual term files (~50) with detailed definitions, usage, and examples |

---

## Articles & Publications

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [Why I Built a Local Memory Engine for AI Agents](../articles/why_i_built_local_memory_engine.md) | Motivation and design philosophy behind VantaDB | Published |
| 2 | [SQLite for AI Agents](../articles/sqlite_for_ai_agents.md) | Comparing embedded databases for agent memory | Published |
| 3 | [How Hybrid Search Works in VantaDB](../articles/how_hybrid_search_works.md) | Technical deep-dive on BM25 + vector fusion | Published |

## Case Studies

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [RAG on Edge Devices](../case_studies/rag_edge_device.md) | Running VantaDB on resource-constrained hardware | Draft |
| 2 | [Agent Local Memory with Ollama](../case_studies/agent_local_memory_ollama.md) | AI agent using VantaDB with local Ollama inference | Draft |

## Migration Guides

| # | Document | Description |
|---|----------|-------------|
| 1 | [From ChromaDB](../migration/FROM_CHROMADB.md) | Migrating from ChromaDB to VantaDB |
| 2 | [From LanceDB](../migration/FROM_LANCEDB.md) | Migrating from LanceDB to VantaDB |

## Experimental / Research

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [IQL — Interactive Query Language](../experimental/IQL.md) | Experimental query language for VantaDB | Draft |
| 2 | [GraphRAG](../graphrag/README.md) | Graph-based RAG integration research | Research |

## Other Documents

| # | Document | Description |
|---|----------|-------------|
| 1 | [Backlog](../Backlog.md) | Full project backlog and feature tracking |
| 2 | [CHANGELOG](../CHANGELOG.md) | Release history and version changelog |
| 3 | [QUICKSTART](../QUICKSTART.md) | Quickstart guide for new users |
| 4 | [Documentation Overview](../README.md) | Docs landing page and reading guide |
| 5 | [Bitácora](../Bitácora.md) | Development log and daily notes |
| 6 | [Investigaciones](../Investigaciones.md) | Research index and investigation notes |

## MPTS Vault

*(6 main section docs archived to [`docs/archive/`](../archive/) — keep English docs as source of truth)*

---

## Progress

See [`docs/CHANGELOG.md`](../CHANGELOG.md) for version history and [`docs/Backlog.md`](../Backlog.md) for active tasks.

---



## Meta / Configuration

| File | Description |
|------|-------------|
| `.opencode/skills/progreso/` | OpenCode skills configuration |
| `opencode.json` | OpenCode agent configuration |
| `Cargo.toml` | Rust project manifest |
