---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
links: "[[master-index]]"
last_reviewed: 2026-07-03
aliases: []
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks, active and postponed.
> **Completed features:** `docs/CHANGELOG.md`

---

## PHASE 3 — Pre-Launch (July-August 2026)

### 3.C Core Engine

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `TSK-09` | OpenTelemetry traces (premature without basic Prometheus) | 🟢 | ✅ |
| `TSK-145` | Normalizar comentarios español/inglés en storage.rs, wal.rs a inglés | 🟢 | ✅ |
| `TSK-146` | Eliminar magic numbers esparcidos (1024 capacity, 64 alignment, 0x8 tombstone, 0.80 RSS) | 🟢 | ✅ |


## PHASE 4 — Launch (Jul-Sep 2026)

### 4.0 Foundational (blocking — do first)

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `REL-01` | Bump workspace version v0.1.5 → v0.2.0 (SemVer: 340+ commits, nuevas APIs, 4 plataformas) | 🔴 | ✅ |
| `LEG-01` | Register trademark "VantaDB" (USPTO + EUIPO) before Show HN | 🔴 | ❌ |
| `LEG-02` | Add Contributor License Agreement (CLA) for future core contributions — Individual + Corporate CLA creados en .github/ | 🟠 | ✅ |

### 4.B Framework Integrations

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `INT-01` | LangChain adapter (PyPI + PR langchain-community) — code exists in `integrations/langchain/`, pending PyPI publish + upstream PR | 🔴 | ❌ |
| `INT-02` | LlamaIndex adapter (PyPI + PR llama-index) — code exists in `integrations/llamaindex/`, pending PyPI publish + upstream PR | 🔴 | ❌ |
| `MEM-02` | Letta (fka MemGPT): VantaDB as memory backend (23K stars) | 🟡 | ✅ |
| `TSK-89` | CrewAI: VantaDBMemory for multi-agent crews | 🟡 | ✅ |
| `TSK-91` | DSPy: VantaDBRM (Retrieval Module) | 🟡 | ✅ |
| `TSK-92` | Haystack: VantaDBDocumentStore | 🟡 | ✅ |
| `TSK-116` | vantadb-openai (optional embedding package) | 🟡 | ✅ |
| `TSK-117` | vantadb-ollama (local offline embedding) | 🟡 | ✅ |
| `TSK-95` | vantadb-litellm (universal gateway embeddings) | 🟡 | ✅ |

### 4.C MCP & WASM Differentiation (Unique Competitive Advantage)

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `MCP-03` | Publish benchmarks and feature comparison vs WASM vector DBs (EdgeVec, minimemory, altor-vec, lattice-db). Establish "most feature-complete WASM vector DB" narrative | 🔴 | ❌ |

| `WASM-03` | Build demo: AI Agent running entirely in browser (Transformers.js + VantaDB WASM + persistent OPFS memory). No competitor enables this | 🟡 | ✅ |
| `WASM-04` | WASM bundle size optimization (target: <500KB gzip). Currently unmeasured | 🟡 | ✅ |
| `WASM-05` | SIMD acceleration for WASM build (expose f32x8 cosine distance in browser) | 🟡 | ✅ |
| `MCP-04` | MCP server: add tool for collection management (list, delete, stats) and streaming search results | 🟡 | ✅ |
| `MCP-05` | MCP server: write integration test suite (currently 9 tests, target 25+) | 🟡 | ✅ |

