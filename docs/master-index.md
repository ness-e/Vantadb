---
title: VantaDB Master Index
type: master-index
status: active
tags: [vantadb, documentation, index, master-index]
last_reviewed: 2026-09-15
aliases: []
related: [Backlog.md, backlog-futuro.md, Backlog-negocio.md, CHANGELOG.md, QUICKSTART.md, avance/README.md]
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
| [ARCHITECTURE.md](dev/architecture/ARCHITECTURE.md) | High-level system architecture overview |
| [TEXT_INDEX_DESIGN.md](dev/architecture/TEXT_INDEX_DESIGN.md) | Tantivy-based text index implementation |
| [MUTATION_RECOVERY_PROTOCOL.md](dev/architecture/MUTATION_RECOVERY_PROTOCOL.md) | Mutation recovery and derived index rebuild protocol |
| [ADVANCED_TOKENIZER.md](dev/architecture/ADVANCED_TOKENIZER.md) | Multilingual text tokenizer with stemming and stopwords |
| [STORAGE_VERSIONING.md](dev/architecture/STORAGE_VERSIONING.md) | Storage versioning strategy |
| [EXPERIMENTAL_GOVERNANCE_DESIGN.md](dev/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md) | Experimental governance design |
| [LISP_ANALYSIS.md](dev/architecture/LISP_ANALYSIS.md) | LISP query language analysis |
| [WASM_STORAGE_REVIEW.md](dev/architecture/WASM_STORAGE_REVIEW.md) | WASM storage backends review and audit |
| [COMP-026: LSM Compaction](dev/architecture/adr/COMP-026-lsm-compaction-design.md) | Multi-level LSM compaction design (proposed ADR) |

---

## API Reference

| Document | Description |
|----------|-------------|
| [Embedded SDK](api/EMBEDDED_SDK.md) | Core Rust SDK reference — `VantaEmbedded` (~45 public methods, all types) |
| [Python SDK](api/PYTHON_SDK.md) | Python bindings — `vantadb-py` |
| [HTTP API](api/HTTP_API.md) | REST / HTTP server specification (regenerated 2026-08-22, GOV-B5) |
| [OpenAPI spec](api/openapi.yaml) | Machine-readable OpenAPI contract for the HTTP API (GOV-B4) |
| [MCP API](api/MCP.md) | MCP server spec — **stub**; single source of truth: [`skills/vantadb-mcp/references/api-reference.md`](../skills/vantadb-mcp/references/api-reference.md) |
| [Proxy](api/PROXY.md) | `vanta-proxy` reference — endpoints, 8 opt-in features, defaults, env |
| [TypeScript SDK](api/TS_SDK.md) | TypeScript / WASM bindings — `vantadb-ts` |
| [IQL](api/IQL.md) | Interactive Query Language reference |
| [VANTA_MEMORY.md](api/VANTA_MEMORY.md) | Vanta memory subsystem API (LLM-free stores, personas, wiki) |
| [GRAPH_RAG.md](api/GRAPH_RAG.md) | GraphRAG public API reference |
| [BINDINGS_NAMESPACES.md](api/BINDINGS_NAMESPACES.md) | Namespace map across Python / TS / WASM bindings |
| [WASM_PERSISTENCE.md](api/WASM_PERSISTENCE.md) | WASM persistence backends (OPFS / IndexedDB / memory) |
| [WASM_STANDALONE.md](api/WASM_STANDALONE.md) | WASM standalone build and usage guide |
| [EMBEDDINGS.md](api/EMBEDDINGS.md) | Embedding providers & local model config |
| [ERROR_HANDLING.md](api/ERROR_HANDLING.md) | Error taxonomy & VantaError mapping |
| [NODE_SDK.md](api/NODE_SDK.md) | Node.js bindings — `vantadb-node` |
| [VERSIONING.md](api/VERSIONING.md) | API versioning & semver guarantees |
| [WASM_API.md](api/WASM_API.md) | WASM high-level API surface |
| [scores.md](api/scores.md) | Score semantics — RRF / cosine / BM25 / zero-norm (RES-04) |

