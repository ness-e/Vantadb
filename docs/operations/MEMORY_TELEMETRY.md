# Memory Telemetry Contract

This document defines the memory observability contract used by VantaDB
operational metrics, Prometheus gauges, and certification harnesses.

## Per-Subsystem Breakdown

VantaDB now reports memory at two levels: **host-scoped** (from hardware
detection) and **process-scoped** (from `sysinfo::Process` + engine internals).

| Metric | Source | Units | Scope |
| --- | --- | --- | --- |
| `HardwareCapabilities::total_memory` | `sysinfo::System::total_memory()` | bytes | Host |
| `process_rss_bytes` | `sysinfo::Process::memory()` | bytes | Process |
| `process_virtual_bytes` | `sysinfo::Process::virtual_memory()` | bytes | Process |
| `hnsw_nodes_count` | `CPIndex::nodes.len()` | count | Engine |
| `hnsw_logical_bytes` | `CPIndex::estimate_memory_bytes()` | bytes | Engine |
| `mmap_resident_bytes` | Mapped file residency when available | bytes or null | Engine/OS |
| `volatile_cache_entries` | `volatile_cache.len()` | count | Engine |
| `volatile_cache_cap_bytes` | Configured max cache capacity | bytes | Engine |

### Prometheus Gauges

The following gauges are registered in the `METRICS_REGISTRY` and exported
via the `/metrics` endpoint:

- `vanta_process_rss_bytes`
- `vanta_process_virtual_bytes`
- `vanta_hnsw_nodes_count`
- `vanta_hnsw_logical_bytes`
- `vanta_mmap_resident_bytes`
- `vanta_volatile_cache_entries`
- `vanta_volatile_cache_cap_bytes`

These gauges are updated at:

1. **Engine startup** — after WAL replay and index load.
2. **Flush** — after persisting the HNSW index and backend data.
3. **Rebuild** — after ANN or text-index rebuilds (future).

### SDK Access

```rust
let metrics = db.operational_metrics();
println!("Process RSS: {} bytes", metrics.process_rss_bytes);
println!("HNSW nodes: {}", metrics.hnsw_nodes_count);
println!("HNSW logical bytes: {}", metrics.hnsw_logical_bytes);
println!("MMap resident bytes: {:?}", metrics.mmap_resident_bytes);
```

```python
metrics = db.operational_metrics()
print(f"Process RSS: {metrics['process_rss_bytes']} bytes")
print(f"HNSW nodes: {metrics['hnsw_nodes_count']}")
print(f"HNSW logical bytes: {metrics['hnsw_logical_bytes']}")
print(f"MMap resident bytes: {metrics['mmap_resident_bytes']}")
```

## What these numbers mean

These numbers are useful for:

- Comparing one certification block against another inside the same process.
- Catching obvious regressions in runtime memory growth.
- Understanding whether a scenario triggers materially larger process usage.
- Tracking HNSW index growth and cache utilization over time.

## What these numbers do **not** mean

These numbers do **not** directly represent:

- Exact allocator footprint for HNSW internals; `hnsw_logical_bytes` is a
  deterministic estimate over vectors, nodes, and neighbor lists.
- Portable MMap residency; `mmap_resident_bytes` is `null` on platforms where
  residency cannot be measured.
- OS page cache or backend-specific allocator internals.
- Full host memory pressure from other processes.

For HNSW-only logical footprint, use `CPIndex::estimate_memory_bytes()`, which
also backs `VantaOperationalMetrics::hnsw_logical_bytes`.

## Confidence Levels

- `process_only`: Trustworthy as process-scoped runtime telemetry.
- `untrusted_for_product_claims`: Do not use alone for "low RAM", "better
  footprint", or competitive marketing claims.

## Controlled Harness

Use the dedicated harness to compare controlled scenarios:

```bash
cargo test --test memory_telemetry -- --nocapture
```

Recommended for local runs:

```bash
set VANTA_CERT_REPORT=target/memory_telemetry.json
cargo test --test memory_telemetry -- --nocapture
```

This avoids appending exploratory runs to the tracked certification log.
