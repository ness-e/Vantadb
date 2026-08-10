---
title: "VantaDB CI & Certification Policy"
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-08-10
aliases: []
---

# VantaDB CI & Certification Policy

To maintain a rapid development iteration cycle and guarantee mathematical precision in our HNSW
engine, VantaDB enforces a split Continuous Integration architecture.

## CI Workflow Inventory

VantaDB has **14 active workflow files** in `.github/workflows/` (numbered by layer for dependency
ordering). Each workflow is documented below.

### Local Verification Scripts — Rutas Canónicas

The CI/Hooks integration table in `.opencode/AGENTS.md` lists the local verification scripts.
Their canonical paths are:

| Script | Assertion scope |
|--------|-----------------|
| `dev-tools/verify_changed.ps1` | Quick verify (~30s): fmt → check → clippy on `vantadb` core. Runs the docs-coverage gate only when `git diff --name-only HEAD` touches `src/`, bindings (`vantadb-python`, `vantadb-ts`, `vantadb-wasm`) or `docs/api/`; otherwise silently skips. |
| `dev-tools/verify.ps1` | Full pre-flight (~2–5 min): fmt → check → clippy → audit → deny → tests → coverage. Runs the docs-coverage gate whenever the script exists. |
| `scripts/validate-docs-coverage.ps1` | Docs coverage gate (Regla 3, mecánica): valida símbolos públicos SDK/config/error/CLI/Python/MCP contra `docs/api/*`. `-ReportOnly` imprime gaps sin fallar; sin el flag, los métodos sin documentar devuelven exit 1 y fallan el script host. |

`scripts/validate-docs-coverage.ps1` is the **single shared docs gate**, referenced from both
`dev-tools/verify_changed.ps1` and `dev-tools/verify.ps1`. The two verify scripts form a
hierarchy (quick → full), not alternative locations for the same gate — this is the canonical
map so the AGENTS.md CI/Hooks table and this policy reconcile at the next docs sync:
Regla 3 ("docs al día") is enforced mechanically by the docs-coverage gate, not by convention.

### 1. Fast Gate (`ci-rust-10.yml`)

The fast gate is triggered automatically on every pull request and push to the `main` branch.
**Goal:** Deliver PR feedback in under 5 minutes.

The fast gate validates the production-facing MVP boundary only: embedded core behavior, stable
SDK/CLI flows, durability, namespace and metadata indexes, vector retrieval, BM25, Hybrid Retrieval
v1, rebuild/audit, and local deterministic integration tests. Historical or experimental surfaces
such as IQL/LISP/DQL, MCP, LLM/Ollama integration, graph traversal beyond stored local edges, and
governance semantics are excluded from the default fast lane.

**Jobs:**

| Job | Description |
|-----|-------------|
| `fmt` | Format Check — `cargo fmt --check` |
| `clippy` | Clippy Lints — `cargo clippy -- -D warnings` |
| `test` | Tests (Linux) — nextest audit profile |
| `test-windows` | Tests (Windows) — nextest ci-windows profile |
| `test-macos` | Tests (macOS) — nextest audit profile |
| `msrv` | MSRV Check (1.94.1) |
| `minimal-versions` | Minimal Versions Check (`-Zminimal-versions`, nightly, continue-on-error) |
| `coverage` | Code Coverage (`cargo-llvm-cov`, ≥59% threshold) |
| `audit` | Security Audit (`cargo audit`) |
| `miri` | Miri UB Detection (nightly) |
| `deny` | Dependency Policy Check (`cargo deny`) |
| `experimental-check` | Experimental Crates Check (continue-on-error, non-blocking) |
| `sanitizer-asan` | AddressSanitizer (nightly, continue-on-error) |
| `sanitizer-tsan` | ThreadSanitizer (nightly, continue-on-error) |

**Strict Rules for the Fast Gate:**

- **Deterministic:** Tests must not rely on random timing or external networking.
- **Local:** No external dependencies are allowed (e.g., no external LLM services, no Ollama
  required).
- **Fast:** Any test exceeding a few seconds must be moved to heavy certification or heavily
  optimized.

### Experimental Crate Circuit Breaker

The workspace includes several **experimental crates** that are not part of the core MVP:

| Crate | Description | Status |
|-------|-------------|--------|
| `vantadb-server` | Local HTTP/gRPC server | Experimental |
| `vantadb-mcp` | Model Context Protocol interface | Experimental |
| `vantadb-wasm` | WASM bindings for JS/TS SDK | Experimental |
| `vantadb-openai` | OpenAI embedding adapter | Experimental |
| `vantadb-ollama` | Ollama embedding adapter | Experimental |
| `vantadb-litellm` | LiteLLM embedding adapter | Experimental |