---

## Architecture Decision Records (ADR)

| Document | Description |
|----------|-------------|
| [ADR-001: Configuración Unificada](dev/architecture/adr/001_unified_config_readonly.md) | Unified config + read-only barrier architecture |
| [ADR-002: WAL CRC32C + Auto-Healing](dev/architecture/adr/002_wal_crc32c_autohealing.md) | WAL physical resilience, CRC32C validation, self-healing |
| [ADR-003: Sync/Async Decoupling](dev/architecture/adr/003_sync_async_decoupling.md) | Concurrent execution isolation architecture |
| [ADR-004: Storage Backend](dev/architecture/adr/004_storage_backend.md) | Storage backend abstraction |
| [ADR-005: HNSW Parameters](dev/architecture/adr/005_hnsw_parameters.md) | HNSW parameter configuration |
| [ADR-006: RRF Constant](dev/architecture/adr/006_rrf_constant.md) | Reciprocal Rank Fusion constant decision |
| [ADR-007: PyO3 Binding Architecture](dev/architecture/adr/007_pyo3_binding_architecture.md) | Python binding architecture |
| [ADR-008: WASM Support Strategy](dev/architecture/adr/008_wasm_support_strategy.md) | WASM build and support strategy |
| [ADR-009: Community Governance Model](dev/architecture/adr/009_community_governance_model.md) | Community governance model |
| [ADR-0001: Adoptamos ADRs](dev/architecture/adr/ADR-0001-ADOPTAMOS-ADRS.md) | Decision to adopt ADR process |

---

## Operations & Configuration

Full listing in [Operations Master Index](user/operations/master-index.md).

Key documents:

| Document | Description |
|----------|-------------|
| [CONFIGURATION.md](user/operations/CONFIGURATION.md) | All runtime configuration knobs, env vars, CLI commands |
| [BENCHMARKS.md](user/operations/BENCHMARKS.md) | Benchmark results and methodology |
| [DURABILITY_GUARANTEES.md](user/operations/DURABILITY_GUARANTEES.md) | WAL durability and crash guarantees |
| [PERFORMANCE_GUIDE.md](user/operations/PERFORMANCE_GUIDE.md) | Performance optimization guide |
| [PERFORMANCE_TUNING.md](user/operations/PERFORMANCE_TUNING.md) | Performance tuning parameters |
| [SECURITY.md](user/operations/SECURITY.md) | Security policies and procedures |
| [RELIABILITY_GATE.md](user/operations/RELIABILITY_GATE.md) | Reliability gate criteria and sign-off |
| [CI_POLICY.md](user/operations/CI_POLICY.md) | CI pipeline configuration and policy |
| [FUZZING.md](user/operations/FUZZING.md) | Fuzzing strategy and results |
| [BACKUP_POLICY.md](user/operations/BACKUP_POLICY.md) | Backup and restore procedures |
| [DEPLOYMENT_GUIDE.md](user/operations/DEPLOYMENT_GUIDE.md) | Deployment procedures and checklist |
| [DISASTER_RECOVERY_RUNBOOK.md](user/operations/DISASTER_RECOVERY_RUNBOOK.md) | Disaster recovery runbook |
| [GRAFANA_SETUP.md](user/operations/GRAFANA_SETUP.md) | Grafana dashboard setup for metrics |
| [MEMORY_TELEMETRY.md](user/operations/MEMORY_TELEMETRY.md) | Memory footprint telemetry design |
| [PYTHON_RELEASE_POLICY.md](user/operations/PYTHON_RELEASE_POLICY.md) | Python SDK release and publishing policy |
| [SQLITE_MIGRATION_GUIDE.md](user/operations/SQLITE_MIGRATION_GUIDE.md) | SQLite migration guide |
| [GC_TTL.md](user/operations/GC_TTL.md) | Garbage collection TTL configuration |
| [chaos-testing.md](chaos-testing.md) | Chaos testing strategy and scenarios |
| [TEST_MAP.md](TEST_MAP.md) | Map of test files to subsystems and coverage intent |

