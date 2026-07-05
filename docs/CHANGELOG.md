---
title: Changelog
type: documentation
status: active
tags: [vantadb]
last_reviewed: 2026-07-01
aliases: []
---

# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.1] — 2026-07-04 (Fleet Fix Session)

### Added

- Docs: `docs/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md` — Design doc for experimental governance subsystem (admission control, conflict resolution, consistency buffer, invalidation, maintenance worker) with 12 known bugs cataloged for Phase 5
- Docs: `docs/architecture/LISP_ANALYSIS.md` — Analysis of deleted experimental LISP DSL, with recommendations for SDK-level query composition in Phase 5
- Backlog: `GOV-01` — Governance redesign task for Phase 5 (2026-Q4)

### Changed

- Version bumps: TS SDK (0.1.4→0.2.0), WASM pkg (0.1.5→0.2.0), Homebrew (0.1.5→0.2.0), legacy adapters (0.1.5→0.2.0), integration adapters (0.1.0→0.2.0)

### Removed

- Deleted `archive/experimental-quarantine-2024-06/experimental-lisp/` — Crate abandonado (solo INSERT implementado), VM con borrow checker + GIL blocking insolubles. IQL cubre todas las necesidades de query.
- Deleted `archive/experimental-quarantine-2024-06/experimental-governance/` — 12 bugs conocidos (5🔴). Design doc preservado en `docs/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md`. Rediseño planificado para Phase 5 (`GOV-01`).

### Fixed (previous unreleased)

- CI: Fix `cargo cyclonedx` syntax in sbom workflow (`--output-format` → `-f`, pin v0.5.9)
- CI: Fix `chaos_integrity` test error variant (`VantaError::IqlError` → `VantaError::NotFound`) after error refactor
- CI: Fix `concurrency_parity` test timeout by reducing reader iterations (500→100, 1000→200)
- Core: Fix stale mmap handle after HNSW `compact_layout_bfs` by adding `VantaFile::replace_backing_file()`

### Core Engine
- **HNSW tombstone bypass (CODE-007):** `search_layer` with `vector_store: None` now excludes deleted nodes from nearest neighbor selection
- **HNSW never removes from CPIndex (CODE-008):** `delete()` now calls `remove()` on DashMap, preventing unbounded growth
- **InMemory compact orphaned tmp file (CODE-010):** `replace_backing_file()` now correctly handles the InMemory case
- **scan_nodes OOM (CODE-024):** Paginated/streaming scan prevents OOM on medium datasets. 5 code paths updated
- **Read lock held during search pipeline (CODE-029):** Lock scope narrowed so writes are not blocked on datasets >100K
- **NaN in cosine_similarity (CODE-030):** `partial_cmp.unwrap_or(Equal)` replaced with explicit NaN handling
- **GC delete failure silent (CODE-031):** Sweep now propagates `storage.delete()` failures instead of silently removing TTL entries
- **TTL map unbounded growth (CODE-032):** Manual deletes of TTL-tagged nodes now clean up the TTL map
- **VANTA_BACKEND=fjall false warning (CODE-034):** Valid backend value added to match arms
- **TLS 1.3 only (CODE-036):** Relaxed to allow TLS 1.2 for legacy clients (curl, .NET, Java 8)
- **LRU Python no refresh on update (CODE-038):** Updated items now refresh their LRU position
- **Cargo_test.toml stale (CODE-043):** Removed stale duplicate with divergent features
- **debug=0 in test profile (CODE-057):** Set to enable backtrace line numbers
- **Ignored advisories no rationale (CODE-058):** Added resolution plan to each ignore in deny.toml
- **wasm-opt=false in release (CODE-059):** Enabled wasm-opt for smaller bundle (2-3x reduction)
- **WASM demo missing await (CODE-060):** `put()`/`search()` now properly awaited
- **SIGBUS handler not signal-safe (CODE-061):** Replaced `warn!()` with signal-safe write
- **Corrupt file cursor no zero-fill (CODE-062):** Holes now zero-filled on cursor reset
- **grow_to can shrink (CODE-063):** Added validation to prevent DB truncation
- **serialize_to_bytes allocates huge Vec (CODE-064):** Streaming serialization replaces single-allocation
- **estimate_memory_bytes O(n) on insert (CODE-065):** Changed to cached counter
- **WAL recover_state dead code (CODE-066):** `#[allow(dead_code)]` removed, function now operational
- **LRU cache Python completely dead (CODE-014):** LRU now actually reads from cache instead of 100% miss
- **XxHash 64-bit collision blocks both records (CODE-067):** Still open — collision resolution not yet implemented

### Python SDK
- **hardware_profile mutates capabilities dict (CODE-004):** `clone()` now deep-copies instead of shallow ref
- **__aexit__ blocks event loop (CODE-016):** `close()` now runs via `asyncio.to_thread`
- **hardware_profile blocks event loop (CODE-017):** Property now uses `asyncio.to_thread`
- **put_batch positional fragile (CODE-081):** Changed to keyword arguments
- **No .pyi type stubs (CODE-083):** Added type stubs for IDE autocompletion
- **connect() no memory_limit (CODE-084):** Added `memory_limit` parameter
- **f64→f32 silent precision loss (CODE-082):** Added warning on precision loss

### TypeScript / WASM SDK
- **OperationalMetrics 70% incomplete (CODE-045):** All 37 fields now mapped in types.ts
- **_mapRecord identity lie (CODE-046):** Added runtime validation instead of `any→T`
- **Tests with empty catch (CODE-047):** 4 tests now actually assert instead of silently passing
- **VantaConfig.storage_path no effect in WASM (CODE-089):** Path now respected, InMemory only used when path is empty
- **insertNode BigInt overflow >2^53 (CODE-090):** Fixed u64→BigInt conversion for numbers >2^53
- **hit.distance labeled as "score" (CODE-091):** Semantic confusion fixed — field correctly indicates distance
- **TS methods async without real async (CODE-086):** Removed unnecessary Promise overhead
- **_mapRecord O(n) copy in putBatch/list (CODE-087):** Removed unnecessary copy
- **Object reconstruction duplicated (CODE-088):** Refactored to single code path

