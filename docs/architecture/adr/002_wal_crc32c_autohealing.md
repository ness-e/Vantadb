---
title: "ADR 002: WAL Physical Resilience, CRC32C Validation, and Auto-Healing Mechanism"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-01
---

# ADR 002: WAL Physical Resilience, CRC32C Validation, and Auto-Healing Mechanism

## Status

Status: Approved

## Context

In production system crash scenarios (e.g., power failures, kernel panics, or abrupt process termination), the Write-Ahead Log (WAL) file may end up in a corrupt or partially-written state in its final section (trailer).
The absence of strict binary integrity validations and fault-tolerant recovery allowed incomplete or corrupt records to be interpreted during engine startup, causing catastrophic system crashes or, worse, silent data corruption that propagated to the HNSW index and relational storage.

## Decision

To harden the engine's physical consistency against catastrophic persistence failures, a three-pillar redesign of the WAL layer was implemented:

1. **Robust Binary Structure with Versioning:** Each WAL record is serialized under a binary structure with explicitly structured headers and a mutable protocol version:
   * `version: u32`: WAL version identifier. `version = 0` is explicitly prohibited and rejected to prevent decoding binary noise from empty or pre-allocated zero-filled files.
   * `payload_len: u32`: Length of the data payload.
   * `crc32c: u32`: Redundant checksum computed over the complete data payload using the CRC32C (Castagnoli) variant.
   * `payload`: Bytes corresponding to the mutation (`Put`, `Delete`, etc.).

2. **Auto-Healing on Catastrophic Crashes:** During startup and recovery, the WAL reader sequentially parses each binary frame. If corruption is detected (CRC32C mismatch, corrupt header, or premature EOF mid-record):
   * It immediately stops decoding in a controlled, safe manner.
   * Emits detailed warning-level logs (`tracing::warn!`) specifying the corrupt byte offset.
   * Executes an **automatic physical truncation routine (auto-healing)** to cut the WAL file exactly at the position of the last consistent and healthy record.
   * Cleans up any corrupt binary residue after the cut, allowing the engine to continue normal startup with transactions committed up to that point.

3. **Checkpoint Coherence Guarantees:** When invoking database consolidation (`checkpoint`), the physical flush to disk (`flush()` and `sync()`) of active tables is ensured before updating the `checkpoint_seq: u64` pointer in the metadata index, guaranteeing an exact consistency point for restoration.

## Consequences

### Benefits

* **Extreme Power-Failure Resilience:** Chaos tests with active binary fault injection demonstrate that VantaDB always starts without catastrophic errors and in a logically consistent state, regardless of how corrupt the WAL tail end becomes.
* **Silent Corruption Prevention:** No partial or corrupt mutation from incomplete disk writes can be injected into the engine's live memory, protecting embedding databases and relational indexes.
* **Integrated Fuzzing:** A dedicated suite in `heavy_certification.yml` systematically stresses the WAL decoder by injecting random faults and interruptions, certifying 100% stability.

### Technical Debt / Costs

* **Controlled Loss of Uncommitted Transactions:** Physical truncation of the trailer implies discarding the last incomplete or unsynchronized mutation at the time of the crash. This is a standard trade-off in industrial databases and is considered the only safe alternative to injecting corrupt data.
* **CPU Checksum Overhead:** CRC32C computation adds minimal overhead when writing hot records. To mitigate this, we use highly optimized libraries that exploit CPU-level SIMD instructions whenever available.
