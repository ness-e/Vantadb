# Operations & Configuration Manual

This document tracks the current runtime knobs for the embedded core and the optional local server wrapper.

## 1. VantaConfig Reference

All configuration fields available in `VantaConfig` (Rust) and via environment variables.

| Field | Type | Default | Env Var | Description |
|-------|------|---------|---------|-------------|
| `storage_path` | `String` | `vantadb_data` | `VANTADB_STORAGE_PATH` | Filesystem path for embedded data directory |
| `host` | `String` | `127.0.0.1` | `VANTADB_HOST` (fallback `HOST`) | Bind address for HTTP server |
| `port` | `u16` | `8080` | `VANTADB_PORT` (fallback `PORT`) | TCP port for HTTP server |
| `memory_limit` | `Option<u64>` | `None` | — | Memory budget hint for backend and mmap selection |
| `read_only` | `bool` | `false` | — | Opens engine in read-only mode |
| `force_mmap` | `bool` | `false` | — | Force memory-mapped I/O for vector store |
| `mmap_hnsw` | `bool` | `true` | — | Enable memory-mapped HNSW index |
| `prefetch_mode` | `PrefetchMode` | `Auto` | `VANTA_PREFETCH`, `VANTA_DISABLE_PREFETCH` | MMap prefetch strategy (Auto/Enabled/Disabled) |
| `rss_threshold` | `f64` | `0.80` | — | RSS pressure threshold for backpressure eviction (0.0-1.0) |
| `eviction_weight_hits` | `f64` | `1.0` | — | Weight for access frequency in eviction score |
| `eviction_weight_confidence` | `f64` | `2.0` | — | Weight for confidence score in eviction |
| `eviction_weight_importance` | `f64` | `3.0` | — | Weight for importance score in eviction |
| `eviction_weight_recency` | `f64` | `1.0` | — | Weight for recency in eviction |
| `eviction_ratio` | `f64` | `0.20` | — | Fraction of cold nodes to evict per cycle |
| `backend_kind` | `BackendKind` | `Fjall` | `VANTA_BACKEND` | KV backend: `fjall`, `rocksdb`, `memory` |
| `max_blocking_threads` | `usize` | `16` | `VANTADB_MAX_BLOCKING_THREADS` | Max threads for blocking thread pool |
| `sync_mode` | `SyncMode` | `Periodic` | — | WAL sync: `Always`, `Periodic`, `Never` |
| `api_key` | `Option<String>` | `None` | `VANTADB_API_KEY` | Bearer token for HTTP auth |
| `rate_limit_rpm` | `u32` | `100` | `VANTADB_RATE_LIMIT_RPM` | Rate limit in requests per minute |
| `tls_cert_path` | `Option<String>` | `None` | `VANTADB_TLS_CERT` | Path to TLS certificate PEM file |
| `tls_key_path` | `Option<String>` | `None` | `VANTADB_TLS_KEY` | Path to TLS private key PEM file |
| `log_format` | `LogFormat` | `Compact` | `VANTADB_LOG_FORMAT`, `VANTADB_LOG_JSON` | Log output: `compact`, `json`, `full` |
| `llm_url` | `String` | `http://localhost:11434` | `VANTA_LLM_URL` | Ollama endpoint for remote embeddings |
| `llm_model` | `String` | `all-minilm` | `VANTA_LLM_MODEL` | Model name for embeddings |
| `llm_summarize_model` | `String` | `llama3` | `VANTA_LLM_SUMMARIZE_MODEL` | Model name for summarization |
| `advanced_tokenizer_config` | `Option<...>` | `None` | — | Advanced tokenizer config (feature-gated) |

### Enums

| Enum | Variants | Description |
|------|----------|-------------|
| `LogFormat` | `Compact`, `Json`, `Full` | Log output format |
| `SyncMode` | `Always` (fsync every write), `Periodic` (fsync every 5s), `Never` | WAL durability sync mode |
| `PrefetchMode` | `Auto` (detect), `Enabled`, `Disabled` | MMap prefetch strategy |
| `BackendKind` | `Fjall` (default), `RocksDb`, `InMemory` | KV storage backend |

## 2. Python Constructor

```python
import vantadb_py as vantadb

db = vantadb.VantaDB(
    "./vanta_data",
    read_only=False,
    memory_limit_bytes=512_000_000,
    backend=None,     # "rocksdb", "memory", or None (fjall)
)
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `db_path` | `str` | required | Filesystem path (maps to `storage_path`) |
| `read_only` | `bool` | `False` | Opens the engine in read-only mode |
| `memory_limit_bytes` | `int \| None` | `None` | Memory budget hint (maps to `memory_limit`) |
| `backend` | `str \| None` | `None` | Backend selection: `"rocksdb"`, `"memory"`, or `None` (fjall) |

## 3. Embedded Runtime Notes

- Fjall is the default storage backend.
- RocksDB remains an explicit fallback path in the core.
- Vector search is cosine-based HNSW.
- Memory records use `namespace + key` identity with scalar metadata and optional vectors.
- Derived namespace/payload indexes are persisted and rebuilt from canonical records.

## 4. Embedded CLI

The CLI uses the embedded core directly and does not require the optional HTTP server.

### Global Flags

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--db` / `-d` | `VANTA_DB` | `./db` | Path to the database directory |
| `--verbose` / `-v` | — | `false` | Enable verbose output |

