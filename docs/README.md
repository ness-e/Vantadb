# VantaDB Documentation

Welcome to the VantaDB documentation index. Organized by audience so you can find what's relevant faster.

---

## 👤 End Users

Getting started, migration guides, case studies, and technical articles.

### Getting Started

* [QUICKSTART.md](QUICKSTART.md) — 5-minute setup guide for Rust, CLI client, and Python SDK.

### Migration Guides

* [From ChromaDB](migration/FROM_CHROMADB.md) — Step-by-step migration from ChromaDB to VantaDB.
* [From LanceDB](migration/FROM_LANCEDB.md) — Step-by-step migration from LanceDB to VantaDB.

### GraphRAG

* [GraphRAG on VantaDB](graphrag/README.md) — Unified vector-graph database for GraphRAG: semantic clusters, confidence scoring, graph traversal, semantic compression engine, and multi-agent support.

### Case Studies

* [Local Memory with Ollama](case_studies/agent_local_memory_ollama.md) — Embedding integration with local LLMs.
* [RAG on Edge Devices](case_studies/rag_edge_device.md) — Edge deployments on resource-restricted systems.

### Technical Articles

* [How Hybrid Search Works](articles/how_hybrid_search_works.md) — Deep dive into combining Tantivy lexical search with HNSW vector search.
* [SQLite for AI Agents](articles/sqlite_for_ai_agents.md) — Why a specialized vector-relational engine is needed instead of traditional SQLite.
* [Why I Built a Local Memory Engine](articles/why_i_built_local_memory_engine.md) — Engineering rationale behind creating a zero-dependency local vector database.

---

## 🔧 Developers

API & SDK reference, architecture, experimental specs, and development guides.

### API & SDK Reference

* [Python SDK Guide](api/PYTHON_SDK.md) — Integration, GIL release policies, Rayon batching, and PyO3 FFI boundary specs.
* [Model Context Protocol (MCP)](api/MCP.md) — Guide to using the MCP server for connecting local AI agents directly.

### Architecture

* [Architecture Overview](architecture/ARCHITECTURE.md) — Core architecture principles (single-crate design, layout alignment, zero-copy memory mapping).
* [Text Index Design](architecture/TEXT_INDEX_DESIGN.md) — System design of the inverted text index for BM25 lexical search.
* [WAL & Mutation Recovery](architecture/MUTATION_RECOVERY_PROTOCOL.md) — WAL specifications, CRC32C validation, and the Scan-Forward Auto-healing protocol.
* [Advanced Tokenizer](architecture/ADVANCED_TOKENIZER.md) — Multilingual text processing, stemming, and Unicode folding based on Tantivy.
* [Architecture Decision Records (ADRs)](architecture/adr/) — Formal record of architectural decisions:
  * [ADR 001: Unified Config and Read-Only Mode](architecture/adr/001_unified_config_readonly.md)
  * [ADR 002: WAL CRC32C and Scan-Forward Auto-healing](architecture/adr/002_wal_crc32c_autohealing.md)
  * [ADR 003: Sync-Async Core Decoupling](architecture/adr/003_sync_async_decoupling.md)
* [Historical Audits](architecture/audits/) — Technical review logs from early development phases:
  * [2026-05-04 Cleanup Candidates](architecture/audits/2026-05-04-cleanup-candidates.md)
  * [2026-05-04 Test Report](architecture/audits/2026-05-04-test-report.md)
  * [2026-05-04 Total Review](architecture/audits/2026-05-04-total-review.md)
  * [2026-05-19 Phase 5 Certification Report](architecture/audits/2026-05-19-fase-5-certification-report.md)
  * [2026-05-19 Performance Action Plan](architecture/audits/2026-05-19-plan-accion-alto-rendimiento.md)

### Experimental & Plans

* [Experimental IQL](experimental/IQL.md) — Specifications of the experimental LISP-like query parser currently in quarantine.
* [Memory Management Plan](plans/2026-06-18-memory-management.md) — Design proposal for memory management improvements.

### Integrations

* **LangChain** — `integrations/langchain/` — `VantaDBVectorStore(VectorStore)` adapter with hybrid search, metadata filtering, and batch operations.
* **LlamaIndex** — `integrations/llamaindex/` — `VantaDBVectorStore(BasePydanticVectorStore)` adapter with graph traversal, hybrid search, and rich filtering.

