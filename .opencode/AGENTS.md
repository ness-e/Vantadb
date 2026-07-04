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

### CI / Hooks Integration

| Script | Qué hace |
|--------|----------|
| `dev-tools/verify.ps1` | Pre-flight completa (fmt → clippy → nextest → Python) — incluye `codegraph affected` al inicio |
| `dev-tools/verify_changed.ps1` | **Quick verify**: usa `codegraph affected` para testear solo archivos impactados. Ideal para iteración rápida |
| `.git/hooks/pre-commit` | Muestra preview no-blocking de tests afectados por staged changes |
| `.git/hooks/pre-push` | Corre `verify.ps1` completo antes de cada push |

**Flujo típico local:**
```
git add .                                           # stage changes
# pre-commit hook muestra preview de tests afectados
dev-tools/verify_changed.ps1                       # quick check (~30s)
git commit -m "feat: ..."                           # commit
# pre-push hook corre verify.ps1 completo (~2-5min)
git push
```
<!-- CODEGRAPH_END -->

<!-- UNDERSTAND_START -->
## Understand-Anything

Understand-Anything produce un **knowledge graph LLM-powered** (1917 nodos, 1120 edges, 32 capas, 14 tour steps) en `.understand-anything/knowledge-graph.json`. Complementa a CodeGraph para preguntas arquitectónicas y narrativa humana.

### CodeGraph vs Understand-Anything — Guía de decisión

| Situación | Herramienta | Por qué |
|-----------|------------|---------|
| "¿Dónde está definida la función X?" | **CodeGraph** | Index pre-construido, respuesta en ms |
| "¿Qué llama a esta función?" | `codegraph_explore` | Call paths precisos, resuelve dispatch dinámico |
| "¿Qué se rompe si cambio X?" | `codegraph impact X` | Blast radius exacto por AST |
| "¿Cómo está estructurada la arquitectura?" | **Understand-Anything** | 32 capas con descripciones narrativas |
| "Dame un tour del código base" | **Understand-Anything** | Tour guiado de 14 pasos desde entry point |
| "Explica este módulo en detalle" | `skill understand-explain` | Análisis narrativo contextual |
| "¿Qué tests ejecutar?" | `codegraph affected` | Determinístico, conectado al git diff |
| "Onboarding para nuevo dev" | `skill understand-onboard` | Genera guía de onboarding interactiva |
| "¿Cuál es el dominio de negocio?" | `skill understand-domain` | Extrae flujos de dominio del grafo |
| "¿Qué cambió en este PR?" | `skill understand-diff` | Analiza diff contra el grafo existente |

**Regla general**: CodeGraph primero para todo lo que sea símbolos/código preciso. Understand-Anything para contexto arquitectónico, narrativa, onboarding y visualización.

### Skills disponibles

