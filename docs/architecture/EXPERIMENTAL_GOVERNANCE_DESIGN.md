---
title: "Experimental Governance — Design Document (2024 → Phase 5)"
type: architecture
status: draft
tags: [vantadb, architecture, governance, admission-control, conflict-resolution, consistency]
links: "[[Backlog]], [[LISP_ANALYSIS]]"
last_reviewed: 2026-07-04
aliases: [gov-design-doc, experimental-governance]
---

# Experimental Governance — Design Document

> **Source:** `archive/experimental-quarantine-2024-06/experimental-governance/` (7 files, 1,010 LOC — **deleted Jul 2026**)
> **Status:** Código eliminado. Design doc preservado como referencia para Phase 5.
> **Action:** Redesign in **Phase 5** (2026-Q4). Concepts captured here will inform the rewrite.

---

## 1. System Architecture Overview

The governance subsystem consists of four interconnected modules plus a maintenance worker:

```
┌─────────────────────────────────────────────────────┐
│                    MaintenanceWorker                  │
│  (background thread, every 10s or on inactivity)     │
│                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │ Admission     │  │ Conflict     │  │ Consistency │ │
│  │ Filter        │  │ Resolver     │  │ Buffer      │ │
│  │ (Bloom)       │  │ (Devil's     │  │ (Pending     │ │
│  │               │  │  Advocate)   │  │  Records)    │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘ │
│         │                 │                  │          │
│         └─────────────────┴──────────────────┘          │
│                           │                             │
│                    ┌──────▼──────┐                      │
│                    │ Invalidation │                      │
│                    │ Dispatcher   │                      │
│                    │ (MPSC)       │                      │
│                    └──────────────┘                      │
└──────────────────────────────────────────────────────────┘
```

### 1.1 Data Flow

```
Insert/Update Request
    │
    ▼
┌────────────────┐
│ Conflict       │── Reject → AdmissionFilter.block_record()
│ Resolver       │── Superposition → ConsistencyBuffer
│ (Confidence     │── Accept → StorageEngine.insert()
│  Arbiter)      │
└────────────────┘
    │
    ▼
┌────────────────┐
│ Consistency    │── Decay (0.9x confidence/ciclo)
│ Buffer         │── Resolution deadline reached → winner inserted
│ (10s TTL max)  │── Low confidence (<0.2) → purge + tombstones
└────────────────┘
    │
    ▼
┌────────────────┐
│ Maintenance    │── Evict cold nodes (hits < 10, last_access > 60s)
│ Worker         │── Compress thread groups via LLM summarization
│ (10s cycle)    │── Purge INVALIDATED nodes → slashing origin role
└────────────────┘
    │
    ▼
┌────────────────┐
│ Invalidation   │── PremiseInvalidated (re-quantization)
│ Dispatcher     │── InvalidatedPurged (node deleted)
│ (MPSC channel) │── EnvironmentDrift (hardware change)
└────────────────┘
```

---

## 2. Module Design

### 2.1 AdmissionFilter (`admission_filter.rs`)

A probabilistic Bloom Filter that prevents re-ingestion of rejected records.

- **Hash Function:** XxHash64 with 3 salts (3 independent hash positions)
- **Capacity:** Auto-sized: `ceil(capacity_hint × 9.585)` bits, minimum 100,000 bits
- **Operations:**
  - `block_record(id: u64)` — adds a record ID to the filter
  - `block_role(owner_role: &str)` — adds an agent role string to the filter
  - `is_blocked(id) / is_role_blocked(role)` — membership check

**Design Issues:**
1. **Bloom Filter Saturation (CODE-GOV-01):** No false-positive rate tracking or reset mechanism. After ~150K blocked records at default capacity, false positive rate exceeds 50% → system effectively read-only (all inserts rejected).
2. **No removal:** Standard Bloom Filters don't support deletion. A blocked role cannot be unblocked without rebuilding the entire filter.
3. **Single-threaded RwLock:** All operations contend on one RwLock despite Bloom Filters being read-mostly.

### 2.2 ConflictResolver (`conflict_resolver.rs`)