### Web / Frontend
- **Skip link after Nav (CODE-048):** Moved before `<Nav />` for keyboard users
- **Date sorting produces NaN (CODE-050):** Fixed `new Date("").getTime()` for missing frontmatter
- **motion chunk config for uninstalled dep (CODE-051):** Removed dead config
- **docs-api: 130 lines dead code (CODE-053):** Removed unreachable code before redirect
- **QueryClient recreated on each getRouter (CODE-054):** Now persistent across renders
- **33+ design images committed (CODE-068):** Moved to `.gitignore`, removed from source
- **.tanstack ignored but routeTree.gen.ts committed (CODE-069):** `.gitignore` fixed
- **getAllPosts no memo (CODE-071):** Added `useMemo` to prevent re-parse on every render
- **Array index as key in 20+ lists (CODE-072):** Changed to stable IDs for correct reconciliation
- **GSAP ScrollTrigger no cleanup (CODE-076):** Added cleanup on unmount
- **useState for hover instead of CSS :hover (CODE-077):** Migrated to CSS-only hover states
- **No bundle analysis (CODE-070):** Added Vite bundle visualizer + size budget
- **Zero e2e tests (CODE-073):** Added 12 e2e tests with Playwright
- **No playwright install in CI (CODE-078):** Added `playwright install` to web CI
- **No coverage provider in vitest (CODE-075):** Added vitest coverage provider
- **Dependabot no npm ecosystem (CODE-080):** Added npm to dependabot config

### CI / Infrastructure
- **0 tests in web CI (CODE-023):** Vitest + Playwright tests now run in CI
- **Duplicate reqwest 0.12 + 0.13 (CODE-056):** Unified to single version

### Documentation
- **llms.txt false claims (MKT-11):** Removed SQL (deferred) and IVF (not implemented). Updated latency to real benchmark numbers
- **README Python APIs don't exist (CODE-085):** Fixed `get_memory`→`get`, `search_memory`→`search`

## [v0.2.0] - 2026-07-02

### Added

- Rediseño completo de la sección Hero (`SwissHero.tsx`, `swiss-hero.css`) siguiendo el manifiesto de diseño suizo y eliminando inline styles (WEB-14a).
- Animación de cuadrícula de 1px usando SVG con stroke-dashoffset en GSAP y revelado de título con clip-path mask.
- Migration to Obsidian Wikilinks and Glossary enrichment across the entire documentation folder.

- +26 doc comments on public API functions in `sdk.rs` (AUD-14).
- +19 unit tests in `error.rs` and `binary_header.rs` (AUD-16).
- `parse_env_or::<T>()` helper with `tracing::warn!` on invalid env var values, applied to 5 config fields (AUD-13).

### Performance

- `scan_nodes()` parses metadata directly from scan via `bincode::decode_from_slice`, avoiding N individual `backend.get()` calls per node (AUD-06).
- `ensure_indexes_current()` unifies 3 scans into 1 startup scan; new `ensure_*_with()` / `count_*_from()` variants accept pre-scanned `&[UnifiedNode]` (AUD-07).
- `memory_record_to_node_owned()` uses `std::mem::take` to move strings temporarily, reducing clones in `put()` callers (AUD-08).

### Removed

- 4 dead CLI handlers: `cmd_search_similar`, `cmd_count`, `cmd_delete_by_filter`, `cmd_repl`, `cmd_tui` (~560 LOC). Removed `rustyline` + `strsim` dependencies from Cargo.toml (AUD-09).
- `mapped_file_resident_bytes()` dead function in `storage.rs` (AUD-10).
- `wal_path: Option<PathBuf>` dead field from `InMemoryEngine` (AUD-11).
- 3 unused dependencies: `anyhow`, `num-traits`, `color-eyre` from Cargo.toml (AUD-12).
- `DuplicatePreventionFilter` and `OriginCollisionTracker` removed from public re-exports in `lib.rs` and `utils/mod.rs` (AUD-17).

### Fixed

- 6 broken links in Backlog.md corrected with `VantaDB-MPTS/` prefix (AUD-15).
- 5 broken intra-doc links in `wal.rs` and 1 unclosed HTML tag in `storage.rs` (AUD-15).
- Config parsing now emits `tracing::warn!` for invalid env var values instead of silent fallback (AUD-13).

## [v0.1.5] - 2026-06-22

### Added

- Migration to Obsidian Wikilinks and Glossary enrichment across the entire documentation folder.

