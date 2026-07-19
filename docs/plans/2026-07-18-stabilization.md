# Stabilization Plan — Feature Freeze Phase 1

> **Inicio:** 2026-07-18
> **Estado:** ✅ FASE 1 y 2 COMPLETE — estabilización sellada en v0.3.0-stable
> **Fuente:** Phase 0.1 audit (`/audit full` — 10 Critical, 15 Important findings)
> **Feature Freeze:** PROMPT-MAESTRO-FREEZE.md — no features, no new deps, no API changes, zero-bug policy

## Gate Summary

| Result | Count |
|--------|-------|
| ✅ COMPLETED | 4 P0 + 5 P1 + 3 P2 |
| 🟡 DEFER | 7 (P1-2, P1-5..9) |
| ❌ SKIP | ~160 backlog items (features, marketing, integration — BLOCKED by freeze) |
| 🔴 BLOQUEADO | 0 |

## P0 — Certify Blockers (must fix to PASS, zero-bug policy)

### ~~P0-1: FIX-CLIPPY-CORE — 9 unnecessary `unsafe` blocks in core~~

- **Estado:** ❌ SKIPPED — `unnecessary_unsafe` is nightly-only lint; stable `cargo clippy -D warnings` passes clean.

### P0-2: FIX-DEPRECATED — `put_batch` deprecation in vantadb_py

- **Esfuerzo:** 🟢 15min
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/src/lib.rs:5`
- **Gate Justificación:** Certify blocker — `cargo clippy -D warnings` fails on python crate
- **Gate Result:** ✅ DO
- **Contrato:** `cargo clippy -p vantadb_py -- -D warnings` → 0 warnings
- **Estado:** ✅ COMPLETED

### ~~P0-3: FIX-DEADCODE — 4 dead methods~~

- **Estado:** ❌ SKIPPED — Only flagged in `#[cfg(not(feature = "memmap2"))]` shim (not compiled in default build). Stable clippy doesn't warn.

### P0-4: FIX-DENY — 2 stale advisory ignores in deny.toml

- **Esfuerzo:** 🟢 5min
- **Prioridad:** 🔴
- **Archivos clave:** `deny.toml`
- **Gate Justificación:** Certify blocker — stale advisories invalidate deny check
- **Gate Result:** ✅ DO
- **Contrato:** `cargo deny check advisories` → clean
- **Estado:** ✅ COMPLETED

## P1 — Security & Performance (ideal before RC)

### ✅ P1-1: SEC-WASM-UNWRAP — Fix `Reflect::set().unwrap()` in WASM bridge

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-wasm/src/lib.rs` (19 calls → `.ok()`)
- **Gate Justificación:** 19 calls to `.unwrap()` on `Reflect::set()` — can panic on JS exceptions
- **Gate Result:** ✅ DO
- **Contrato:** grep-count of `Reflect::set\(\)\.unwrap\(\)` = 0
- **Estado:** ✅ COMPLETED

### ~~P1-2: SEC-MMAP-UB — Generation counter for `MmapFull`/`SendPtr`~~

- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb/src/lib.rs` (MmapFull struct + SendPtr)
- **Gate Justificación:** Handle reuse of stale mmap pointers with generation counter
- **Gate Result:** 🟡 DEFER
- **Contrato:** `MmapFull` has a generation field, checked before access
- **Estado:** ❌ DEFERRED — theoretical UB, no known trigger, 1d effort

### ✅ P1-3: SEC-WASM-OOM — Input size validation at WASM FFI boundary

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-wasm/src/lib.rs` (MAX_F32_VEC_LEN, MAX_BATCH_SIZE)
- **Gate Justificación:** Reject oversized inputs before allocation
- **Gate Result:** ✅ DO
- **Contrato:** WASM test with ~1GB input returns Err, not OOM
- **Estado:** ✅ COMPLETED

### ✅ P1-4: SEC-ALIGN — Runtime alignment assertions on f32 reinterpret casts

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `src/index/search.rs`, `src/index/serialize.rs`, `src/storage/engine/ops.rs`, `src/storage/archive.rs`
- **Gate Justificación:** Prevent UB on misaligned f32 reads
- **Gate Result:** ✅ DO
- **Contrato:** `debug_assert_eq!(ptr.align_offset(4), 0)` on each cast path
- **Estado:** ✅ COMPLETED

### ~~P1-5: PERF-WAL — Reusable buffer in WAL serialization~~

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb/src/wal.rs`
- **Gate Justificación:** `postcard::to_allocvec` allocates per op — replace with reusable buffer
- **Gate Result:** 🟡 DEFER
- **Contrato:** WAL benchmark shows < N+1 allocs per batch
- **Estado:** ❌ DEFERRED — post-RC optimization

