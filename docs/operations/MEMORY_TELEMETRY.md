# Memory Telemetry Contract

This document defines the current memory contract used by VantaDB benchmarks and certification output.

## What the contract measures

The certification harness now records **process-scoped** memory from `sysinfo` with explicit units:

- `process_memory_current_mb`
- `process_memory_delta_mb`
- `process_virtual_memory_current_mb`
- `process_virtual_memory_delta_mb`

These values are sourced from:

- `sysinfo::Process::memory()`
- `sysinfo::Process::virtual_memory()`

The report also stores:

- `schema_version`
- `memory_source`
- `memory_confidence`

## What these numbers mean

These numbers are useful for:

- comparing one certification block against another inside the same process
- catching obvious regressions in runtime memory growth
- understanding whether a scenario triggers materially larger process usage

## What these numbers do **not** mean

These numbers do **not** directly represent:

- logical HNSW memory footprint
- mmap residency
- OS page cache
- backend-specific allocator internals
- full host memory pressure

For HNSW-only logical footprint, use the explicit estimate reported by benchmark code such as `estimate_memory_bytes()` in `tests/certification/stress_protocol.rs`.

## Confidence levels

- `process_only`: trustworthy as process-scoped runtime telemetry
- `untrusted_for_product_claims`: do not use alone for “low RAM”, “better footprint”, or competitive marketing claims

## Controlled harness

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
