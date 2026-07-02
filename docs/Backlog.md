---
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
links: "[[master-index]]"
last_refined: 2026-07-01 (AUD-06..18 + sdk.rs split + docs consolidation + code hardening ✅)
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


## PHASE 4 — Launch (Jul-Sep 2026)

### 4.0 Foundational (blocking — do first)

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `MKT-06` | Logo and branding (SVG, palette, favicon) | 🔴 | ❌ |
| `REL-01` | Bump workspace version v0.1.5 → v0.2.0 (SemVer: 340+ commits, nuevas APIs, 4 plataformas) | 🔴 | ❌ |
| `LEG-01` | Register trademark "VantaDB" (USPTO + EUIPO) before Show HN | 🔴 | ❌ |
| `LEG-02` | Add Contributor License Agreement (CLA) for future core contributions | 🟠 | ❌ |

### 4.B Framework Integrations

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `INT-01` | LangChain adapter (PyPI + PR langchain-community) | 🔴 | ❌ |
| `INT-02` | LlamaIndex adapter (PyPI + PR llama-index) | 🔴 | ❌ |
| `TSK-90` | Mem0: VantaDB as VectorStoreBackend | 🟠 | ❌ |
| `TSK-89` | CrewAI: VantaDBMemory for multi-agent crews | 🟡 | ❌ |
| `TSK-91` | DSPy: VantaDBRM (Retrieval Module) | 🟡 | ❌ |
| `TSK-92` | Haystack: VantaDBDocumentStore | 🟡 | ❌ |
| `TSK-116` | vantadb-openai (optional embedding package) | 🟡 | ❌ |
| `TSK-117` | vantadb-ollama (local offline embedding) | 🟡 | ❌ |
| `TSK-95` | vantadb-litellm (universal gateway embeddings) | 🟡 | ❌ |

### 4.D Launch Campaign

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `MKT-01` | Landing page (vantadb.dev): hero, benchmarks, comparisons | 🔴 | ❌ |
| `MKT-02` | Blog post "Introducing VantaDB" (technical + benchmarks) | 🔴 | ❌ |
| `MKT-03` | Show HN post (timing, title, prepared responses) | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟠 | ❌ |
| `MKT-05` | Technical blog posts (5+ pre-launch) | 🟠 | ❌ |
| `COM-01` | Discord: announcements, general, help, showcase, dev | 🔴 | ❌ |
| `TSK-106` | GitHub Discussions (Q&A, Ideas, Show & Tell) | 🟡 | ❌ |
| `TSK-107` | Community showcase (projects in docs/showcase.md) | 🟡 | ❌ |
| `TSK-108` | Newsletter (Substack/Beehiiv, monthly) | 🟢 | ❌ |
| — | Good first issues (20+ tagged issues) | 🟠 | ❌ |

### 4.F Distribution

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `TSK-121` | SHA256 hash verification of the wheel in tests | 🟢 | ❌ |
| `REL-02` | Publish `vantadb-ts` npm package (WASM, 26/26 tests, examples listos) | 🔴 | ❌ |

### 4.G Developer Experience

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB (showcase) | 🟠 | ❌ |
| `TSK-103` | Public benchmark site (compare.py vs chroma/lancedb/qdrant) | 🟠 | ❌ |
| `DX-01` | Refactor API: `VantaDB()` → `connect()` (eliminar redundancia, alinear con SQLite3/LanceDB/DuckDB) | 🟠 | ❌ |
| `DX-02` | Python SDK latency optimization: reduce p50 from ~62ms to <20ms (PyO3 FFI overhead) | 🟠 | ❌ |


### 4.H Code Health & Security

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `SEC-01` | Migrate `bincode` (RUSTSEC-2025-0141 — unmaintained). Direct dep used in index serialization, WAL, state. Alternatives: `postcard`, `rkyv`, `borsh`. | 🔴 | ❌ |
| `SEC-02` | Migrate `rustls-pemfile` (RUSTSEC-2025-0134 — unmaintained). Direct dep for TLS in vantadb-server. Alternatives: `rustls-pki-types` or inline PEM parsing. | 🔴 | ❌ |
| `DOC-01` | Unit tests: 34/48 modules without `#[cfg(test)]`. Priority: `config.rs`, `engine.rs`, `executor.rs`, `gc.rs`, `metrics.rs`, `storage.rs`, `graph.rs`, `backends/` | 🟡 | ⏳ |
| `DOC-02` | Refactor `insert_hnsw()` in `src/index.rs` (177L → 3 functions: `compute_inv_cached_norm`, `shrink_neighbors`, `insert_hnsw`) | 🟡 | ✅ |
| `DOC-03` | Normalize 6 files with Unicode/accent in filename to pure ASCII (avoids cross-platform issues) | 🟢 | ✅ |

