---
title: VantaDB Master Index
type: master-index
status: active
last_reviewed: 2026-08-22
tags: [vantadb, documentation, index, master-index]
---

# VantaDB Master Index

> Global index of all documentation, architecture decisions, API references, and operational guides.

> **Maintenance rule:** every new doc or first-level folder under `docs/` MUST be indexed here **in the same PR** that adds it. Deliberate exclusions are listed at the bottom of this file.

- **Project:** VantaDB — cross-platform memory layer for AI agents
- **Repository:** `https://github.com/ness-e/Vantadb`
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
- [Audit Reports & Reviews](#audit-reports--reviews)
- [Pipeline Reports](#pipeline-reports)
- [Plans](#plans)
- [Progress & Planning](#progress--planning)
- [Research & Investigations](#research--investigations)
- [CI Workflows](#ci-workflows)
- [Web Frontend Docs](#web-frontend-docs)
- [Benchmarks](#benchmarks)
- [Book](#book)
- [Community & Examples](#community--examples)
- [Other Documents](#other-documents)
- [Deliberately Not Indexed](#deliberately-not-indexed)

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
| [HTTP API](api/HTTP_API.md) | REST / HTTP server specification (regenerated 2026-08-22, GOV-B5) |
| [OpenAPI spec](api/openapi.yaml) | Machine-readable OpenAPI contract for the HTTP API (GOV-B4) |
| [MCP API](api/MCP.md) | MCP server spec — **stub**; single source of truth: [`skills/vantadb-mcp/references/api-reference.md`](../skills/vantadb-mcp/references/api-reference.md) |
| [TypeScript SDK](api/TS_SDK.md) | TypeScript / WASM bindings — `vantadb-ts` |
| [IQL](api/IQL.md) | Interactive Query Language reference |
| [VANTA_MEMORY.md](api/VANTA_MEMORY.md) | Vanta memory subsystem API (LLM-free stores, personas, wiki) |
| [GRAPH_RAG.md](api/GRAPH_RAG.md) | GraphRAG public API reference |
| [BINDINGS_NAMESPACES.md](api/BINDINGS_NAMESPACES.md) | Namespace map across Python / TS / WASM bindings |
| [WASM_PERSISTENCE.md](api/WASM_PERSISTENCE.md) | WASM persistence backends (OPFS / IndexedDB / memory) |
| [WASM_STANDALONE.md](api/WASM_STANDALONE.md) | WASM standalone build and usage guide |

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
| [chaos-testing.md](chaos-testing.md) | Chaos testing strategy and scenarios |
| [TEST_MAP.md](TEST_MAP.md) | Map of test files to subsystems and coverage intent |

---

## Strategy & Vision

| Document | Description |
|----------|-------------|
| [ROADMAP.md](strategy/ROADMAP.md) | Engineering roadmap, phases, and execution plan |
| [GO_TO_MARKET.md](strategy/GO_TO_MARKET.md) | Go-to-market and ecosystem strategy |
| [VANTADB-PRO-FEATURES.md](strategy/VANTADB-PRO-FEATURES.md) | Open Core boundary — features Pro vs community |
| [VANTADB-PRO-DELIVERY.md](strategy/VANTADB-PRO-DELIVERY.md) | Delivery and distribution of VantaDB Pro |
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

Runnable code samples live in [`examples/`](examples/) (`fnd05_python_context_manager.py`, `fnd05_ts_async_dispose.ts`).

---

## Case Studies

Archivados a `docs/archive/case-studies-unverified/` (2026-08-22, GOV-B1): material interno no-público, escenarios ilustrativos SIN verificación. El case study real llega vía CLD-04.

## Glossary

The glossary lives in two complementary locations:

| Location | Description |
|----------|-------------|
| [glosario/](glosario/) | 57 individual term files with detailed definitions (English) |
| [glosario/README.md](glosario/README.md) | Categorized index with quick descriptions |

---

## Articles & Blog

Published blog posts (in `docs/blog/`):

| Article | Description |
|---------|-------------|
| [Why I Built a Local Memory Engine for AI Agents in Rust](blog/why_i_built.md) | Motivation and design philosophy |
| [How Hybrid Search Works: BM25 + HNSW + RRF](blog/how_hybrid_search_works.md) | Technical deep-dive on hybrid search |
| [SQLite for AI Agents: Benchmarks and Architecture](blog/sqlite_for_ai_agents.md) | Comparing embedded databases for agent memory |
| [Introducing VantaDB](blog/introducing_vantadb.md) | Product announcement |
| [Benchmarks vs LanceDB & Chroma](blog/benchmarks_vs_lancedb_chroma.md) | Competitive benchmark write-up |
| [GraphRAG Benchmark](blog/graphrag-benchmark.md) | GraphRAG performance evaluation post |
| [Campaign: AI Agent Memory](blog/campaign-ai-agent-memory.md) | Campaign narrative on agent memory |

---

## GraphRAG

| Document | Description |
|----------|-------------|
| [GraphRAG README](graphrag/README.md) | Graph-based RAG integration research |
| [GraphRAG API](api/GRAPH_RAG.md) | Public GraphRAG API reference |

---

## Audit Reports & Reviews

Audit and review reports generated by `/audit`, `/review`, and `unified-review` live in `docs/reviews/`.

| Document | Description |
|----------|-------------|
| [auditoria-documentacion-2026-08-21.md](reviews/auditoria-documentacion-2026-08-21.md) | Documentation audit that motivated the GOV campaign |
| [audit-full-20260812-231204.md](reviews/audit-full-20260812-231204.md) | Full audit run 2026-08-12 |
| [review-certify-2026-08-05-2025.md](reviews/review-certify-2026-08-05-2025.md) | Certification review 2026-08-05 |
| [stabilization-report.md](reviews/stabilization-report.md) | Stabilization phase report |
| Historical runs (including `audit-full-2026-07-18`) are archived under [`reviews/archive/`](reviews/archive/) |

---

## Pipeline Reports

Structured reports produced by pipelines and evals live in `docs/reports/`.

| Document | Description |
|----------|-------------|
| [INDEX.md](reports/INDEX.md) | Index of all pipeline reports |
| [dora.md](reports/dora.md) | DORA metrics report |
| [northstar.md](reports/northstar.md) | Northstar tracking report |
| [pipeline-evals.md](reports/pipeline-evals.md) | Pipeline evaluation report |

---

## Plans

Active plans in `docs/plans/`; completed plans move to `plans/archive/`.

| Document | Description |
|----------|-------------|
| [PROMPT-MAESTRO-FREEZE.md](plans/archive/PROMPT-MAESTRO-FREEZE.md) | Prompt maestro freeze plan (archived) |
| [ACTION_PLAN → ROADMAP v2.0](strategy/ROADMAP.md) | Archived — superseded by ROADMAP.md v2.0 |

---

## Progress & Planning

Spanish-language planning material (allowed exception to the English docs rule).

| Document | Description |
|----------|-------------|
| [progreso/README.md](progreso/README.md) | Unified progress log and development history |
| [Backlog.md](Backlog.md) | Full project backlog and feature tracking |
| [backlog-futuro.md](backlog-futuro.md) | Deferred / future backlog items |
| [CHANGELOG.md](CHANGELOG.md) | Release history and version changelog |
| [avance/](avance/README.md) | Progress tracking workspace (activo, auditoría, decisiones, historial) |

---

## Research & Investigations

| Document | Description |
|----------|-------------|
| [Investigaciones → research/](research/) | Spanish-language research notes: FND-*, INV-*, TIR-*, competitive analyses (see folder README per series) |
| [research/human-facing-db-ui/](research/human-facing-db-ui/) | Research on human-facing DB UI concepts |
| [research/tdam/](research/tdam/) | TDAM (Tiered Document Attention Model) research |
| [wasm/CRASH_MODEL.md](wasm/CRASH_MODEL.md) | WASM crash model research |

---

## CI Workflows

Per-workflow documentation mirrors `.github/workflows/`.

| Document | Description |
|----------|-------------|
| [ci-gate.md](workflow/ci-gate.md) | Fast Gate workflow |
| [ci-rust-10.md](workflow/ci-rust-10.md) | Rust CI workflow |
| [ci-web-11.md](workflow/ci-web-11.md) | Web CI workflow |
| [gate-docs-21.md](workflow/gate-docs-21.md) | Docs gate workflow |
| [fuzz-40.md](workflow/fuzz-40.md) | Fuzzing workflow |
| [chaos-45.md](workflow/chaos-45.md) | Chaos testing workflow |
| [perf-bench-40.md](workflow/perf-bench-40.md) | Performance benchmark workflow |
| [heavy-bench-nightly-51.md](workflow/heavy-bench-nightly-51.md) | Nightly heavy benchmark workflow |
| [heavy-certification-50.md](workflow/heavy-certification-50.md) | Heavy certification workflow |
| [sec-codeql-30.md](workflow/sec-codeql-30.md) | CodeQL security workflow |
| [release-wheels-60.md](workflow/release-wheels-60.md) | Wheels release workflow |
| [release-npm-61.md](workflow/release-npm-61.md) | NPM release workflow |
| [release-adapters-62.md](workflow/release-adapters-62.md) | Adapters release workflow |
| [release-binaries-63.md](workflow/release-binaries-63.md) | Binaries release workflow |
| [release-sbom-64.md](workflow/release-sbom-64.md) | SBOM release workflow |

---

## Web Frontend Docs

Documentation for the Next.js web frontend lives in `docs/web/`.

| Document | Description |
|----------|-------------|
| [web/README.md](web/README.md) | Web docs landing page |
| [design-rules-es-tutorial.md](archive/design-rules-es-tutorial.md) | Tutorial ES de diseño (ARCHIVADO — ver banner) |
| [QA.md](web/QA.md) | Web QA checklist |
| Sub-folders: [`audit/`](web/audit/), [`guides/`](web/guides/), [`reference/`](web/reference/), [`standards/`](web/standards/) |

---

## Benchmarks

Raw benchmark artifacts and analyses live in `docs/benchmarks/`. Canonical claims belong in [`operations/BENCHMARKS.md`](operations/BENCHMARKS.md).

| Document | Description |
|----------|-------------|
| [COMPETITIVE_ANALYSIS.md](benchmarks/COMPETITIVE_ANALYSIS.md) | Competitive analysis narrative |
| [COMPETITIVE_SDK_BENCH.md](benchmarks/COMPETITIVE_SDK_BENCH.md) | SDK benchmark vs competitors |
| [ivf_bench.md](benchmarks/ivf_bench.md) | IVF index benchmark notes |

---

## Book

| Document | Description |
|----------|-------------|
| [book/](book/book.toml) | mdBook project (`mdbook build` from `docs/book/`) — narrative book on VantaDB internals |

---

## Community & Examples

| Document | Description |
|----------|-------------|
| [discord/README.md](discord/README.md) | Discord community workspace (server config, bilingual strategy) |
| [examples/](examples/) | Runnable code samples referenced by tutorials |

Assets (images used by docs): [`assets/`](assets/) — `demo_terminal.png`, `social-preview.png`.

Agent-facing reference files (bug workflow, troubleshooting, nextest output): [`references/`](references/).

Historical material moved out of the main tree: [`archive/`](archive/) (incl. archived case studies and legacy docs inventories).

---

## Other Documents

| Document | Description |
|----------|-------------|
| [QUICKSTART.md](QUICKSTART.md) | Quickstart guide for new users |
| [FAQ.md](FAQ.md) | Frequently asked questions |
| [COMPARISON.md](COMPARISON.md) | Honest comparison vs sqlite-vec / LanceDB / Qdrant / Chroma — qualitative table, benchmark provenance per Regla 11, practical limits |
| [ci-cd-guide.md](ci-cd-guide.md) | CI/CD setup and operations guide |
| [README.md](README.md) | Documentation landing page and reading guide |

---

## See Also

- [Operations Master Index](operations/master-index.md) — Detailed operations document listing
- [Pipeline Reports Index](reports/INDEX.md) — Detailed pipeline report listing
- [GitHub Repository](https://github.com/ness-e/Vantadb) — Source code and issues
- [CHANGELOG](CHANGELOG.md) — Version history

---

## Deliberately Not Indexed

First-level entries excluded from this index, with reason:

| Entry | Reason |
|-------|--------|
| `_templates/` | Internal doc templates (ADR, glossary term, note) — not reader-facing documentation |
| `.obsidian/` | Personal Obsidian vault configuration — not project documentation |
| `TDAM-VANTADB/` | Empty directory, pending deletion |
