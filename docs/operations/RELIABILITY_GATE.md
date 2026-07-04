---
title: VantaDB Reliability Gate & Certification Policy
type: operations
status: active
tags: [vantadb, operations, reliability]
last_reviewed: 2026-07-01
aliases: []
---

# VantaDB Reliability Gate & Certification Policy

This document consolidates the operational certification policy, acceptance thresholds (gating), and practical procedures for validating VantaDB's resilience under memory stress, catastrophic fault injection, and data corruption.

---

## Section 1: RSS Stability Gate (Confined Heap Memory)

Establishes the operational procedure for validating VantaDB's physical memory (RSS) stability under sustained loads and certifying that the global `mimalloc` allocator mitigates long-term heap fragmentation (Acceptance Criterion ST2.2.2).

### 1.1 Purpose

The quick CI test validates the absence of catastrophic hot leaks. However, to formally certify long-term fragmentation mitigation under dynamic storage engine cycles, a continuous **30-minute** run is required.

### 1.2 Prerequisites

Build the executable or library with the `custom-allocator` feature enabled to activate `mimalloc`:

```powershell
cargo build --release --features custom-allocator
```

### 1.3 Manual Execution Procedure

The script runs a continuous loop of bulk insertions and reads through the Python SDK interface, evaluating Resident Set Size (RSS) drift over time.

#### Test Code (Save as `tests/stress_rss_30m.py`)

```python
import time
import os
import psutil
import vantadb_py as vanta

def run_stress_test(duration_minutes=30):
    print(f"Starting RSS stress test for {duration_minutes} minutes...")
    db_path = "./temp_stress_db"
    if not os.path.exists(db_path):
        os.makedirs(db_path)
        
    db = vanta.VantaDB(db_path)
    process = psutil.Process(os.getpid())
    rss_initial = process.memory_info().rss
    print(f"Initial RSS: {rss_initial / 1024 / 1024:.2f} MB")
    
    start_time = time.time()
    end_time = start_time + (duration_minutes * 60)
    count = 0
    while time.time() < end_time:
        for i in range(1000):
            vector = [1.0] * 128
            db.insert(count, f"content_{count}", vector)
            count += 1
            
        db.flush()
        profile = db.hardware_profile()
        current_rss = profile["process_rss_bytes"]
        elapsed = (time.time() - start_time) / 60
        print(f"[{elapsed:.2f} min] Current RSS: {current_rss / 1024 / 1024:.2f} MB | Nodes: {count}")
        time.sleep(1)
        
    rss_final = process.memory_info().rss
    drift = (rss_final / rss_initial) - 1.0
    print("\n" + "="*40)
    print("TEST COMPLETED")
    print(f"Initial RSS: {rss_initial / 1024 / 1024:.2f} MB")
    print(f"Final RSS: {rss_final / 1024 / 1024:.2f} MB")
    print(f"Residual RSS Drift: {drift * 100:.2f}%")
    print("="*40)
    
    if drift < 0.10:
        print("✅ Certification Successful: Residual RSS growth below 10%.")
    else:
        print("❌ Certification Failed: Excessive leak or fragmentation detected.")

if __name__ == "__main__":
    run_stress_test(30)
```

### 1.4 Acceptance Thresholds (Gating)

1. **RSS Drift < 10%** measured between memory stabilization at minute 5 (after warm-up) and the end at minute 30.
2. **Fragmentation Coherence**: Logical [[hnsw|HNSW]] memory (`hnsw_logical_bytes`) and mapped physical resident memory (`mmap_resident_bytes`) must reflect RAM stability, without exponential growth or divergent global process RSS.

---

## Section 2: Chaos Integrity Gate (Fault-Injection and Recovery)

Ensures VantaDB is tolerant to catastrophic disk I/O, memory, and index serialization faults injected at critical points, guaranteeing 100% corruption-free self-recovery without loss of ACID consistency.

### 2.1 Instrumented Chaos Points (Failpoints)

VantaDB instruments controlled discrete error injections via the `failpoints` feature flag:

| Failpoint Name | Code Location | Simulated Behavior |
| :--- | :--- | :--- |
| `wal_append_fail` | `src/wal.rs` | Error when writing records to the Write-Ahead Log. |
| `storage_insert_fail` | `src/storage.rs` | Catastrophic I/O error when inserting into storage structures. |
| `mmap_flush_fail` | `src/storage.rs` | Error when syncing to disk (`msync` / `flush`) the memory-mapped file. |
| `hnsw_serialize_fail` | `src/index.rs` | I/O error when serializing and persisting the HNSW vector index to disk. |

### 2.2 Verification Procedures

#### A. Quick CI Verification (Nextest)

For rapid chaos testing in CI or pre-push environments:

```powershell
cargo nextest run --profile chaos --features failpoints
```

#### B. Manual Chaos Loop Certification (Sustained Resilience)

To validate the absence of leaks, deadlocks, or residual corruption during repetitive error injection runs, execute the chaos loop script:

```powershell
.\dev-tools\chaos_loop.ps1 -Iterations 1000 -Release
```

### 2.3 Acceptance Thresholds (Gating)

1. **Success Rate: 100.00%** over 1,000 iterations (zero unhandled failures).
2. **Self-Recovery Guarantee**: Any operation returning `Err` during fault injection must execute successfully (`Ok`) immediately after deactivating the failpoint.
3. **Transactional Consistency**: The engine state must be readable and correct, recovering all committed data prior to the fault.

---

## Section 3: WAL Durability and Cold-Start Recovery

Validates that upon a forced process termination (abrupt crash), data committed to the WAL is not lost and the engine automatically rebuilds to its last known consistent state.

### 3.1 Validation Suite

WAL durability resilience and cold-start recovery are verified through dedicated suites:

- **`tests/storage/wal_resilience.rs`**: Certifies WAL parser integrity, CRC32C checksums, and recovery from truncated fragments.
- **`tests/durability_recovery.rs`**: Certifies cold reconstruction of the vector and relational index from WAL records after simulated process crashes.

### 3.2 Execution Commands

For manual certification:

```powershell
cargo test --test wal_resilience --release
cargo test --test durability_recovery --release
```

### 3.3 Acceptance Thresholds (Gating)

1. **Zero consistency leaks**: All nodes inserted and committed to the WAL before the abrupt shutdown must be recoverable via `get()` after cold restart (`StorageEngine::open`).
2. **Checksum Integrity**: Any physical corruption introduced in the WAL file must be detected at the page/record level via CRC32C, raising read errors or applying auto-repair up to the last consistent point in the log.
