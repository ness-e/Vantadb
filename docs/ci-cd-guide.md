# VantaDB CI/CD Guide

## Overview

VantaDB uses GitHub Actions for continuous integration and delivery. The CI/CD pipeline is organized into 12 workflows, each with a clear category and purpose.

### File Naming Convention

All workflow files follow: `<category>-<name>-<number>.yml`

| Number range | Category | Description | Bloqueante |
|---|---|---|---|
| 10-19 | **CI** | Build, lint, tests | Sí |
| 20-29 | **GATE** | Validation gates (docs) | Sí |
| 30-39 | **SEC** | Security scanning | Sí |
| 40-49 | **PERF** | Performance benchmarks | No |
| 50-59 | **HEAVY** | Long-running certification | No (schedule) |
| 60-69 | **RELEASE** | Publish to registries | No (tag-only) |

Numbering is sequential within each category (10, 11, 12...).

### Workflow Standard Structure

Every workflow follows this structure:

```yaml
name: '<CATEGORY>: <Name> — <Description>'

on:
  push: ...
  pull_request: ...
  workflow_dispatch: ...

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read    # minimum required, override only when needed

env:               # set only when needed
  ...

jobs:
  job-name:        # kebab-case
    name: Job Name # human-readable
    runs-on: ubuntu-latest
    timeout-minutes: N
    ...
```

---

## Workflow Reference

### CI (10-)

#### `ci-rust-10.yml` — CI: Rust — Build & Lint + Tests

**Trigger**: Push/PR to main (Rust code paths)

**Jobs** (all run in parallel):

| Job | Tool | Timeout | Purpose |
|-----|------|---------|---------|
| `fmt` | `cargo fmt --check` | 10min | Code formatting |
| `clippy` | `cargo clippy` | 15min | Lint checks |
| `test` | `cargo nextest --profile audit` | 30min | Full test suite (Linux) |
| `test-windows` | `cargo nextest --profile ci-windows` | 30min | Windows smoke test |
| `coverage` | `cargo llvm-cov nextest` | 30min | Code coverage |
| `audit` | `cargo audit` | 5min | Security advisory check |
| `deny` | `cargo deny check` | 5min | License & policy check |

#### `ci-web-11.yml` — CI: Web — Build & Test

**Trigger**: Push/PR to main (web/ paths)

**Jobs**: Build, lint, typecheck, unit tests, e2e (Playwright)

### GATE (20-)

#### `gate-docs-21.yml` — GATE: Docs — Lint & Frontmatter

**Trigger**: Push/PR to main (docs/ paths)

**Jobs**: Markdown linting, frontmatter validation

### SEC (30-)

#### `sec-codeql-30.yml` — SEC: CodeQL — Analysis

**Trigger**: Push/PR + weekly schedule

**Jobs**: CodeQL static analysis for Rust

### PERF (40-)

#### `perf-bench-40.yml` — PERF: Benchmarks — Python Integration

**Trigger**: Push to main (Rust/Python paths), manual dispatch

**Jobs**: Builds Python wheel, runs ingestion/search benchmarks

### HEAVY (50-)

#### `heavy-certification-50.yml` — HEAVY: Certification — All Tests

**Trigger**: Weekly schedule (Sunday 3AM), manual dispatch

**Jobs**: 10 parallel stress/certification test suites

#### `heavy-bench-nightly-51.yml` — HEAVY: Benchmarks — Nightly Regression

**Trigger**: Nightly schedule (3AM), manual dispatch

**Jobs**: Criterion benchmarks with regression detection + GitHub Issues

### RELEASE (60-)

#### `release-wheels-60.yml` — RELEASE: Wheels — Build & Publish

**Trigger**: Tag `v*`, PR (wheel build), manual dispatch

**Jobs**: Build matrix (3 OS) + TestPyPI/PyPI publish + verify

#### `release-npm-61.yml` — RELEASE: NPM — Publish

**Trigger**: Tag `wasm-v*`/`ts-v*`, manual dispatch

**Jobs**: Publish wasm + TypeScript packages to npm

#### `release-adapters-62.yml` — RELEASE: Adapters — PyPI Publish

**Trigger**: Tag `adapters-v*`, manual dispatch

**Jobs**: Test, build, publish LangChain/LlamaIndex adapters to PyPI

#### `release-binaries-63.yml` — RELEASE: Binaries — Build & Upload

**Trigger**: GitHub Release published

**Jobs**: Build + upload native binaries (Linux x86_64, macOS x86_64/ARM)

#### `release-sbom-64.yml` — RELEASE: SBOM — Generate

**Trigger**: Tag `v*`, manual dispatch

**Jobs**: Generate CycloneDX SBOM, upload as artifact

---

## Composite Action: `rust-setup`

Location: `.github/actions/rust-setup/action.yml`

Provides common Rust setup for all workflows. Accepts inputs:

| Input | Default | Description |
|-------|---------|-------------|
| `toolchain` | `stable` | Rust toolchain channel |
| `components` | `''` | Rustup components (comma-separated) |
| `swap-mb` | `0` | Swap file size in MB (0 to skip) |
| `free-disk` | `false` | Free disk space |
| `cache-bin` | `false` | Cache cargo binaries |
| `install-nextest` | `false` | Install cargo-nextest |
| `install-llvm-cov` | `false` | Install cargo-llvm-cov |
| `install-system-deps` | `true` | Install system libraries |

Usage:
```yaml
- uses: ./.github/actions/rust-setup
  with:
    swap-mb: 4096
    free-disk: true
    install-nextest: true
    install-llvm-cov: false
```

---

## Performance Expectations

| Metric | Before | After |
|--------|--------|-------|
| CI wall time (push) | ~30 min | ~15-20 min |
| CI wall time (windows) | ~20 min | ~20 min (parallel) |
| CI wall time (total) | ~30 min | ~20 min (max job) |
| Runner-minutes (push) | ~30 min | ~100 min (7 jobs) |

The trade-off: faster feedback (2x) for more runner-minute consumption.

---

## Security Notes

- All workflows use `contents: read` by default (minimum permission)
- Release workflows override permissions only when needed (`id-token: write`, `contents: write`)
- Pinned commit SHAs are used for all third-party actions
- `cancel-in-progress` is disabled for tag-triggered release workflows

---

## Future Improvements

- **Merge Queue**: When team grows beyond 1 developer, enable merge queue for `main` with `merge_group` trigger on CI workflows
- **Dependabot**: Configure `dependabot.yml` to auto-update pinned action SHAs
- **Windows coverage**: Expand `release-binaries` matrix to include Windows targets
- **Workflow Telemetry**: Add GitHub Actions usage metrics collection
