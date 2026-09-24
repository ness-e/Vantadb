---
title: "Experimental Governance — Proposed Design (NOT Implemented)"
type: architecture
status: draft
implemented: false
tags: [vantadb, architecture, governance, admission-control, conflict-resolution, consistency]
links: "[[Backlog]], [[LISP_ANALYSIS]]"
last_reviewed: 2026-08-05
aliases: [gov-design-doc, experimental-governance]
---

# Experimental Governance — Design Document

> **Original source:** `archive/experimental-quarantine-2024-06/experimental-governance/` (7 files, 1,010 LOC — **archived Jul 2026**; ⚠️ origin path could not be re-verified on 2026-08-05)
> **Current implementation:** ⚠️ **NONE — RETRACTED (2026-08-05).** An earlier draft claimed a "current implementation" in `src/governance/` (`admission.rs`, `conflict.rs`, `consistency.rs`, `worker.rs`, `mod.rs`). **That directory does not exist** (`Test-Path` = false). The governance subsystem was never implemented beyond the archived experiment. This is the only (marked) historical mention of `src/governance` left in the document.
> **Status:** ⚠️ **PROPOSED DESIGN — NOT IMPLEMENTED.** No code corresponds to this design. Every "FIXED in current code" / "current implementation" statement below describes *intended* fixes in the archived experiment — **none is verified against live code**.
> **Action:** Redesign in **Phase 5** (2026-Q4). Concepts captured here will inform the rewrite.

---

## 0. Implementation Status & Mapping to Existing Code

> **Retraction (2026-08-05):** this document previously claimed verification against
> `src/governance/` on 2026-07-21. The directory does not exist; the claim was false.
> The governance subsystem (admission control, conflict resolution, consistency
> buffer, maintenance worker) is **unimplemented** — it is a proposed design for
> Phase 5, not a shipped feature.

The only existing code related to this document is `src/gds.rs`
(`GraphDataScience`, 403 lines) — but it is a **different subsystem** and does
**not** implement any governance concept:

| Exists today (`src/gds.rs`) | Governance design (future — NOT implemented) |
|------------------------------|-----------------------------------------------|
| `GraphDataScience::page_rank(roots, max_iterations, damping, tolerance)` (gds.rs:38) — iterative PageRank over the subgraph reachable from `roots` | `AdmissionFilter` — Bloom Filter + CountMinSketch (design §2.1) |
| `GraphDataScience::degree_centrality(roots)` (gds.rs:145) — in/out degree counts per node | `ConflictResolver` — version vectors + friction metric (design §2.2) |
| Single-threaded graph algorithms over `StorageEngine` via `GraphTraverser` (~10K nodes) | `ConsistencyBuffer` — TTL-based pending records (design §2.3) |
| Unit-tested in `src/gds.rs` (`#[cfg(test)]` module) | `MaintenanceWorker` — background maintenance cycle (design §2.5) |

> `src/gds.rs` is **Graph Data Science** (ranking/centrality), not admission
> control or conflict resolution. The two share only the `StorageEngine`
> dependency. Do not cite this document as describing implemented behavior.

## 1. System Architecture Overview

The governance subsystem consists of four interconnected modules plus a maintenance worker:

```
┌─────────────────────────────────────────────────────────────┐
│                    MaintenanceWorker                         │
│  (background thread, every 10s or on inactivity)            │
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │ AdmissionFilter │  │ Conflict       │  │ Consistency    │ │
│  │ (Bloom + Count- │  │ Resolver       │  │ Buffer         │ │
│  │  Min Sketch)    │  │ (Version Vect) │  │ (TTL-based)    │ │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘ │
│          │                   │                    │          │
│          └───────────────────┴────────────────────┘          │
│                              │                               │
│                     (no InvalidationDispatcher —             │
│                      removed in current implementation)      │
└──────────────────────────────────────────────────────────────┘
```

### 1.1 Data Flow

```
Insert/Update Request
    │
    ▼
┌────────────────┐
│ Conflict       │── Reject → AdmissionFilter.block_record()
│ Resolver       │── Superposition → ConsistencyBuffer
│ (Version       │── Accept → StorageEngine.insert()
│  Vectors)      │
└────────────────┘
    │
    ▼
┌────────────────┐
│ Consistency    │── TTL expiry → winner inserted or tombstone
│ Buffer         │── force_flush() → picks highest confidence
│ (TTL-based)    │── Buffer full → backpressure error
└────────────────┘
    │
    ▼
┌────────────────┐
│ Maintenance    │── Bloom filter auto-reset (FP rate threshold)
│ Worker         │── Conflict log GC (1h TTL)
│ (10s on        │── Buffer flush + expiry
│  inactivity)   │
└────────────────┘

Note: InvalidationDispatcher (MPSC channel) was removed from the
current implementation. Invalidation events are handled directly
by their respective modules.
```