---

## Strategy & Vision

| Document | Description |
|----------|-------------|
| [ROADMAP.md](dev/strategy/ROADMAP.md) | Engineering roadmap, phases, and execution plan |
| [GO_TO_MARKET.md](dev/strategy/GO_TO_MARKET.md) | Go-to-market and ecosystem strategy |
| [VANTADB-PRO-FEATURES.md](dev/strategy/VANTADB-PRO-FEATURES.md) | Open Core boundary — features Pro vs community |
| [VANTADB-PRO-DELIVERY.md](dev/strategy/VANTADB-PRO-DELIVERY.md) | Delivery and distribution of VantaDB Pro |
| [VISION.md](dev/vision/VISION.md) | Product vision and strategic positioning |
| [SHOW_HN_PREP.md](dev/strategy/SHOW_HN_PREP.md) | Hacker News launch preparation |

---

## Tutorials & Migration

| Document | Description |
|----------|-------------|
| [Tutorials index](user/tutorials/index.md) | Structured learning path (ordered by complexity) |
| [01: AI Agent Memory](user/tutorials/01-ai-agent-memory.md) | Building AI agent memory with VantaDB |
| [02: Local RAG Pipeline](user/tutorials/02-local-rag-pipeline.md) | Local RAG pipeline tutorial |
| [04: Hybrid Search](user/tutorials/04-hybrid-search-basics.md) | Vector, BM25, and hybrid search modes |
| [05: Embedding Providers](user/tutorials/05-embedding-integrations.md) | OpenAI, Ollama, LiteLLM embedding patterns |
| [03: Migrating from ChromaDB](user/tutorials/03-migrating-from-chromadb.md) | Migration guide from ChromaDB to VantaDB |
| [Migrating from LanceDB](user/tutorials/migration-from-lancedb.md) | Migration guide from LanceDB to VantaDB |

Runnable code samples live in [`examples/`](user/examples/) (`fnd05_python_context_manager.py`, `fnd05_ts_async_dispose.ts`).

---

## Case Studies

Archivados a `docs/dev/archive/case-studies-unverified/` (2026-08-22, GOV-B1): material interno no-público, escenarios ilustrativos SIN verificación. El case study real llega vía CLD-04.

## Glossary

The glossary lives in two complementary locations:

| Location | Description |
|----------|-------------|
| [glosario/](user/glosario/) | 57 individual term files with detailed definitions (English) |
| [glosario/README.md](user/glosario/README.md) | Categorized index with quick descriptions |

---

## Articles & Blog

Published blog posts (in `docs/user/blog/`):

| Article | Description |
|---------|-------------|
| [Why I Built a Local Memory Engine for AI Agents in Rust](user/blog/why_i_built.md) | Motivation and design philosophy |
| [How Hybrid Search Works: BM25 + HNSW + RRF](user/blog/how_hybrid_search_works.md) | Technical deep-dive on hybrid search |
| [SQLite for AI Agents: Benchmarks and Architecture](user/blog/sqlite_for_ai_agents.md) | Comparing embedded databases for agent memory |
| [Introducing VantaDB](user/blog/introducing_vantadb.md) | Product announcement |
| [Benchmarks vs LanceDB & Chroma](user/blog/benchmarks_vs_lancedb_chroma.md) | Competitive benchmark write-up |
| [GraphRAG Benchmark](user/blog/graphrag-benchmark.md) | GraphRAG performance evaluation post |
| [Campaign: AI Agent Memory](user/blog/campaign-ai-agent-memory.md) | Campaign narrative on agent memory |

---

## GraphRAG

