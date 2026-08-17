# Contributing to VantaDB

Thank you for your interest in contributing! This guide covers the development
workflow, testing requirements, and specialized tooling like fuzzing.

---

## Development Prerequisites

- **Rust stable** (see `rust-toolchain.toml`)
- **cargo-nextest**: `cargo install cargo-nextest`
- **Python 3.11+** with `venv` support

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
cargo nextest run --profile experimental --workspace --build-jobs 2
```

---

## Code Quality

All PRs must pass the pre-flight gate. The single entry point is:

```bash
just verify            # fmt + clippy + test + deny (full gate, ~2-5 min)
just verify-quick      # CodeGraph-optimized quick check, ~30s
```

`just verify` is equivalent to running, in order:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --profile audit --workspace --build-jobs 2
cargo deny check
```

The same gate runs in CI on every PR/push (`.github/workflows/ci-gate.yml`) and
is enforced locally by the git hooks (`.githooks/pre-commit`, `.githooks/pre-push`).
Never push with `--no-verify` — fix the error and re-run instead (see
`.opencode/AGENTS.md` → Regla 1).

---

## Commit Convention

VantaDB uses **Conventional Commits** — `release-plz` parses commit messages to
decide the next semver bump, so the message format is load-bearing:

| Commit | Bump | Example |
|--------|------|---------|
| `feat:` | minor | `feat: add cosine distance metric` |
| `fix:` | patch | `fix: overflow in take_bytes bounds` |
| `docs:` | patch | `docs: update QUICKSTART.md` |
| `test:` | patch | `test: add edge case for empty index` |
| `perf:` | patch | `perf: reduce clone in hot path` |
| `refactor:` | patch | `refactor: extract hnsw builder` |
| `ci:` | no release | `ci: fix timeout in fuzz workflow` |
| `chore:` | no release | `chore: bump getrandom to 0.4` |
| `feat!:` / `BREAKING CHANGE:` | major | `feat!: redesign search API` |

Rules:
- `feat:` always implies minor (may contain breaking changes until 1.0.0).
- If a change is breaking even in `0.x`, use `feat!:`.
- Commits without a conventional prefix are ignored by release-plz — they don't
  trigger a release and don't appear in the changelog.
- The full normative rule is `.opencode/AGENTS.md` → Regla 7.

---

## Branch & PR Flow

The repo uses **`main` as the release branch, `develop` as the working branch**:

| Branch | Purpose | Rule |
|--------|---------|------|
| `main` | Releases only | Never commit directly — only PRs from `develop` |
| `develop` | Daily work | Every change starts here |

```
change code on develop → commit → push → PR to main → merge to main
                                                          ↓
              release-plz detects the push to main (GitHub Actions)
              → analyzes conventional commits since the last tag
              → bumps version automatically (major/minor/patch)
              → updates docs/CHANGELOG.md
              → opens a Release PR (e.g. "chore: release v0.4.1")
              → you review and merge the Release PR
              → release-plz tags and publishes to crates.io
              → RELEASE Wheels / NPM / Binaries workflows fire
```

**Never touch by hand** — `release-plz` owns these:
- The version field in `Cargo.toml`
- `docs/CHANGELOG.md` (regenerated via `git-cliff`, config `cliff.toml`)
- Git tags

### Pre-merge validation (run before merging to main)

```bash
just verify
dev-tools/setup_venv.ps1  # or setup_venv.sh on Unix
dev-tools/scripts/validate_python_sdk.ps1  # or validate_python_sdk.sh
```

### CI gates

Two tiers (see `docs/operations/CI_POLICY.md`):

1. **Fast Gate** (every PR/push): fmt, clippy, unit + fast integration tests.
   Must stay < 5 min, deterministic, offline.
2. **Heavy Certification** (manual/scheduled): stress, HNSW validation, SIFT,
   chaos, WAL resilience — up to 2h, never in the Fast Gate.

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