---

## 2. Module Design

### 2.1 AdmissionFilter (`admission.rs` → original: `admission_filter.rs`)

A Bloom Filter + CountMinSketch that prevents re-ingestion of rejected records.

- **Hash Function:** XxHash64 with 3 seeds (3 independent hash positions)
- **Capacity:** Auto-sized: `ceil(capacity_hint × 9.585)` bits, minimum 100,000 bits
- **Operations:**
  - `block_record(id: u64)` — adds a record ID to the filter
  - `block_role(owner_role: &str)` — adds an agent role string to the filter
  - `is_blocked(id) / is_role_blocked(role)` — membership check
  - `record_frequency(id)` — frequency estimation via CountMinSketch
  - `reset_filter()` — manual reset; auto-reset when FP rate exceeds threshold
  - `estimated_fp_rate()` — live false-positive rate estimation

**Design Issues:**
1. ~~Bloom Filter Saturation (GOV-01):~~ **FIXED in current code.** Auto-reset mechanism triggers when `estimated_fp_rate() > reset_threshold` (default 5%). FP rate is tracked live via `estimated_fp_rate()`.
2. **No removal:** Standard Bloom Filters don't support deletion. A blocked role cannot be unblocked without rebuilding the entire filter (mitigated by auto-reset).
3. **Single-threaded RwLock:** All operations contend on one RwLock despite Bloom Filters being read-mostly.
4. ~~No reset mechanism (GOV-10):~~ **FIXED in current code.** Both manual `reset_filter()` and auto-reset on threshold exist.

### 2.2 ConflictResolver (`conflict.rs` → original: `conflict_resolver.rs`)

Implements version-vector conflict resolution with exponential backoff and friction-based rejection.

- **Core Mechanism:** Version vectors for causal ordering. Concurrent writes enter friction-based resolution.
- **Friction Metric:** `1.0 / (log2(total_collisions) + 1.0 + epsilon)` — higher collisions → lower friction → harder to pass.
- **Backoff:** Exponential backoff per node (capped at 64), bounded per-conflict counter.
- **Resolution Logic:**
  1. If version vectors are causally ordered (Before/After) → winner is the later version
  2. If concurrent AND values equal → merge version vectors
  3. If concurrent AND values differ → compute friction + backoff:
     - If friction ≥ threshold → Superposition (ConsistencyBuffer)
     - Otherwise → challenger accepted after backoff
- **Audit:** Every resolution logs a `ConflictRecord` with node_id, origins, resolution, nonce, timestamp.

**Design Issues:**
1. ~~Friction Barrier Inverted (GOV-02):~~ **FIXED in current code.** Formula inverted: `1.0 / (log2(total) + 1.0 + epsilon)`. Higher collisions → lower friction value → harder to exceed threshold. Comment in source explicitly notes "(GOV-02 fix)".
2. **No timeout on collision tracking:** Backoff counters (`conflict_backoff` HashMap) grow without bound — no TTL eviction.
3. **O(1) friction computation** (was O(n) in original): Only looks up the two involved origins, not all origins.

### 2.3 ConsistencyBuffer (`consistency.rs`)

Temporal buffer for conflicting records that cannot be immediately resolved.

- **Storage:** `HashMap<u64, PendingRecord<T>>` behind RwLock, bounded by `max_size`
- **PendingRecord:** Contains node_id, candidates, state (PendingConflict / ResolvedAccept / ResolvedReject), injection timestamp, TTL deadline, last_touched
- **TTL Expiry:** `expire_entries()` removes records past their deadline — no confidence decay
- **Flush:** `flush_all()` returns a structured `FlushResult` with `accepted`, `rejected`, and `tombstones` lists — full audit trail
- **Backpressure:** `try_insert()` returns `Err(BufferFull)` when buffer is at capacity
- **Touch:** `touch()` extends a record's deadline (for long-lived conflicts)

**Design Issues:**
1. ~~Confidence Death Spiral (GOV-03):~~ **FIXED in current code.** Replaced with TTL-based expiry (deadline on each `PendingRecord`). No confidence decay.
2. ~~force_flush() drops data (GOV-04):~~ **FIXED in current code.** `flush_all()` returns `FlushResult` with accepted/rejected/tombstone lists. Every candidate is recorded. Traceable.
3. **force_flush() never called (archived):** The original design had an unused `force_flush()`. Current `flush_all()` is called by `MaintenanceWorker.should_flush()` when count or time threshold is exceeded.
4. ~~`_shrinks_deadline` dead code (GOV-11):~~ **FIXED in current code.** No such variable exists. Deadline management uses `Instant` directly.