### Development Tooling

* [CI Policy](operations/CI_POLICY.md) — Rules for PR testing gates, coverage constraints, and lint enforcement.
* [Fuzzing Guide](operations/FUZZING.md) — Harnesses and guidelines for running fuzz tests using cargo fuzz.
* [Repository Checklist](operations/REPO_CHECKLIST.md) — Code hygiene, documentation and packaging check sheets.
* [Editor Setup Guidelines](operations/EDITOR_INTEGRATIONS.md) — VS Code and NeoVim configuration.
* [Agent Guidelines](operations/AGENT_INSTRUCTIONS.md) — Directives and governance rules for AI coding agents.
* [Public Issues & Good First Issues](operations/PUBLIC_ISSUE_DRAFTS.md) — Automated templates for first-time open-source contributors.

---

## ⚙️ Operators

Configuration, deployment, monitoring, reliability, and community operations.

### Configuration & Features

* [Configuration Schema](operations/CONFIGURATION.md) — Detail of parameters in `VantaConfig`, cache limits, and query boundaries.
* [Product & Feature Boundary](operations/EXPERIMENTAL_FEATURES.md) — Categorized matrix of stable (MVP), optional wrappers, and experimental subsystems.
* [Python Release Policy](operations/PYTHON_RELEASE_POLICY.md) — Automated wheels compilation via cibuildwheel and signing with GitHub Attestations (SLSA Level 2).

### Performance & Monitoring

* [Benchmarks & Performance](operations/BENCHMARKS.md) — Performance metrics, memory footprints, and competitive analysis vs. LanceDB and ChromaDB.
* [Memory Telemetry](operations/MEMORY_TELEMETRY.md) — Metrics for tracking physical RAM RSS and mapped page tables.
* [Grafana Setup](operations/GRAFANA_SETUP.md) — Monitoring dashboard setup instructions.
* [Grafana Dashboard JSON](operations/grafana-dashboard.json) — Pre-built dashboard configuration.

### Reliability & Durability

* [Backup & Durability Policy](operations/BACKUP_POLICY.md) — Hot checkpoints, filesystem backups, and index snapshot strategies.
* [Durability Guarantees](operations/DURABILITY_GUARANTEES.md) — Formal durability contract and guarantees.
* [Reliability & Chaos Testing](operations/RELIABILITY_GATE.md) — Test suites for chaos loop, failpoints injection, and hardware profiles.
* [Historical Repository Snapshots](operations/snapshots/) — Full backups of codebase status at key points:
  * [Snapshot 2026-06-09](operations/snapshots/snapshot_2026-06-09.md)
  * [Snapshot 2026-06-12](operations/snapshots/snapshot_2026-06-12.md)

### Audits & Milestones

* [Executive Technical Audit](operations/EXECUTIVE_TECHNICAL_AUDIT.md) — Executive summary of architectural review, findings, and remediation.
* [Text Index Phase 1 Closeout](operations/TEXT_INDEX_PHASE_1_CLOSEOUT.md) — Phase 1 completion report for text index implementation.
* [Milestone v0.2.0](operations/MILESTONE_V0.2.0.md) — Planning and progress for the v0.2.0 milestone.
* [Release v0.1.1](operations/RELEASE_V0.1.1.md) — Release notes and checklist for v0.1.1.

### Community & Publishing

* [Community Governance](operations/COMMUNITY_GOVERNANCE.md) — Contribution process, RFC timelines, and Code of Conduct.
* [Pilot Program Plan](operations/PILOT_PROGRAM.md) — Framework and guidelines for early developer pilots.
* [HackerNews Launch Preparation](operations/SHOW_HN_PREP.md) — Post copywriting drafts and Q&A for technical critiques.

---

## 📖 Reference

* [Changelog](CHANGELOG.md) — Summary of releases, versions, and changes history.
* [Unified Progress History](progreso/README.md) — Consolidated summary of all development phases and milestones completed.

---

*All file paths are relative to the `docs/` directory. For the repository root README, see [`README.md`](../README.md).*