### 4.D Launch Campaign

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `MKT-01` | Landing page (vantadb.dev): hero, benchmarks, comparisons | 🔴 | ✅ |
| `MKT-02` | Blog post "Introducing VantaDB" + 3 more published | 🔴 | ✅ |
| `MKT-06` | Logo and branding (SVG, palette, favicon) | 🔴 | ✅ |
| `MKT-03` | Show HN post (timing, title, prepared responses) | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟠 | ❌ |
| `MKT-05` | Technical blog posts (5+ pre-launch) | 🟠 | ❌ |
| `MKT-10` | "AI Agent Memory" narrative campaign: blog posts, token reduction demos, benchmarks vs full-context | 🟠 | ❌ |
| `COM-01` | Discord: announcements, general, help, showcase, dev | 🔴 | ❌ |
| `TSK-106` | GitHub Discussions (Q&A, Ideas, Show & Tell) | 🟡 | ❌ |
| `TSK-107` | Community showcase (projects in docs/showcase.md) | 🟡 | ❌ |
| `TSK-108` | Newsletter (Substack/Beehiiv, monthly) | 🟢 | ❌ |
| — | Good first issues (20+ tagged issues) | 🟠 | ❌ |

### 4.E Backend Performance (N+1 Patterns & Bottlenecks)

| ID | Task | Priority | Status |
|----|-------|-----------|--------|

| `PERF-02` | Refactor WAL Mutex contention (`Mutex<Option<WalWriter>>` serializes all writes). Evaluate `async-lock` or sharded WAL segments | 🟡 | ✅ |
| `PERF-03` | Make spawn_blocking semaphore cap configurable and dynamic (default 16 is hard limit) — configurable via `VANTADB_MAX_BLOCKING_THREADS` ✅, dynamic auto-scale ✅ (detecta CPU cores × 2) | 🟠 | ✅ |
| `PERF-04` | Refactor `Execution(String)` catch-all → typed error variants (TODO in source) | 🟡 | ✅ |
| `PERF-05` | Split monolithic files: `storage.rs` (2624L), `index.rs` (2044L), `metrics.rs` (1300L), `cli_server.rs` (687L) into modules | 🟡 | ✅ |
| `PERF-06` | Eliminate duplicated `append_to_vstore` / `write_node_to_vstore` (40L near-identical, storage.rs:1170-1257) | 🟢 | ✅ |
| `PERF-07` | Global edge index + referential integrity (ON DELETE CASCADE for dangling edges) | 🟡 | ✅ |
| `PERF-08` | Secondary scalar indexes for `filter_field()` — currently does full table scan | 🟡 | ✅ |
| `PERF-09` | Dynamic quantization governor: auto-transition f32→SQ8 for cold nodes based on hit frequency | 🟢 | ✅ |
| `PERF-10` | Memory governor with eviction metrics visible via `/metrics` | 🟠 | ✅ |
| `PERF-11` | Batch KV loader (`get_many`/`multi_get`) para eliminar N+1 en graph traversal, physical scan, vector search, hybrid explain | 🔴 | ✅ |
| `PERF-12` | Refactor patrón WAL repetitivo en engine.rs (`if let Some(ref mut wal) = ... wal.append(...)`) a helper method | 🟡 | ✅ |
| `PERF-13` | Refactor `read_only` check repetido 5× en sdk/api.rs a helper method | 🟢 | ✅ |
| `PERF-14` | Refactor `init_telemetry` masivo (cli_server.rs:280-438): eliminar bloques if/else repetidos JSON/full/compact × mcp/no-mcp | 🟡 | ✅ |
| `PERF-15` | Agregar `#![warn(missing_docs)]` en todos los crates del workspace (14 crates) | 🟢 | ✅ |

### 4.F Distribution

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `TSK-121` | SHA256 hash verification of the wheel in tests | 🟢 | ❌ |
| `REL-02` | Publish `vantadb-ts` npm package (WASM, 26/26 tests, examples listos) | 🔴 | ❌ |
| `DEVOPS-05` | Publish LangChain + LlamaIndex adapters to PyPI and submit PRs upstream (langchain-community, llama-index) | 🔴 | ❌ |
| `DEVOPS-02` | Build ARM64 wheels for Python SDK (Apple Silicon, AWS Graviton, Raspberry Pi) | 🟠 | ❌ |
| `DEVOPS-06` | Homebrew formula for vanta-cli (macOS/Linux) | 🟢 | ❌ |