### 2.4 InvalidationDispatcher (`invalidations.rs` — ARCHIVED)

**This module does not exist in any implementation (see §0 retraction).** It was part of the original experimental design and was removed during the rewrite. Invalidation events would be handled directly by their respective modules without a central dispatcher.

**Original Design (archived):**
- Synchronous MPSC channel for invalidation events (PremiseInvalidated, InvalidatedPurged, EnvironmentDrift)
- `mpsc::channel()`, producer-consumer with a background listener thread
- Unbounded channel — no backpressure

**Original Design Issues (all N/A — removed):**
1. ~~Unbounded channel (GOV-05):~~ Removed. No central invalidation channel exists.
2. ~~Blocking send:~~ Removed.
3. ~~eprintln! logging:~~ Removed. Current code uses `tracing` throughout.

### 2.5 MaintenanceWorker (`worker.rs` → original: `maintenance_worker.rs`)

Background thread that cycles every 10s on inactivity (>5s), performing bloom reset checks, conflict GC, and buffer housekeeping.

- **Trigger:** `now - last_activity > inactivity_threshold_ms` (5000ms). Runs ONLY when inactive — avoids competing with live traffic.
- **Stages:**
  1. Bloom filter FP rate check → auto-reset if threshold exceeded; warning at 80% of threshold
  2. Conflict log GC (remove entries older than 1h)
  3. Buffer expiry (TTL-based) + flush (if count/time threshold reached)
  4. Health status update

**Design Issues:**
1. **No load-aware backpressure (GOV-06):** Inactivity-based scheduling avoids competing with traffic but doesn't consider CPU or WAL pressure explicitly. Good enough for current scale; Phase 5 should add load metrics.
2. ~~Confidence reset on half cycles (GOV-07):~~ **FIXED in current code.** Hit decay removed entirely. No `hits *= 0.5` exists.
3. ~~Compression deletes originals (GOV-09):~~ **REMOVED.** LLM summarization (`execute_data_compression`) does not exist in current implementation.
4. ~~Deadlock risk (GOV-08):~~ **FIXED in current code.** `run_maintenance_cycle` takes locks sequentially (admission → conflict → buffer), no nested lock patterns.
5. ~~Emergency trigger fire-once (GOV-12):~~ **REMOVED.** No emergency trigger mechanism exists in current code.

---

## 3. Bug Catalog — Status vs Current Implementation

> ⚠️ **Not verified (2026-08-05).** The bugs below were identified in the original experimental code. They were **NOT** verified against any live implementation — see §0 retraction. Status reflects the *intended* design of the archived experiment, not verified code.

### 🔴 Critical (Data Loss / System Blockage)

| ID | Original File | Bug | Current File | Status |
|----|--------------|-----|-------------|--------|
| GOV-01 | `admission_filter.rs:16-25` | Bloom filter saturates at ~150K inserts → FP rate > 50% | `admission.rs:151-153` | ✅ **FIXED** — auto-reset when `estimated_fp_rate()` > `reset_threshold` (default 5%) |
| GOV-03 | `maintenance_worker.rs:129` | Confidence score decays 0.9× every cycle → records silently purged | `consistency.rs:141-156` | ✅ **FIXED** — replaced with TTL-based expiry on each `PendingRecord` |
| GOV-04 | `consistency.rs:111-144` | `force_flush()` picks 1 winner, drops all others, no audit trail | `consistency.rs:175-219` | ✅ **FIXED** — `flush_all()` returns `FlushResult` with accepted/rejected/tombstones |
| GOV-07 | `maintenance_worker.rs:268` | `node.hits *= 0.5` every cycle → active nodes evicted in 40s | `worker.rs` | ✅ **FIXED** — hit decay removed. No `hits *= 0.5` exists |
| GOV-09 | `maintenance_worker.rs:433-447` | Deletes originals during compression before summarizing complete | N/A | ❌ **N/A** — LLM compression feature removed from current implementation |

### 🟠 Severe (Functional / Security)

