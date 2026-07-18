---
title: VantaDB Master Index
type: master-index
status: active
last_reviewed: 2026-07-13
review_interval_days: 90
language: en
aliases: [Index, Documentation Index, Master Index]
tags: [vantadb, documentation, index, master-index]
---

# VantaDB Master Index

> Index of all documentation, architecture decisions, protocol definitions, and operational references for the VantaDB project.

- **Project**: VantaDB — cross-platform memory layer for AI agents
- **Version**: 0.2.0
- **Last Updated**: 2026-07-13
- **Repository**: `https://github.com/ness-e/Vantadb`
- **Owner**: Eros

---

## Navigation

- [[#Vision|Vision]]
- [[#Strategy|Strategy]]
- [[#Architecture Docs|Architecture Docs]]
- [[#API Reference|API Reference]]
- [[#Operations & Configuration|Operations & Configuration]]
- [[#Architecture Decision Records (ADR)|Architecture Decision Records (ADR)]]
- [[#Glossary (glosario)|Glossary (glosario)]]
- [[#Articles & Publications|Articles & Publications]]
- [[#Case Studies|Case Studies]]
- [[#Migration Guides|Migration Guides]]
- [[#Experimental / Research|Experimental / Research]]
- [[#Other Documents|Other Documents]]
- [[#Progress|Progress]]
- [[#Meta / Configuration|Meta / Configuration]]

---

## Vision

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[vision/VISION.md\|VISION.md]] | UVP, ICP personas, competitive matrix, positioning, moat strategy, success metrics | Done |

---

## Strategy

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[strategy/ROADMAP.md\|ROADMAP.md]] | Engineering phases (Phase 3-5), exit criteria, architectural decisions pending, Q3 2026–Q2 2027 timeline | Done |
| 2 | [[strategy/GO_TO_MARKET.md\|GO_TO_MARKET.md]] | Distribution channels, integration tiers, 3-vertical market segmentation, business model, pricing tables, community/DevRel plan, GTM roadmap | Done |

---

## Architecture Docs

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[architecture/ARCHITECTURE.md\|ARCHITECTURE.md]] | High-level system architecture overview | Done |
| 2 | [[architecture/TEXT_INDEX_DESIGN.md\|TEXT_INDEX_DESIGN.md]] | Tantivy-based text index implementation | Done |
| 3 | [[architecture/MUTATION_RECOVERY_PROTOCOL.md\|MUTATION_RECOVERY_PROTOCOL.md]] | Mutation recovery and derived index rebuild protocol | Done |
| 4 | [[architecture/ADVANCED_TOKENIZER.md\|ADVANCED_TOKENIZER.md]] | Multilingual text tokenizer with stemming and stopwords | Done |

---

## API Reference

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[api/EMBEDDED_SDK.md\|EMBEDDED_SDK.md]] | Core Rust SDK reference — `VantaEmbedded` (~45 public methods, all types) | Done |
| 2 | [[api/PYTHON_SDK.md\|PYTHON_SDK.md]] | Python bindings — `vantadb-py` | Done |
| 3 | [[api/HTTP_API.md\|HTTP_API.md]] | REST / HTTP server specification — `GET /health`, `GET /metrics`, `POST /api/v2/query` | Done |
| 4 | [[api/MCP.md\|MCP.md]] | MCP (Model Context Protocol) server specification | Done |
| 5 | [[api/TS_SDK.md\|TS_SDK.md]] | TypeScript SDK — `vantadb-ts` (WASM bindings) | Done |

---

## Operations & Configuration

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[operations/CONFIGURATION.md\|CONFIGURATION.md]] | All runtime configuration knobs, env vars, CLI commands | Done |
| 2 | [[operations/MEMORY_TELEMETRY.md\|MEMORY_TELEMETRY.md]] | Memory footprint telemetry design | Done |
| 3 | [[operations/BENCHMARKS.md\|BENCHMARKS.md]] | Benchmark results and methodology | Done |
| 4 | [[operations/AGENT_INSTRUCTIONS.md\|AGENT_INSTRUCTIONS.md]] | Instructions for AI coding agents working with VantaDB | Done |
| 5 | [[operations/BACKUP_POLICY.md\|BACKUP_POLICY.md]] | Backup and restore procedures | Done |
| 6 | [[operations/CI_POLICY.md\|CI_POLICY.md]] | CI pipeline configuration and policy | Done |
| 7 | [[operations/COMMUNITY_GOVERNANCE.md\|COMMUNITY_GOVERNANCE.md]] | Community guidelines and governance model | Done |
| 8 | [[operations/DURABILITY_GUARANTEES.md\|DURABILITY_GUARANTEES.md]] | WAL durability and crash guarantees | Done |
| 9 | [[operations/EDITOR_INTEGRATIONS.md\|EDITOR_INTEGRATIONS.md]] | IDE / editor integration notes | Done |
| 10 | [[archive/EXECUTIVE_TECHNICAL_AUDIT.md\|EXECUTIVE_TECHNICAL_AUDIT.md]] | Full technical audit report (Archived) | Archived |
| 11 | [[operations/EXPERIMENTAL_FEATURES.md\|EXPERIMENTAL_FEATURES.md]] | Feature flags and experimental functionality | Done |
| 12 | [[operations/FUZZING.md\|FUZZING.md]] | Fuzzing strategy and results | Done |
| 13 | [[operations/GRAFANA_SETUP.md\|GRAFANA_SETUP.md]] | Grafana dashboard setup for metrics | Done |
| 14 | [[archive/MILESTONE_V0.2.0.md\|MILESTONE_V0.2.0.md]] | V0.2.0 milestone plan and tracking (Archived) | Done |
| 15 | [[operations/PILOT_PROGRAM.md\|PILOT_PROGRAM.md]] | Early access pilot program docs | Done |
| 16 | [[operations/PUBLIC_ISSUE_DRAFTS.md\|PUBLIC_ISSUE_DRAFTS.md]] | Public issue templates and drafts | Done |
| 17 | [[operations/PYTHON_RELEASE_POLICY.md\|PYTHON_RELEASE_POLICY.md]] | Python SDK release and publishing policy | Done |
| 18 | [[archive/RELEASE_V0.1.1.md\|RELEASE_V0.1.1.md]] | V0.1.1 release notes and tracking (Archived) | Done |
| 19 | [[operations/RELIABILITY_GATE.md\|RELIABILITY_GATE.md]] | Reliability gate criteria and sign-off | Done |
| 20 | [[operations/REPO_CHECKLIST.md\|REPO_CHECKLIST.md]] | Repository setup and maintenance checklist | Done |
| 21 | [[strategy/SHOW_HN_PREP.md\|SHOW_HN_PREP.md]] | Hacker News launch preparation | Done |
| 22 | [[archive/TEXT_INDEX_PHASE_1_CLOSEOUT.md\|TEXT_INDEX_PHASE_1_CLOSEOUT.md]] | Text index phase 1 closeout report (Archived) | Done |

---

## Architecture Decision Records (ADR)

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[architecture/adr/001_unified_config_readonly.md\|ADR-001: Unified Configuration]] | Unified config + read-only barrier architecture | Done |
| 2 | [[architecture/adr/002_wal_crc32c_autohealing.md\|ADR-002: WAL Physical Resilience]] | WAL physical resilience, CRC32C validation, self-healing | Done |
| 3 | [[architecture/adr/003_sync_async_decoupling.md\|ADR-003: Sync/Async Decoupling]] | Concurrent execution isolation architecture | Done |
| 4 | [[architecture/adr/004_storage_backend.md\|ADR-004: Storage Backend]] | Fjall vs RocksDB storage backend selection | Done |
| 5 | [[architecture/adr/005_hnsw_parameters.md\|ADR-005: HNSW Parameters]] | HNSW graph parameters M, M_max0, ef_construction, ef_search, ml | Done |
| 6 | [[architecture/adr/006_rrf_constant.md\|ADR-006: RRF Constant]] | RRF constant k=60 for Reciprocal Rank Fusion | Done |
| 7 | [[architecture/adr/007_pyo3_binding_architecture.md\|ADR-007: PyO3 Binding]] | PyO3 binding architecture for Python SDK | Done |
| 8 | [[architecture/adr/008_wasm_support_strategy.md\|ADR-008: WASM Strategy]] | WASM support strategy and browser deployment | Done |
| 9 | [[architecture/adr/009_community_governance_model.md\|ADR-009: Community Governance]] | Community governance model and contribution process | Done |

---

## Glossary (`glosario`)

The glossary lives in two complementary locations:

| Location | Description |
|----------|-------------|
| [[glosario/README.md\|Glossary Index]] | Categorized index with quick descriptions and cross-concept relationships |
| [[glosario/\|Glossary Folder]] | Directory containing individual term files with definitions, usage, and examples |

---

## Articles & Publications

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | Blog: Why I Built VantaDB | Motivation and design philosophy — URL TBD | Draft |
| 2 | Blog: SQLite for AI Agents | Comparing embedded databases — URL TBD | Draft |
| 3 | Blog: How Hybrid Search Works | Technical deep-dive on BM25 + vector fusion — URL TBD | Draft |

---

## Case Studies

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[case_studies/rag_edge_device.md\|RAG on Edge Devices]] | Running VantaDB on resource-constrained hardware | Draft |
| 2 | [[case_studies/agent_local_memory_ollama.md\|Agent Local Memory with Ollama]] | AI agent using VantaDB with local Ollama inference | Draft |

---

## Migration Guides

Migrated from `migration/` to `tutorials/` (consolidated into the tutorial series).

| # | Document | Description |
|---|----------|-------------|
| 1 | [[tutorials/03-migrating-from-chromadb.md\|From ChromaDB]] | Migrating from ChromaDB to VantaDB |
| 2 | [[tutorials/migration-from-lancedb.md\|From LanceDB]] | Migrating from LanceDB to VantaDB |

---

## Experimental / Research

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[experimental/IQL.md\|IQL — Interactive Query Language]] | Experimental query language for VantaDB | Draft |
| 2 | [[graphrag/README.md\|GraphRAG]] | Graph-based RAG integration research | Research |

---

## Web Site

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[web/README.md\|Web Site Overview]] | Documentation index for the VantaDB website project | Active |
| 2 | [[web/design/REDESIGN_V2_PLAN.md\|Design System]] | Master design specification (Swiss+Neubrutalism) | Active |
| 3 | [[web/design/TOKEN_SYSTEM.md\|Token System]] | Design tokens: typography, color, spacing, shadows | Active |
| 4 | [[web/design/COMPONENT_SPEC.md\|Component Spec]] | Component library specification (Nb system) | Active |
| 5 | [[web/product/PRODUCT.md\|Product]] | Product purpose, users, and personality | Active |
| 6 | [[web/product/SITE_MAP.md\|Site Map]] | Complete route inventory with status | Active |

---

## Other Documents

| # | Document | Description |
|---|----------|-------------|
| 1 | [[Backlog.md\|Backlog]] | Full project backlog and feature tracking |
| 2 | [[CHANGELOG.md\|CHANGELOG]] | Release history and version changelog |
| 3 | [[QUICKSTART.md\|QUICKSTART]] | Quickstart guide for new users |
| 4 | [[README.md\|Documentation Overview]] | Docs landing page and reading guide |
| 5 | [[progreso/bitacora.md\|Devlog (Bitacora)]] | Development log and daily notes |
| 6 | [[FAQ.md\|FAQ]] | Frequently Asked Questions |
| 7 | [[DESIGN_RULES.md\|Design Rules]] | Swiss + Neubrutalism visual design rules |
| 8 | [[backlog-guide.md\|Backlog Guide]] | Backlog management conventions |
| 9 | [[ci-cd-guide.md\|CI/CD Guide]] | CI/CD pipeline guide for contributors |

---

## Progress

See [[CHANGELOG.md]] for version history, [[Backlog.md]] for active tasks, and [[progreso/README.md\|Progress Dashboard]] for a complete checklist of completed tasks.

---

## Reviews & Audits

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 1 | [[reviews/FINAL-REVIEW.md\|Final Review]] | Skills ecosystem evaluation and cleanup plan | Active |
| 2 | [[archive/FULL_CODEBASE_AUDIT_2026-07-09.md\|Codebase Audit (Jul 9)]] | Full Rust core + bindings + web audit (Archived, superseded) | Archived |
| 3 | [[reviews/FULL_CODEBASE_AUDIT_2026-07-11.md\|Codebase Audit (Jul 11)]] | Updated full codebase audit (replaces Jul 9) | Active |
| 4 | [[reviews/2026-07-13-full-review.md\|Full Review (Jul 13)]] | Latest comprehensive review | Active |
| 5 | [[reviews/analisis_proyecto.md\|Project Analysis]] | Architecture, security, concurrency, code quality | Active |

---

## References & Troubleshooting

| # | Document | Description |
|---|----------|-------------|
| 1 | [[references/troubleshooting.md\|Troubleshooting]] | Common Windows build/runtime issues |
| 2 | [[references/bug-workflow.md\|Bug Workflow]] | Bug reporting and triage process |
| 3 | [[references/reading-nextest-output.md\|Nextest Output]] | How to read cargo-nextest test results |

---

## Research

| # | Document | Description |
|---|----------|-------------|
| 1 | [[research/VantaDB_RESEARCH_UNIFIED.md\|Unified Research]] | Consolidated cross-agent research report |
| 2 | [[research/VantaDB_ANALISIS_COMPLETO.md\|Complete Analysis]] | Full project analysis and decisions |
| 3 | [[research/ACID_TRANSACTIONS.md\|ACID Transactions]] | ACID compliance research |
| 4 | [[research/SIGNED_RELEASES.md\|Signed Releases]] | Sigstore/SLSA release signing research |
| 5 | [[research/VantaDB_RESEARCH_VALIDADO.md\|Validated Research]] | Validated and cross-checked research findings |
| 6 | [[archive/DOCS_TOOLS_RESEARCH.md\|Docs Tools Research]] | Documentation tool evaluation (Archived) |
| 7 | [[archive/DOCS_AUDIT_REPORT.md\|Docs Audit Report]] | Comprehensive documentation audit (Archived) |
| 8 | [[archive/COGNEE_EVALUATION.md\|Cognée Evaluation]] | Evaluation of the Cognée project (Archived) |
| 9 | [[archive/SQL_ANALYSIS.md\|SQL Analysis]] | SQL-based query language analysis (Archived) |

---

## Meta / Configuration

| File | Description |
|------|-------------|
| `Cargo.toml` | Rust project manifest |
| `opencode.jsonc` | OpenCode agent configuration |
