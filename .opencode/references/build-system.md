# Build System

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar cuando toques Cargo.toml, profiles, features o el build de la workspace. Si editas, actualiza también el puntero en AGENTS.md.

## Build System

- **Rust**: stable (rust-toolchain.toml: `1.94.1`+)
- **Profile `ci`** (no LTO, opt-level=2, 16 codegen-units) — used by CI Fast Gate
- **Profile `release`** (thin LTO, opt-level=3, 1 codegen-unit)
- **Profile `dev`** (opt-level=1, debug=0) — faster local iteration
- **Profile `test`** (opt-level=0, debug=0)
- **Profile `audit`** — used by nextest for pre-flight/release validation
- **Windows MSVC stack overflow workaround**: Always pass `--build-jobs 2` to nextest
- **Windows linker**: `.cargo/config.toml` forces `link.exe` (rust-lld causes STATUS_STACK_BUFFER_OVERRUN with large crates)

## Rust Build Optimization

`jobs = 2` en `.cargo/config.toml` es necesario por RAM (sin cambios de código posibles).
Estrategias para mantener `cargo check` rápido:

**Sin cambiar código (workflow):**

| Comando | Por qué es más rápido |
|---------|----------------------|
| `cargo check -p vantadb` | Solo la crate core, ignora las otras 15 del workspace |
| `cargo check -p vantadb -p vantadb-server -p vantadb-mcp` | Solo las 3 que tocas |
| `cargo check -p vantadb --no-default-features -F "fjall,cli"` | Excluye rocksdb, arrow, tantivy, server, prometheus |
| `cargo check -p vantadb` (sin flag) | El profile `check` nativo ya usa opt-level=0, debug=0, codegen-units=256 |
| `cargo check --timings -p vantadb` | Genera HTML con el desglose exacto de cada crate |
| `cargo check --workspace --exclude vantadb-langchain --exclude vantadb-ollama --exclude vantadb-openai --exclude vantadb-litellm --exclude vantadb-haystack --exclude vantadb-dspy --exclude vantadb-crewai --exclude vantadb-letta --exclude vantadb-mem0 --exclude vantadb-llamaindex` | Workspace completo sin los 10 adapters (cada uno tira pyo3) |

**Prioridad: `-p vantadb` es el que más impacto da.** Los adapters casi nunca cambian.

## Default Features

`cli` + `arrow` + `fjall` + `roaring` + `advanced-tokenizer` + `memmap2` + `fs2` + `sysinfo` + `rayon`

(`rocksdb` y `prometheus` NO están en default — activarlos opt-in cuando se necesiten.)

Key optional features:
- `failpoints` — required for `chaos_integrity` test
- `remote-inference` — enables `llm` module (reqwest-based)
- `server` — enables axum HTTP server + tokio
- `python_sdk` — enables PyO3 bindings
