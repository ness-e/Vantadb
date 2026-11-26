---
title: "WASM threads: lock-free multi-thread design (STRAT-04)"
kind: concept
status: active
description: "Design and measured first slice for lock-free multi-threaded WASM (SharedArrayBuffer + atomics), with COOP/COEP requirements and single-thread fallback"
tags: [wasm, concurrency, architecture]
---

# WASM threads: lock-free multi-thread design (STRAT-04)

> **Status:** design + first slice measured (2026-10-06). Product integration is
> tracked as a follow-up (see [§Integration path](#integration-path)).
> Task: `docs/dev/tasks/STRAT-04.md` (plan `2026-10-04-master-plan-0.9.0.md`, Task 64).

## Why

The WASM binding has no parallel compute today: `vantadb-wasm/src/worker.rs`
offloads OPFS I/O to one Web Worker (message passing, no shared compute), and
the only atomics in the crate are flags/counters. The Kuzu-successor claim
(`Backlog-negocio.md`) needs an answer to "can the browser build run work in
parallel?" — Kuzu itself ships a dual build (single-thread default +
`multithreaded` variant that requires cross-origin isolation), and DuckDB-WASM
ships single-threaded by default with experimental multithreading.

## Decision

**SharedArrayBuffer + atomics with a custom lock-free kernel, not
`wasm-bindgen-rayon`, for the first slice.** Evidence:

| Factor | Custom SAB + atomics | wasm-bindgen-rayon |
|---|---|---|
| Benchmark reproducibility | Node `worker_threads` — Node supports SAB + shared `WebAssembly.Memory` with no headers (verified 2026-10-06) | JS glue is browser-only (its tests/demo use browsers); requires `--target web` |
| Supply chain | no new runtime dependency | GoogleChromeLabs repo archived 2024-07-17 → single-maintainer personal fork |
| Memory layout control | host owns the arena layout (zero-copy vector data) | rayon parallelizes over Rust slices; layout stays opaque |
| Ergonomics | manual work distribution (`fetch_add` cursor) | `par_iter` |

`wasm-bindgen-rayon` remains the better choice for complex data-parallel
algorithms later — re-evaluate when the integration needs more than the
chunk-claim pattern (documented in the task file Spec).

## Build requirements (declared)

- **Toolchain:** nightly Rust + `-Zbuild-std` + `-C target-feature=+atomics` +
  linker flags `--shared-memory --max-memory --import-memory`. The precompiled
  std/core ship without atomics (tracking issue rust-lang/rust#77839 is still
  open), so `build-std` is mandatory. The whole `vantadb-wasm` graph compiles
  under this configuration (reproducible: `RUSTFLAGS='--cfg
  getrandom_backend="wasm_js" -C target-feature=+atomics' cargo +nightly check
  -p vantadb-wasm --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort`
  → exit 0; log in the task file session, `target/strat04-s5-check.log`).
- **Browser host:** `SharedArrayBuffer` and shared `WebAssembly.Memory` require
  a **secure context + cross-origin isolation**:
  `Cross-Origin-Opener-Policy: same-origin` + `Cross-Origin-Embedder-Policy:
  require-corp` (MDN). Node has no such requirement.
- **Fallback (mandatory):** feature-detect (`crossOriginIsolated`,
  `wasm-feature-detect` threads) and fall back to the single-thread build. The
  shipped npm package stays single-thread; the threads build is opt-in. This is
  the same shape Kuzu and DuckDB-WASM use.

## The first slice (measured)

`vantadb-wasm/threads-kernel/` — a standalone `no_std` cdylib (deliberately not
a workspace member) with one kernel: batch cosine scoring over a flat f32 arena
in shared memory, work-distributed with a **lock-free chunk cursor**
(`AtomicU32::fetch_add` — no locks; each chunk claimed exactly once).

- Build: `pwsh vantadb-wasm/threads-kernel/build.ps1` (nightly + build-std,
  ~20 s; output ≈ 1.7 KB `.wasm` importing `env.memory`).
- Bench (100k — regenerates `benchmarks/wasm_threads_results.json`):
  `node benchmarks/wasm_threads_bench.mjs --n-vectors 100000 --dims 128
  --workers 1,2,4,8,12 --repeats 7` (Node harness, `worker_threads`;
  workers=1 is the single-thread baseline).
- Bench (400k — regenerates `benchmarks/wasm_threads_results_400k.json`):
  `node benchmarks/wasm_threads_bench.mjs --n-vectors 400000 --dims 128
  --workers 1,2,4,8,12 --repeats 5 --output benchmarks/wasm_threads_results_400k.json`.

Results (Windows VM, 12 vCPU, Node 26.8.1, 2026-10-06; max abs error vs f64
reference 5.4e-8; the harness NaN-gate proves no index is left unscored —
exactly-once semantics are proven by the kernel's native multi-thread test in
`src/lib_tests.rs`):

| Workers | Wall (median) | Compute (max worker) | Speedup (compute) | Vectors/s |
|---|---|---|---|---|
| 1 | 10.15 ms | 9.94 ms | 1.00x | 9.9 M |
| 2 | 6.37 ms | 6.07 ms | 1.64x | 15.7 M |
| 4 | 3.30 ms | 3.08 ms | 3.23x | 30.3 M |
| 8 | 2.34 ms | 2.13 ms | 4.68x | 42.8 M |
| 12 | 1.99 ms | 1.79 ms | 5.55x | 50.3 M |

At 400k vectors × 128 dims the same curve holds (5.10x wall / 5.16x compute at
12 workers). Sub-linear at the top end (VM core scaling + message orchestration),
but clearly positive.

### Two findings that shape any integration

1. **Per-instance stacks are mandatory.** Every instance shares ONE linear
   memory, so `__stack_pointer` must point at a disjoint region per worker or
   concurrent frames corrupt each other. `wasm-bindgen`'s threads transform does
   this via a walrus-injected start function; a raw module must export
   `__stack_pointer` (`--export=__stack_pointer`, a mutable global the host
   assigns) and the host must carve per-worker stack regions beyond the module's
   static layout (`--initial-memory` fixes that boundary). The harness does
   both.
2. **V8 tier-up dominates naive benchmarks.** The first call to a wasm function
   runs in the Liftoff baseline (~7x slower); timed runs must warm up first.
   A cold-start harness reports *negative* scaling (the cold call cost scales
   with worker count). The harness keeps workers alive and warms up before
   measuring — any future wasm benchmark must do the same.

## Integration path

1. **Feature `threads` in `vantadb-wasm`** (build recipe proven above): kernel
   module + JS glue that spawns workers, creates the shared memory, assigns
   per-worker stacks, and falls back to single-thread when
   `crossOriginIsolated` is false.
2. **Data path:** the core `Embedded`/`StorageEngine` is not thread-safe by
   design (`Arc<RwLock<…>>` + domain locks); do NOT share one engine instance
   across workers. Options: (a) per-worker instances over disjoint shards
   (message-passing), or (b) a shared arena for the hot path (vector data) with
   the kernel pattern above, engine state staying per-instance. (b) is the one
   this slice de-risks.
3. **Benchmark gate:** extend `benchmarks/wasm_threads_bench.mjs` with the real
   search path; browser validation requires COOP/COEP headers (the repo's
   static server / demo must set them).

## Sources (all consulted 2026-10-06)

- MDN, SharedArrayBuffer — security requirements (page edited 2026-02-10):
  https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer
- Rust tracking issue #77839 (wasm atomics, open; `-Zbuild-std` required):
  https://github.com/rust-lang/rust/issues/77839
- rustc book, `wasm32-unknown-unknown`:
  https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html
- wasm-bindgen-rayon (fork) README — build recipe + COOP/COEP:
  https://github.com/RReverser/wasm-bindgen-rayon
- wasm-bindgen threads transform (per-thread stacks + TLS):
  https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/cli-support/src/transforms/threads
- Kuzu-Wasm docs (dual build; multithreaded variant requires cross-origin
  isolation; edited 2025-10-10): https://kuzudb.github.io/docs/client-apis/wasm/
- DuckDB-WASM README ("default mode is single threaded"):
  https://github.com/duckdb/duckdb-wasm
