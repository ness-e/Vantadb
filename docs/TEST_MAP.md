# Test Map — 2026-07-22

> If you change X, which test suite do you run? Quick reference for contributors.

## Fast Decision Table

| If you change… | Run this first | Then run | CI gate |
|---|---|---|---|
| **Core engine** (`src/`) | `cargo nextest run --profile audit -p vantadb` | `just verify` | ci-rust-10 |
| **Core + heavy tests** | `just test` (audit profile) | `just certify` for weekly-level coverage | heavy-certification-50 |
| **Storage layer** (`tests/storage/`) | `cargo nextest run --profile audit --test <name>` | `cargo test --release --test storage --test wal_resilience --test …` | heavy-certification-50 (storage-persistence) |
| **HNSW / vector index** (`src/index/hnsw/`) | `cargo nextest run --profile audit --test hnsw` | `cargo test --release --test hnsw_validation --test hnsw_recall_certification` | heavy-certification-50 (hnsw-*) |
| **Query logic / parser** (`src/query/`, `src/lang/`) | `cargo nextest run --profile experimental` | `cargo test --release --test integration --test executor --test governor` | heavy-certification-50 (other-heavy) |
| **Python bindings** (`vantadb-python/`) | `cd vantadb-python && cargo test` | `maturin develop && pytest tests/` | ci-rust-10 (via workspace) |
| **Python integrations** (`integrations/*/`) | `pytest integrations/<name>/tests/` | — | — (no CI gate) |
| **Integration adapters** (`vantadb-{openai,langchain,…}/`) | `cargo test -p vantadb-<name>` | `pytest` if Python-side tests exist | ci-rust-10 (compilation) |
| **WASM crate** (`vantadb-wasm/`) | `cargo check -p vantadb-wasm` | `wasm-pack test --chrome --headless` | ci-rust-10 (check + wasm job) |
| **TS SDK** (`vantadb-ts/`) | `cd vantadb-ts && npx vitest run` | `cd vantadb-ts && npm run build` | — (no CI gate for TS SDK) |
| **Web frontend** (`web/`) | `cd web && npx vitest run` | `cd web && npx playwright test` | ci-web-11 |
| **Server** (`vantadb-server/`) | `cargo nextest run -p vantadb-server` | `cargo test --release -p vantadb-server --test e2e` | heavy-certification-50 (other-heavy) |
| **MCP** (`vantadb-mcp/`) | `cargo nextest run -p vantadb-mcp` | `cargo test --release -p vantadb-mcp --test mcp_tests` | heavy-certification-50 (other-heavy) |
| **Documentation** (`docs/`) | — | `npx markdownlint-cli2 "docs/**/*.md"` | gate-docs-21 |
| **CI config** (`.github/`) | `just fmt && just clippy` | `just verify` | ci-rust-10 |
| **Dependencies** (`Cargo.toml`, `Cargo.lock`) | `just deny && just audit-cargo` | `just machete` | ci-rust-10 (deny + audit) |
| **Benchmarks** (`benches/`, `tests/benchmark_*`) | `cargo bench` (criterion) | `cargo test --release --test benchmark_internal` | heavy-bench-nightly-51 |

## Justfile Targets

