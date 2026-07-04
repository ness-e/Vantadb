---
title: "ADR 004: Storage Backend Selection (Fjall vs RocksDB)"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-03
aliases: []
---

# ADR 004: Storage Backend Selection (Fjall vs RocksDB)

## Status

Status: Approved

## Context

VantaDB required an embedded LSM-tree storage engine to serve as the physical persistence layer for its key-value and relational stores. Two candidates were evaluated: Fjall (pure Rust LSM engine) and RocksDB (C++ engine via C bindings through `rust-rocksdb`).

The evaluation criteria were:
1. **Ecosystem Fit:** Ability to compile as a pure Rust dependency without native linker requirements.
2. **Performance Characteristics:** Throughput, write amplification, and compaction behavior under typical vector-database workloads (small payloads, high write throughput during index build, bursty compaction).
3. **Maintenance Overhead:** Build-time complexity, CI matrix expansion from native compilation requirements, and upstream breakage risk.
4. **Deployment Portability:** Cross-compilation targets (musl, ARM, WASM) and avoiding OpenSSL/native dependency chains.

## Decision

1. **Fjall as Default Backend:** The `vantadb` storage layer defaults to Fjall for all embedded scenarios. Fjall is selected for the following reasons:
   - Pure Rust implementation with zero C dependencies, eliminating cross-compilation friction.
   - Direct integration with the Rust memory model, simplifying lifetime management and error propagation.
   - LSM-tree architecture with native `bincode`-level segment encoding, well suited for the small-to-medium payload sizes common in vector metadata storage.
   - Native support for snapshot isolation and range queries without FFI boundary overhead.
   - Significantly faster CI pipeline due to elimination of RocksDB native compilation in default builds.

2. **RocksDB Retained for Legacy Compatibility:** RocksDB remains an optional storage backend, gated behind the `rocksdb` Cargo feature flag. This ensures:
   - Existing deployments that already consume RocksDB for operational monitoring or integration with adjacent systems can migrate incrementally.
   - Benchmarks against a well-known baseline can be maintained.
   - Users with extreme write throughput requirements (millions of writes/sec sustained) can opt into RocksDB's mature compaction engine.

3. **Unified Storage Trait:** Both backends are exposed through a common `StorageBackend` trait in `vantadb-core`, ensuring that all higher-level abstractions (WAL, HNSW persistence, relational engine) are backend-agnostic.

## Consequences

### Benefits

- **Embedded-First Default:** A standard `cargo add vantadb` produces a build with no external C toolchain requirements, dramatically lowering the barrier for new users.
- **Cross-Compilation Simplicity:** Targets such as `aarch64-unknown-linux-musl`, `wasm32-wasi`, and `x86_64-pc-windows-msvc build without RocksDB complications.
- **Feature-Gated Complexity:** Users who never need RocksDB are never exposed to its build system or licensing (BSD 3-Clause vs. Fjall's Apache 2.0/MIT dual license).
- **CI Velocity:** Default-feature tests complete without the 3-5 minute RocksDB C++ compilation, accelerating development iteration.

### Technical Debt / Costs

- **Performance Ceiling Under Extreme Load:** Fjall may not match RocksDB's years of tuned compaction strategies under sustained, petabyte-scale write loads. Users at that scale are expected to enable the `rocksdb` feature.
- **Dual Engine Testing Burden:** All integration tests must pass against both backends, requiring a CI matrix expansion and careful feature-flag-aware test annotations.
- **Feature-Drift Risk:** Without deliberate stewardship, the abstraction boundary may leak backend-specific semantics, requiring periodic audits of the `StorageBackend` trait.