Implements "Devil's Advocate" adversarial conflict resolution using a friction metric.

- **Friction Metric (F_ax):** `sum over origins of (log2(count + 1) × confidence_score)`
- **Resolution Logic:**
  1. If challenger role is slashed (confidence ≤ 0.0) → Reject
  2. If vector cosine similarity > 0.95 AND incumbent is pinned with importance ≥ 0.8:
     - Record collision in OriginCollisionTracker
     - Compute friction: if F_ax < threshold (importance × 10.0) → Reject ("Consistency Barrier")
  3. If challenger confidence < incumbent confidence → Superposition (ConsistencyBuffer)
  4. Otherwise → Accept

**Design Issues:**
1. **Friction Barrier Inverted (CODE-GOV-02):** Higher collision count increases friction, making it EASIER for malicious actors to pass the barrier (more collisions = higher F_ax = higher chance of accepting bad data from a known bad actor). Should be: more collisions = lower F_ax = harder to pass.
2. **No timeout on collision tracking:** Origins accumulate forever. A role slashed 6 months ago still occupies memory.
3. **O(n) friction computation:** Iterates all origins on every collision — O(n) in a RwLock critical section.

### 2.3 ConsistencyBuffer (`consistency.rs`)

Temporal buffer for conflicting records that cannot be immediately resolved.

- **Storage:** `HashMap<u64, ConsistencyRecord>` behind RwLock
- **ConsistencyRecord:** Contains node_id, candidates (max 3), state (PendingConflict / ResolvedAccept / ResolvedReject), injection timestamp, resolution deadline
- **Resolution:** Best-confidence winner selection, tombstone for losers via `AuditableTombstone` stored in BackendPartition::TombstoneStorage

**Design Issues:**
1. **Confidence Death Spiral (CODE-GOV-03):** `confidence_score *= 0.9` every maintenance cycle (line 129). After ~22 cycles (220s), all pending records fall below 0.2 and are purged. Records in legitimate long-term conflict are silently deleted.
2. **force_flush() drops data (CODE-GOV-04):** Picks highest-importance candidate, discards all others silently. No audit trail. Called with no backpressure or circuit breaker.
3. **force_flush() never called:** No monitoring triggers `force_flush()`. It exists but no OOM guard or timer invokes it.
4. **`_shrinks_deadline` variable unused:** Computed but `_shrinks_deadline` is assigned with underscore prefix and never read (dead code). The deadline reduction on line 125-127 only applies when this would be true, but it's never checked.

### 2.4 InvalidationDispatcher (`invalidations.rs`)

Synchronous MPSC channel for invalidation events.

- **Events:** PremiseInvalidated, InvalidatedPurged, EnvironmentDrift
- **Pattern:** `mpsc::channel()`, producer-consumer with a background listener thread
- **Capacity:** Unbounded channel — no backpressure

**Design Issues:**
1. **Unbounded channel (CODE-GOV-05):** Under high invalidation load (e.g., mass re-quantization), the channel grows without bound → OOM.
2. **Blocking send:** `sender.send(event)` blocks if receiver is slow. Combined with unbounded channel, receiver can fall arbitrarily far behind.
3. **eprintln! logging:** Production events logged to stderr instead of tracing.

### 2.5 MaintenanceWorker (`maintenance_worker.rs`)

Background thread that cycles every 10s or on inactivity (>5s), performing eviction, persistence, compression, and compaction.

- **Trigger:** Emergency flag OR `now - last_activity > inactivity_threshold_ms` (5000ms)
- **Stages:**
  1. ConsistencyBuffer decay + resolution
  2. Volatile cache eviction (hits decay 0.5x, remove if hits < 10 AND last_access > 60s)
  3. Consolidation (persist to backend)
  4. Purge INVALIDATED nodes + role slashing
  5. Data compression (LLM summarization of thread groups, `remote-inference` feature)
  6. Disk compaction if tombstone volume > 10,000