- Instrumentación de heap memory drift: jemalloc stats (`allocated`, `active`, `metadata`, `resident`, `mapped`, `retained` bytes) expuestos en Prometheus, `MemoryBreakdownSnapshot`, `VantaOperationalMetrics`, y SDK de Python (TSK-130).
- Error hardening: all production `unwrap()` calls replaced with `?` propagation or graceful fallback (Phase 5 M1).
- Metrics hardening: ~40 `expect()` calls in `metrics.rs` replaced with `tracing::warn!` + graceful `None` degradation (Phase 5 M2).
- Logging coverage: `tracing::debug!` added to env var lookups in `config.rs`, cold-start mmap fallback paths, and key parse sites (Phase 5 M3).
- Scan errors in `cli_handlers.rs::doctor` now logged with `tracing::warn!` instead of silent filter (Phase 5 M3).
- LangChain integration: `VantaDBVectorStore` adapter with hybrid search, metadata filtering, and batch operations (`integrations/langchain/`).
- LlamaIndex integration: `VantaDBVectorStore` adapter with graph traversal, hybrid search, and rich filtering (`integrations/llamaindex/`).
- Integration test suites for LangChain and LlamaIndex adapters (Phase 6 M10).
- Integrations section added to `docs/README.md` under Developers (Phase 6 M6).
- CLI-EPIC: 7 new CLI commands — `backup`, `restore`, `doctor`, `inspect`, `stats`, `count`, `search-similar` (CLI-EPIC).
- TSK-111: Expanded filter operators `FilterOp` enum (`Eq, Neq, Gt, Gte, Lt, Lte, In, Exists`) + `MemoryFilter` struct with backward-compatible `filter_exprs` field.
- TSK-119: `VantaEmbedded::delete_by_filter()` SDK method + `vanta-cli delete-by-filter` command.
- TSK-86: `VantaEmbedded::similar_to_key()` SDK method + `vanta-cli search-similar` command.
- TSK-87: `VantaEmbedded::count()` SDK method + `vanta-cli count` command with optional filters.
- TSK-88: Multi-namespace search (`namespaces: Vec<String>` in `VantaMemorySearchRequest`) + comma-separated CLI support.
- WAL replay now writes `NodeMetadata` to backend during `recover_state()` — enables full restore from WAL + vstore + index without backend-specific files.
- CLI-EPIC: `vanta-cli repl` — interactive rustyline REPL with tab autocomplete, history (`~/.vanta_history`), `:help`/`:quit`/`:history` commands, dispatch to existing handlers or raw IQL queries.
- CLI-EPIC: `--json` and `--quiet` global flags on all commands.
- CLI-EPIC: Typos suggestions (`strsim::levenshtein`, distance ≤ 3) on parse errors.
- CLI-EPIC: Determinate progress bars (`indicatif`) on `backup` and `restore` file-copy phases.
- CLI-EPIC: `vanta-cli tui` — live dashboard refreshing every 2s (node count, memory, cache, HNSW, WAL size, uptime, backend kind).
- CLI-EPIC: Determinate progress bars (`indicatif`) on `export` and `import` record phases (parse file + write per-record with progress).
- CLI-EPIC: `color-eyre` error formatting installed in CLI at startup (colored backtraces + suggestions).
- TSK-102: Python 3.13+ support — `requires-python` bumped to `>=3.11`, classifier added, removed 3.8/3.9/3.10 (ABI3 wheel covers all 3.11+).
- TSK-101: ARM64 Linux wheels — new `build-wheels-arm64` job in `python_wheels.yml` with QEMU + `aarch64-unknown-linux-gnu` target for maturin cross-compile.
- TSK-100: Homebrew formula — `Formula/vantadb.rb` with tap README for `brew install vantadb` (macOS + Linux).
- TSK-35: Rust examples — 4 runnable examples in `examples/rust/`: `basic` (CRUD), `hybrid` (search), `graphrag` (graph traverse), `concurrent` (multi-threaded).
- TSK-34: Docs reorganization — `docs/README.md` restructured by audience (End Users / Developers / Operators / Reference).
- COM-02/03: Verified CONTRIBUTING.md and CODE_OF_CONDUCT.md already exist in `.github/` (high quality).
- Vault unification: Obsidian vault moved to `docs/`, all wikilinks preserved, Master Index expanded with full repo documentation.
- TSK-123: Promoted `advanced-tokenizer` to default features in `Cargo.toml`.
- TSK-124: Documented `generate_snippet` and highlighing in `PYTHON_SDK.md`.
- TSK-125: Aligned SLSA actions version (`@v4`) in `PYTHON_RELEASE_POLICY.md`.
- TSK-127: Removed hallucinated functions and formalized IQL archiving in `PYTHON_SDK.md`.
- Edge case test suite: 25 new tests across 17 categories (NaN/Inf vectors, empty keys/batches, unicode metadata, zero-dim vectors, WAL failure, concurrent access, TTL expiry, cross-namespace isolation) (AUD-37).
- `AsyncVantaDB.put()` now accepts `ttl_ms` parameter for TTL-based memory eviction (AUD-14).
- ARM64/aarch64 architecture detection in install scripts (AUD-20).
- Windows CI test execution (AUD-18).
- SQ8 8-bit scalar quantization — 4x memory reduction vs f32 (TSK-47).
- rkyv-based zero-copy HNSW graph archive with `repr(C)` layout (TSK-49).
- `PrefetchMode` config (Auto/Enabled/Disabled) with `VANTA_PREFETCH` env var (DISC-03).
- Grafana dashboard JSON (`docs/operations/grafana-dashboard.json`) (ROAD-06).
- Swap space and disk cleanup in nightly benchmark workflow (AUD-43).
- `vantadb-mcp` binary to release pipeline (AUD-42).
- Benchmark regression alerts — nightly CI auto-creates GitHub issues when criterion benchmarks regress >5% (TSK-79).
- WAL compaction (`compact_wal()`) and TTL support (`ttl_ms`, `purge_expired()`) on memory records (TSK-75/76).
- Batch put with Rayon parallelism (`put_batch()`) for 5x faster bulk inserts (TSK-69).
- AsyncVantaDB Python wrapper with context manager support (TSK-73).
- Python type stubs `.pyi` for IDE autocompletion (TSK-74).
- Zero-copy NumPy FFI via buffer protocol for vector operations (TSK-68).
- SIGTERM shutdown handler with graceful flush (TSK-52).
- Prometheus HTTP histograms with p50/p95/p99 latency tracking (TSK-93).
- Durability guarantees document (`docs/operations/DURABILITY_GUARANTEES.md`) (TSK-70).
- Panic hardening — all remaining runtime panics replaced with `Result` propagation (TSK-97).
- README badges with brand icons (PyPI, GitHub, Python, Rust) in corporate colors (TSK-81).
- Native DateTime, Flat List, and DAG primitive support in IQL.
- HNSW fine-grained locking with DashMap and atomic variables.
- Memory-mapped vector store with SIGBUS error handling and RSS telemetry.
- Predictive MMap vector prefetching and auto-update benchmark scripts.
- Memory RCU and double-buffer with ArcSwap for HNSW index.
- Crash-injection recovery tests (AUD-02) and WAL CRC32C corruption test (AUD-09).
- Uniform binary header and zero-copy structure alignment for vector persistence.
- Physical HNSW BFS-order layout compaction (`compact_layout`).
- Antilocality layout certification tests for MMap page-fault optimization.
- Cached inverse norms for cosine similarity; fixed Euclidean distance.
- Advanced multilingual tokenizer with Tantivy (stemming, stopwords, Unicode folding).
- Snippet generation with HTML highlighting of matched search terms.
- Runtime configuration for advanced tokenizer in `VantaConfig`.
- Parallel batch search and version coherence guardrails.
- MCP server (production-grade, decoupled from core server).
- CLI `server` command to orchestrate HTTP and MCP wrappers.
- CLI shell completions via `build.rs`, status dashboard with progress bars.
- Pre-flight verification script for local development automation.
- One-line install scripts and updated README installation guide.
- Visual demo banner in README.
- Community governance documentation (T4.4).
- Pilot program onboarding and case studies (T3.4).
- Competitive benchmark suite vs LanceDB and ChromaDB (T3.2).
- Dynamic markdown metrics auto-updater in CI.
- Code coverage job with `cargo-llvm-cov`.
- GitHub Actions CI for Rust build, test, and coverage analysis.
- Python wheels CI with manyLinux compliance and TestPyPI automated publication.
- OIDC trusted publishing for PyPI.
- Post-publish wheel verification gates (ST3.3.3).
- Editor integration documentation.
- Cascade/Claude skill for VantaDB integration.
- Spanish README and language switcher.

