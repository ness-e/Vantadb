---
title: "Memory Telemetry — Schema and Validation"
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-07-01
---

# Memory Telemetry — Schema and Validation

This document defines the VantaDB memory observability contract, the operational metrics exposed via Prometheus, and the historical baseline for in-memory data persistence.

---

## 1. Telemetry Breakdown by Subsystem

VantaDB reports memory usage at the host level (total hardware) and at the process level (engine internals and OS RSS).

| Metric | Code Source | Unit | Purpose / Scope |
| :--- | :--- | :--- | :--- |
| `HardwareCapabilities::total_memory` | `sysinfo::System::total_memory()` | Bytes | Host total memory capacity. |
| `process_rss_bytes` | `sysinfo::Process::memory()` | Bytes | Physical resident memory of the process. |
| `process_virtual_bytes` | `sysinfo::Process::virtual_memory()` | Bytes | Virtual memory allocated to the process. |
| `hnsw_nodes_count` | `CPIndex::nodes.len()` | Count | Nodes loaded in the vector index. |
| `hnsw_logical_bytes` | `CPIndex::estimate_memory_bytes()` | Bytes | Deterministic logical estimation of the graph. |
| `mmap_resident_bytes` | Syscalls `mincore` (Unix) / `QueryWorkingSetEx` (Win) | Bytes | Resident pages of mapped files. |
| `volatile_cache_entries` | `volatile_cache.len()` | Count | Active entries in the LRU cache. |

### Prometheus Metrics

The following gauges are registered in `METRICS_REGISTRY` and exposed at `/metrics` in `vantadb-server`:
* `vanta_process_rss_bytes`
* `vanta_process_virtual_bytes`
* `vanta_hnsw_nodes_count`
* `vanta_hnsw_logical_bytes`
* `vanta_mmap_resident_bytes`

---

## 2. Data Contract and Derived State

VantaDB operates with a structured contract across memory and disk to ensure consistency:

* **Identity (Keys):** Generated from `namespace + "\0" + key`.
* **Payload:** Serialized UTF-8 content.
* **Metadata:** Flat attributes composed solely of `FieldValue` enum values (Strings, Integers, Floats, Booleans). Nested JSON objects are not supported to preserve efficiency.
* **Vectors:** Optionally stored in contiguous `f32` precision format within `vector_store.vanta`.

### In-Memory Derived Indexes
The engine reads the canonical key-value storage and materializes the following structures cold, which can be fully rebuilt via `rebuild_index`:
1. **`NamespaceIndex`:** Maps namespace prefix to logical node identifiers.
2. **`PayloadIndex`:** Maps scalar fields for fast relational filtered lookups.
3. **`TextIndex`:** [[bm25|BM25]] inverted indexes for full-text lexical search.

---

## 3. Memory Telemetry Verification

To run local stability measurements and profile memory consumption under continuous inserts, execute the test harness:

```powershell
# Set report file path
$env:VANTA_CERT_REPORT="target/memory_telemetry.json"
cargo test --test memory_telemetry -- --nocapture
```
This validates that RSS memory does not leak and that MMap pages are released correctly when the index is flushed to disk.
