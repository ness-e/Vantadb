---
title: "ADR-020: Storage backend default — Fjall (retroactive consolidation of ADR 004)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, storage, backend, fjall, rocksdb]
created: 2026-08-16
last_reviewed: 2026-08-16
related: [004_storage_backend.md]
---

# ADR-020: Storage backend default — Fjall (retroactive consolidation of ADR 004)

## Status

Accepted (retroactive). Consolidates ADR 004 with the current on-disk evidence.

## Context

The choice of Fjall as the default embedded storage backend was already made in
code before any ADR documented it with code evidence. ADR 004
(`004_storage_backend.md`, accepted 2026-07-21) records the decision rationale
(Fjall default, RocksDB opt-in behind the `rocksdb` feature) but cites no
`file:line` evidence. This ADR retro-documents the decision against the current
source so the rationale and the code never drift apart again (FND-21, P20d).

## Decision

**Fjall is the default storage backend; RocksDB is opt-in behind the `rocksdb`
Cargo feature; InMemory is available for tests/performance mode.** Verified
current state:

- `Cargo.toml:97` — `default = ["cli", "arrow", "fjall", "roaring", "advanced-tokenizer", "memmap2", "fs2", "sysinfo", "rayon"]`: `fjall` is compiled by default; `rocksdb` is **not** part of default features.
- `Cargo.toml:44-46` — both backends are optional dependencies (`fjall = "3.1"`, `rocksdb = "0.24.0"` with `lz4`, `default-features = false`).
- `src/config.rs:582-598` — `VantaConfig::default()` resolves `backend_kind` from the `VANTA_BACKEND` env var; `None` → `BackendKind::Fjall` (`:594`); unrecognized values warn and fall back to Fjall (`:587-593`).
- `src/storage/engine/init.rs:269-289` — runtime dispatch: `BackendKind::RocksDb` requires the `rocksdb` feature (`:270-278`, `ValidationError` otherwise), `BackendKind::Fjall` requires the `fjall` feature (`:279-287`), `BackendKind::InMemory` always available (`:288`).

Rationale (from ADR 004, still valid): pure-Rust dependency graph (no C
toolchain, no OpenSSL chain), cross-compilation simplicity (musl/ARM/WASM),
and CI velocity — default builds skip RocksDB's 3-5 min C++ compile.

## Consequences

- **Positive:** default `cargo add vantadb` builds with zero native dependencies; the `StorageBackend` trait keeps the engine backend-agnostic.
- **Negative:** Fjall's compaction is less battle-tuned than RocksDB under sustained petabyte-scale write load — users at that scale opt into `rocksdb`; the dual-engine surface requires feature-flag-aware test annotations.
- **Debt:** RocksDB's error paths (`init.rs:270-278`) and the `StorageBackend` trait boundary are not covered by tests (no covering tests found for `FjallBackend`/`RocksDbBackend` at discovery time); a future ADR or audit should close that gap.
- **No drift:** the default has not changed since ADR 004; this ADR pins the evidence so a future backend switch requires an explicit ADR update.

## Related

- ADR 004 (`004_storage_backend.md`) — the canonical decision record; this ADR adds its code evidence.