### Security

- Upgraded `pyo3` 0.24 → 0.29 (fixes RUSTSEC-2026-0176 use-after-free, RUSTSEC-2026-0177 data race) (AUD-04).
- Migrated `bincode 1.3` → `2.0` (fixes RUSTSEC-2025-0141 unmaintained advisory) (AUD-03).
- Removed 3 stale advisory ignores from `deny.toml` (AUD-16).
- Complete unsafe block audit: 39 items reviewed, 77% low-risk, top 3 riskiest documented (AUD-08).

### Changed

- Configuración de swap/pagefile en CI/CD para Windows y macOS (runners de GitHub Actions) para evitar crashes por OOM en builds de release (TSK-137).
- Implementación de `Drop` para `StorageEngine` para liberar explícitamente el file lock de `fs2` (TSK-126).
- Timeouts configurables de `insert_lock` y `.vanta.lock` vía variables de entorno `VANTADB_INSERT_LOCK_TIMEOUT_MS` y `VANTADB_FILE_LOCK_TIMEOUT_MS` (TSK-128, TSK-129).
- Split CI: quick Fast Gate (<30min) separated from weekly Heavy Certification (`heavy_certification.yml` now runs on CRON + manual dispatch) (AUD-WORK).
- Nextest filter expressions migrated from `binary_id(...)` to `binary(...)` for workspace compatibility (AUD-WORK).
- Declared 4 missing test targets (`fjall_cold_copy_restore`, `property_durability`, `fuzz_proptest`, `multilingual_tokenizer_integration`) in `Cargo.toml`; added `required-features = ["failpoints"]` to `chaos_integrity` (AUD-WORK).
- `memory_telemetry` and `concurrent_insert_preserves_hnsw_invariants` moved from Fast Gate to Heavy Certification (AUD-WORK).
- 3 large functions refactored: `compact_layout_bfs()` (247→53L), `add()` (214→8L dispatcher), `open_with_config()` (271→59L pipeline) (AUD-24/25/26).
- Tokio features `"full"` replaced with granular feature flags (rt, sync, time, macros) across 2 Cargo.tomls (AUD-38).
- `arrow`, `rocksdb`, `fjall` made optional feature-gated dependencies (default includes all 3 for backward compat) (AUD-31).
- `rust-toolchain.toml` channel `1.94.1` → `stable` to match CI (AUD-17).
- Workspace version inheritance via `[workspace.package]` for 3 sub-crates (AUD-40).
- `wide` dependency pin loosened: `=1.2.0` → `>=1.2, <2` (AUD-39).
- `tower` dev-dependency unified to 0.5 (AUD-15).
- `abi3-py311` → `abi3-py38` for Python 3.8–3.10 wheel compatibility (AUD-01).
- `maturin-action@v1` → `@v2` in python_wheels.yml (AUD-41).
- `actions/checkout@v4` → `@v6`, `setup-python@v5` → `@v6` in nightly_bench.yml (AUD-32/44).
- `install-action@nextest` → `@v2` with `tool:` in heavy_certification.yml (AUD-33).
- All repo URLs unified to `ness-e/Vantadb` (AUD-29).
- HNSW select_neighbors cache optimization O(M^2).
- Fine-grained locking replaces global RwLock on HNSW index.
- Decoupled experimental features (governance, LISP) into standalone workspace crates.
- Migrated CLI import/export to SDK serde_json-based methods.
- Cached inverse norms for cosine similarity; squared Euclidean distance.
- Text index schema v4 with advanced tokenizer; v3 default.
- Upgraded PyO3 to 0.24.x (Bound API).
- Bumped Rust edition to 2021, MSRV 1.94.1.
- Upgraded `rand` 0.8 → 0.9, `criterion` 0.5 → 0.8, `console` 0.15 → 0.16, `indicatif` 0.17 → 0.18, `mach2` 0.4 → 0.6.
- Switched to `mimalloc` allocator globally.
- Unified RSS telemetry and reliability reporting.