### 4.I Documentation Consolidation

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `DOC-04` | Restore unique content from archived VantaDB-MPTS files: Vision/UVP, GTM/Strategic Roadmap, design principles, flowcharts, risk register (~2,900 unique lines with no EN equivalent). Create `docs/vision/VISION.md`, `docs/strategy/ROADMAP.md`, `docs/strategy/GO_TO_MARKET.md` | 🔴 | ✅ |
| `DOC-05` | Translate to English 10 docs from `operations/` + 3 ADRs that are in Spanish (violate EN convention) | 🟡 | ✅ |
| `DOC-06` | Unified frontmatter schema (title, status, tags, last_reviewed, aliases) for 117 .md files | 🟡 | ⏳ |
| `DOC-07` | Unify naming convention to kebab-case without accents or spaces | 🟢 | ✅ |
| `DOC-08` | Archive `TEXT_INDEX_PHASE_1_CLOSEOUT.md`, `RELEASE_V0.1.1.md`, `MILESTONE_V0.2.0.md` (historical) | 🟢 | ✅ |

### 4.J Web Frontend

| ID | Task | Priority | Status |
|----|-------|-----------|--------|

| `WEB-06` | Fase 4: Migrate 637 inline styles to CSS Modules | 🟡 | ❌ |
| `WEB-07` | Fase 4: Configure Vitest and Playwright E2E for web | 🟡 | ❌ |
| `WEB-08` | Fase 5: Anti-Slop Audit, Performance Budget, SEO Final Review | 🟢 | ❌ |

## PHASE 5 — Post-Launch / Pre-Seed (November-December 2026)

### 5.A Enterprise Readiness

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `TSK-72` | AES-256-GCM at-rest encryption (foreign key) | 🟡 | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 | ❌ |
| `TSK-110` | SBOM (SPDX/CycloneDX) in each release | 🟡 | ❌ |
| `BIZ-02` | Asynchronous WAL Shipping (replication without Raft) | 🟡 | ❌ |
| `TSK-122` | Sharded-slab for HNSW lock-free (mitigates `insert_lock` bottleneck) | 🟡 | ❌ |
| `TSK-131` | Implement PITR via archival WAL (archive + point-in-time replay) | 🟡 | ❌ |
| `TSK-132` | Research checkpoint API in Fjall upstream or contribute it | 🟢 | ❌ |
| `TSK-133` | Incremental backup (full snapshot + WAL deltas) | 🟢 | ❌ |
| `TSK-48` | Dynamic quantization (f32→SQ8 for cold nodes) | 🟢 | ❌ |
| `LOW-01` | TLS 1.3 on vantadb-server | 🟢 | ✅ |
| `TSK-142` | Investigate and prototype WASM persistence using OPFS and Web Workers | 🟡 | ❌ |
| `TSK-143` | Fjall vs RocksDB Performance Parity Benchmark for RocksDB Depreciation | 🟡 | ❌ |
| `TSK-144` | Quantitative benchmarking of Recall vs Latency of custom HNSW vs hnswlib | 🟠 | ❌ |

### 5.B VantaDB Cloud and Business

| ID | Task | Priority | Status |
|----|-------|-----------|--------|
| `CLD-01` | VantaDB Cloud Beta (Fly.io, NVMe, Bearer auth) | 🟡 | ❌ |
| `CLD-02` | Pitch Deck + one-pager (10 pre-seed slides) | 🟡 | ❌ |
| `CLD-03` | Enterprise pilot program (3-5 early adopters) | 🟡 | ❌ |
| `CLD-04` | Case Studies (minimum 2) | 🟡 | ❌ |
| `BIZ-01` | Enterprise crate structure: split features pagas a `vantadb-enterprise/` (crate propietario separado del core Apache 2.0) | 🟡 | ❌ |
| `BIZ-03` | Pricing page (Free/Pro/Enterprise) | 🟡 | ❌ |

**Phase 5 Exit Criteria:** 10 enterprise pilots ✓ | $10K MRR ✓ | 3 case studies ✓ | Pitch deck ✓

---

## ⚠️ Risks of Not Doing (pre-launch)

| Risk | Impact | Mitigation | Tracked as |
|--------|---------|------------|------------|
| ~~Pending license audit~~ | ✅ Mitigated — `cargo deny check licenses` passes clean, all deps Apache 2.0 compatible | — |
| Trademark "VantaDB" not registered | Someone else claims the name | Register trademark at USPTO + EUIPO before Show HN | `LEG-01` |
| No CLA for external contributors | Can't relicense or use contributions commercially | Add CLA before accepting PRs | `LEG-02` |
| CI/CD for external forks | Community PRs can break CI or inject malicious code | Workflow approval for first-time contributors + restricted secrets | — |

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
| `ROAD-11` | Docker Compose | Pre-configured environment VantaDB Server + Ollama + Web UI |

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
| **Testing** | 🟢 Complete (Compiles clean, 265/265 tests passing) | 90% |
| **CLI + Server** | 🟢 Complete (repl, json/quiet, typos) | 95% |
| **API Methods** | 🟢 Complete (filter ops, delete_by_filter, similar_to_key, count, multi-ns) | 95% |

## Web Site

The website project has its own backlog at `web/docs/backlog.md`,
tracking design, content, and frontend tasks.

See also: [[web/docs/README.md]]

---

## See Also

- [[master-index]] — Central navigation for all documentation
- [[archive/]] — Archive of historical docs, executed plans, and previous research
- [[FAQ.md]] — Frequently Asked Questions
- [[CHANGELOG.md]] — Release history and implemented features