Los skills están en `C:\Users\Eros\.agents\skills\` y se cargan con `skill <nombre>`:

| Skill | Comando | Qué hace |
|-------|---------|----------|
| `understand` | `/understand [path] [--full\|--review\|--auto-update]` | Pipeline completo: escanea, analiza y genera grafo |
| `understand-chat` | `/understand-chat [query]` | Responde preguntas sobre el codebase usando el grafo |
| `understand-dashboard` | `/understand-dashboard` | Lanza visor web interactivo del grafo |
| `understand-explain` | `skill understand-explain` | Explicación profunda de un archivo/función/módulo |
| `understand-diff` | `skill understand-diff` | Analiza git diff o PR contra el grafo existente |
| `understand-domain` | `skill understand-domain` | Extrae conocimiento de dominio de negocio del grafo |
| `understand-knowledge` | `skill understand-knowledge` | Analiza bases de conocimiento estilo wiki → grafo interactivo |
| `understand-onboard` | `skill understand-onboard` | Genera guía de onboarding para nuevos miembros |

### Estado actual

El grafo ya está generado en `.understand-anything/knowledge-graph.json` (commit `17171dd8`). Para regenerarlo o actualizarlo:

```
skill understand
/understand                    # incremental update si hay cambios
/understand --full             # rebuild completo
/understand --review           # auditoría LLM de calidad
/understand --auto-update      # actualización automática en cada commit
/understand --language es      # generar contenido en español
```

### Flujo recomendado: CodeGraph + Understand-Anything sin conflictos

1. **Para navegación diaria** → usa CodeGraph (`codegraph_explore`). Es más rápido, determinístico, y no gasta tokens LLM en re-análisis.
2. **Para entender arquitectura** → carga `understand-chat` y pregunta. El grafo ya existe, no necesita regenerarse.
3. **Para onboarding/review** → carga `understand-explain` o `understand-onboard`. Usan el grafo existente.
4. **Solo regenera si**: cambia la estructura del proyecto (nuevos módulos grandes) o quieres un análisis más fresco.
5. **NUNCA** ejecutes `/understand --full` a menos que sea necesario — el pipeline actual ya cubre 790 archivos y consumió ~158s de subagentes.

### Referencia del grafo

```json
{
  "nodes": [{"id": "file:src/engine.rs", "type": "file", "name": "engine.rs", "summary": "In-memory storage engine", "tags": ["storage", "core"]}],
  "edges": [{"source": "file:src/engine.rs", "target": "file:src/storage/mod.rs", "type": "imports", "direction": "directed", "weight": 0.7}],
  "layers": [{"id": "layer:core-engine", "name": "Core Engine", "description": "In-memory engine and storage backends", "nodeIds": ["file:src/engine.rs", ...]}],
  "tour": [{"order": 1, "title": "Project Overview", "description": "Start with README", "nodeIds": ["document:README.md"]}]
}
```

### Capas arquitectónicas (32 total)

Las principales: `core-engine`, `storage-backends`, `vector-index`, `web-frontend`, `python-bindings`, `typescript-sdk`, `integration-wrappers`, `dev-tooling`, `tests`, `documentation`, `ci-cd`, `wasm`, `enterprise`, `mcp`.
<!-- UNDERSTAND_END -->

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

:: Quick local verify (CodeGraph-optimized, ~30s)
dev-tools/verify_changed.ps1
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

:: Experimental tests (parser, executor, governor) — NOTE: experimental-lisp and experimental-governance deleted Jul 2026

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
- **OpenCode MCP config**: `opencode.jsonc` at root (CodeGraph MCP server)
- **CodeGraph CI hooks**: verify.ps1/verify_changed.ps1 + pre-commit/pre-push hooks

## MCP Servers Disponibles

Configurados globalmente en `%USERPROFILE%\.config\opencode\opencode.json`.

### Activos

| MCP | Comando | Propósito |
|-----|---------|-----------|
| **CodeGraph** | `codegraph serve --mcp` | Grafo de conocimiento del código (7.3K símbolos). Resuelve símbolos, flujos, blast radius |
| **Pencil** | `mcp-server-windows-x64.exe` | Editor de archivos `.pen` — diseño UI visual, reemplazo de Figma |
| **Playwright** | `@playwright/mcp` | Automatización de navegador: navegar, click, screenshot, snapshot, redes |
| **Recraft** | `@recraft-ai/mcp-recraft-server` | Generación/edición de imágenes por IA (upscale, vectorizar, remover fondo) |
| **cargo-mcp** | `cargo-mcp serve` | Ejecutar comandos Cargo: `check`, `clippy`, `test`, `build`, `fmt`, `add`, `remove`, `bench`, `run` |
| **rust-analyzer-mcp** | `rust-analyzer-mcp` | LSP completo: goto def, hover, references, completions, diagnostics, rename, format |
| **rust-mcp-server** | `rust-mcp-server` | Bridge completo: build, test, deps, clippy, doc, project management, dependency management |

### Referencia rápida para agentes

- Para preguntas de código → **CodeGraph** (siempre primero, antes de grep/read)
- Para diseño UI/visual → **Pencil** (archivos `.pen`)
- Para web scraping/testing → **Playwright**
- Para generar/editar imágenes → **Recraft** (requiere `RECRAFT_API_KEY`)
- Para tareas Rust/Rust → **cargo-mcp**, **rust-analyzer-mcp**, **rust-mcp-server**