| ID | Original File | Bug | Current File | Status |
|----|--------------|-----|-------------|--------|
| GOV-02 | `conflict_resolver.rs:130-137` | Friction barrier inverted (more collisions = easier to pass) | `conflict.rs:233-248` | ✅ **FIXED** — formula inverted: `1.0 / (log2(total) + 1.0)`. Source explicitly notes `(GOV-02 fix)` |
| GOV-05 | `invalidations.rs:30` | Unbounded MPSC channel with blocking send → OOM | N/A | ❌ **N/A** — `InvalidationDispatcher` module removed from current implementation |
| GOV-06 | `maintenance_worker.rs:52` | No backpressure — maintenance runs at peak traffic | `worker.rs:84-103` | ⚠️ **PARTIAL** — inactivity-based scheduling (runs only when inactive >5s) but no CPU/WAL load metrics |
| GOV-08 | `maintenance_worker.rs:219` | `volatile_cache.write()` while other locks held → deadlock risk | `worker.rs:133-199` | ✅ **FIXED** — `run_maintenance_cycle` takes locks sequentially with no nesting |

### 🟡 Minor (Logic / Performance)

| ID | Original File | Bug | Current File | Status |
|----|--------------|-----|-------------|--------|
| GOV-10 | `admission_filter.rs:27-36` | No reset mechanism for Bloom filter → permanent degradation | `admission.rs:232-244` | ✅ **FIXED** — `reset_filter()` + auto-reset on threshold |
| GOV-11 | `consistency.rs:124` | `_shrinks_deadline` dead code — deadline reduction never executes | N/A | ✅ **FIXED** — variable removed. Deadline managed via `Instant` |
| GOV-12 | `maintenance_worker.rs:56-65` | Emergency trigger reset race (fire-once) | N/A | ❌ **N/A** — Emergency trigger mechanism removed from current implementation |

---

## 4. Phase 5 Redesign Recommendations

### 4.1 Bloom Filter → Cuckoo Filter + SBF

Replace `AdmissionFilter` with a Cuckoo Filter (supports deletion) combined with a Scalable Bloom Filter (auto-resize on saturation). Track false positive rate and emit warning at 1%, block at 5%.

### 4.2 Friction Metric Fix

Invert the friction computation: `F_ax = sum(1 / (log2(count + 1) × confidence + epsilon))`. Higher collisions → lower friction → harder to pass. Add a time decay to collision history (last-24h window).

### 4.3 Consistency Buffer with Persistence

Replace in-memory `HashMap` with a persistent queue (WAL-backed). Replace confidence decay with a staleness metric based on wall-clock time, not cycle count. `force_flush()` should tombstone ALL candidates (not just discard).

### 4.4 Bounded Invalidation Channel

Use `crossbeam::bounded` channel with backpressure or `tokio::sync::mpsc` with `reserve()`. Drop oldest events under pressure instead of blocking.

### 4.5 Maintenance Worker with Backpressure

Skip maintenance if CPU > 80% or WAL queue depth > 1000. Replace hit decay with exponential moving average (EMA) over wall-clock time. Add cancellation token for shutdown coordination.

### 4.6 Audit Trail

All governance actions (block, resolve, purge, slash) should write to an append-only audit log. `AuditableTombstone` is a good start but needs timestamps and causal ordering.

---

## 5. Key Terminology

| Term | Definition |
|------|------------|
| **Admission Control** | Preventing known-bad records from being ingested |
| **Friction Metric (F_ax)** | Adversarial resistance score — higher = harder for bad actors to force conflicts |
| **Slashing** | Setting an agent origin's confidence to 0.0, permanently banning them |
| **Superposition** | State where conflicting candidates are held pending temporal resolution |
| **Consistency Decay** | Progressive confidence reduction of pending records over time |
| **Devil's Advocate** | Adversarial resolution strategy: challengers must overcome a friction barrier |
| **Premise Invalidated** | A node's quantized representation diverged from ground truth (soft invalidation) |
| **Invalidated Purged** | A node was hard-deleted with role slashing (hard invalidation) |
| **Environment Drift** | Hardware profile changed (CPU features, memory), requiring re-benchmark |

---

## 6. Relationship to Other Systems

| System | Relationship |
|--------|-------------|
| **LISP DSL (deleted)** | Governance was designed as a companion to LISP — LISP would define query semantics, Governance would enforce consistency. Both were experimental. |
| **IQL (current)** | IQL is a flat query language with no governance features. Phase 5 may optionally add governance-aware query modifiers (e.g., `AFTER <version>`, `CONSENSUS <min_confidence>`). |
| **WAL (current)** | Governance decisions should be WAL-logged for crash recovery. The current implementation does not yet integrate with the WAL subsystem. |
| **StorageEngine** | Governance hooks into `StorageEngine::insert()` via conflict resolution. Needs explicit hook points rather than ad-hoc calls. |

---

## See Also

- [[Backlog]] — `GOV-01`: Rediseño de governance (Phase 5, Q4 2026)
- [[LISP_ANALYSIS]] — Capabilities from the deleted LISP experiment that influenced governance design
- [[docs/strategy/ROADMAP.md]] — Phase 5 definition and timeline