### 4.G Developer Experience

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB (showcase) | 🟠 | ❌ |
| `TSK-103` | Public benchmark site (compare.py vs chroma/lancedb/qdrant) | 🟠 | ❌ |
| `DX-01` | Refactor API: `VantaDB()` → `connect()` (eliminar redundancia, alinear con SQLite3/LanceDB/DuckDB) | 🟠 | ✅ |
| `DX-02` | Python SDK latency optimization: reduce p50 from ~62ms to <20ms (PyO3 FFI overhead) | 🟠 | ✅ |
| `DX-04` | TypeScript SDK: improve from 18 tests to 50+ covering edge cases, error handling, concurrent access | 🟡 | ✅ |


### 4.H Code Health & Security

| ID | Task | Priority | Status |
|----|-------|-----------|--------|

| `SEC-04` | Auth hardening: constant-time comparison (`subtle::ConstantEq`), rate limiting on auth failures, make `/metrics` auth-required | 🟠 | ✅ |
| `SEC-05` | RBAC design: scoped API tokens (read-only, namespace-scoped, time-limited) for multi-user server deployments | 🟡 | ✅ |
| `SEC-06` | SBOM (SPDX/CycloneDX) generation in each release | 🟡 | ✅ |
| `SEC-07` | CodeQL + cargo-deny in CI for vulnerability scanning on every PR | 🟡 | ✅ |
| `SEC-08` | Migrar `rustls-pemfile` → `rustls-pki-types` (RUSTSEC-2025-0134, vulnerabilidad activa) | 🔴 | ✅ |
| `SEC-09` | Reemplazar `bincode` 2.0 por `postcard` (RUSTSEC-2025-0141, unmaintained) | 🟡 | ✅ |
| `SEC-10` | Security test suite: IQL injection, auth bypass, input validation fuzzing, timing attack | 🔴 | ✅ |
| `DOC-01` | Unit tests: 91 nuevos tests en config/engine/executor/gc/metrics/backends | 🟡 | ✅ |
| `DOC-02` | Refactor `insert_hnsw()` in `src/index.rs` (177L → 3 functions: `compute_inv_cached_norm`, `shrink_neighbors`, `insert_hnsw`) | 🟡 | ✅ |
| `DOC-03` | Normalize 6 files with Unicode/accent in filename to pure ASCII (avoids cross-platform issues) | 🟢 | ✅ |

### 4.I Documentation Consolidation

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `DOC-04` | Restore unique content from archived VantaDB-MPTS files: Vision/UVP, GTM/Strategic Roadmap, design principles, flowcharts, risk register (~2,900 unique lines with no EN equivalent). Create `docs/vision/VISION.md`, `docs/strategy/ROADMAP.md`, `docs/strategy/GO_TO_MARKET.md` | 🔴 | ✅ |
| `DOC-05` | Translate to English 10 docs from `operations/` + 3 ADRs that are in Spanish (violate EN convention) | 🟡 | ✅ |
| `DOC-06` | Unified frontmatter schema (title, status, tags, last_reviewed, aliases) for 124 .md files | 🟡 | ✅ |
| `DOC-07` | Unify naming convention to kebab-case without accents or spaces | 🟢 | ✅ |
| `DOC-08` | Archive `TEXT_INDEX_PHASE_1_CLOSEOUT.md`, `RELEASE_V0.1.1.md`, `MILESTONE_V0.2.0.md` (historical) | 🟢 | ✅ |
| `DOC-09` | Create `.github/` directory with SECURITY.md, SUPPORT.md, CODE_OF_CONDUCT.md, issue/PR templates | 🔴 | ✅ |
| `DOC-10` | Fix broken links in README.md and README_ES.md | 🔴 | ✅ |
| `DOC-11` | Fix factual errors in blog: License MIT→Apache 2.0, GitHub URL `vantadb/vantadb`→`ness-e/Vantadb` in `web/content/blog/introducing-vantadb.md` | 🟡 | ✅ |
| `DOC-12` | Update `web/public/llms.txt` with current version (currently says v0.4.0→v0.6.0, project is v0.2.0) | 🟡 | ✅ |
| `DOC-13` | Create missing ADRs (Architecture Decision Records): Fjall vs RocksDB criteria, HNSW params (M=32, ef_construction=200), RRF k=60, PyO3 architecture, WASM strategy, community governance. 6 ADRs creados (004-009) | 🟡 | ✅ |
| `DOC-14` | Write official Performance Tuning Guide: HNSW params, memory limits, backend selection, sync modes, quantization tradeoffs — 479 líneas | 🟡 | ✅ |
| `DOC-15` | Create OpenAPI/Swagger spec for HTTP API (currently 3 endpoints documented in 149 lines — EMBEDDED_SDK has 428L) | 🟡 | ✅ |
| `DOC-16` | Create tutorial series in `docs/tutorials/`: AI Agent Memory with VantaDB, Local RAG Pipeline walkthrough, Migrating from ChromaDB step-by-step — 3 tutorials creados | 🟡 | ✅ |
| `DOC-17` | Crear diagramas de arquitectura formales (reemplazar ASCII art en ARCHITECTURE.md) — 5 diagramas Mermaid | 🟡 | ✅ |
| `DOC-18` | Expandir HTTP_API.md (149L → ~504L) al nivel de detalle de EMBEDDED_SDK (428L) | 🟡 | ✅ |
| `DOC-19` | Agregar términos faltantes al glosario: `similar_to_key`, `put_batch`, `compaction`, `serialization`, `heuristic_search` — 5 términos creados | 🟢 | ✅ |

