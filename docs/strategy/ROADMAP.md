---
title: VantaDB Engineering Roadmap
type: strategy
status: stable
tags: [vantadb, roadmap, milestones, phases, timeline, engineering]
last_reviewed: 2026-07-01
aliases: [Roadmap, Milestones, Engineering Plan, Timeline]
---

# VantaDB Engineering Roadmap

> **Domain:** Product & Engineering
> **Detail:** [Backlog](../Backlog.md) contains per-task breakdown
> **Purpose:** Define phases, exit criteria, and architectural decisions. Does not duplicate backlog.

---

## Roadmap Philosophy

1. **Engineering goals, not commercial dates** — each phase has quantitative exit criteria
2. **Stability before features** — "done" means "tested, documented, production-ready"
3. **Radical transparency** — public roadmap, weekly status, documented known issues

---

## Current Status (v0.1.4 — June 2026)

| Phase | Status | Description |
|-------|--------|-------------|
| **FASE 0** | ✅ 100% | Post-quarantine stabilization |
| **FASE 1** | ✅ 100% | [[hnsw\|HNSW]] scalability & performance |
| **FASE 2** | ✅ 100% | Architectural hardening |
| **FASE 3** | ✅ 100% | Pre-launch: AUD-01→44 complete, crates.io published |
| **FASE 4** | 🔄 ~50% | Community launch + ecosystem |
| **FASE 5** | ⬜ 0% | Post-launch / Pre-seed |

---

## Phase 1: [[hnsw|HNSW]] Scalability & Performance ✅

| Metric | Target | Achieved |
|--------|--------|----------|
| Recall@10 (100K) | ≥0.95 | 0.998 |
| p50 Latency (100K) | <20ms | 12.4ms |
| Memory Efficiency | <1500 bytes/vector | 1172 bytes/vector |
| Scale Factor | <6x (10K→50K) | 4.83x (linear O(N)) |

## Phase 2: Architectural Hardening ✅

| Risk | Severity | Status |
|------|----------|--------|
| AUD-01: [[wal\|WAL]] durability (fsync before ACK) | 🔒 Blocking | ✅ Resolved |
| AUD-02: [[wal\|WAL]] missing checksums | 🔒 Blocking | ✅ Resolved ([[crc32c\|CRC32C]]) |
| AUD-03: Rebuild concurrency | ⚠️ High | ✅ Resolved (exclusive lock) |
| AUD-04: Missing file locking | ⚠️ High | ✅ Resolved (fs2) |
| AUD-05: GIL not released consistently | ⚠️ High | ✅ Resolved (py.allow_threads) |

---

## Phase 3: Pre-Launch ✅

**Timeline:** 2026-05-01 → 2026-08-31

### Sub-Phases
- **3.A** Critical blockers (CI, telemetry, SIGTERM)
- **3.B** Python SDK performance (zero-copy FFI, async, stubs, batch)
- **3.C** Core Engine ([[mmap]] [[hnsw|HNSW]] ✅, [[sq8|SQ8]] quantization ✅, [[wal|WAL]] vacuum ✅, TTL ✅, backpressure ✅, eviction ✅)
- **3.D** Testing & Quality (real datasets, proptest, regression gates)
- **3.E** Observability (Prometheus, JSON logging, Grafana dashboard) ✅
- **3.F** Essential Documentation (GraphRAG, durability, migration guides, CHANGELOG) ✅

### Key Technical Milestones Achieved

- [x] Chaos testing: 30 iterations crash injection (between writes + tight loop)
- [x] CI/CD audit: toolchain @stable, runner fixes, FORCE_NODE24 removed
- [x] NaN/Inf validation in Python FFI (TSK-53)
- [x] Text index audit with no critical issues (TSK-36)
- [x] Extended [[bm25|BM25]] corpus (8 validations, TSK-38)
- [x] Windows functional skill scripts (TSK-23)
- [x] CLI tests (33), server tests (14 unit + 6 E2E), MCP tests (9)
- [x] HTTP server merged into `vanta-cli` (TSK-30)
- [x] [[wal|WAL]] compaction/vacuum with 256MB auto-trigger (TSK-75)
- [x] Windows file locking tests: FILE_SHARE_READ, DELETE, stale lock (DISC-02)
- [x] [[sq8|SQ8]] quantization: 4x RAM reduction (TSK-47)
- [x] rkyv zero-copy archives (TSK-49)
- [x] Official Grafana dashboard (ROAD-06)
- [x] TTL on memory records: `expires_at_ms`, `ttl_ms`, `purge_expired()` (TSK-76)
- [x] [[wasm|WASM]] core build: 5 optional deps, Vec-backed [[mmap]] shim, cfg-gated metrics (TSK-71)

### Exit Criteria