**Circuit breaker rules:**

1. **Removed from `default-members`** in root `Cargo.toml` — `cargo check`, `cargo build`, and
   `cargo test` without `--workspace` skip them entirely.
2. **Excluded from `--workspace` clippy** — the `--exclude` flag is used for all three platforms
   (Linux, Windows, macOS) so lint failures in experimental code do not block CI.
3. **Excluded from `--workspace` coverage** — the `cargo llvm-cov nextest` step uses `--exclude` so
   compilation or test failures in experimental code do not block coverage reporting.
4. **Dedicated `experimental-check` job** — runs `cargo check` on all experimental crates with
   `continue-on-error: true`. This provides visibility into experimental crate health without
   blocking the fast lane.

**To promote an experimental crate to stable**, remove it from the exclusion list in
`ci-rust-10.yml` and re-add it to `default-members` in `Cargo.toml`.

**Review 2026-08-05 (TECH-08):** Decision: **keep `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`
EXPERIMENTAL** — not promoted to `default-members`. Evidence: all three compile together
(`cargo check -p vantadb-server -p vantadb-mcp -p vantadb-wasm` → OK, 49s) and their test suites are
green, but the circuit-breaker policy is deliberate: a failure in an experimental crate must not block
core CI, and the planned desktop build (`DESKTOP-01b`) depends on being able to consume these crates
with an empty `[workspace]` decoupling. Re-evaluate after desktop ships.

### Experimental Suite

Experimental tests are retained for local/manual diagnostics but do not define the v0.1.x MVP. Run
them explicitly with:

```bash
cargo nextest run --profile experimental --workspace --features experimental
```

Failures in this suite should be triaged, but they do not block the Fast Gate unless the failure is
caused by a change to production-facing MVP behavior.

### 2. Heavy Certification (`heavy-certification-50.yml`)

The heavy certification suite validates the engine's capability to run under production stress,
ensuring recall guarantees and scaling limits. **Goal:** Validate engine stability, recall, and
scale capabilities without bottlenecking daily development.

**Jobs:**

| Job | Description |
|-----|-------------|
| `stress-protocol` | Dynamic scaling (10K, 50K, 100K vectors), persistence, latency, 0.95+ Recall@10 |
| `hnsw-validation` | HNSW validation |
| `hnsw-recall` | HNSW recall certification |
| `sift-validation` | SIFT-1M validation (manual opt-in) |
| `competitive-bench` | Competitive benchmarks vs FAISS/HNSWlib (manual opt-in) |
| `failpoint-tests` | `chaos_integrity`, `wal_resilience`, `crash_injection` — crash recovery with `--features failpoints` |
| `storage-persistence` | Storage persistence & recovery (backend, durability, GC, schema evolution, etc.) |
| `text-index` | Text index recovery (`text_index_recovery`) |
| `memory-concurrency` | Memory & concurrency heavy tests (concurrency, memory brutality, fuzz proptest) |
| `other-heavy` | Remaining heavy tests: CLI, benchmarks, hybrid ranking, MCP (`vantadb-mcp`), columnar, `concurrent_insert_preserves_hnsw_invariants` |

**Why are these tests separated?** Running `stress_protocol` can take close to 2 hours on hosted
runners and requires significant system resources (AVX2 plus heavy swap). It runs in its own
scheduled/manual job with a 150 minute step timeout so it can complete without blocking the other
certification checks. Running this on every PR would paralyze development velocity.

### Coverage Gate — Minimum Mechanized Coverage (P2-06)

**Tier:** Heavy Certification (not the Fast Gate). A PR can reduce coverage in a hot module without
any gate failing; this gate closes that gap by enforcing a minimum line-coverage threshold
mechanically. **Task P2-06** from `docs/plans/2026-08-10-p2-p3-structural-quality.md`. The
threshold starts conservative and ratchets upward.

**Command:**

```bash
cargo llvm-cov --fail-under-lines <UMBRAL>
```

(The CI variant runs under nextest: `cargo llvm-cov nextest run --profile audit --workspace
--fail-under-lines <UMBRAL>`.) `--fail-under-lines` exits with status 1 when total **line**
coverage falls below the threshold — that is the fail-under behavior of this gate.

**Scope:** Applies to the hot modules `src/vector/` and `src/engine/`. The threshold is evaluated
on the whole workspace report (the hot modules dominate it); to enforce a strict per-module floor,
narrow the report with `--ignore-filename-regex` to those paths.