### 4.J Web Frontend

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `WEB-06` | Migrate 637 inline styles to CSS Modules (engine.tsx 1085L inline, architecture.tsx 557L inline) — ~125 migraciones a Tailwind | 🟡 | ✅ |

| `WEB-07` | Eliminar redundancia de animation libraries: mantener solo GSAP, reemplazar Motion route transitions y AnimeJS text-scramble con GSAP — Motion eliminado, AnimeJS no estaba en uso | 🟡 | ✅ |
| `WEB-08` | Anti-Slop Audit, Performance Budget, SEO Final Review | 🟢 | ✅ |
| `WEB-14` | Implement missing GSAP animations per DiseñoNuevo.md: scroll-trigger reveals, count-up numbers | 🟡 | ✅ |
| `WEB-17` | Evaluate TanStack Router necessity: 23 mostly-static pages. React Router would be simpler with fewer deps and no `routeTree.gen.ts` — evaluación completada, recomendación: mantener por ahora | 🟡 | ✅ |
| `WEB-18` | Crear componente `<VsTable data={...} />` reusable para patrón "Legacy vs VantaDB" (duplicado en 7+ archivos) — 10 tests, todos pasando | 🟡 | ✅ |
| `WEB-19` | Implementar `React.lazy()` / code splitting automático por ruta para reducir bundle inicial — lazy loading implementado con TanStack Router nativo | 🟡 | ✅ |
| `WEB-20` | Agregar `React.memo`, `useMemo`, `useCallback` en componentes clave (Nav, SwissFooter, SwissSubpageHero) — Nav memoizado, SwissFooter y SubpageHero ya lo estaban | 🟡 | ✅ |
| `WEB-21` | Eliminar mutación directa del DOM en handlers onMouseEnter/onMouseLeave (about/index.tsx, about/community.tsx, SwissArchSection.tsx) — 25 patrones fijados en 11 archivos | 🟡 | ✅ |

### 4.K Testing Gaps

| ID | Task | Priority | Status |
|----|-------|-----------|--------|