### Commands

| Command | Description |
|---------|-------------|
| `put --namespace <ns> --key <k> --payload <text> [--vector <v>]` | Save a key-value pair to persistent memory |
| `get --namespace <ns> --key <k>` | Retrieve a value from persistent memory |
| `delete --namespace <ns> --key <k>` | Delete a record by namespace and key |
| `list --namespace <ns> [--limit <N>]` | List keys and values in a namespace |
| `search --namespace <ns> --query <q> [--query_vector <v>] [--limit <N>] [--json]` | Search records semantically across a namespace |
| `query <iql_string> [--limit <N>]` | Execute a structured IQL/hybrid query |
| `status` | Display database health diagnostics and system status |
| `rebuild-index` | Rebuild all database indexes (HNSW, text index, derived indexes) |
| `audit-index [--namespace <ns>] [--json] [--deep]` | Validate text index integrity without repairing |
| `repair-text-index` | Repair text index if inconsistencies are detected |
| `export [--namespace <ns>] --out <path>` | Export records to a JSON file |
| `import --in <path>` | Import records from a JSON file |
| `namespace list` | List all namespaces |
| `namespace info --namespace <ns>` | Show record count and details for a namespace |
| `server [--http] [--mcp] [--port <N>] [--host <host>]` | Start the HTTP or MCP server wrapper |
| `completions --shell <bash|zsh|fish|powershell>` | Generate shell completion scripts |

### Examples

```bash
vanta-cli put --db ./vanta_data --namespace agent/main --key memory-1 --payload "hello"
vanta-cli get --db ./vanta_data --namespace agent/main --key memory-1
vanta-cli list --db ./vanta_data --namespace agent/main
vanta-cli search --db ./vanta_data --namespace agent/main --query "hello world" --limit 5
vanta-cli status --db ./vanta_data
vanta-cli audit-index --db ./vanta_data --deep
vanta-cli rebuild-index --db ./vanta_data
vanta-cli export --db ./vanta_data --namespace agent/main --out ./agent-main.jsonl
vanta-cli import --db ./vanta_data --in ./agent-main.jsonl
vanta-cli namespace list --db ./vanta_data
vanta-cli namespace info --db ./vanta_data --namespace agent/main
vanta-cli server --http --port 8080 --db ./vanta_data
vanta-cli completions --shell powershell
```

## 5. Operational Metrics

The embedded SDK exposes diagnostic metrics for:

- startup duration
- WAL replay duration and records replayed
- ANN and derived-index rebuild duration
- exported/imported record counts
- import errors
- HNSW logical bytes and mmap resident bytes
- lexical queries, hybrid queries, planner routes

These metrics are for engineering decisions and reliability gates. They should not be presented as memory-footprint or competitive benchmark claims.

## 6. Memory Telemetry Caveat

Current telemetry must be interpreted carefully:

- process memory and logical index memory are tracked separately
- process-scoped metrics do not equal mmap residency or page cache
- memory claims should use the contract in [MEMORY_TELEMETRY.md](MEMORY_TELEMETRY.md)

## 7. Cargo Features

Build-time feature flags in `Cargo.toml`:

| Feature | Deps Enabled | Description |
|---------|-------------|-------------|
| `default` | `cli`, `arrow`, `rocksdb`, `fjall`, `sysinfo`, `memmap2`, `fs2`, `prometheus`, `rayon` | Default feature set for production |
| `cli` | `indicatif`, `console`, `clap`, `clap_complete`, `rustyline`, `strsim`, `color-eyre` | CLI binary + console UX |
| `server` | `cli` + `tokio`, `axum`, `tower`, `tower_governor`, `tower-http` | HTTP/MCP server |
| `tls` | `axum-server`, `rustls-pemfile` | TLS for HTTP server |
| `python_sdk` | `pyo3` | Python bindings via PyO3 |
| `wasm` | *(none — shim-based)* | WASM build support (wasm32-wasip1 / wasm32-unknown-unknown) |
| `advanced-tokenizer` | `tantivy` | Multilingual tokenizer with stemming/stopwords |
| `remote-inference` | `reqwest` | LLM inference over HTTP (Ollama) |
| `opentelemetry` | `opentelemetry`, `tracing-opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` | OpenTelemetry tracing/export |
| `rocksdb` | `rocksdb` | RocksDB backend |
| `fjall` | `fjall` | Fjall backend (default) |
| `arrow` | `arrow` | Apache Arrow IPC support |
| `rkyv-serialization` | `rkyv` | Zero-copy rkyv archives for HNSW |
| `failpoints` | `fail` | Fault-injection testing |
| `custom-allocator` | `mimalloc` | mimalloc global allocator |

## 8. SIMD and Build Behavior

VantaDB still uses the runtime hardware profile to choose fast paths where available, but public claims should stay tied to validated behavior rather than to a specific SIMD tier alone.