**Initial threshold (PRUDENT):** 60% line coverage. No coverage artifacts exist in the repo as of
2026-08-10, so this number is **to be set from the first llvm-cov run** — the first Heavy
Certification pass must record the real current level and adjust the threshold to match it (never
set it below the current level).

**Escalation policy:** the threshold ratchets upward over time (e.g. +5 points per quarter or per
release), converging with the Fast Gate coverage job (currently ≥59%). The threshold is never
lowered without a documented justification and review.

**Merge / union:** multiple coverage runs under different feature/test conditions are merged with
`--no-report` runs followed by `cargo llvm-cov report`; `--failure-mode any|all` controls whether
merge failures fail the gate.

**Test exclusion:** cargo-llvm-cov excludes `tests/` directories and `*_tests.rs` files from the
report by default, so test code itself is not counted toward the threshold.

**Local verify wiring:** `dev-tools/verify.ps1` runs this gate (with the same `--fail-under-lines`
threshold) only when `cargo-llvm-cov` is installed; otherwise it prints a warning and continues, so
the default local `just verify` flow is never blocked by a missing tool.

### 3. Web CI (`ci-web-11.yml`)

Builds and lints the web frontend (`web/` directory — Next.js 16). Runs `npm ci`, `npm run lint`,
`npx tsc --noEmit`, and `npm run build` on push/PR to `main` that touches `web/**`. No test infra
— the Next.js SPA is client-only (`"use client"` everywhere). Triggered by `workflow_dispatch` as
well.

### 4. Docs Gate (`gate-docs-21.yml`)

Lints Markdown files in `docs/**` with `markdownlint-cli2`. Triggered on push/PR to `main`
touching docs.

### 5. Security Scan (`sec-codeql-30.yml`)

CodeQL analysis for Rust. Runs on push/PR to `main` and weekly. Triggers via `workflow_dispatch`.

### 6. Fuzzing (`fuzz-40.yml`)

LibFuzzer corpus + regression via `cargo fuzz`. Scheduled weekly (Monday 06:00 UTC) or
`workflow_dispatch`.

### 7. Performance Benchmarks (`perf-bench-40.yml`)

Python integration performance benchmarks. Triggered on push to `main` touching core or
Python paths, or via `workflow_dispatch` with configurable vector/queries/dim inputs.

### 8. Nightly Benchmarks (`heavy-bench-nightly-51.yml`)

Nightly benchmark regression suite (daily CRON plus `workflow_dispatch`). Runs light benchmarks
and heavy benchmarks across multiple package scopes.

### 9. Python Wheel Build & Publish (`release-wheels-60.yml`)

Builds the Python SDK on Linux, macOS, and Windows with `maturin`, installs the generated wheel
by resolved path, and runs the Python SDK smoke suite. Manual TestPyPI upload is available only
through an explicit workflow input and the `TEST_PYPI_API_TOKEN` secret. Production PyPI
publication and signing remain deferred.

### 10. Release Workflows

| Workflow | File | Trigger |
|----------|------|---------|
| Release Automation | `release.yml` | Push to `main` — `release-plz` auto-version, changelog, tag, publish |
| NPM Publish | `release-npm-61.yml` | Tag `wasm-v*.*.*` / `ts-v*.*.*` or `workflow_dispatch` |
| PyPI Adapters | `release-adapters-62.yml` | Tag `adapters-v*.*.*` or `workflow_dispatch` (TestPyPI) |
| Binary Builds | `release-binaries-63.yml` | Release published or `workflow_dispatch` |
| SBOM Generation | `release-sbom-64.yml` | Tag `v*` or `workflow_dispatch` |

## External Dependencies (Ollama/LLMs)

VantaDB integrates with external LLMs for embeddings and semantic queries. However, **integration
tests requiring network access to LLMs (like Ollama) are strictly excluded from the Fast Gate.**
They are either marked with `#[ignore]` or gated behind environment variables (e.g.,
`VANTADB_RUN_LLM_TESTS=1`). This ensures the core engine can be built and tested completely offline.

## Running Heavy Certification Manually

The `heavy-certification-50.yml` workflow runs automatically via a CRON schedule (weekly on
Sundays at 03:00 UTC). The scheduled lane runs the local deterministic core certification jobs.
SIFT-1M validation and competitive benchmarks are manual opt-ins because they require external
datasets. You can also trigger it manually from the GitHub Actions UI:

1. Navigate to the **Actions** tab in the repository.
2. Select **HEAVY: Certification — All Tests** from the left sidebar.
3. Click **Run workflow**.
4. You can optionally check the boxes to include `SIFT-1M validation` or `Competitive benchmarks`.