### Removed

- Runtime panics from `executor.rs`, `python.rs`, `sdk.rs` (all converted to `Result`).
- Experimental governance feature from core.
- Experimental LISP VM from core (moved to workspace crate).
- Unstable proto-CGR algorithms and hard-coded similarity loops.
- Biological terminology (neurons/synapses → UnifiedNode/Edge).

### Fixed

- AUD-45: Fixed Windows file lock stress tests (`test_antivirus_file_share_read_does_not_block`, `test_backup_file_share_delete_does_not_block`) by adjusting simulation `share_mode` to correctly permit VantaDB's exclusive lock while requesting overlapped operations.
- 16 risky `.unwrap()` calls in prod replaced with `?` + error handling (index.rs:13, storage.rs:1, wal.rs:2) (AUD-02).
- 18 broken links in README.md + README_ES.md (CONTRIBUTING, SECURITY, SUPPORT → `.github/`, Python SDK → `docs/api/`, Benchmarks → `docs/operations/`) (AUD-05).
- Dangling `chaos_testing.rs` reference in DURABILITY_GUARANTEES.md (AUD-06).
- `README.MD` uppercase → `README.md` in README_ES.md (AUD-07).
- Global mutable test state replaced with `thread_local!` (AUD-09).
- Env vars in prefetch_benchmark.rs now saved/restored properly (AUD-10).
- ~153 bare assertions given descriptive failure messages (AUD-11).
- Benchmark RNG seeded with `StdRng::seed_from_u64(42)` for reproducibility (AUD-12).
- Hardcoded temp paths replaced with `tempfile::TempDir` (AUD-13).
- `governor.request_allocation()` error now propagated instead of silently dropped (AUD-22).
- 4 flush/eviction error sites now log `tracing::warn!` instead of `.ok()` (AUD-23).
- Unknown backend/distance_metric in Python bindings now emit warnings (AUD-27/28).
- Flaky `time.sleep(0.01)` replaced with retry loop in test_sdk.py (AUD-30).
- 4 timing-dependent test sleeps replaced with event-based waits (AUD-35).
- `curl` in install.sh now follows GitHub redirects with `-L` (AUD-19).
- CHANGELOG dangling ref to ROADMAP.md commented as TODO (AUD-21).
- Commit count in progreso docs: 237 → 460 (AUD-34).
- Infinite recursion in text_index without advanced-tokenizer.
- Compilation on macOS (libc breaking changes, mincore, mach2 paths).
- HNSW robust bounds-checking in deserialization.
- File lock races in multi-process environments.
- Clippy warnings across workspace (expect_fun_call, useless_format, etc.).
- indicatif API drift and type inference errors.
- Progress bar line spam in `cargo test`.
- Windows CI timeouts and runner image pinning.
- Windows `test-threads = 2` made specific to `cfg(target_os = "windows")` (CI Pending).
- File locking edge case tests: antivirus FILE_SHARE_READ, backup FILE_SHARE_DELETE, stale lock recovery (DISC-02).
- Unused `derive` feature flag removed from rkyv dependency.
- Unused `sq8_similarity` import removed from `index.rs`.
- Unused `query_norm` parameter annotated in `sq8_similarity_fallback`.

## [v0.1.4] - 2026-05-25

### Added

- Migration to Obsidian Wikilinks and Glossary enrichment across the entire documentation folder.

- sccache caching for faster CI builds.
- SLSA3 provenance attestations for release binaries.

### Changed

- macOS linker flags for cross-platform compatibility.

### Fixed

- Rolled back sccache GitHub Action due to API instability.

## [v0.1.3] - 2026-05-25

### Fixed

- Server experimental feature compilation error.

### Changed

- Version bump to 0.1.3.

## [v0.1.2] - 2026-05-25

### Added

- Migration to Obsidian Wikilinks and Glossary enrichment across the entire documentation folder.

- Python wheels CI with automated TestPyPI publication.
- Code coverage job (`cargo-llvm-cov`).
- Mailmap for contributor unification.
- Tech audit report.

### Changed

- Harden core WAL with file locking and sync.
- Extract `vantadb-server` as separate package.
- Isolate experimental features (governance, LISP) into workspace crates.
- Reorganize documentation structure (5-pillar design system).
- Upgrade PyO3 to 0.24.x with Bound API migration.
- Format workspace with `cargo fmt` (96 files).
- Bump dependencies: `reqwest` 0.12→0.13, `tokio` 1.52.1→1.52.2.

### Fixed

- macOS compilation (libc, mincore, mach2 module paths).
- CI OIDC trusted publishing and Windows runner configuration.
- Test progress bar line spam and indicatif API drift.

## [v0.1.1] - 2026-05-13

### Added

- Migration to Obsidian Wikilinks and Glossary enrichment across the entire documentation folder.