| Command | What it runs | When to use | Est. time |
|---|---|---|---|
| `just check` | `cargo check --workspace` | Quick compile check before any test run | ~30s |
| `just clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Before opening a PR | ~60s |
| `just fmt` | `cargo fmt --check` | Verify formatting | ~10s |
| `just test` | `cargo nextest run --profile audit --workspace` | **Primary**: CI-matching test suite (audit profile, excludes heavy) | ~5–10m |
| `just test-one <name>` | `cargo nextest run --profile audit --workspace --test <name>` | Run a single test binary | ~2–5m |
| `just test-all` | `cargo nextest run --workspace` (no filter) | All tests including heavy | ~30–60m |
| `just test-experimental` | nextest `experimental` profile (logic/parser/executor) | During active work on query engine | ~2–4m |
| `just verify` | `fmt + clippy + test + deny` | **Pre-PR gate** — what CI checks | ~8–12m |
| `just verify-quick` | `dev-tools/verify_changed.ps1` | CodeGraph-optimized ~30s check | ~30s |
| `just deny` | `cargo deny check` | License + advisory policy check | ~10s |
| `just audit-cargo` | `cargo audit` | Advisory-only check | ~10s |
| `just certify` | `dev-tools/nocturnal_suite.ps1` | Full heavy certification (local) | ~60–120m |
| `just python-test` | `dev-tools/scripts/validate_python_sdk.ps1` | Python SDK after changes | ~3–5m |
| `just machete` | `cargo machete` | Unused dependency detection | ~15s |
| `just outdated` | `cargo outdated --exit-code 1` | Check for stale deps | ~30s |
| `just watch` | `cargo watch -x check -x "nextest run --profile audit"` | Iterative dev loop | continuous |
| `just ci` | `fmt + clippy + test + deny + audit-cargo` | Full pre-merge CI | ~10–15m |
| `just docs` | `scripts/validate-docs-coverage.ps1` | Docs coverage verification | ~10s |

## CI Gate Summary

| Gate workflow | What it covers | Frequency | Required for merge |
|---|---|---|---|
| **ci-rust-10** | Build, clippy, nextest audit (Linux/Win/macOS), coverage ≥59%, cargo-deny, cargo-audit, miri (UB), MSRV, ASan/TSan | Every push/PR to main | **Yes** (fmt+clippy+test+deny+audit) |
| **ci-web-11** | Web build, lint, tsc, vitest unit, playwright e2e | Every push/PR touching `web/` | **Yes** for web changes |
| **heavy-certification-50** | Full test suite: stress, HNSW validation, storage persistence, failpoints, text index, memory concurrency, benchmarks | Weekly (Sun) + manual trigger | No (weekly quality signal) |
| **heavy-bench-nightly-51** | Performance benchmarks | Nightly | No (regression signal) |
| **gate-docs-21** | Markdown lint + frontmatter validation | Every push/PR touching `docs/` | **Yes** for doc changes |
| **fuzz-40** | Fuzz testing (Linux, nightly) | On demand | No |
| **sec-codeql-30** | CodeQL security analysis | On push to main | No |
| **release-*** | Binary, wheel, npm, SBOM publishing | Tag/release | N/A |

## Per-Crate Test Commands

| Crate | Test command | Features / notes |
|---|---|---|
| `vantadb` (core) | `cargo nextest run -p vantadb` | `--profile audit` in CI; `--features "cli,arrow,tls,opentelemetry"` |
| `vantadb-python` | `cargo test -p vantadb-python && maturin develop && pytest tests/` | Requires Python venv; `--features pyo3/extension-module` |
| `vantadb-server` | `cargo nextest run -p vantadb-server` | Tests: `server`, `e2e`, `benchmarks`, `mcp_integration` |
| `vantadb-mcp` | `cargo nextest run -p vantadb-mcp` | Test: `mcp_tests` |
| `vantadb-wasm` | `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` | WASM CI job: `wasm-pack test --chrome --headless` (best-effort, non-blocking) |
| `vantadb-{openai,langchain,…}` | `cargo test -p vantadb-<name>` | Thin integration adapters, compilation-only in CI |

## Test Binary Quick Reference

Tests are organized in `tests/` (core crate) by category:

| Directory | Test binary | What it validates |
|---|---|---|
| `tests/core/` | `basic_node`, `graph`, `hnsw`, `regression_certification`, `snapshot_certification`, `vector_scale_check` | Core data structures, graph operations, HNSW index integrity, snapshot stability |
| `tests/storage/` | `storage`, `mutations`, `core_invariants`, `gc`, `mmap_index`, `antilocality_layout`, `backend_tests`, `wal_resilience`, `tombstone_ann_vstore`, `crash_injection`, `multi_process_lock`, `chaos_integrity` | Storage backend (fjall/rocksdb), WAL, tombstone GC, mmap, crash recovery, failpoints |
| `tests/logic/` | `integration`, `parser`, `executor`, `governor`, `columnar` | Query parsing, execution, governance, columnar (arrow) |
| `tests/certification/` | `stress_protocol`, `hnsw_validation`, `hnsw_recall_certification`, `sift_validation`, `competitive_bench`, `hybrid_retrieval_quality`, `hybrid_ranking_metrics`, `hardware_profiles` | Heavy recall/quality validation, cross-backend parity |
| `tests/memory/` | `backpressure`, `eviction`, `mmap_hnsw` | Memory management, LRU eviction, mmap index |
| `tests/security/` | `security_audit` | Security boundary audit |
| `tests/api/` | `structured_api_v2`, `python_sdk_boundary` | Public API contract, Python FFI boundary |
| `tests/` (root) | `durability_recovery`, `index_reconstruction`, `schema_evolution`, `concurrency_parity`, `memory_*`, `derived_*`, `text_index_recovery`, `version_coherence`, `edge_cases`, `fuzz_proptest`, `cli_tests`, `file_locking_stress`, `benchmark_*`, `prefetch_benchmark`, `property_durability`, `fjall_cold_copy_restore`, `multilingual_tokenizer_integration`, `miri_unsafe` | Root-level integration, durability, concurrency, fuzz, benchmarks |

## Coverage

- **CI threshold**: ≥59% line coverage (enforced in `ci-rust-10.yml` via `cargo-llvm-cov`)
- **Coverage exclusions**: `tests/`, `benches/`, `packages/experimental`, `crash_injection` source
- **Report**: Generated to `lcov.info` artifact in CI

## Python Integration Tests

| Package | Test path | Run command |
|---|---|---|
| `integrations/openai/` | `integrations/openai/tests/` | `cd integrations/openai && pip install -e . && pytest tests/` |
| `integrations/crewai/` | `integrations/crewai/tests/` | same pattern |
| `integrations/dspy/` | `integrations/dspy/tests/` | same pattern |
| `integrations/langchain/` | `integrations/langchain/tests/` | same pattern |
| `integrations/haystack/` | `integrations/haystack/tests/` | same pattern |
| `integrations/llamaindex/` | `integrations/llamaindex/tests/` | same pattern |
| `integrations/letta/` | `integrations/letta/tests/` | same pattern |
| `integrations/ollama/` | `integrations/ollama/tests/` | same pattern |
| `integrations/mem0/` | `integrations/mem0/tests/` | same pattern |

## Non-Rust Test Suites

| Project | Test runner | Config | Run command |
|---|---|---|---|
| **TypeScript SDK** (`vantadb-ts/`) | vitest | `vantadb-ts/vitest.config.ts` | `cd vantadb-ts && npx vitest run` |
| **Web** (`web/`) | vitest + Playwright | `web/vitest.config.*` (local), `web/playwright.config.*` (e2e) | `cd web && npx vitest run && npx playwright test` |

## Nextest Profiles (`.config/nextest.toml`)

| Profile | Filter | Use case |
|---|---|---|---|
| `default` | Excludes ~50 heavy/slow test binaries | Local dev quick check |
| `audit` | No filter (runs everything except default-excluded); fail-fast=false, 60s timeout | **CI default** |
| `ci-windows` | Extends `audit`; test-threads=2 | Windows CI (page file limit) |
| `experimental` | Only: integration, parser, executor, governor, columnar, graph, mcp_integration, structured_api_v2 | During query engine work |
| `chaos` | Only: `chaos_integrity_failpoints`; test-threads=1 | Failpoint validation |

---

## Adapter Tier Classification (ADR-001)

See `docs/archived-decisions/ADR-001-ADAPTER-TIERS.md` for full rationale.

| Tier | Label | Adapters | Score range | CI gate |
|---|---|---|---|---|
| 🟢 **Tier 1** | Core | `vantadb-openai`/`ollama`/`litellm` (Rust), `vantadb-haystack`/`llamaindex`/`langchain`/`openai`/`ollama` (Python), `vantadb-python` (SDK) | 10/10 | ci-rust-10 (Rust), manual (Python) |
| 🟡 **Tier 2** | Community | `vantadb-letta`, `vantadb-mem0`, `vantadb-crewai`, `vantadb-dspy` | 9–10/10 | manual |
| 🟠 **Tier 3** | Experimental | `vantadb-wasm`, `vantadb-mcp`, `vantadb-server`, `vantadb-ts` | varies | check/build only |
| 🏗️ **Tier 4** | Platform | `web/` | — | ci-web-11 |

### Per-Adapter Test Status (Python Rust adapters only)

| Adapter | Rust tests | Python tests | Score | Tier |
|---|---|---|---|---|
| `vantadb-openai` (Rust) | `cargo test -p vantadb-openai` | `pytest providers/openai/tests/` | 10/10 | 🟢 |
| `vantadb-ollama` (Rust) | `cargo test -p vantadb-ollama` | `pytest providers/ollama/tests/` | 10/10 | 🟢 |
| `vantadb-litellm` (Rust) | `cargo test -p vantadb-litellm` | `pytest providers/litellm/tests/` | 10/10 | 🟢 |
| `vantadb-haystack` | — | `pytest integrations/haystack/tests/` | 10/10 | 🟢 |
| `vantadb-llamaindex` | — | `pytest integrations/llamaindex/tests/` | 10/10 | 🟢 |
| `vantadb-langchain` | — | `pytest integrations/langchain/tests/` | 10/10 | 🟢 |
| `vantadb-openai` (Python) | — | `pytest integrations/openai/tests/` | 10/10 | 🟢 |
| `vantadb-ollama` (Python) | — | `pytest integrations/ollama/tests/` | 10/10 | 🟢 |
| `vantadb-letta` | — | `pytest integrations/letta/tests/` | 10/10 | 🟡 |
| `vantadb-mem0` | — | `pytest integrations/mem0/tests/` | 10/10 | 🟡 |
| `vantadb-crewai` | — | `pytest integrations/crewai/tests/` | 9/10 | 🟡 |
| `vantadb-dspy` | — | `pytest integrations/dspy/tests/` | 9/10 | 🟡 |