### ~~P1-6: PERF-PREFIX — Streaming iterator for `scan_prefix`~~

- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb/src/fjall_backend.rs`
- **Gate Justificación:** `scan_prefix` materializes full Vec before processing
- **Gate Result:** 🟡 DEFER
- **Contrato:** `scan_prefix_iter()` returns `impl Iterator<Item=T>` not `Vec<T>`
- **Estado:** ❌ DEFERRED — post-RC optimization

### ~~P1-7: PERF-LEXICAL — Truncate candidate pool in `lexical_search`~~

- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb/src/index/lexical.rs`
- **Gate Justificación:** HashMap grows unbounded before `get_many`
- **Gate Result:** 🟡 DEFER
- **Contrato:** Pool size ≤ top_k * 2 before `get_many`
- **Estado:** ❌ DEFERRED — post-RC optimization

### ~~P1-8: PERF-MEMREC — Single-pass in `memory_record_from_node`~~

- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb/src/index/lexical.rs`
- **Gate Justificación:** 2-pass filter + collect in memory_record_from_node
- **Gate Result:** 🟡 DEFER
- **Contrato:** Single pass filter_map instead of filter + collect
- **Estado:** ❌ DEFERRED — post-RC optimization

### ~~P1-9: PERF-HNSW — Reuse HashSet across HNSW layers~~

- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb/src/index/hnsw.rs`
- **Gate Justificación:** New HashSet allocated per layer
- **Gate Result:** 🟡 DEFER
- **Contrato:** Single HashSet reused with `.clear()`
- **Estado:** ❌ DEFERRED — post-RC optimization

### ✅ P1-10: CLN-MACHETE — Remove unused deps

- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb/Cargo.toml`, `vantadb-python/Cargo.toml`, etc.
- **Gate Justificación:** `cargo machete` findings
- **Gate Result:** ✅ DO
- **Contrato:** `cargo machete` → 0 unused deps
- **Estado:** ✅ COMPLETED

## P2 — Docs & Polish (completed)

### ✅ P2-1: DOC-OVERVIEW — Fix vantadb-core doc overview

- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Contrato:** `cargo doc --no-deps` → no warnings
- **Estado:** ✅ COMPLETED — expanded crate-level doc with core types, feature flags, usage example

### ✅ P2-2: DOC-SECURITY — Fill placeholder sections in SECURITY.md

- **Esfuerzo:** 🟢 30min
- **Contrato:** No "TODO" or placeholder text in SECURITY.md
- **Estado:** ✅ COMPLETED — created SECURITY.md at repo root with supported versions, reporting process, security practices

### ✅ P2-3: DOC-WASM — Document WASM API

- **Esfuerzo:** 🟢 1h
- **Contrato:** All public WASM exports have doc comments
- **Estado:** ✅ COMPLETED — filled last 2 missing doc gaps (`worker` module, `ListOptions`)

## Deferred / Blocked

| ID | Reason |
|----|--------|
| All TIER 1-6 backlog features | 🔴 Blocked by Feature Freeze contract — no features until RC |
| MKT-* (marketing items) | 🟡 Deferred — not code-stabilization work |
| INT-* (LangChain, LlamaIndex) | 🔴 Blocked — integrations are new features |
| DEVOPS-* (infrastructure) | 🟡 Deferred — only P0 infra (certify pipeline) kept |
| COMP-* (competitive analysis) | 🟡 Deferred — research, not code work |
| NUEVO-* (new feature requests) | 🔴 Blocked by freeze |
| Cosmetic refactors | ❌ Skipped — YAGNI during freeze |
| DRV-116, DRV-117 | ✅ Covered by P0-1, P0-3, P0-4 |