| Document | Description |
|----------|-------------|
| [GraphRAG README](dev/graphrag/README.md) | Graph-based RAG integration research |
| [GraphRAG API](api/GRAPH_RAG.md) | Public GraphRAG API reference |

---

## Audit Reports & Reviews

Audit and review reports generated by `/audit`, `/review`, and `unified-review` live in `docs/dev/reviews/`.

| Document | Description |
|----------|-------------|
| [auditoria-documentacion-2026-08-21.md](dev/reviews/archive/auditoria-documentacion-2026-08-21.md) | Documentation audit that motivated the GOV campaign |
| [audit-full-20260812-231204.md](dev/reviews/archive/audit-full-20260812-231204.md) | Full audit run 2026-08-12 |
| [review-certify-2026-08-05-2025.md](dev/reviews/archive/review-certify-2026-08-05-2025.md) | Certification review 2026-08-05 |
| [stabilization-report.md](dev/reviews/archive/stabilization-report.md) | Stabilization phase report |
| Historical runs (including `audit-full-2026-07-18`) are archived under [`reviews/archive/`](dev/reviews/archive/) |

---

## Pipeline Reports

Structured reports produced by pipelines and evals live in `docs/dev/reports/`.

| Document | Description |
|----------|-------------|
| [INDEX.md](dev/reports/INDEX.md) | Index of all pipeline reports |
| [dora.md](dev/reports/dora.md) | DORA metrics report |
| [northstar.md](dev/reports/northstar.md) | Northstar tracking report |
| [pipeline-evals.md](dev/reports/pipeline-evals.md) | Pipeline evaluation report |

---

## Plans

Active plans in `docs/dev/plans/`; completed plans move to `plans/archive/`.

| Document | Description |
|----------|-------------|
| [PROMPT-MAESTRO-FREEZE.md](dev/plans/archive/PROMPT-MAESTRO-FREEZE.md) | Prompt maestro freeze plan (archived) |
| [ACTION_PLAN → ROADMAP v2.0](dev/strategy/ROADMAP.md) | Archived — superseded by ROADMAP.md v2.0 |

---

## Progress & Planning

Spanish-language planning material (allowed exception to the English docs rule).

| Document | Description |
|----------|-------------|
| [avance/README.md](dev/avance/README.md) | Unified progress log and development history |
| [Backlog.md](dev/Backlog.md) | Full project backlog and feature tracking |
| [backlog-futuro.md](dev/backlog-futuro.md) | Deferred / future backlog items |
| [CHANGELOG.md](CHANGELOG.md) | Release history and version changelog |
| [avance/](dev/avance/README.md) | Progress tracking workspace (activo, auditoría, decisiones, historial) |

---

## Research & Investigations

| Document | Description |
|----------|-------------|
| [Investigaciones → research/](dev/research/) | Spanish-language research notes: FND-*, INV-*, TIR-*, competitive analyses (see folder README per series) |
| [research/human-facing-db-ui/](dev/research/human-facing-db-ui/) | Research on human-facing DB UI concepts |
| [research/tdam/](dev/research/tdam/) | TDAM (Tiered Document Attention Model) research |
| [wasm/CRASH_MODEL.md](dev/wasm/CRASH_MODEL.md) | WASM crash model research |

---

## CI Workflows

Per-workflow documentation mirrors `.github/workflows/`.

