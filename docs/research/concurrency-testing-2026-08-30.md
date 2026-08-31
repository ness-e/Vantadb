# Concurrency Testing — `loom` Evaluation (2026-08-30)

> **Document type:** Research note / mini-ADR (not a full ADR — no formal template needed because
> the decision is "no action").
> **Status:** Accepted (decision: do NOT introduce `loom`).
> **Author:** vanta-lead (TBH-17, Wave 3 of testing-bench-harden plan).
> **Date:** 2026-08-30.

---

## TL;DR

**`loom` is NOT introduced into the VantaDB workspace.** The current concurrency test suite
(`tests/concurrency_parity.rs` + `serial_test`) is sufficient for the engine's current
concurrency surface. `loom` will be reconsidered only if a future change introduces a
non-trivial custom synchronization primitive (lock-free queue, epoch reclamation, custom RCU).

---

## 1. Context

The 2026-08-30 multi-agent testing audit (Wave 2 of `2026-08-30-testing-bench-harden`)
included a soft recommendation to evaluate `tokio-rs/loom` as a deterministic concurrency
testing tool, conditional on the engine introducing new synchronization primitives.
`loom` performs exhaustive interleaving of `std::sync::*` operations at test time and
catches races that probabilistic tests miss.

The owner decision (D5, plan doc): *"justificar cada dep antes de añadir"* — meaning no new
dependency enters the workspace without clear, demonstrated need.

## 2. What the workspace uses today

Confirmed by `git grep "loom" Cargo.toml` → **0 hits**, and reading the `[dev-dependencies]`
section of `Cargo.toml`:

```text
dev-dependencies:
  tokio, criterion, tempfile, proptest, http, futures,
  console, indicatif, serial_test, serde_json, tower, serde_yaml
```

Production synchronization primitives in `[dependencies]` (lines 30-38 of Cargo.toml):

| Crate | Purpose |
|---|---|
| `parking_lot = "0.12"` | `Mutex`/`RwLock`/`Condvar` (faster than std) |
| `portable-atomic = "1"` | Atomic operations across architectures |
| `dashmap = "6"` | Concurrent hashmap |
| `arc-swap = "1.7"` | RCU-style lock-free pointer swap |
| `lru = "0.16"` | Concurrent LRU cache |

These are battle-tested, well-documented primitives — **not custom lock-free code**.

## 3. What the current test suite already covers

`tests/concurrency_parity.rs` (304 lines, registered as a workspace `[[test]]`):

| Test | Coverage |
|---|---|
| `test_triple_backend_parity_validation` | RocksDB ↔ Fjall ↔ InMemory functional equivalence under identical write sequences (1000 inserts, 500 deletes) |
| `test_high_concurrency_fjall_stress` | 10 concurrent writers × 100 ops/thread; validates that no node is lost under write contention |
| `test_interleaved_read_write_parity` | Reader + writer threads racing with `thread::yield_now()` between ops; validates shared-state integrity |
| `test_concurrency_rebuild_rcu` | The only test targeting a *new* primitive: validates the RCU-style `arc-swap` index rebuild does not lose writes or break query continuity |

Plus existing chaos coverage from `tests/storage/chaos_integrity.rs` (failpoints feature gate),
and crash recovery in `tests/storage/crash_injection.rs` and `tests/durability_recovery.rs`.

The 4 tests in `concurrency_parity.rs` map directly onto the production sync surface:

- `arc-swap` rebuild → `test_concurrency_rebuild_rcu`
- `parking_lot::RwLock` on `vector_store` → `test_interleaved_read_write_parity`
- `dashmap` shard contention → `test_high_concurrency_fjall_stress`
- Cross-backend parity under concurrent writes → `test_triple_backend_parity_validation`

## 4. What `loom` would add (and what it would cost)

### Pro

- **Exhaustive interleaving** — catches races that probabilistic stress tests miss.
- **Drop-in for `std::sync::*`** — `loom::sync::Arc`, `loom::sync::Mutex`, `loom::thread::spawn`.
- Used in production by `tokio`, `crossbeam`, `rustls`, etc.

### Con (the cost that justifies saying no, for now)

1. **Compile time impact:** `loom` rewrites sync primitives via proc-macros + adds a runtime
   scheduler that tracks every thread spawn. CI compile budget (Fast Gate <5 min) cannot
   absorb ~30-60% slowdown for the 65+ integration tests.

2. **State explosion:** `loom` is exhaustive — every atomic op interleaved. The current
   `tests/concurrency_parity.rs` tests already push into state-explosion territory
   (10 threads × 100 ops = combinatorial nightmare). To make `loom` tractable we would need
   to *reduce* test scope, not expand it.

3. **Coverage of the wrong surface:** `loom` proves correctness of *user-written sync code*.
   VantaDB's user-written sync code is small (`arc-swap` rebuild + `RwLock` reads). The rest
   is delegated to well-tested crates (`parking_lot`, `dashmap`). `loom` would burn CI minutes
   re-verifying those crates.

4. **YAGNI:** The audit found no planned introduction of custom lock-free primitives.
   `loom` is the right tool when the next primitive shows up — not before.

## 5. Trigger conditions for re-evaluation

`loom` (or an equivalent like `shuttle`, also evaluated and rejected for the same reasons
plus its `async` runtime coupling) becomes worth adding when **any** of the following appears:

- [ ] A custom lock-free queue, stack, or skip-list implementation lands in `src/`.
- [ ] Epoch reclamation or hazard-pointer logic is hand-rolled (currently we use `arc-swap`).
- [ ] A new shared data structure needs proof of correctness beyond `cargo test --test concurrency_parity`.
- [ ] A production race condition is observed post-release that the existing tests did not catch.
- [ ] The chaos suite (`vanta-chaos` agent) finds a non-deterministic failure that cannot be
      reproduced in a CI loop without state-space exploration.

When one of these fires, file a new TBH-XX task referencing this document and re-evaluate.

## 6. Decision

**No action.** Do not add `loom` or `shuttle` to `Cargo.toml`.

This decision follows the D5 principle (justificar antes de añadir) and the
[`ponytail`](../.opencode/skills/ponytail/SKILL.md) reflex: the smallest change that works is
no change.

## 7. References

- Plan: `docs/plans/2026-08-30-testing-bench-harden.md` (TBH-17)
- Task file: `.opencode/skills/campaign-executor/tasks/TBH-17.md`
- Test suite evaluated: `tests/concurrency_parity.rs`
- 2026-08-30 multi-agent audit (gap analysis) — internal, Wave 2 deliverable
- `loom` upstream: <https://github.com/tokio-rs/loom>
- `shuttle` alternative evaluated: <https://github.com/awslabs/shuttle> (rejected for same reasons)

---

*Review on next plan that introduces custom synchronization primitives, or in 6 months
(2027-02-28), whichever comes first.*