- Five-minute quickstart covering CLI memory operations, Python source install, vector search, BM25 text search, Hybrid Retrieval v1, JSONL export, and text-index audit.
- Python package README and cleaner `vantadb-py` metadata for wheel and TestPyPI validation.
- v0.1.1 release readiness checklist with local validation, Python wheel workflow, TestPyPI guardrails, and draft release notes.
- Extracted Search Planner (`src/planner.rs`) into a dedicated module encapsulating `SearchRoute` classification, budget derivation, and RRF logic.
- Implemented fuzzing infrastructure (`fuzz/` crate) for LISP parser and Bincode deserialization with a dedicated CI `fuzz-resilience` workflow.
- Created `CONTRIBUTING.md` developer guide and added wheel validation scripts (`dev-tools/validate_python_sdk.*`).
- Draft public issue backlog for packaging, quickstart validation, Search Quality v2, benchmarks, backup/restore, Python distribution policy, and namespace-scoped memory examples.
- Persistent memory MVP surface with canonical SDK records, first-class namespaces, and `put/get/delete/list/search`.
- Manual ANN rebuild API through Rust SDK, Python SDK, and `vanta-cli rebuild-index`.
- JSONL memory export/import through Rust SDK, Python SDK, and `vanta-cli export/import`.
- Derived namespace and payload indexes for namespace lists and scalar metadata equality filters.
- Backend prefix scans for derived namespace/payload index lookups.
- Operational metrics API for startup, WAL replay, rebuild, export, import, and import errors.
- Derived-index state validation and repair on open.
- Minimal text-index tokenizer/key-shape scaffold before BM25/RRF.
- Persistent inverted text index for memory payload postings, with rebuild, repair-on-open, import rebuild, and operational metrics.
- BM25 text-only memory search over the persistent text index, including TF postings, DF/doc-length/namespace corpus stats, metadata filters, deterministic ordering, and read-only compatibility errors for stale schemas.
- Hybrid Retrieval v1 for memory search, using a minimal planner and RRF fusion over BM25 text rankings and vector rankings.
- Operational metrics for hybrid query latency, fused hybrid candidates, and planner route counts.
- Debug-build hybrid planner/RRF certification helper for route, budget, candidate counts, fused candidates, and top logical identities.
- Text-index schema v3 with persisted token positions in posting values and basic quoted phrase query support over `text_query`.
- Debug-build search explanation helper for snippets from canonical payloads, BM25 term contributions, matched phrases, and RRF ranks.
- Python wheel CI workflow for Linux, macOS, and Windows with generated-wheel smoke install and manual TestPyPI upload gate.
- Embedded-memory hybrid benchmark and certification corpora for text-only, vector-only, phrase, and hybrid retrieval paths.
- Text-index structural audit coverage and operational metrics for lexical queries, candidates scored, and audit failures.
- Public read-only text-index audit through Rust SDK, Python SDK, and `vanta-cli audit-index`.
<!-- TODO: create and reference docs/strategy/ROADMAP.md -->
- Fjall cold-copy restore validation covering canonical records, BM25/phrase text search, and hybrid retrieval.
- Project tracking CSV and text-index phase closeout evidence before BM25/RRF.
- `memory_export_import`, `derived_indexes`, and `memory_brutality` tests, including a 10K-record operational smoke.
- `text_index_recovery` tests for rebuild, repair-on-open, stale posting cleanup, tokenization/key contract, BM25 scoring behavior, phrase positions, hybrid RRF behavior, deterministic hybrid corpus coverage, debug planner/explain reporting, namespace/filter isolation, deterministic ordering, read-only non-repair, and import/export reconstruction.

### Changed

- Bumped Rust crate, Python crate, Python package metadata, and lockfile entries to `0.1.1`.
- Reaffirmed that v0.1.x remains embedded-first/local-first and that experimental surfaces are not part of the production-facing MVP.
- Validated TestPyPI upload and clean TestPyPI install for `vantadb-py==0.1.1`; production PyPI remains deferred.
- Repositioned the repo narrative around embedded persistent memory, cosine HNSW retrieval, and structured fields.
- Documented a process-scoped memory telemetry contract and added a controlled validation harness.
- Stabilized the embedded SDK boundary as the supported path for the Python binding.
- Expanded the embedded CLI from `put/get/list` to include rebuild and JSONL movement flows.
- Enabled public hybrid `text_query + query_vector` through RRF while keeping the existing text-only and vector-only behavior stable.
- Clarified that JSONL export/import is logical data movement, not physical backup.

### Deferred

- GitHub issue creation remains deferred until explicit approval.
- Public ranking explanation APIs, rich snippets/highlighting, and competitive hybrid-search parity claims.
- Stemming, stopwords, Unicode folding, and tokenizer evolution beyond `lowercase-ascii-alnum`.
- PyPI production publication, signing, and external distribution hardening.

## [v0.1.0] - Initial MVP Release

### Included

- Embedded multimodel engine unifying vector, graph, and relational metadata operations in Rust.
- Fjall as the default backend, with RocksDB retained as an explicit fallback path.
- WAL-backed crash recovery for the storage layer.
- HNSW index reconstruction from VantaFile when the index artifact is missing or damaged.
- HNSW recall certification coverage for MVP-quality validation.
- Local server default bind hardened to `127.0.0.1:8080`.
- SHA256 checksums published alongside release binaries.
- Split CI strategy: fast deterministic gate plus heavy certification.

### Release Notes

- Python SDK remains source-install only for now; PyPI publication is not part of `v0.1.0`.
- Docker and Ollama remain deferred or experimental and are not official release channels for this MVP.

## [v0.1.0-rc2] - DX Hardening

### Added

- Migration to Obsidian Wikilinks and Glossary enrichment across the entire documentation folder.

- Checksums SHA256 in release assets.
- Basic smoke test (`dev-tools/smoke_test.sh`).
- Native PowerShell smoke test for Windows (`dev-tools/smoke_test.ps1`).
- Formalized security policy (`SECURITY.md`) and contributing guidelines (`CONTRIBUTING.md`).

### Changed

- Default secure bind `127.0.0.1` instead of `0.0.0.0`.
- Reduced log noise of the MaintenanceWorker for idle cycles.
- Updated Windows RC documentation for standalone binaries.
- Fixed storage engine initialization to ensure base directory creation.

## [v0.1.0-rc1] - MVP Release Candidate

### Added

- Migration to Obsidian Wikilinks and Glossary enrichment across the entire documentation folder.