| `TEST-04` | Regression test suite: dedicated tests for each fixed bug to prevent regressions (12 tests in `tests/core/regression_certification.rs`) | 🟡 | ✅ |
| `TEST-05` | Snapshot testing: HNSW recall certification snapshots, export/import format versioning, WAL format integrity (7 tests) | 🟡 | ✅ |
| `TEST-06` | Load/stress tests for Python (9) and TypeScript (6) SDKs | 🟡 | ✅ |
| `TEST-07` | Fix `test-threads = 2` global: make OS-specific config (Windows needs 2, Linux/macOS can use more parallelism) — global `test-threads=2` removed ✅, Windows-specific override ✅ | 🟢 | ✅ |
| `TEST-08` | Fix `chaos_integrity` missing `required-features = ["failpoints"]` in Cargo.toml | 🟠 | ✅ |
| `TEST-09` | Implementar tests WASM reales (39 tests en 11 categorías, 744 líneas) | 🔴 | ✅ |
| `TEST-10` | Configurar Vitest + React Testing Library para frontend (40 tests en 6 suites) | 🔴 | ✅ |

### 4.L Pricing & Monetization Strategy

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `MKT-07` | Design and publish Pricing page (Free/Pro/Enterprise tiers). Signal pricing model before Show HN even if cloud is not ready | 🔴 | ✅ |
| `MKT-08` | Register trademark "VantaDB" (USPTO + EUIPO) — pre-Show HN requirement to prevent name squatting | 🔴 | ❌ |
| `MKT-09` | Contributor License Agreement (CLA) for future core contributions — see LEG-02 ✅ | 🟠 | ✅ |
| `BIZ-01` | Design enterprise crate structure: separate paid features to `vantadb-enterprise/` (proprietary crate, Apache 2.0 core stays free) — crate creado con encryption, audit, RBAC, replication módulos | 🟡 | ✅ |
| `BIZ-04` | Cloud architecture design doc: WAL shipping to object storage (S3/R2), serverless read replicas, usage-based pricing model | 🟡 | ❌ |
| `BIZ-05` | Competitive pricing analysis: model $0 self-hosted → $29/mo Pro (1M vectors, 10GB) → $149/mo Business (10M) → $499/mo Enterprise (unlimited) | 🟡 | ❌ |
| `BIZ-06` | Pitch Deck + one-pager for pre-seed fundraising (10 slides) | 🟡 | ❌ |

### 4.M DevOps & CI/CD Infrastructure

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `DEVOPS-07` | Revisar/mejorar Dockerfile multi-stage existente (cache mounts, labels, HEALTHCHECK, non-root user) | 🟡 | ✅ |
| `DEVOPS-08` | Add docs build verification in CI (docs-check.yml: markdownlint + lychee + frontmatter validation) | 🟢 | ✅ |
| `DEVOPS-09` | Auto-deploy web a Vercel/Cloudflare Pages en push a main (web-deploy.yml) | 🟡 | ✅ |
| `DEVOPS-10` | Sign Windows releases (resolver SmartScreen warning) — research doc completado | 🟡 | ✅ |
| `DEVOPS-11` | CodeQL analysis en CI para todos los PRs | 🟡 | ✅ |

### 4.N Database Constraints & Evolution

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `DB-01` | Implementar migration runner (`vanta-cli migrate`) con MigrationEngine en src/migration.rs para schema evolution de formatos en disco | 🔴 | ✅ |
| `DB-02` | Diseñar estrategia de versionado de formatos físicos (VantaFile, WAL, índices) para upgrades seguros | 🔴 | ✅ |
| `DB-03` | Evaluar e implementar sistema ACID transaccional (BEGIN TRANSACTION / COMMIT / ROLLBACK) — research doc creado en docs/research/ACID_TRANSACTIONS.md | 🟡 | ✅ |
| `DB-04` | Expandir bitset más allá de 128 bits para sistemas multi-tenant con muchas categorías — FilterBitset dinámico (Vec<u64>) implementado | 🟢 | ✅ |

## PHASE 5 — Post-Launch / Pre-Seed (November-December 2026)

