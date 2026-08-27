---
title: "VantaDB CI & Certification Policy"
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-08-25
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
| `coverage` | Code Coverage (`cargo-llvm-cov`, gate root crate ≥80% por ADR-018) |
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

### Fast Gate Test Exclusions

The local fast gate (`dev-tools/verify.ps1`, `nextest` and `coverage` steps) applies an explicit
`-E` nextest filter that removes three tests. These exclusions are **not** flakiness waivers — all
three tests are deterministic and still run in full local suites and Heavy Certification; they are
removed only from the fast lane because they deliberately stress runner resources.

**Category taxonomy:** these use a dedicated `RESOURCE-GUARD` category, alongside the existing
EXPERIMENTAL / BEST-EFFORT / NON-CRITICAL / INFORMATIONAL categories used for
`continue-on-error:` annotations in `.github/workflows/`. A `RESOURCE-GUARD` test is one whose
input is intentionally large or hostile enough to threaten the runner itself (OOM, page-file
exhaustion), not one that is broken or slow by accident.

| Excluded test | Lives in source | Why excluded | Category | Where the exclusion is enforced |
|---------------|-----------------|--------------|----------|--------------------------------|
| `deserialize_absurd_node_count` | `src/index/core.rs:414` | Deserializes a crafted buffer with `u64::MAX` node count — designed as a memory bomb for the deserializer path; allocating it on a shared runner risks OOM-killing unrelated jobs | RESOURCE-GUARD | `dev-tools/verify.ps1` `-E` filter (`nextest` + `coverage` steps) and the non-nextest fallback `--skip` list |
| `test_search_with_bizarre_text_query` | `tests/security.rs:639` | Feeds giant malformed text queries (100KB strings, NUL bytes, astral-plane chars) into search; robust behavior against such inputs belongs to the dedicated fuzzing lane (`fuzz-40.yml`), not the fast gate | RESOURCE-GUARD | Same |
| `test_malformed_payload_extremely_large` | `tests/security.rs:324` | Ingests a 1MB payload plus 10KB of metadata; same rationale — hostile-input coverage is delegated to fuzzing | RESOURCE-GUARD | Same |

**Structural exclusions in `.config/nextest.toml`:** the `default-filter` of the `audit` profile
additionally excludes ~55 heavy test binaries (stress_protocol, chaos_integrity, wal_resilience,
sift_validation, competitive_bench, etc.) via package-qualified `not (package(X) and binary(Y))`
clauses (BND-06 scope-safe form). That list implements the two-tier split documented in this file
(Fast Gate vs Heavy Certification) and changes only together with `heavy-certification-50.yml`.

**Rules for any new exclusion:**

1. Every fast-gate test exclusion must be listed in this table with its category and rationale.
2. Exclusions must be traceable: `dev-tools/verify.ps1` carries an inline comment pointing back to
   this policy (Regla 2 traceability).
3. **Who can add or revert an exclusion:** only the project lead (`vanta-lead`) after review;
   re-enabling a `RESOURCE-GUARD` test in the fast gate requires evidence that the input size was
   reduced below runner-risk thresholds (e.g. bounded allocations in the test itself).

### Experimental Crate Circuit Breaker

The workspace includes several **experimental crates** that are not part of the core MVP:

| Crate | Description | Status |
|-------|-------------|--------|
| `vantadb-server` | Local HTTP server + MCP stdio binary | Experimental |
| `vantadb-mcp` | Model Context Protocol interface (lib-only; served by `vanta-cli server --mcp`) | Experimental |
| `vantadb-wasm` | WASM bindings for JS/TS SDK | Experimental |
| `providers/openai` | OpenAI embedding adapter (NOT a workspace member — checked via `--manifest-path`) | Experimental |
| `providers/ollama` | Ollama embedding adapter (idem) | Experimental |
| `providers/litellm` | LiteLLM embedding adapter (idem) | Experimental |

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
`ci-rust-10.yml` and re-add it to `default-members` in `Cargo.toml`. The full
promotion DoD (10 checks, per-crate cost table, wall-time budget and
reversibility) is defined in **[ADR-031: Promotion to default-members](../architecture/adr/ADR-031-default-members-promotion.md)** — no crate may be promoted without passing ADR-031 in 3 consecutive clean runs; see ADR-031 §Question to Owner for the Fast Gate `<5 min` vs Heavy threshold gate (STABLE-00).