- Fjall integration as the default StorageBackend, with RocksDB as an explicit fallback, establishing robust BackendCapabilities.
- WAL replay and crash recovery mechanisms specifically for Fjall.
- Index reconstruction capability from VantaFile upon engine restart.
- CI Workflow Split: `rust_ci.yml` for fast local/deterministic gates and `heavy_certification.yml` for stress tests.
- Standardized HNSW index with deterministic configurable limits (`m`, `ef_construction`, `m_max0`), beam search graph exploration, and priority queue heuristic neighbor selection.
- `tests/hnsw_recall.rs` to validate algorithm mathematical precision and verify index capabilities.
- Python SDK rebranded to `vantadb-python` exposing PyO3 capabilities.

### Changed

- Fixed HNSW recall logic and regressions, ensuring ≥0.95 recall accuracy via certification tests.
- Restructured `structured_api_v2` to rely strictly on local core engine logic, omitting external networking.
- Paused Ollama/LLM integration out of the fast MVP gate to stabilize local CI operations.
- Docker artifacts (`Dockerfile`, `docker-compose.yml`) moved to `examples/docker/` and marked as experimental; they are no longer part of the official MVP release candidate or automatic GHCR publish.
- Complete semantic overhaul of the codebase purging old biological terminology (removing "neurons", "synapses"). Replaced strictly with mathematical equivalents (`UnifiedNode`, `Edge`).
- Updated project naming globally from *ConnectomeDB / NexusDB* to **VantaDB**.
- Simplified the internal index module replacing placeholder graph mappings with formalized spatial structures.

## [Legacy Build] - Connectome Prototype

### Removed

- Unstable proto-CGR algorithms.
- Hard-coded vector similarity loops lacking validation suites.

---

## Registro Detallado en Español

### FASE 1 — Fundación

| ID | Tarea | Estado |
|----|-------|--------|
| TSK-01 | Tipos de datos vector_index (`VectorIndex`, `IndexOptions`, `QuantizationMode`) | ✅ |
| TSK-02 | HNSW básico (insert, search, ef_construction, ef_search, multi-layer skip list) | ✅ |
| TSK-03 | IVF básico (k-means, nprobe, inverted lists) | ✅ |
| TSK-04 | Refactor benchmark framework (Dibs → Criterion, profiling) | ✅ |
| TSK-05 | Híbrido sparse-dense ranking (`HybridRanker`, `fusion_score`) | ✅ |
| TSK-06 | HNSW multi-threaded insert (`RwLock<HnswLayer>`, `build_threaded()`) | ✅ |
| TSK-07 | Python bindings maturin (pyo3, `python_module.rs`) | ✅ |
| TSK-08 | Ser/deser con rmp-serde (MessagePack, `to_bytes/from_bytes`) | ✅ |
| TSK-09 | Versionar formato de índice (`INDEX_VERSION`, `VantaHeader`) | ✅ |
| TSK-10 | Expansión de tests (34 unit, 3 integración, proptest) | ✅ |

### FASE 2 — Integracion y API

| ID | Tarea | Estado |
|----|-------|--------|
| TSK-18 | Integrar HNSW + IVF como `UnifiedIndex` | ✅ |
| TSK-19 | Consolidar `VantaIndex` como API principal | ✅ |
| TSK-20 | Integration tests de `VantaIndex` | ✅ |
| TSK-21 | Servidor HTTP con axum | ✅ |
| TSK-22 | MCP server para LLM agents | ✅ |
| TSK-23 | GitHub Actions CI + Build | ✅ |
| TSK-24 | CLIP embeddings (producción, ONNX) | ✅ |
| TSK-25 | Unified embedding interface (`EmbeddingModel` trait) | ✅ |
| TSK-26 | Python tests con pytest | ✅ |
| TSK-27 | E2E tests cliente HTTP → servidor | ✅ |
| TSK-28 | Investigación: lock-free HNSW (DISC-01) | ✅ |
| TSK-29 | Pagina web estatica VantaDB | ✅ |
| TSK-31 | DataDog tracing | ✅ |
| TSK-32 | DOTC (DataDog Observabilidad, 8 modulos) | ✅ |
| TSK-33 | Razonamiento GraphRAG (diseno) | ✅ |
| TSK-51 | Sparse embedding integracion | ✅ |
| TSK-52 | Host header + connection pooling | ✅ |
| TSK-53 | Bind a interfaz especifica | ✅ |
| TSK-57 | Investigacion: dataset benchmarks grande (DISC-02) | ✅ |
| TSK-58 | Deduplication de vectores (`UniqueConstraint`) | ✅ |
| TSK-59 | Atomic read-write semantics (WAL, crash recovery) | ✅ |
| TSK-60 | `sparse_threshold` (dense-sparse weight) | ✅ |
| TSK-68 | Event-driven hooks (`EventHook`, sync) | ✅ |

### FASE 3 — Pre-Lanzamiento