### 5.A Enterprise Readiness

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `TSK-72` | AES-256-GCM at-rest encryption | 🟡 | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 | ❌ |
| `TSK-110` | SBOM (SPDX/CycloneDX) in each release — implemented via `SEC-06`, `.github/workflows/sbom.yml` | 🟡 | ✅ |
| `BIZ-02` | Asynchronous WAL Shipping (replication without Raft) | 🟡 | ❌ |
| `TSK-122` | Sharded-slab for HNSW lock-free (mitigates `insert_lock` bottleneck) | 🟡 | ❌ |
| `TSK-131` | Implement PITR via archival WAL (archive + point-in-time replay) | 🟡 | ❌ |
| `TSK-132` | Research checkpoint API in Fjall upstream or contribute it — `StorageEngine::create_checkpoint()` + `WalRecord::Checkpoint` implemented | 🟢 | ✅ |
| `TSK-133` | Incremental backup (full snapshot + WAL deltas) | 🟢 | ❌ |
| `TSK-48` | Dynamic quantization (f32→SQ8 for cold nodes) — same as `PERF-09`, implemented in `src/vector/governor.rs` | 🟢 | ✅ |
| `LOW-01` | TLS 1.3 on vantadb-server | 🟢 | ✅ |
| `TSK-142` | Investigate and prototype WASM persistence using OPFS and Web Workers | 🟡 | ❌ |
| `TSK-143` | Fjall vs RocksDB Performance Parity Benchmark for RocksDB Depreciation | 🟡 | ❌ |
| `TSK-144` | Quantitative benchmarking of Recall vs Latency of custom HNSW vs hnswlib | 🟠 | ❌ |
| `ENT-01` | SOC 2 compliance preparation (access controls, audit trails, data retention policies) | 🟡 | ❌ |
| `ENT-02` | HIPAA compliance assessment and BAA readiness documentation | 🟡 | ❌ |
| `ENT-03` | Multi-tenant namespace isolation with resource quotas (RAM, IOPS, storage per tenant) | 🟡 | ❌ |
| `ENT-04` | Connection pooling with circuit breaker for server-mode clients | 🟡 | ❌ |
| `ENT-05` | Plugin system for custom storage backends (WASM-based plugin architecture) | 🟡 | ❌ |

### 5.B VantaDB Cloud and Business

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `CLD-01` | VantaDB Cloud Beta (Fly.io, NVMe, Bearer auth) | 🟡 | ❌ |
| `CLD-02` | Pitch Deck + one-pager (10 pre-seed slides) | 🟡 | ❌ |
| `CLD-03` | Enterprise pilot program (3-5 early adopters) | 🟡 | ❌ |
| `CLD-04` | Case Studies (minimum 2: AI agent memory, local RAG) | 🟡 | ❌ |
| `CLD-05` | Cloud architecture: WAL shipping to S3/R2, serverless read replicas, usage-based billing | 🟡 | ❌ |
| `CLD-06` | Stripe integration for cloud self-service signup + billing | 🟡 | ❌ |
| `CLD-07` | Web dashboard (admin panel for cloud: collections, usage, billing, team management) | 🟡 | ❌ |
| `BIZ-01` | Enterprise crate structure: split paid features to `vantadb-enterprise/` (proprietary crate, Apache 2.0 core stays free) | 🟡 | ❌ |
| `BIZ-03` | Pricing page (Free/Pro/Enterprise) | 🟡 | ✅ |

**Phase 5 Exit Criteria:** 10 enterprise pilots ✓ | $10K MRR ✓ | 3 case studies ✓ | Pitch deck ✓

---

## ⚠️ Risks of Not Doing (pre-launch)

