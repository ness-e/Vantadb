# Architecture

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar para preguntas estructurales de la workspace. Si editas, actualiza también el puntero en AGENTS.md.

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