#### Promotion to `default-members` — ADR-031 DoD (STABLE-00, P47)

`default-members` today is `[ ".", "vantadb-python" ]` (`Cargo.toml:636`). Candidates
`vanta-memory`, `vanta-proxy`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`
(Rust) and `vantadb-ts`/`vantadb-node` (npm, equivalent gate) must each pass
the **10-check DoD** before promotion; the checks and their exact commands are
the single source of truth in ADR-031 (gates 1-10: `cargo check`+`fmt`+`clippy -D warnings`,
`cargo nextest --profile audit`, `cargo deny check`, `validate-docs-coverage`,
workflow `paths:` + no `continue-on-error` + `timeout-minutes <5 min` measured,
`cargo package --dry-run`, `wasm-pack`/`wasm32` for `wasm`, `napi` 7-target matrix +
`npm pack` for `node`, `verify.ps1` wall time `<5 min` with all crates included,
ADR with cost + rollback). **Rollback is 1 line:** `git revert` of `Cargo.toml:636`
(the `publish = false` crates never affect `cargo publish`).

Until ADR-031 is `accepted` (Owner answers STABLE-00 question A vs B on `<5 min`
vs Heavy), promotion is **blocked** — STABLE-01..08 may validate per-crate gates
but STABLE-09 must not merge. Gate #9 is measured on branch `test/default-all`
with expanded `default-members` (`verify.ps1` / `just verify` cold cache).

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
mechanically. **Task P2-06** from `docs/plans/archive/2026-08-10-p2-p3-structural-quality.md (archivado)`. The
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
release), converging with the Fast Gate coverage job (gate canónico ADR-018: root crate ≥80%). The threshold is never
lowered without a documented justification and review.

**Merge / union:** multiple coverage runs under different feature/test conditions are merged with
`--no-report` runs followed by `cargo llvm-cov report`; `--failure-mode any|all` controls whether
merge failures fail the gate.

**Test exclusion:** cargo-llvm-cov excludes `tests/` directories and `*_tests.rs` files from the
report by default, so test code itself is not counted toward the threshold.

**Local verify wiring:** `dev-tools/verify.ps1` runs this gate (with the same `--fail-under-lines`
threshold) only when `cargo-llvm-cov` is installed; otherwise it prints a warning and continues, so
the default local `just verify` flow is never blocked by a missing tool.

**Policy decision (COV-004, 2026-08-09):** the strategic coverage policy — root crate vs workspace
aggregate vs per-runner binding measurement — is decided in
[ADR-015](../architecture/adr/ADR-015-coverage-policy.md) (accepted, owner TBD). In force:

- Gate canónico (ADR-018, supersede ADR-015 §D1): root crate antadb ≥ 80% line (baseline
  81.40%). Workspace aggregate (root + antadb-python, 72.76% medida) se reporta solo para
  visibilidad. Nunca bajar el umbral de 80% para acomodar un baseline.
  to accommodate a baseline.
- Bindings are measured on their native runners: Python wrapper coverage ≥ 85% via pytest;
  WASM/MCP/server carry no coverage gate while experimental (Tier 3).
- No new `--fail-under` gate is added for the aggregate. Revisit on release or when a binding
  graduates from experimental.

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
| NPM Publish | `release-npm-61.yml` | Tag `v*.*.*`, push to `main` with `vantadb-ts/**`/`vantadb-wasm/**` paths, `pull_request` with same paths, or `workflow_dispatch` — includes Fast Gate job `tests` (`npm ci && npm run build && npx vitest run`, measured 27s <5min, no `continue-on-error`, PR+push gate per TS-06) |
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

