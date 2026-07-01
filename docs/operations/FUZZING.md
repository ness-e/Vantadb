---
title: Fuzzing Guide for VantaDB
type: operations
status: active
tags: [vantadb, operations, testing, fuzzing]
last_reviewed: 2026-07-01
---

# Fuzzing Guide for VantaDB

## Validation Strategy

VantaDB uses a dual fuzzing approach to maximize coverage and compatibility:

| Method | Platform | Purpose | Execution |
|--------|-----------|-----------|----------|
| **`proptest`** | ✅ Windows, Linux, macOS | Cross-platform validation during development | `cargo test fuzz_proptest` |
| **`cargo-fuzz`** | 🐧 Linux/macOS only | Intensive fuzzing with sanitizers (CI) | `cd fuzz && cargo +nightly fuzz run <target>` |

---

## Windows Validation (Proptest)

Run property-based tests that generate random data to validate robustness:

```powershell
# Run fuzzing tests
cargo test fuzz_proptest

# Run with more cases (default 256)
PROPTEST_CASES=1000 cargo test fuzz_proptest
```

**What it validates:**
- ✅ Safe deserialization of `WalRecord` and `UnifiedNode` against corrupt bytes
- ✅ Correct roundtrip with random payloads and vectors
- ✅ Zero panics in critical parsing code

**Acceptance criteria:**
- All `fuzz_proptest` tests must pass
- No panics or crashes during deserialization

---

## Advanced Fuzzing (Linux/macOS + Nightly)

Requires `cargo-fuzz` and `rustc nightly`. Use in CI or WSL2:

### Prerequisites
```bash
rustup install nightly
cargo install cargo-fuzz
```

### Execution
```bash
cd fuzz/

# Deserialization fuzzing (WAL + Nodes)
cargo +nightly fuzz run fuzz_node_deserialize -- -max_total_time=300

# LISP/query parser fuzzing
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

### Reproducing Crashes
If `cargo-fuzz` finds a crash, the case is saved in `fuzz/artifacts/`:
```bash
cargo +nightly fuzz run fuzz_node_deserialize fuzz/artifacts/fuzz_node_deserialize/crash-<hash>
```

### CI Integration (GitHub Actions)
```yaml
# .github/workflows/fuzz.yml (example for Linux)
name: Fuzzing
on: [push, pull_request]

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: rustup install nightly
      - run: cargo install cargo-fuzz
      - run: cd fuzz && cargo +nightly fuzz run fuzz_node_deserialize -- -max_total_time=60
```

---

## CI Acceptance Criteria

- [ ] `proptest` passes on all platforms (Windows/Linux/macOS)
- [ ] `cargo-fuzz` runs for 300s without crashes on Linux (no-panic)
- [ ] 0 memory leaks detected by LSan/ASan (Linux only)
- [ ] Any crash found is documented in `docs/architecture/audits/fuzz-crashes.md`

---

## Technical Notes

### Why `cargo-fuzz` Does Not Work on Native Windows
- `cargo-fuzz` depends on `libFuzzer`, which requires sanitizers (ASan/LSan)
- LLVM sanitizers are only officially supported on Linux/macOS
- On Windows, use `proptest` for logic validation and reserve `cargo-fuzz` for Linux CI

### Workspace Configuration
The `fuzz/` directory is excluded from the workspace in `Cargo.toml`:
```toml
[workspace]
exclude = ["fuzz"]  # Prevents conflict with cargo-fuzz on Windows
```

This allows:
- Running `cargo test` on Windows without workspace errors
- Running `cargo +nightly fuzz run` on Linux without interference

---

## Quality Metrics

| Metric | Target | Tool |
|---------|----------|-------------|
| **Fuzzing coverage** | >1M inputs without crash | `cargo-fuzz` (Linux) |
| **Regression-free duration** | 300s continuous | `cargo-fuzz -- -max_total_time=300` |
| **Property tests** | 256+ cases per test | `proptest` (default) |
| **Deserialization panics** | 0 | Both methods |

---

> **Note for developers**: If you add new critical serializable types (e.g., new WAL records, index structures), consider adding a corresponding `proptest` test in this file to maintain fuzzing coverage.