| Document | Description |
|----------|-------------|
| [ci-gate.md](dev/workflow/ci-gate.md) | Fast Gate workflow |
| [ci-rust-10.md](dev/workflow/ci-rust-10.md) | Rust CI workflow |
| [ci-web-11.md](dev/workflow/ci-web-11.md) | Web CI workflow |
| [gate-docs-21.md](dev/workflow/gate-docs-21.md) | Docs gate workflow |
| [fuzz-40.md](dev/workflow/fuzz-40.md) | Fuzzing workflow |
| [chaos-45.md](dev/workflow/chaos-45.md) | Chaos testing workflow |
| [perf-bench-40.md](dev/workflow/perf-bench-40.md) | Performance benchmark workflow |
| [heavy-bench-nightly-51.md](dev/workflow/heavy-bench-nightly-51.md) | Nightly heavy benchmark workflow |
| [heavy-certification-50.md](dev/workflow/heavy-certification-50.md) | Heavy certification workflow |
| [sec-codeql-30.md](dev/workflow/sec-codeql-30.md) | CodeQL security workflow |
| [release-wheels-60.md](dev/workflow/release-wheels-60.md) | Wheels release workflow |
| [release-npm-61.md](dev/workflow/release-npm-61.md) | NPM release workflow |
| [release-adapters-62.md](dev/workflow/release-adapters-62.md) | Adapters release workflow |
| [release-binaries-63.md](dev/workflow/release-binaries-63.md) | Binaries release workflow |
| [release-sbom-64.md](dev/workflow/release-sbom-64.md) | SBOM release workflow |

---

## Web Frontend Docs

Documentation for the Next.js web frontend lives in `docs/user/web/`.

| Document | Description |
|----------|-------------|
| [web/README.md](user/web/README.md) | Web docs landing page |
| [design-rules-es-tutorial.md](dev/archive/design-rules-es-tutorial.md) | Tutorial ES de diseño (ARCHIVADO — ver banner) |
| [QA.md](user/web/QA.md) | Web QA checklist |
| Sub-folders: [`audit/`](user/web/audit/), [`guides/`](user/web/guides/), [`reference/`](user/web/reference/), [`standards/`](user/web/standards/) |

---

## Benchmarks

Raw benchmark artifacts and analyses live in `docs/user/benchmarks/`. Canonical claims belong in [`operations/BENCHMARKS.md`](user/operations/BENCHMARKS.md).

| Document | Description |
|----------|-------------|
| [COMPETITIVE_ANALYSIS.md](user/benchmarks/COMPETITIVE_ANALYSIS.md) | Competitive analysis narrative |
| [COMPETITIVE_SDK_BENCH.md](user/benchmarks/COMPETITIVE_SDK_BENCH.md) | SDK benchmark vs competitors |
| [ivf_bench.md](user/benchmarks/ivf_bench.md) | IVF index benchmark notes |

---

## Book

| Document | Description |
|----------|-------------|
| [book/](user/book/book.toml) | mdBook project (`mdbook build` from `docs/user/book/`) — narrative book on VantaDB internals |

---

## Community & Examples

| Document | Description |
|----------|-------------|
| [discord/README.md](user/discord/README.md) | Discord community workspace (server config, bilingual strategy) |
| [examples/](user/examples/) | Runnable code samples referenced by tutorials |

Assets (images used by docs): [`assets/`](../assets/)

Agent-facing reference files (bug workflow, troubleshooting, nextest output): [`references/`](dev/references/).

Historical material moved out of the main tree: [`archive/`](dev/archive/) (incl. archived case studies and legacy docs inventories).

---

## Other Documents

| Document | Description |
|----------|-------------|
| [QUICKSTART.md](user/QUICKSTART.md) | Quickstart guide for new users |
| [FAQ.md](user/FAQ.md) | Frequently asked questions |
| [COMPARISON.md](COMPARISON.md) | Honest comparison vs sqlite-vec / LanceDB / Qdrant / Chroma — qualitative table, benchmark provenance per Regla 11, practical limits |
| [ci-cd-guide.md](ci-cd-guide.md) | CI/CD setup and operations guide |
| [README.md](README.md) | Documentation landing page and reading guide |
| [../desktop/DESIGN_DECISIONS.md](../desktop/DESIGN_DECISIONS.md) | Studio design decisions: token decoupling, light-only web, WCAG palette |

---

## See Also

- [Operations Master Index](user/operations/master-index.md) — Detailed operations document listing
- [Pipeline Reports Index](dev/reports/INDEX.md) — Detailed pipeline report listing
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