| Criterion | Target |
|-----------|--------|
| Python SDK p50 latency | <20ms |
| Windows CI | ✅ Green |
| RAM telemetry | Correct (RSS vs mmap) |
| Chaos tests | In CI + 30/30 pass |
| 1M vectors | No OOM on 16GB RAM |
| Documentation | 90%+ coverage |

---

## Phase 4: Community Launch 🔄

**Timeline:** 2026-06-01 → 2026-10-31

### Sub-Phases
- **4.0** Foundational (crates.io ✅, SECURITY.md 🔴, logo 🔴, WASM ✅)
- **4.A** TypeScript SDK (WASM, npm, examples)
- **4.B** Framework Integrations (LangChain, LlamaIndex, Mem0, CrewAI, DSPy)
- **4.C** API Completeness (expanded filters, delete_by_filter, similar_to_key)
- **4.D** Launch Campaign (landing page, blog, Show HN, Discord, CONTRIBUTING)
- **4.E** CLI Polish (backup, restore, doctor, stats, inspect, REPL, TUI)
- **4.F** Distribution (ARM64 wheels, Homebrew, Python 3.13)
- **4.G** Developer Experience (demo app, benchmark site, Rust examples)

### Exit Criteria

| Metric | Target |
|--------|--------|
| GitHub Stars | 1,000+ |
| PyPI Downloads/mo | 10,000+ |
| Discord Members | 500+ |
| Contributors | 20+ |
| TypeScript SDK | Published on npm |
| LangChain + LlamaIndex | Published on PyPI |

---

## Phase 5: Enterprise / Pre-Seed ⬜

**Planned Start:** 2026-Q4

### Sub-Phases
- **5.A** Enterprise Readiness (encryption, audit logs, [[wal|WAL]] shipping)
- **5.B** VantaDB Cloud + Business (beta, pitch deck, enterprise pilots, pricing)

### Exit Criteria

| Metric | Target |
|--------|--------|
| Enterprise Pilots | 10+ |
| MRR | $10K+ |
| Case Studies | 3+ published |
| Pitch Deck | Complete |

---

## Pending Architectural Decisions

### 1. Query Language

| Option | Pros | Cons |
|--------|------|------|
| **A:** Programmatic API only | ✅ Simplicity | ❌ Power users limited |
| **B:** Simple DSL (Mongo-like) | ✅ More flexible | ❌ Medium complexity |
| **C:** SQL subset | ✅ Familiar | ❌ Poor fit for vectors |
| **Decision:** Deferred to Phase 5 (evaluate user feedback) |

### 2. Distributed Mode

| Option | Pros | Cons |
|--------|------|------|
| **A:** Stay single-node | ✅ Embedded coherence | ❌ Enterprise ceiling |
| **B:** Async master-slave replication | ✅ Read scalability | ❌ Eventual consistency |
| **C:** Sharding + Raft | ✅ Full scale | ❌ Massive complexity |
| **Decision:** Deferred to Phase 5+. [[wal\|WAL]] shipping (BIZ-02) as intermediate step. |

### 3. Licensing

| Option | Pros | Cons |
|--------|------|------|
| **A:** Apache 2.0 (current) | ✅ Maximum adoption | ❌ Difficult monetization |
| **B:** Open core (Apache + BSL enterprise) | ✅ Balance | ❌ Legal complexity |
| **C:** AGPL | ✅ Cloud protection | ❌ Limits enterprise adoption |
| **Decision:** Apache 2.0 for now. Re-evaluate at Phase 5. |

---

## High-Level Timeline

```
Q3 2026 (Jul-Sep)
├── Jul ── 4.0 Foundational (crates.io, SECURITY.md, logo, WASM), 4.C API ops
├── Aug ── 4.A TypeScript SDK, 4.B Integrations (LangChain, LlamaIndex), 4.E CLI
└── Sep ── 4.D Launch Campaign (landing, blog, Show HN, Discord), 🚀 LAUNCH

Q4 2026 (Oct-Dec)
├── Oct ── 4.F Distribution (ARM64, Homebrew), 4.G DevEx
├── Nov ── Phase 5: TSK-72 (encryption), BIZ-02 ([[wal|WAL]] shipping), enterprise pilots
└── Dec ── CLD-01 (Cloud alpha), CLD-02 (pitch deck), CLD-04 (case studies)

Q1 2027
├── Enterprise pilots expansion
├── Pre-seed fundraising preparation
└── [[wal|WAL]] shipping MVP

Q2 2027
├── VantaDB Cloud beta (Fly.io)
├── 3+ enterprise pilots
└── Target: $10K MRR
```

---

## See Also

- [Master Index](../VantaDB-MPTS/Master%20Index.md) — Parent document
- [Backlog](../Backlog.md) — Detailed per-phase tasks
- [VISION.md](../vision/VISION.md) — Why these phases matter
- [GO_TO_MARKET.md](GO_TO_MARKET.md) — How the launch executes
- [OPERATIONS.md](../archive/Operaciones,%20Calidad%20y%20Riesgos.md) — Risks being mitigated