Fuzzing runs as a scheduled job in `.github/workflows/heavy-certification-50.yml`
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

## Where to Find Work

The project backlog lives in `docs/Backlog.md`. Each row follows the format
`ID | Description | Files | Effort | Priority | Status`. Look for items marked
`📝 Pendiente` with green priority, or pick issues labeled `good first issue`.

Relevant labels on GitHub issues:

| Label | Meaning |
|-------|---------|
| `bug` | Defect report — see Issue Triage below |
| `triage` | Needs classification (default on new bug reports) |
| `enhancement` | Feature request |
| `flaky` | Test fails intermittently — never hide with `continue-on-error` |

Normative rules for contributors live in `.opencode/AGENTS.md` (Reglas 1-11)
and the per-area rule files in `.opencode/rules/` (read the file for the area
you touch before editing code).

---

## Issue Triage

Every new issue is classified, labeled, and routed to the right owner domain.
The goal is a fast path from "reported" to "in front of the right agent".

### 1. Classification

| Type | How to recognize | Action |
|------|------------------|--------|
| **Bug** | Wrong behavior, crash, panic, data loss, unexpected result | Reproduce first: version + OS + steps + logs. Label `bug`. Add `flaky` if it fails intermittently. |
| **Feature** | New capability, API, SDK, or enhancement request | Label `enhancement`. Align with the local-first, embedded memory boundary (see `feature_request.yml`). |
| **Performance** | Slow search/ingestion, high latency, memory blowup, regression vs `docs/operations/BENCHMARKS.md` | Label `perf`. Require a reproducible benchmark command + before/after numbers (AGENTS.md Regla 9/11). |
| **Security** | Auth, trust boundary, unsafe, CVE, data exfiltration, secret leak | **Do not file publicly.** Report via `SECURITY.md` / the security policy (linked from `.github/ISSUE_TEMPLATE/config.yml`). |

### 2. Routing by domain

Once classified, route to the owner domain (agent system mapping, see
`.opencode/AGENTS.md` → "Límites de herramientas por rol"):

| Domain | Route to | Examples |
|--------|----------|----------|
| Core engine / storage / indexes | `vanta-engine` | HNSW, WAL, durability, vector/text search, `src/` |
| Bindings / SDK / integrations | `vanta-worker` | Python SDK, TS/WASM/Node, MCP, server |
| Security / vulnerabilities | `vanta-audit` | CVE lookup, unsafe audit, dependency advisories |
| Performance / benchmarking | `vanta-tuner` | Benchmarks, P99 regressions, telemetry |
| Stress / concurrency / deadlocks | `vanta-chaos` | Fuzzers, stress tests, lock-order audits |
| Documentation | `vanta-docs` | API docs, quickstart, ADRs, examples |
| PR review (after implementation) | `vanta-review` | Verify verdicts — review never implements |

### 3. Triage checklist

1. **Reproduce** the issue (or confirm it's a feature request).
2. **Classify** bug / feature / perf / security (table above).
3. **Label** — `bug` + `triage` (default on bug reports), `enhancement`, `perf`, `flaky`.
4. **Route** to the owner domain per the routing table.
5. **Security?** Close the public issue and redirect to `SECURITY.md` — never
   discuss exploit details in a public issue.

---

## Release Checklist

> ⚠️ Version bumps and publish are automated via **release-plz**. Never manually edit `Cargo.toml` version, `CHANGELOG.md`, or git tags.

1. Merge `develop` into `main` — trigger `release-plz` via Release PR
2. Wait for release-plz CI to pass and merge the auto-generated Release PR
3. Verify the release on crates.io, PyPI (`vantadb-py`), and npm (`vantadb`)
4. Confirm the new git tag `v{{ version }}` is published on GitHub

> Pre-merge validation (gate before merging to main) is listed above under
> **Branch & PR Flow** — run `just verify` plus the Python SDK validation
> scripts.