**Design Issues:**
1. **No backpressure (CODE-GOV-06):** Maintenance runs regardless of load. During peak traffic, it evicts cache entries that are actively being queried.
2. **Confidence reset on half cycles (CODE-GOV-07):** `node.hits *= 0.5` every cycle (line 268). After 4 cycles (40s), a node with 100 hits drops to 6 hits and is eligible for eviction.
3. **Compression deletes originals before summarizing:** In the `execute_data_compression` path (line 433-447), originals are deleted after summarizing. If the server crashes between the delete loop and the completion, data is permanently lost.
4. **Deadlock risk (CODE-GOV-08):** `run_maintenance_cycle` acquires `volatile_cache.write()` while holding implicit locks from the caller. If the caller holds any other lock, this creates a lock ordering hazard.
5. **Emergency trigger is fire-once:** `emergency_maintenance_trigger` is set to `true` by some monitor but reset to `false` at the start of the cycle (line 62-63). If the monitor sets it again during the cycle, the flag is missed until next iteration.

---

## 3. 12 Bugs Catalog

### 🔴 Critical (Data Loss / System Blockage)

| ID | File:Line | Bug | Impact |
|----|-----------|-----|--------|
| GOV-01 | `admission_filter.rs:16-25` | Bloom filter saturates at ~150K inserts → false positive rate > 50% | System permanently read-only (all inserts rejected as "blocked") |
| GOV-03 | `maintenance_worker.rs:129` | Confidence score decays 0.9× every cycle | Records in legitimate conflict silently purged after ~220s |
| GOV-04 | `consistency.rs:111-144` | `force_flush()` picks 1 winner, drops all others, no audit trail | Data loss under memory pressure |
| GOV-07 | `maintenance_worker.rs:268` | `node.hits *= 0.5` every maintenance cycle | Active nodes evicted after 40s of inactivity |
| GOV-09 | `maintenance_worker.rs:433-447` | Deletes originals during compression before summarizing is complete | Data loss on crash during compression |

### 🟠 Severe (Functional / Security)

| ID | File:Line | Bug | Impact |
|----|-----------|-----|--------|
| GOV-02 | `conflict_resolver.rs:130-137` | Friction barrier is inverted (more collisions = easier to pass) | Malicious actors with high collision count bypass conflict resolution |
| GOV-05 | `invalidations.rs:30` | Unbounded MPSC channel with blocking send | OOM under high invalidation load |
| GOV-06 | `maintenance_worker.rs:52` | No backpressure — maintenance runs at peak traffic | Evicts actively queried cache entries |
| GOV-08 | `maintenance_worker.rs:219` | `volatile_cache.write()` lock acquired while other locks may be held | Deadlock potential |

### 🟡 Minor (Logic / Performance)

| ID | File:Line | Bug | Impact |
|----|-----------|-----|--------|
| GOV-10 | `admission_filter.rs:27-36` | No reset mechanism for Bloom filter | Permanent filter degradation |
| GOV-11 | `consistency.rs:124` | `_shrinks_deadline` dead code (variable never read) | Deadline reduction logic never executes |
| GOV-12 | `maintenance_worker.rs:56-65` | Emergency trigger reset race (fire-once semantics) | Emergency maintenance may be missed |

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
| **LISP DSL (deleted)** | Governance was designed as a companion to LISP — LISP would define query semantics, Governance would enforce consistency. Both were experimental and neither was completed. |
| **IQL (current)** | IQL is a flat query language with no governance features. Phase 5 may optionally add governance-aware query modifiers (e.g., `AFTER <version>`, `CONSENSUS <min_confidence>`). |
| **WAL (current)** | Governance decisions should be WAL-logged for crash recovery. The current `force_flush()` bypasses WAL entirely. |
| **StorageEngine** | Governance hooks into `StorageEngine::insert()` via conflict resolution. Needs explicit hook points rather than ad-hoc calls. |

---

## See Also

- [[Backlog]] — `GOV-01`: Rediseño de governance (Phase 5, Q4 2026)
- [[LISP_ANALYSIS]] — Capabilities from the deleted LISP experiment that influenced governance design
- [[docs/strategy/ROADMAP.md]] — Phase 5 definition and timeline
