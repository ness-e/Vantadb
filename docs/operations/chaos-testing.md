# Chaos Testing in VantaDB

## Overview

Chaos testing validates that VantaDB survives **injected I/O failures** without
losing or corrupting data. The system uses the [`fail`] crate to inject
controlled failures at strategic persistence points.

[`fail`]: https://docs.rs/fail/latest/fail/

**Principle**: Every failpoint simulates a transient I/O error. The engine must
always recover when the failpoint is removed — no corruption, no panics, no
silent data loss.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     ChaosTestHarness                         │
│  ┌──────────────┐  ┌──────────────────┐  ┌───────────────┐  │
│  │   TempDir     │  │  StorageEngine   │  │  failpoints   │  │
│  │  (backing     │  │  (Arc-wrapped)   │  │  (RefCell<    │  │
│  │   storage)    │  │                  │  │   Vec<String>>)│  │
│  └──────────────┘  └──────────────────┘  └───────────────┘  │
│         │                  │                      │         │
│         │            fail::cfg()               track(+)     │
│         │            fail::remove()            track(-)     │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                  Injection Points                            │
│                                                              │
│  wal_append_fail       ─── src/wal.rs (append, batch_append)│
│  storage_insert_fail   ─── src/storage/engine/ops.rs        │
│  mmap_flush_fail       ─── src/storage/vfile.rs             │
│  hnsw_serialize_fail   ─── src/index/serialize.rs           │
│  edge_write_fail       ─── src/edge_index.rs                │
│  snapshot_serialize_fail ── src/storage/engine/maintenance.rs│
└─────────────────────────────────────────────────────────────┘
```

## Available Failpoints

| Failpoint               | Location                          | What it simulates              |
|-------------------------|-----------------------------------|--------------------------------|
| `wal_append_fail`       | `src/wal.rs` (append+batch)       | WAL write I/O failure          |
| `storage_insert_fail`   | `src/storage/engine/ops.rs`       | Storage backend insert failure |
| `mmap_flush_fail`       | `src/storage/vfile.rs`            | Memory-mapped file flush fail  |
| `hnsw_serialize_fail`   | `src/index/serialize.rs`          | HNSW index serialization fail  |
| `edge_write_fail`       | `src/edge_index.rs`               | Edge index insert failure      |
| `snapshot_serialize_fail`| `src/storage/engine/maintenance.rs` | Snapshot persist failure     |

---

## How to Add a New Failpoint

### Step 1: Insert the failpoint in the source

```rust
// In the function where you want to inject failure:
#[cfg(feature = "failpoints")]
{
    fail::fail_point!("my_new_failpoint", |_| {
        Err(VantaError::IoError(std::io::Error::other(
            "Simulated failure description",
        )))
    });
}
```

For functions that return `Result<()>`, the closure returns the error.
For functions returning `()`, omit the closure — the failpoint returns
`()` early:

```rust
#[cfg(feature = "failpoints")]
fail::fail_point!("my_new_failpoint");
```

### Step 2: Enable the failpoint in tests

```rust
use vantadb::testing::chaos::ChaosTestHarness;

let chaos = ChaosTestHarness::new().unwrap();
chaos.enable("my_new_failpoint", "return");

// ... operation should fail ...

chaos.disable("my_new_failpoint");

// ... operation should succeed after removal ...
chaos.assert_recovery();
chaos.destroy();
```

### Step 3: Update this doc

Add the new failpoint to the table above.

---

## How to Run Chaos Tests

### Prerequisites

```bash
rustup update stable
cargo install cargo-nextest --locked
```

### Run all chaos tests

```bash
# Using the dedicated chaos nextest profile:
cargo nextest run --profile chaos --features failpoints -p vantadb

# Or directly:
cargo nextest run --features failpoints -p vantadb -- chaos_integrity
```

### Run a specific scenario

```bash
cargo test --features failpoints -p vantadb -- chaos_integrity_failpoints_certification --nocapture
```

### CI

Chaos tests run automatically via `.github/workflows/chaos-45.yml` on pushes
and PRs touching `src/`, `tests/`, or the failpoint infrastructure.

---

## Expected Behavior

1. **Failpoint active** → the affected operation returns `Err(...)`.
   The engine never panics or enters an undefined state.

2. **Failpoint removed** → subsequent operations succeed normally.
   The engine state is consistent — no partial writes, no stale data.

3. **Concurrent failpoints** → each failpoint acts independently.
   The `ChaosTestHarness` disables all tracked failpoints on drop.

4. **Recovery invariant**: after `chaos.assert_recovery()`:
   - A sentinel node (`id = u128::MAX`) can be inserted and read back.
   - The engine serves reads without error.

---

## Harness API

```rust
// ─── Setup ──────────────────────────────────────────────────────
ChaosTestHarness::new() -> Result<Self>

// ─── Failpoint control ──────────────────────────────────────────
harness.enable(name: &str, action: &str)    // cfg + track
harness.disable(name: &str)                 // remove + untrack
harness.disable_all()                       // remove all tracked

// ─── Verifications ──────────────────────────────────────────────
harness.assert_recovery()                   // insert+read sentinel

// ─── Cleanup ────────────────────────────────────────────────────
harness.destroy()                           // disable_all + drop
// Also called automatically in Drop.
```

### Fields

| Field    | Type                  | Description                    |
|----------|-----------------------|--------------------------------|
| `dir`    | `TempDir`             | Temporary backing directory    |
| `engine` | `Arc<StorageEngine>`  | Shared storage engine under test |
