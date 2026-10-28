# Contributing to VantaDB

Thank you for contributing to VantaDB! This guide outlines the development setup, testing standards, commit conventions, and pull request workflow.

## Prerequisites

- **Rust stable** (pinned in `rust-toolchain.toml`)
- **cargo-nextest**: `cargo install cargo-nextest`
- **Python 3.11+** with `venv` support (required for Python SDK bindings)

### Python SDK Setup

Python development uses an isolated hermetic environment under `target/audit-venv`:

```bash
# Windows
powershell -ExecutionPolicy Bypass -File dev-tools/setup_venv.ps1
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -v

# Unix / macOS
./dev-tools/setup_venv.sh
target/audit-venv/bin/python -m pytest vantadb-python/tests/test_sdk.py -v
```

## Building & Running Tests

All contributions must pass the verification gate before submission. You can run all checks using `just verify` or execute individual commands:

```bash
# Run test suite with the audit profile
cargo nextest run --profile audit --workspace --build-jobs 2

# Code formatting check
cargo fmt --check

# Linter checks across all features and targets
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Dependency security and license checks
cargo deny check
```

## Commit Convention

VantaDB follows **Conventional Commits**. Commit messages drive automated semantic versioning and changelog generation via `release-plz`:

| Prefix | Semver Bump | Description | Example |
|--------|-------------|-------------|---------|
| `feat:` | minor | New feature | `feat: add cosine distance metric` |
| `fix:` | patch | Bug fix | `fix: handle empty slice in buffer` |
| `docs:` | patch | Documentation changes | `docs: update quickstart guide` |
| `test:` | patch | Adding or modifying tests | `test: add boundary check for hnsw` |
| `perf:` | patch | Performance improvements | `perf: avoid allocation in search path` |
| `refactor:` | patch | Code refactoring without behavior changes | `refactor: extract graph builder` |
| `chore:` | none | Maintenance tasks, dependencies | `chore: update dependencies` |

Use `feat!:` or `BREAKING CHANGE:` in the commit body to denote breaking changes.

## Pull Request Workflow

1. **Branching model**:
   - `develop`: Integration branch where all feature and bugfix PRs must be targeted.
   - `main`: Production release branch. Releases are automated.
2. **Automated releases**:
   - Do **not** manually edit versions in `Cargo.toml`.
   - Do **not** edit `docs/CHANGELOG.md` or create release git tags manually.
   - When PRs merge into `main` from `develop`, `release-plz` automatically calculates the version bump, updates the changelog, and opens release PRs.
3. **Pre-flight checks**:
   - Ensure `just verify` passes cleanly with no warnings or formatting diffs.
   - Keep pull requests focused on a single concern.

## Fuzzing

Resilience testing uses `cargo-fuzz` under the `fuzz/` directory (requires Rust nightly):

```bash
cd fuzz
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
cargo +nightly fuzz run fuzz_node_deserialize -- -max_total_time=300
```

Available targets include `fuzz_parser`, `fuzz_node_deserialize`, `fuzz_wal`, and `fuzz_archive`. When triaging crashes, isolate the reproducer into a regression test in `tests/`.
