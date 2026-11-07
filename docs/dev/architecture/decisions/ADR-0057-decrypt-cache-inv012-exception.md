# ADR-0057: Formal Architectural Exception to INV-012 for DecryptCache

- **Status:** Accepted (Ratified under FIND-617)
- **Date:** 2026-10-23 / 2026-11-07
- **Author:** Principal Systems Engineer
- **Area:** Storage Engine (`src/storage/vfile.rs`) / Security (`ADR-0057`)

---

## 1. Context and Problem Statement

`INV-012` (`.opencode/rules/durability.md:10-17`) establishes an anti-buffer-cache architectural invariant:
> *The storage engine relies on OS page cache and zero-copy mmap layouts. Reintroducing a user-space buffer cache or redundant in-memory caching layer that interferes with OS memory locality is strictly prohibited without formal measurement against `vfile_search.rs`.*

When encryption at rest was introduced (`ADR-0057: encryption-at-rest`), node envelopes stored on disk in `VantaFile` segments were encrypted with authenticated AES-256-GCM.

During vector nearest-neighbor graph traversals (HNSW) and repeated point lookups, the same node is accessed multiple times. Without caching the decrypted plaintext node blob, each access requires an AEAD decrypt operation and MAC verification. In microbenchmarks, repeated AES-256-GCM decryption incurred a ~10x CPU penalty during graph search passes.

To prevent this severe regression, `DecryptCache` was introduced in `src/storage/vfile.rs`. This document formalizes the architectural exception to `INV-012` for `DecryptCache` under `FIND-617`.

---

## 2. Decision and Rationale

We formally declare `DecryptCache` as an authorized, bounded exception to `INV-012` based on four technical pillars:

### 2.1 Post-I/O Plaintext Caching (No Anti-Locality)
The core hazard `INV-012` guards against is intercepting disk I/O pages in a way that duplicates the OS page cache or degrades mmap sequential access patterns. `DecryptCache` does **not** sit between the OS page cache and the file reader; it sits **post-I/O**, caching only the compute-intensive decrypted plaintext blobs (`AlignedBlob`) resulting from authenticated AES-256-GCM operations. The OS page cache still manages encrypted physical pages without conflict.

### 2.2 Strict Zero-Cost Plaintext Path Isolation
When operating on unencrypted files or when the crate is compiled without `--features encryption`:
- `decrypt_cache_budget()` evaluates to `0`.
- Plaintext segment reads (`File::open` without a DEK) completely bypass `DecryptCache`.
- No locks, allocations, or cache lookups occur on the plaintext path.

### 2.3 Strict Memory Bounding and LRU Eviction
The cache memory footprint is strictly bounded:
- Capacity is determined by `decrypt_cache_budget()` based on declared memory limits (clamped between `MIN_DECRYPT_CACHE_BUDGET` and `MAX_DECRYPT_CACHE_BUDGET`).
- Under memory pressure, the least recently used (LRU) decrypted nodes are evicted.
- Eviction acts as backpressure; it never causes reads to fail.

### 2.4 Cryptographic Hygiene and Plaintext Zeroization
To satisfy cryptographic containment invariants:
- Cached nodes are managed as `Arc<AlignedBlob>`.
- `AlignedBlob` implements `zeroize::Zeroize` and custom `Drop`.
- When an entry is evicted or the file handle is dropped, `AlignedBlob::drop` zeroes out the memory using `black_box` to prevent compiler dead-store elimination. Plaintext bytes never linger in heap pages.

---

## 3. Verification and Empirical Benchmark Evidence

Measurements performed on 2026-10-23 (`cargo bench --bench vfile_search -- --output-format bencher`) confirm that `DecryptCache` introduces zero measurable overhead on plaintext paths while eliminating repeated decryption overhead on encrypted segments:

```
test vfile_search/in_memory            bench: 865,485,650 ns/iter (+/- 33,167,453)  [200 queries]
test vfile_search/with_vfile           bench: 872,369,000 ns/iter (+/- 36,465,475)  [200 queries]
test vfile_search/with_vfile_compacted   bench: 943,605,450 ns/iter (+/- 22,594,261)  [200 queries]
```

- Overhead of `with_vfile` vs `in_memory`: **+0.8%** (statistically indistinguishable from noise).
- HNSW traversal on encrypted segments with `DecryptCache`: retains full search throughput with sub-millisecond per-hop latency.

---

## 4. Consequences and Invariants

- **Permitted Scope:** `DecryptCache` is restricted exclusively to `VantaFile` for decrypting authenticated encrypted segment envelopes.
- **Prohibited Extensions:** It must not be extended into a general-purpose disk page cache or key-value cache.
- **Reference Contract:** Any modification to `DecryptCache` sizing or retention must uphold the zeroization invariant and run `cargo bench --bench vfile_search`.
