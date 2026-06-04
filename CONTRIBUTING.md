# Contributing to VantaDB

Thank you for your interest in contributing! This guide covers the development
workflow, testing requirements, and specialized tooling like fuzzing.

---

## Development Prerequisites

- **Rust stable** (see `rust-toolchain.toml`)
- **cargo-nextest**: `cargo install cargo-nextest`
- **Python 3.8+** with `venv` support

### Python SDK (hermetic audit venv)

Local Python work must use `target/audit-venv` so tests never pick up a stale global `vantadb-py` install:

```powershell
# Windows — create venv and install bindings in develop mode
powershell -ExecutionPolicy Bypass -File dev-tools/setup_venv.ps1

# Run SDK tests
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -v
```

```bash
# Unix/macOS — equivalent
./dev-tools/setup_venv.sh
target/audit-venv/bin/python -m pytest vantadb-python/tests/test_sdk.py -v
```

---

## Running Tests

```bash
# Full test suite (audit profile — used for CI and release validation)
# On Windows, limit build jobs to avoid MSVC stack overflows during test linking:
cargo nextest run --profile audit --workspace --build-jobs 2

# Experimental tests (parser, executor). Pass features on the CLI:
cargo nextest run --profile experimental --workspace --features experimental-lisp,experimental-governance --build-jobs 2
```

---

## Code Quality

All PRs must pass:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --profile audit --workspace
```

---

## Fuzzing

VantaDB uses [`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html)
for resilience testing. Fuzzing requires `cargo-fuzz`, a nightly toolchain, and AddressSanitizer support.

- **Rust nightly**: `rustup toolchain install nightly`
- **cargo-fuzz**: `cargo install cargo-fuzz`

> **Note on OS Support**: Our CI runs fuzzing exclusively on Linux where AddressSanitizer support is most stable. Windows support for `cargo-fuzz` is strictly best-effort and may require specific MSVC AddressSanitizer setups.

### Available Targets

| Target                 | Description                                              |
|------------------------|----------------------------------------------------------|
| `fuzz_parser`          | LISP expression parser, query parser, statement parser   |
| `fuzz_node_deserialize`| `UnifiedNode` and `WalRecord` bincode deserialization    |

### Running a Fuzz Target

```bash
# Navigate to the fuzz crate (it's excluded from the workspace on purpose)
cd fuzz

# Run the parser fuzzer for 5 minutes
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300

# Run the node deserializer fuzzer
cargo +nightly fuzz run fuzz_node_deserialize -- -max_total_time=300
```

### Reproducing a Crash

If fuzzing finds a crash, a corpus artifact is saved under
`fuzz/artifacts/<target>/`. To reproduce it:

```bash
cargo +nightly fuzz run fuzz_parser fuzz/artifacts/fuzz_parser/crash-<hash>
```

### Crash Triage

When a crash artifact is produced:

1. **Reproduce**: Run the command above to confirm the panic and get a backtrace.
2. **Isolate**: Extract the raw bytes or text from the artifact.
3. **Regression Test**: Create a deterministic unit test in `tests/` or inside the relevant module with the exact crashing input.
4. **Fix**: Patch the code until the new unit test passes cleanly.

### CI Integration

Fuzzing runs as a scheduled job in `.github/workflows/heavy_certification.yml`
on Linux runners only. It is **not** part of standard PR validation because it
requires nightly and long wall-clock time.

---

## Workspace Structure

```text
vantadb/          ← core library crate (src/)
vantadb-python/   ← PyO3 Python SDK
fuzz/             ← cargo-fuzz targets (Linux nightly only, excluded from workspace)
benches/          ← Criterion benchmarks
tests/            ← integration test suite
dev-tools/        ← validation scripts
docs/             ← project documentation
```

---

## Release Checklist

1. `cargo fmt --check` — zero formatting issues
2. `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
3. `cargo nextest run --profile audit --workspace` — all tests pass
4. `dev-tools/setup_venv.ps1` then `dev-tools/scripts/validate_python_sdk.ps1` (Windows) or `validate_python_sdk.sh` (Linux/macOS)
5. Update `CHANGELOG.md` and bump version in `Cargo.toml`