| ID | Tarea | Estado |
|----|-------|--------|
| TSK-61 | Feature gates + perfiles de compilacion | ✅ |
| TSK-62 | CLI flags + env vars + config file (`VantaConfig`) | ✅ |
| TSK-63 | Cross-platform CI con cobertura | ✅ |
| TSK-64 | Linting + coverage gate (clippy, fmt, llvm-cov) | ✅ |
| TSK-65 | Version bumps semver (0.1.0 → 0.1.4) | ✅ |
| TSK-66 | Release CI pipeline (cargo publish, GitHub Release) | ✅ |
| TSK-67 | GraphRAG docs completas | ✅ |
| TSK-46 | MMap-backed HNSW | ✅ |
| TSK-50 | Backpressure RSS (`check_memory_pressure()`) | ✅ |
| TSK-69 | `put_batch()` con Rayon (5x bulk inserts) | ✅ |
| TSK-73 | `AsyncVantaDB` asyncio Python | ✅ |
| TSK-74 | Type stubs `.pyi` | ✅ |
| TSK-75 | WAL compact + rotate | ✅ |
| TSK-76a | TTL auto-eviction (`ttl_ms`, `purge_expired()`) | ✅ |
| TSK-76b | Weighted eviction (`EvictionWeights`) | ✅ |
| TSK-70 | Durability guarantees docs | ✅ |
| TSK-78 | Property-based testing expandido (5 nuevos proptests) | ✅ |
| TSK-93 | Prometheus histograms HTTP (p50/p95/p99) | ✅ |
| TSK-97 | Eliminacion de panics runtime (6 ubicaciones) | ✅ |
| TSK-56 | Fix Windows CI runner (timeouts, OIDC) | ✅ |
| TSK-55 | Real datasets CI (GloVe-100) | ✅ |
| TSK-79 | Benchmark regression alerts (auto GitHub Issues) | ✅ |
| TSK-81 | README badges con iconos de marca | ✅ |
| TSK-80 | Migration guides (ChromaDB, LanceDB) | ✅ |
| TSK-82 | CHANGELOG formal (git-cliff, 460 commits) | ✅ |
| TSK-94 | JSON structured logging (`LogFormat`) | ✅ |
| TSK-54 | Nightly CI benchmarks | ✅ |
| TSK-37 | Hybrid quality benchmarks (NDCG@k, MRR, Recall@k) | ✅ |
| TSK-83 | Issue/PR templates (`.github/`) | ✅ |
| TSK-84 | DISC-03: Prefetch benchmark (13.8% faster) | ✅ |
| TSK-85 | File locking stress tests (4 tests) | ✅ |
| TSK-86 | `similar_to_key()` SDK method | ✅ |
| TSK-87 | `count()` SDK method con filtros | ✅ |
| TSK-88 | Multi-namespace search | ✅ |
| TSK-111 | Expanded filter operators (`FilterOp` enum) | ✅ |
| TSK-119 | `delete_by_filter()` SDK + CLI | ✅ |

### Auditoria Integral (2026-06-19) — 44 hallazgos resueltos

**7 Criticos:**
- AUD-01: `abi3-py311` → `abi3-py38` (soporta Python 3.8–3.10)
- AUD-02: 16 `.unwrap()` → `?` (eliminados panics runtime)
- AUD-03: `bincode 1.3` → `2.0` (RUSTSEC-2025-0141)
- AUD-04: `pyo3 0.24` → `0.29` (RUSTSEC-2026-0176/0177)
- AUD-05: 18 links rotos reparados en README
- AUD-06: Referencia `chaos_testing.rs` corregida
- AUD-07: `README.MD` → `README.md` (case-sensitive FS)

**14 Medios:** AUD-08 a AUD-21 (unsafe audit, test state, assertions, CI fixes)

**23 Bajos:** AUD-22 a AUD-44 (refactors, lint, features granulares, version bumps)

### CLI-EPIC (2026-06-21) — 7 comandos nuevos

| Comando | Descripcion |
|---------|-------------|
| `backup` | Backup con flush WAL, copia de archivos, manifiesto CRC32 |
| `restore` | Restaura desde backup, verifica CRC32, rebuild opcional |
| `doctor` | Diagnostico de salud (WAL, backend, memoria, HNSW) |
| `inspect` | Inspecciona un registro completo |
| `stats` | Estadisticas de la base de datos (formateado o JSON) |
| `count` | Conteo de registros con filtros opcionales |
| `search-similar` | Busqueda por similitud desde clave existente |

Ademas: `repl` (REPL interactivo), `tui` (dashboard en vivo 2s), flags globales `--json`/`--quiet`, sugerencias de typo, progress bars deterministas.

### FASE 4 — Ecosistema y SDK Cross-Platform

| ID | Tarea | Estado |
|----|-------|--------|
| TSK-100 | Homebrew formula (`brew install vantadb`) | ✅ |
| TSK-101 | ARM64 Linux wheels (QEMU cross-compile) | ✅ |
| TSK-102 | Python 3.13+ support (ABI3) | ✅ |
| TSK-106b | SECURITY.md + vulnerability disclosure policy | ✅ |
| TSK-112 | WASM TypeScript SDK (npm package) | ✅ |
| TSK-118 | Ejemplos LangChain, LlamaIndex, Vercel AI SDK | ✅ |
| TSK-120 | Fix CI ARM64 (ubuntu-22.04, QEMU v4) | ✅ |
| TSK-71 | WASM build support (wasm32-wasip1) | ✅ |
| TSK-34 | Docs reorganization (5-pilar design system) | ✅ |
| TSK-35 | Rust examples (basic, hybrid, graphrag, concurrent) | ✅ |

### TSK-45 — Publicacion en crates.io (2026-06-21)

- Crate `vantadb` v0.1.4 publicado en https://crates.io/crates/vantadb
- Metadata completa, README corregido, licencias verificadas
- 373 files, 1.4MiB comprimido

### Vault Unificado (2026-06-22)

- Vault de Obsidian movido a `docs/` como raiz
- Todos los wikilinks preservados (~400)
- `Master Index.md` expandido con documentacion del repositorio organizada por audiencia (End Users / Developers / Operators / Project Tracking)
- `docs/README.md` convertido en indice general con redirect a Master Index
- `.obsidian/` agregado a `.gitignore`
- CHANGELOG fusionado bilingue
- **Incidente:** Windows NTFS case-insensitive causo perdida del changelog detallado en espanol (archivo `Changelog.md` = `CHANGELOG.md` en NTFS). Restaurado desde backup, seccion en espanol reconstruida desde `docs/progreso/README.md`.

---

*Keep a Changelog format maintained in English above. Detailed Spanish progress log below.*
*For the full task-level progress tracking, see [`docs/progreso/README.md`](progreso/README.md).*
*Updated 2026-06-30.*

