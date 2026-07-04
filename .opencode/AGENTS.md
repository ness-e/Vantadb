# VantaDB — AGENTS.md

<!-- CODEGRAPH_START -->
## CodeGraph

CodeGraph tiene un índice pre-construido del código fuente de VantaDB (7.3K símbolos, 24.7K edges). **Úsalo SIEMPRE antes de grep/find/Read** para preguntas estructurales.

### Guía de decisión

| Situación | Qué usar |
|-----------|----------|
| "¿Cómo funciona X?", "¿Qué hace este flujo?" | `codegraph_explore "X"` — devuelve source + call paths + blast radius en 1 call |
| "¿Dónde está definido X?" | MCP search o `codegraph query "X"` |
| "¿Qué llama a esta función?" | `codegraph callers "function_name"` |
| "¿Qué llama esta función?" | `codegraph callees "function_name"` |
| "¿Qué se rompe si cambio X?" | `codegraph impact "X" --depth 3"` |
| "¿Qué tests ejecutar tras mis cambios?" | `git diff --name-only \| codegraph affected --stdin` |
| Leer un archivo con line numbers | `codegraph node "src/storage/engine.rs"` |

### Reglas

- **Confía en el resultado** — no re-verifiques con grep/read. El source que devuelve es idéntico byte por byte al del Read tool.
- **No uses grep para buscar definiciones** — CodeGraph ya las tiene indexadas.
- **Staleness**: si ves `⚠️ Pending sync:` tras una edición, lee el archivo directamente. El auto-sync tarda ~2s.
- **Sin `.codegraph/`** → ignora CodeGraph, usa herramientas normales.

### Ejemplos VantaDB

```
codegraph_explore "how does a search query reach StorageEngine"
codegraph_explore "VantaEmbedded open_with_config callers"
codegraph_impact "StorageEngine" --depth 3
```
<!-- CODEGRAPH_END -->

## Skills Manifest

**Todas las skills están centralizadas en:**
- `.agents/skills/` (proyecto, 154 skills)
- Referencia completa en: `SKILLS-MANIFEST.md` (raíz del proyecto)

**Siempre preferir la copia del proyecto sobre la global.**
Para cargar: `skill <nombre>` o leer el SKILL.md correspondiente.

### Skill Loading Guide

- **Diseño UI/Frontend**: `vanta-design-orchestrator` → `impeccable` → `design-taste-frontend`
- **Animación**: `motion (motion.dev)` (preferido), `gsap-core` (alternativa GSAP)
- **Corrección de bugs**: `systematic-debugging` → `writing-plans`
- **Features multi-paso**: `brainstorming` → `writing-plans` → skills relevantes
- **SEO**: `ai-seo` → `seo-audit` → `audit-website`
- **Video/presentaciones**: `hyperframes` → deck skills según necesidad
- **Branding/Arte**: `brandkit`, `canvas-design`, `algorithmic-art`, `theme-factory`, `color-expert`, `platform-design`

## Progreso Skill (MUST USE)

Load `progreso` at start and before completing every task:
- **Start**: `skill progreso` — reads backlog, checks for in-progress work
- **Complete**: `skill progreso` (Trigger 1) — moves done tasks from `docs/Backlog.md` → `docs/progreso/README.md` BEFORE any summary

## Doc Language Split

| Language | Content |
|----------|---------|
| **English** (source of truth) | `docs/api/`, `docs/architecture/`, `docs/operations/`, `docs/QUICKSTART.md` |
| **Spanish** (planning only) | `docs/VantaDB-MPTS/`, `docs/Backlog.md`, `docs/progreso/`, `docs/Investigaciones/` |

Technical docs stay in English. Never duplicate technical content in Spanish.

**Doc-Driven Development**: For new features, write/update `docs/api/` or `docs/operations/` docs FIRST, then implement. Never leave docs behind code.

## Pre-Flight Checks

```bash
:: Order matters — stop on first failure
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --profile audit --workspace --build-jobs 2
scripts/validate-docs-coverage.ps1   # final step before marking done
```

## Build System

