# VantaDB Documentation Directory

Welcome to the VantaDB documentation registry. This index maps all available guides, policies, design records, and technical specifications, classified by target audience and domain.

---

## 🚀 Getting Started & Developer Experience (DX)

* [QUICKSTART.md](QUICKSTART.md) — 5-minute setup guide for Rust, CLI client, and Python SDK.
* [BENCHMARKS.md](BENCHMARKS.md) — Performance metrics, memory footprints, and competitive analysis vs. LanceDB and ChromaDB.
* [EDITOR_INTEGRATIONS.md](EDITOR_INTEGRATIONS.md) — Editor setup guidelines for VS Code and NeoVim.
* [MCP.md](MCP.md) — Guide to using the Model Context Protocol (MCP) server for connecting local AI agents directly.
* [Changelog](CHANGELOG.md) — Summary of releases and stable versions history.

---

## 🏛️ Architecture & System Design

* [Architecture Overview](architecture/ARCHITECTURE.md) — Core architecture principles (Single-crate design, layout alignment, and zero-copy memory mapping).
* [Text Index Design](architecture/TEXT_INDEX_DESIGN.md) — System design of the inverted text index for BM25 lexical search.
* [WAL & Mutation Recovery](architecture/MUTATION_RECOVERY_PROTOCOL.md) — WAL specifications, CRC32C validation, and the Scan-Forward Auto-healing protocol.
* [Advanced Tokenizer](ADVANCED_TOKENIZER.md) — Multilingual text processing, stemming, and Unicode folding based on Tantivy.
* [Architecture Decision Records (ADRs)](adr/) — Formal record of architectural decisions:
  * [ADR 001: Unified Config and Read-Only Mode](adr/001_unified_config_readonly.md)
  * [ADR 002: WAL CRC32C and Scan-Forward Auto-healing](adr/002_wal_crc32c_autohealing.md)
  * [ADR 003: Sync-Async Core Decoupling](adr/003_sync_async_decoupling.md)

---

## 💻 API & SDK Reference

* [IQL Grammar](api/IQL.md) — Reference syntax and grammar rules for the *Index Query Language* (IQL).
* [Python SDK Guide](operations/PYTHON_SDK.md) — Integration, GIL release policies, Rayon batching, and PyO3 FFI boundary specs.
* [Experimental IQL](experimental/IQL.md) — Specifications of the experimental LISP-like query parser currently in quarantine.
* [Agent Cognitive Memory](ai/agent.md) — High-level concepts of integrating relational-semantic databases with AI frameworks.

---

## ⚙️ Operations, Policies & Telemetry

* [Configuration Schema](operations/CONFIGURATION.md) — Detail of parameters in `VantaConfig`, cache limits, and query boundaries.
* [Memory Telemetry](operations/MEMORY_TELEMETRY.md) — Metrics for tracking physical RAM RSS and mapped page tables.
* [Backup & Durability Policy](operations/BACKUP_POLICY.md) — Hot checkpoints, filesystem backups, and index snapshot strategies.
* [Continuous Integration (CI)](operations/CI_POLICY.md) — Rules for PR testing gates, coverage constraints, and lint enforcement.
* [Python Release Policy](operations/PYTHON_RELEASE_POLICY.md) — Automated wheels compilation via cibuildwheel and signing with GitHub Attestations (SLSA Level 2).
* [Product & Feature Boundary](operations/EXPERIMENTAL_FEATURES.md) — Categorized matrix of stable (MVP), optional wrappers, and experimental subsystems.
* [Reliability & Chaos Testing](operations/RELIABILITY_GATE.md) — Test suites for chaos loop, failpoints injection, and hardware profiles.
* [Fuzzing Guide](operations/FUZZING.md) — Harnesses and guidelines for running fuzz tests using cargo fuzz.

---

## 👥 Community, Launch & Outreach

* [Case Studies](case_studies/) — Real-world deployment scenarios:
  * [Local Memory with Ollama](case_studies/agent_local_memory_ollama.md) — Embedding integration with local LLMs.
  * [RAG on Edge Devices](case_studies/rag_edge_device.md) — Edge deployments on resource-restricted systems.
* [Pilot Onboarding Pack](operations/PILOT_ONBOARDING.md) — DX materials and forms for private beta testers.
* [Pilot Outreach Plan](operations/PILOT_OUTREACH.md) — Channels and copy for enrolling pilot developers.
* [Community Governance](operations/COMMUNITY_GOVERNANCE.md) — Contribution process, RFC timelines, and SLA response policies.
* [HackerNews Launch Preparation](operations/SHOW_HN_PREP.md) — Post copywriting drafts and Q&A to answer technical critiques.
* [Public Issues & Good First Issues](operations/PUBLIC_ISSUE_DRAFTS.md) — Automated templates for first-time open-source contributors.

---

## ✍️ Technical Articles & Publications

* [How Hybrid Search Works](articles/how_hybrid_search_works.md) — Deep dive into combining Tantivy lexical search with HNSW vector search.
* [SQLite for AI Agents](articles/sqlite_for_ai_agents.md) — Why a specialized vector-relational engine is needed instead of traditional SQLite.
* [Why I Built a Local Memory Engine](articles/why_i_built_local_memory_engine.md) — Engineering rationale behind creating a zero-dependency local vector database.

---

## 📊 Reports, Milestones & Snapshots

* [Project Status Audit](operations/PROJECT_STATUS_AUDIT.md) — Static review report of code status, crates, and workspace structure.
* [Text Index Phase 1 Closeout](operations/TEXT_INDEX_PHASE_1_CLOSEOUT.md) — Closing metrics for Tantivy integration and lexical capabilities.
* [Memory MVP Baseline](operations/MEMORY_MVP_BASELINE.md) — Raw latency benchmarks for the initial memory model.
* [Repository Checklist](operations/REPO_CHECKLIST.md) — Code hygiene, documentation and packaging check sheets.
* [Release Notes v0.1.1](operations/RELEASE_V0.1.1.md) — Changelog and assets compilation for stable release v0.1.1.
* [Milestone v0.2.0](operations/MILESTONE_V0.2.0.md) — Action plans, deadlines, and criteria of completion for the next milestone.
* [Executive Technical Report](reports/executive-technical-report.md) ([HTML version](reports/executive-technical-report.html)) — Executive summary of architectural review, findings, and remediation.
* [Lista Maestra de Tareas Consolidada](reports/LISTA_MAESTRA_TAREAS_CONSOLIDADA.md) — Master implementation roadmaps and checklists.
* [Historical Repository Snapshots](snapshots/) — Full backups of codebase status at key points:
  * [Snapshot 2026-06-09](snapshots/snapshot_2026-06-09.md)
  * [Snapshot 2026-06-12](snapshots/snapshot_2026-06-12.md)
* [Historical Progress Snapshots](progreso/) — Traceability directory of snapshots for every completed feature branch (e.g. `soporte-datetime-listas-y-dag`, `correccion-inconsistencias-docs`).
* [Historical Audits](audits/) — Technical review logs from early development phases:
  * [2026-05-04 Cleanup Candidates](audits/2026-05-04-cleanup-candidates.md)
  * [2026-05-04 Test Report](audits/2026-05-04-test-report.md)
  * [2026-05-04 Total Review](audits/2026-05-04-total-review.md)
  * [2026-05-19 Phase 5 Certification Report](audits/2026-05-19-fase-5-certification-report.md)
  * [2026-05-19 Performance Action Plan](audits/2026-05-19-plan-accion-alto-rendimiento.md)