| Risk | Impact | Mitigation | Tracked as |
|--------|---------|------------|------------|
| ~~Pending license audit~~ | ✅ Mitigated — `cargo deny check licenses` passes clean, all deps Apache 2.0 compatible | — |
| Trademark "VantaDB" not registered | Someone else claims the name | Register trademark at USPTO + EUIPO before Show HN | `LEG-01` / `MKT-08` |
| No CLA for external contributors | Can't relicense or use contributions commercially | Add CLA before accepting PRs | `LEG-02` |
| CI/CD for external forks | Community PRs can break CI or inject malicious code | Workflow approval for first-time contributors + restricted secrets | — |
| Mem0 picks another backend as default | Losing 57K-star distribution channel | Integrate VantaDB as Mem0 VectorStoreBackend before they standardize on another | `MEM-01` |
| WASM vector DB space consolidates | Competitor (EdgeVec/minimemory) becomes de-facto WASM standard | Publish benchmarks, ship OPFS, lead the "most complete WASM vector DB" narrative | `MCP-03` / `WASM-02` |
| ~~Docker adoption barrier~~ | ✅ Mitigated — Docker Compose "Local LLM Stack" shipped | — | `DX-03` |
| No pricing signal | Community assumes project is unmonetizable or abandoned | Publish pricing page even before cloud is ready | `MKT-07` |
| ~~Weaviate MCP threat~~ | ✅ Mitigated — VantaDB MCP stabilized with config, error handling, timeouts, graceful shutdown, metrics | — | `MCP-02` |

---

## ⏸️ Icebox — Postponed / No Assigned Priority

Tasks that don't fit in the current roadmap but are kept as a record. No priority, no assigned phase.

### Roadmap v2 (Visualization and Tools)

| ID | Task | Description |
|----|-------|-------------|
| `ROAD-02` | Backup/Restore to S3 | Export .vantadb snapshots to network storage |
| `ROAD-03` | Web UI Explorer | Visualize HNSW topology + vector dispersion (UMAP/t-SNE) |
| `ROAD-04` | Bulk Import CLI | Optimized import of millions of nodes from JSON/CSV with progress bar |
| `ROAD-05` | Multi-model Hooks | Integration with local LLMs (Ollama) and remote (OpenAI) for automatic embeddings |
| `ROAD-07` | Connection Pooling | Reusable connection queue with circuit breaker |
| `ROAD-08` | Schema Validation | Optional strict type validations per namespace |
| `ROAD-09` | Query Caching | LRU cache for hybrid-search with TTL |
| `ROAD-12` | Full-text BM25 v2: improve phrase positions, stemming, and relevance scoring beyond current basic implementation | Native BM25 phrase positions — compete with Weaviate hybrid search quality |
| `ROAD-13` | Query logging and analytics dashboard | Track slow queries, popular collections, cache hit rates |
| `ROAD-14` | Built-in embedding models (lightweight, optional feature) | Ollama integration for automatic embedding generation — reduce setup friction |
| `ROAD-15` | Cron-based collection TTL and compaction scheduler | Auto-maintenance for long-running server deployments |

### Distributed and Multi-node Scaling (v2.0+)

| ID | Task | Description |
|----|-------|-------------|
| `DIST-01` | Raft Consensus | Integration of `openraft` in vantadb-server |
| `DIST-02` | Hash Sharding | Consistent key distribution by hash + cross-shard queries |
| `DIST-03` | Zero-Downtime Upgrades | Rolling restarts without service loss |
| `DIST-04` | ML Cost-Based Optimizer | Heuristic optimizer based on decision trees |
| `DIST-05` | Auto-Indexing | Automatic index creation on frequently filtered fields |
| `DIST-06` | Adaptive TEMPERATURE | Hyperparameter variation based on agent read frequency |
| `DIST-07` | Query Recommendations | Spelling suggestions and corrections in text queries |
| `DIST-08` | Anomaly Detection | Resource spike monitoring in clusters |
| `DIST-09` | Multi-Tenant Isolation | Strict RAM, IOPS and indexing quotas per tenant |
| `DIST-10` | Plugin Marketplace | Sandboxed execution of custom WASM modules |
| `DIST-11` | Edge Federation | Eventual P2P sync between disconnected agents |
| `DIST-12` | Time-Series Mode | Operators and aggregation functions over time windows |
| `DIST-13` | GraphQL API | Query namespaces, graphs and relationships with GraphQL |
| `DIST-14` | CDC (Change Data Capture) | WAL events via WebSocket to external clients |

### VantaLISP / VantaScript (Cognitive Primitives)