- **Rust**: stable (rust-toolchain.toml: `1.94.1`+)
- **Profile `ci`** (no LTO, opt-level=2, 16 codegen-units) — used by CI Fast Gate
- **Profile `release`** (thin LTO, opt-level=3, 1 codegen-unit)
- **Profile `dev`** (opt-level=1, debug=0) — faster local iteration
- **Profile `test`** (opt-level=0, debug=0)
- **Profile `audit`** — used by nextest for pre-flight/release validation
- **Windows MSVC stack overflow workaround**: Always pass `--build-jobs 2` to nextest
- **Windows linker**: `.cargo/config.toml` forces `link.exe` (rust-lld causes STATUS_STACK_BUFFER_OVERRUN with large crates)

## Default Features

`cli` + `arrow` + `rocksdb` + `fjall` + `sysinfo` + `memmap2` + `fs2` + `prometheus` + `rayon` + `advanced-tokenizer`

Key optional features:
- `failpoints` — required for `chaos_integrity` test
- `remote-inference` — enables `llm` module (reqwest-based)
- `server` — enables axum HTTP server + tokio
- `python_sdk` — enables PyO3 bindings

## Test Suite

```bash
:: Fast Gate (audit profile)
cargo nextest run --profile audit --workspace --build-jobs 2

:: Single test (adapt to use nextest or cargo test as needed)
cargo nextest run --profile audit -p vantadb --test <test_name>

:: Tests that require specific features:
cargo nextest run --profile audit --features "failpoints" --test chaos_integrity
cargo nextest run --profile audit --features "cli" --test cli_tests
cargo nextest run --profile audit --features "arrow" --test columnar

:: Experimental tests (parser, executor, governor)
cargo nextest run --profile experimental --workspace --features experimental-lisp,experimental-governance

:: Fuzzing (requires nightly + Linux, in fuzz/ dir excluded from workspace)
cd fuzz && cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

Test categories: `tests/core/`, `tests/storage/`, `tests/logic/`, `tests/api/`, `tests/certification/`, `tests/memory/`, plus root-level integration tests.

## CI Architecture (Two-Tier)

1. **Fast Gate** (every PR/push): fmt, clippy, unit + fast integration tests. Must stay <5 min, deterministic, offline.
2. **Heavy Certification** (manual/scheduled): stress_protocol, hnsw validation, SIFT, competitive_bench, chaos_integrity, wal_resilience. Takes up to 2hrs. Never in Fast Gate.

See `docs/operations/CI_POLICY.md`.

## Python SDK

```bash
:: Hermetic venv (tests MUST use this — never a global install)
dev-tools/setup_venv.ps1         # creates target/audit-venv + maturin build
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -v

:: Editable install from source
pip install -e ./vantadb-python

:: PyPI name differs from import
pip install vantadb-py      # distribution
import vantadb_py            # module (underscore)
```

Built via `maturin` with PyO3. Requires Python ≥3.11.

## Architecture

```
vantadb/ (src/)            ← core library (primary crate)
  sdk/                     ← primary embedded API (VantaEmbedded, connect(), Vanta* types)
  engine.rs                ← in-memory engine
  storage/                 ← persistent backends (Fjall default, RocksDB fallback)
  wal.rs                   ← Write-Ahead Log
  vector/                  ← HNSW, distance metrics
  node.rs                  ← UnifiedNode, FieldValue
  cli.rs                   ← vanta-cli binary (#[cfg(feature = "cli")])
  api/                     ← HTTP routes (feature-gated, stub)
vantadb-python/            ← PyO3 bindings
vantadb-server/            ← standalone HTTP server binary
vantadb-wasm/              ← WASM build
vantadb-mcp/               ← MCP integration
vantadb-{openai,ollama,mem0,letta,crewai,dspy,haystack,litellm}/  ← thin integration crates
packages/                  ← LangChain + LlamaIndex adapter packages
fuzz/                      ← cargo-fuzz targets (nightly Linux only, excluded from workspace)
benches/                   ← Criterion benchmarks ([[bench]] in Cargo.toml)
```

## Key Conventions

- **Commit style**: Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`, `perf:`) — see `.github/CONTRIBUTING.md`
- **Changelog**: `docs/CHANGELOG.md` via `git-cliff` (config: `cliff.toml`)
- **Licensing**: `cargo-deny` configured in `deny.toml` (MIT/Apache-2.0 only)
- **Markdown linting**: `.markdownlint-cli2.yaml` — line length disabled, HTML `div`/`h1`/`p`/`br` allowed
- **WASM**: vanta-wasm binary uses `opt-level = "s"` + strip in release
- **No opencode.json**: only `package.json` with `@opencode-ai/plugin` dependency