| ID | Task | Description |
|----|-------|-------------|
| `LISP-01` | Bytecode JIT | Translation of relational queries to bytecode for direct mmap execution |
| `LISP-02` | Multimodal Unification | Semantic-lexical operators `~` and `FOLLOWS` in IQL |
| `LISP-03` | Fuel 2.0 | Compute limits dynamically bound to CPU/RAM telemetry |
| `LISP-04` | Metacognition | Relationship rehydration and reordering algorithms based on conversation flow |
| `LISP-05` | Monotonic Logic | Coordinated distributed logic without global clock for agents |
| `LISP-06` | Execution Sandbox | FFI restrictions to prevent engine from calling unsafe routines |
| `LISP-07` | LISP-definable CRDTs | Data types for deterministic merge |
| `LISP-08` | Multi-hop | Recursive semantic reasoning paths crossing graph edges |
| `LISP-09` | Parser Fuzzing | Random token injection for compiler robustness |
| `LISP-10` | VantaScript / Inference Logic | Renamed to more readable standards for JS/Python devs |

### Low ROI / Non-Priority

| ID | Task | Reason |
|----|------|--------|
| `LOW-02` | Background compaction in Fjall | Fjall handles its own compaction |

---

## ❌ Do Not Do (until post-seed with team)

| Feature | Reason |
|---------|-------|
| Full SQL | 3-6 months, ICP doesn't need it, pgvector already has it |
| Distributed / Raft | 6-12 months, contradicts embedded philosophy |
| IVF-PQ disk-based | LanceDB does this better, not VantaDB's market |
| GPU acceleration | Breaks zero-config, doesn't solve real bottleneck |
| RBAC / SSO in core | Cloud managed only, post-seed |
| Embedding models bundled | Destroys zero-config (500MB+ wheel) |
| GraphQL API | ICP prefers REST API, MCP already available |
| Git-style versioning | Not ICP pain point, LanceDB already has it |
| Time-series mode | Different product, out of scope |
| 1.5/2-bit Quantization | Marginal returns for datasets <1M |

---

## 📊 Verdict: Actual Project Status

| Aspect | Status | Confidence |
|---------|--------|-----------|
| **Core Engine (Rust)** | 🟢 Solid | 95% |
| **Persistence (WAL, mmap)** | 🟢 Implemented | 90% |
| **Indexes (HNSW, BM25)** | 🟢 Functional | 85% |
| **Python SDK** | 🟢 Complete | 90% |
| **Documentation** | 🟢 Consolidated (Wikilinks, Glossary, Unicode normalized) | 95% |
| **Testing** | 🟡 Rust good (342 lib), WASM 45, Frontend 23, MCP 24, TS 55 | 80% |
| **CLI + Server** | 🟢 Complete (repl, json/quiet, typos) | 95% |
| **API Methods** | 🟢 Complete (filter ops, delete_by_filter, similar_to_key, count, multi-ns) | 95% |
| **Security** | 🟡 Auth básico, sin RBAC, timing attack, security test suite 30 | 70% |
| **DevOps** | 🟡 Docker ready, signed releases pending, test-threads global | 55% |
| **Frontend Architecture** | 🟡 Over-engineered routing, inline styles, GSAP unified, 23 tests | 70% |
| **WASM** | 🟡 Funcional con OPFS, 45 tests | 55% |
| **MCP Protocol** | 🟡 GA-ready (error handling, timeouts, metrics, graceful shutdown) | 65% |
| **Backend Performance** | 🟡 WAL mutex contention, monolito files, batch KV loader done | 70% |
| **Competitive Differentiation** | 🟡 Ocupa nicho único (embebido+WASM+MCP+hybrid+IQL) pero sin distribución | 50% |

## Web Site

The website project is tracked directly in this backlog (section 4.J).
Design spec: [[web/design/DiseñoNuevo.md]].

---

## See Also

- [[master-index]] — Central navigation for all documentation
- [[archive/]] — Archive of historical docs, executed plans, and previous research
- [[FAQ.md]] — Frequently Asked Questions
- [[CHANGELOG.md]] — Release history and implemented features
