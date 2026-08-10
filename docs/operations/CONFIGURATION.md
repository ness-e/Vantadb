---
title: "Operations & Configuration Manual"
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-07-07
aliases: []
---

# Operations & Configuration Manual

This document tracks the current runtime knobs for the embedded core and the optional local server wrapper.

## 1. VantaConfig Reference

All configuration fields available in `VantaConfig` (Rust) and via environment variables.

| Field | Type | Default | Env Var | Description |
|-------|------|---------|---------|-------------|
| `storage_path` | `String` | `vantadb_data` | `VANTADB_STORAGE_PATH` | Filesystem path for embedded data directory |
| `host` | `String` | `127.0.0.1` | `VANTADB_HOST` (fallback `HOST`) | Bind address for HTTP server |
| `port` | `u16` | `8080` | `VANTADB_PORT` | TCP port for HTTP server |
| `memory_limit` | `Option<u64>` | `None` | — | Memory budget hint for backend and mmap selection |
| `read_only` | `bool` | `false` | — | Opens engine in read-only mode |
| `force_mmap` | `bool` | `false` | — | Force memory-mapped I/O for vector store |
| `mmap_hnsw` | `bool` | `true` | — | Enable memory-mapped [[hnsw\|HNSW]] index |
| `prefetch_mode` | `PrefetchMode` | `Disabled` | `VANTA_PREFETCH`, `VANTA_DISABLE_PREFETCH` | MMap prefetch strategy (Auto/Enabled/Disabled; default OFF, PERF-04) |
| `rss_threshold` | `f64` | `0.80` | — | RSS pressure threshold for backpressure eviction (0.0-1.0) |
| `eviction_weight_hits` | `f64` | `1.0` | — | Weight for access frequency in eviction score |
| `eviction_weight_confidence` | `f64` | `2.0` | — | Weight for confidence score in eviction |
| `eviction_weight_importance` | `f64` | `3.0` | — | Weight for importance score in eviction |
| `eviction_weight_recency` | `f64` | `1.0` | — | Weight for recency in eviction |
| `eviction_ratio` | `f64` | `0.20` | — | Fraction of hot nodes to evict when memory pressure triggers |
| `backend_kind` | `BackendKind` | `Fjall` | `VANTA_BACKEND` | KV backend: `[[fjall]]`, `[[rocksdb]]`, `memory` |
| `max_blocking_threads` | `usize` | `16` | `VANTADB_MAX_BLOCKING_THREADS` | Max threads for blocking thread pool |
| `max_connections` | `usize` | `max_blocking_threads * 2` | `VANTADB_MAX_CONNECTIONS` | Max concurrent HTTP query pool permits |
| `pool_acquire_timeout_ms` | `u64` | `5000` | `VANTADB_POOL_ACQUIRE_TIMEOUT_MS` | Timeout acquiring a pool permit before the query fails fast with 503 |
| `circuit_breaker_failure_threshold` | `u32` | `5` | `VANTADB_CIRCUIT_BREAKER_FAILURE_THRESHOLD` | Consecutive 5xx failures before the circuit breaker opens |
| `circuit_breaker_open_timeout_secs` | `u64` | `30` | `VANTADB_CIRCUIT_BREAKER_OPEN_TIMEOUT_SECS` | Seconds the breaker stays open before probing half-open |
| `sync_mode` | `SyncMode` | `Periodic` | — | [[wal\|WAL]] sync: `Always`, `Periodic`, `Never` |
| `insert_lock_timeout_ms` | `u64` | `5000` | `VANTADB_INSERT_LOCK_TIMEOUT_MS` | [[hnsw\|HNSW]] insert lock timeout in ms |
| `file_lock_timeout_ms` | `u64` | `1000` | `VANTADB_FILE_LOCK_TIMEOUT_MS` | .vanta.lock file lock timeout in ms |
| `api_key` | `Option<String>` | `None` | `VANTADB_API_KEY` | Bearer token for HTTP auth |
| `rate_limit_rpm` | `u32` | `100` | `VANTADB_RATE_LIMIT_RPM` | Rate limit in requests per minute |
| `trusted_proxies` | `Vec<IpAddr>` | `[]` | `VANTADB_TRUSTED_PROXIES` | Comma-separated reverse-proxy IPs whose `X-Forwarded-For` header is honored for client-IP resolution (rate limiter / logs). Empty = header ignored; direct socket addr is authoritative (clients cannot spoof their IP). |
| `allowed_origins` | `Vec<String>` | `[]` | `VANTADB_ALLOWED_ORIGINS` | Comma-separated origins allowed to make cross-origin (CORS) requests to the HTTP server (e.g. `https://app.example.com,https://admin.example.com`). Empty (default) = CORS middleware omitted; the server sends no `Access-Control-Allow-Origin` header and browsers block cross-origin web calls. Repeatable via `VantaConfig::with_allowed_origins`. |
| `tls_cert_path` | `Option<String>` | `None` | `VANTADB_TLS_CERT` | Path to TLS certificate PEM file |
| `tls_key_path` | `Option<String>` | `None` | `VANTADB_TLS_KEY` | Path to TLS private key PEM file |
| `log_format` | `LogFormat` | `Compact` | `VANTADB_LOG_FORMAT`, `VANTADB_LOG_JSON` | Log output: `compact`, `json`, `full` |
| `llm_url` | `String` | `http://localhost:11434` | `VANTA_LLM_URL` | Ollama endpoint for remote embeddings |
| `llm_model` | `String` | `all-minilm` | `VANTA_LLM_MODEL` | Model name for embeddings |
| `llm_summarize_model` | `String` | `llama3` | `VANTA_LLM_SUMMARIZE_MODEL` | Model name for summarization |
| `wal_shards` | `usize` | `4` | `VANTADB_WAL_SHARDS` | Number of round-robin [[wal\|WAL]] shard files for write parallelism |
| `wal_buffer_size` | `Option<usize>` | `65536` (64KB) | `VANTADB_WAL_BUFFER_SIZE` | Per-shard WAL buffer in bytes (`None` = OS default) |
| `flush_threshold` | `Option<usize>` | `10000` | `VANTADB_FLUSH_THRESHOLD` | Auto-flush after N nodes inserted (`None` = disabled) |
| `advanced_tokenizer_config` | `Option<...>` | `None` | — | Advanced tokenizer config (feature-gated) |
| `batch_size` | `Option<usize>` | `None` (1000) | `VANTADB_BATCH_SIZE` | Max nodes per batch ingestion operation |
| `bulk_commit_interval` | `Option<usize>` | `None` (10000) | `VANTADB_BULK_COMMIT_INTERVAL` | Number of records per batch commit during bulk import |
| `encryption_key` | `Option<String>` | `None` | `VANTADB_ENCRYPTION_KEY` | AES-256-GCM key (hex 32-byte) for at-rest encryption (feature-gated: `encryption`) |
| `flat_threshold` | `Option<usize>` | `10000` | `VANTADB_FLAT_THRESHOLD` | Brute-force flat scan threshold; ≤ this many nodes skips HNSW |
| `hot_reload_config` | `Arc<RwLock<HotReloadConfig>>` | `HotReloadConfig::default()` | — | Hot-reloadable config snapshot (feature-gated: `hot-reload`, not in `default` features). See [Hot-Reload JSON](#hot-reload-json) |
| `rbac_config` | `RbacConfig` | `{ token_role_map: {} }` | — | RBAC config mapping API tokens to roles |
| `require_auth` | `bool` | `false` | `VANTADB_REQUIRE_AUTH` | Refuse to start unless `api_key` is configured |
| `token_role_map` | `HashMap<String, String>` | `{}` | — | `RbacConfig` field: token → role name mapping |
| `export_base_dir` | `Option<PathBuf>` | `None` | `VANTADB_EXPORT_BASE_DIR` | Base directory for export/import path validation. When set, export and import paths are resolved canonically against this directory (symlink protection included). When `None`, only bare `..` traversal is blocked. |
| `audit_log_path` | `Option<PathBuf>` | `None` | `VANTADB_AUDIT_LOG_PATH` | Append-only JSONL audit log (ISO 8601 timestamp + op per write/delete/export/import). When `None`, audit is disabled. |
| `segment_optimizer` | `SegmentOptimizerConfig` | `{enabled: true, vacuum_threshold_pct: 15.0, auto_run_interval_secs: 3600, max_pipeline_duration_secs: 300}` | — | Segment optimizer configuration: master switch, tombstone vacuum threshold (%), auto-run interval (s), max pipeline duration (s), and per-level LSM compaction config. See also `pipeline()` / `optimizer_config()` / `set_optimizer_config()` in the SDK. |

### Audit Log Format

Each line is one JSON object:

```json
{"timestamp":"2026-08-02T12:34:56Z","op":"put","namespace":"docs","key":"a","outcome":"ok","reason":null}
```

- `timestamp`: ISO 8601 UTC (RFC 3339, second precision).
- `op`: `put`, `put_batch`, `delete`, `delete_by_filter`, `export_namespace`, `export_all`, `import_file`.
- `outcome`: `ok` or `err`. Failures still record the attempt; error details go to the `reason` field where available.
- `reason`: optional contextual detail (e.g. `memory delete` on delete, deleted count on `delete_by_filter`).
- Read-only operations (`search`, `get`, `list`) are **not** audited.
- The file is opened in append mode; each record is flushed immediately. Set `VANTADB_AUDIT_LOG_PATH` or `VantaConfig.audit_log_path` via the Rust builder `with_audit_log_path(path)` to enable.

### Enums

| Enum | Variants | Description |
|------|----------|-------------|
| `LogFormat` | `Compact`, `Json`, `Full` | Log output format |
| `SyncMode` | `Always` (fsync every write), `Periodic` (fsync every 5s), `Never` | [[wal\|WAL]] durability sync mode |
| `PrefetchMode` | `Disabled` (default), `Enabled`, `Auto` (behaves like Enabled) | MMap prefetch strategy; default OFF (PERF-04) |
| `BackendKind` | `[[fjall\|Fjall]]` (default), `[[rocksdb\|RocksDb]]`, `InMemory` | KV storage backend |

### Builder API

Configuration in Rust uses the builder pattern: `VantaConfig::default()` reads **all** environment
variables listed above, `VantaConfig::from_env()` is a thin alias of `default()`, and each `with_*`
method overrides a single field on the returned builder. Builder methods are additive — fields you do
not touch keep their env-var or default value. **No `config.toml` (or any other config file) is read
at startup.** The only file-based mechanism is the optional JSON hot-reload watcher described in
[Hot-Reload JSON](#hot-reload-json), which applies a subset of fields at runtime and never replaces
startup configuration.

```rust
use vantadb::VantaConfig;

let cfg = VantaConfig::from_env()
    .with_storage_path("./vanta_data".to_string())
    .with_memory_limit(512_000_000)
    .with_wal_buffer_size(64 * 1024)
    .with_flush_threshold(10_000)
    .with_flat_threshold(Some(10_000))
    .with_tls("cert.pem".to_string(), "key.pem".to_string())
    .with_audit_log_path("./audit.jsonl");
```

### Hot-Reload JSON

Behind the `hot-reload` Cargo feature (not included in `default`; pulls in `notify`),
`VantaConfig::watch_config(config, path, on_reload)` spawns a background thread that watches a single
JSON file and atomically applies safe-to-reload fields at runtime. It is the **only file-based
configuration mechanism**; there is no `config.toml` and no config file is read at startup.

Mechanism (`#[cfg(feature = "hot-reload")]`, `src/config.rs`):

- `watch_config(config: Arc<RwLock<VantaConfig>>, path, on_reload)` returns
  `io::Result<mpsc::Sender<()>>`. Dropping the returned `Sender` shuts the watcher thread down.
- The file is parsed as **JSON only** (`serde_json`); invalid JSON or files larger than 1 MB are
  ignored with a warning. The parent directory is watched non-recursively; only
  `Modify(Data)` events on the exact path trigger a reload.
- Only the safe `HotReloadConfig` subset is applied — `prefetch_mode`, `log_format`,
  `rate_limit_rpm`, `batch_size`, `wal_buffer_size`, `flush_threshold`, `insert_lock_timeout_ms`,
  `sync_mode`. Changes to storage paths, backend, TLS, API keys, or audit settings are ignored.
- `on_reload` is invoked only when at least one field actually changed.

Example file (all keys optional):

```json
{
  "prefetch_mode": "enabled",
  "log_format": "json",
  "rate_limit_rpm": 100,
  "batch_size": 1000,
  "wal_buffer_size": 65536,
  "flush_threshold": 10000,
  "insert_lock_timeout_ms": 5000,
  "sync_mode": "periodic"
}
```

| Key | Type | Accepted values |
|-----|------|-----------------|
| `prefetch_mode` | string | `auto`, `enabled`, `disabled` |
| `log_format` | string | `compact`, `json`, `full` |
| `rate_limit_rpm` | u32 | 0 = disabled |
| `batch_size` | number \| `null` | `null` → `None` |
| `wal_buffer_size` | number \| `null` | bytes; `null` → `None` |
| `flush_threshold` | number \| `null` | node count; `null` → `None` |
| `insert_lock_timeout_ms` | u64 | ms |
| `sync_mode` | string | `always`, `never`; anything else → `periodic` |

```rust
use std::sync::{Arc, RwLock};
use vantadb::VantaConfig;

let cfg = Arc::new(RwLock::new(VantaConfig::from_env()));
let watcher = VantaConfig::watch_config(
    Arc::clone(&cfg),
    "./hot-reload.json",
    || tracing::info!("config hot-reloaded"),
)
.expect("failed to start hot-reload watcher");
// later: drop(watcher) stops the background thread
```

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
*Note: Available backends include [[rocksdb]] and [[fjall]] (default).*

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `db_path` | `str` | required | Filesystem path (maps to `storage_path`) |
| `read_only` | `bool` | `False` | Opens the engine in read-only mode |
| `memory_limit_bytes` | `int \| None` | `None` | Memory budget hint (maps to `memory_limit`) |
| `backend` | `str \| None` | `None` | Backend selection: `"[[rocksdb]]"`, `"memory"`, or `None` ([[fjall]]) |

## 3. Embedded Runtime Notes

- [[fjall|Fjall]] is the default storage backend.
- [[rocksdb|RocksDB]] remains an explicit fallback path in the core.
- Vector search is cosine-based [[hnsw|HNSW]].
- Memory records use `namespace + key` identity with scalar metadata and optional vectors.
- Derived namespace/payload indexes are persisted and rebuilt from canonical records.

## 4. Embedded CLI

The CLI uses the embedded core directly and does not require the optional HTTP server.

### Global Flags

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--db` / `-d` | `VANTA_DB` | `./db` | Path to the database directory |
| `--verbose` / `-v` | — | `false` | Enable verbose output |
| `--json` | — | `false` | Output in JSON format |
| `--quiet` | — | `false` | Suppress non-essential output |

> **Note (ADR-012, 2026-08-05):** `VANTA_DB` is the CLI flag env (clap, global). `VANTADB_STORAGE_PATH` is the config env (`VantaConfig::from_env`). Precedence: CLI flag `--db` > `VANTA_DB` env > `VANTADB_STORAGE_PATH` > defaults. The `vantadb-server` child sets **both** vars from `--db` so the MCP/config path resolves correctly (fix TECH-01). See `docs/architecture/adr/012_env_var_naming.md`.

### Commands

| Command | Description |
|---------|-------------|
| `put --namespace <ns> --key <k> --payload <text> [--vector <v>]` | Save a key-value pair to persistent memory |
| `get --namespace <ns> --key <k>` | Retrieve a value from persistent memory |
| `delete --namespace <ns> --key <k>` | Delete a record by namespace and key |
| `delete-by-filter --namespace <ns> --filter <json>` | Delete records matching metadata filters |
| `count [--namespace <ns>] [--filter <json>]` | Count records, optionally filtered |
| `list --namespace <ns> [--limit <N>]` | List keys and values in a namespace |
| `search --namespace <ns> --query <q> [--query-vector <v>] [--limit <N>] [--json]` | Search records semantically across a namespace |
| `search-multi --namespaces <ns1,ns2> --query <q> [--query-vector <v>] [--top-k <N>] [--json]` | Search across multiple namespaces and merge results by score |
| `search-all --query <q> [--query-vector <v>] [--top-k <N>] [--json]` | Search across ALL known namespaces and merge results by score |
| `similar-to-key --namespace <ns> --key <k> [--top-k <N>] [--json]` | Find records similar to a given key using vector similarity search |
| `query <iql_string> [--limit <N>]` | Execute a structured IQL/hybrid query |
| `status` | Display database health diagnostics and system status |
| `stats [--json]` | Database statistics (formatted or JSON) |
| `doctor` | Health diagnostics (WAL, backend, memory, HNSW) |
| `inspect --key <k>` | Inspect a complete record |
| `rebuild-index` | Rebuild all database indexes (HNSW, text index, derived indexes) |
| `audit-index [--namespace <ns>] [--json] [--deep]` | Validate text index integrity without repairing |
| `repair-text-index` | Repair text index if inconsistencies are detected |
| `backup --out <path>` | Full backup with WAL flush, file copy, CRC32 manifest |
| `restore --from <path> [--rebuild]` | Restore from backup, verify CRC32, optional rebuild |
| `check [--namespace <ns>]` | Validate database structural integrity |
| `migrate [--target-version <v>]` | Migrate storage format between versions |
| `plan` | Preview migration steps without executing |
| `run` | Execute a pre-planned migration |
| `export [--namespace <ns>] --out <path>` | Export records to a JSONL file |
| `import --in <path>` | Import records from a JSONL file |
| `namespace list` | List all namespaces |
| `namespace info --namespace <ns>` | Show record count and details for a namespace |
| `snapshot create --name <name>` | Create an instant filesystem snapshot by hard-linking all data files (copy on Windows) |
| `snapshot list` | List all existing snapshots |
| `wal compact` | Compact the WAL: flush all data, archive the current WAL file, and start a fresh one |
| `wal vacuum` | Remove tombstoned nodes from HNSW and reclaim space |
| `server [--http] [--mcp] [--port <N>] [--host <host>]` | Start the HTTP or MCP server wrapper |
| `repl` | Interactive rustyline REPL with tab autocomplete |
| `tui` | Live dashboard refreshing every 2s |
| `completions --shell <bash|zsh|fish|powershell>` | Generate shell completion scripts |

### Examples

```bash
vanta-cli put --db ./vanta_data --namespace agent/main --key memory-1 --payload "hello"
vanta-cli get --db ./vanta_data --namespace agent/main --key memory-1
vanta-cli list --db ./vanta_data --namespace agent/main
vanta-cli search --db ./vanta_data --namespace agent/main --query "hello world" --query-vector "0.1,0.2,0.3" --limit 10
vanta-cli search-similar --db ./vanta_data --namespace agent/main --key memory-1 --limit 5
vanta-cli count --db ./vanta_data --namespace agent/main
vanta-cli status --db ./vanta_data
vanta-cli stats --db ./vanta_data --json
vanta-cli doctor --db ./vanta_data
vanta-cli audit-index --db ./vanta_data --deep
vanta-cli rebuild-index --db ./vanta_data
vanta-cli backup --db ./vanta_data --out ./vanta_data.bak
vanta-cli export --db ./vanta_data --namespace agent/main --out ./agent-main.jsonl
vanta-cli import --db ./vanta_data --in ./agent-main.jsonl
vanta-cli namespace list --db ./vanta_data
vanta-cli namespace info --db ./vanta_data --namespace agent/main
vanta-cli server --http --port 8080 --db ./vanta_data
vanta-cli repl --db ./vanta_data
vanta-cli tui --db ./vanta_data
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
| `default` | `cli`, `arrow`, `fjall`, `roaring`, `advanced-tokenizer`, `memmap2`, `fs2`, `sysinfo`, `rayon` | Default feature set for production |
| `cli` | `indicatif`, `console`, `clap`, `clap_complete` | CLI binary + console UX |
| `server` | `cli` + `tokio`, `axum`, `tower`, `tower_governor`, `tower-http` | HTTP/MCP server |
| `tls` | `axum-server`, `rustls` | TLS for HTTP server |
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

## 8. OpenTelemetry Tracing Environment

Activated by building with `--features opentelemetry`. All spans are exported via OTLP (gRPC) to a collector.

| Env Var | Default | Description |
|---------|---------|-------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://localhost:4317` | OTLP gRPC endpoint |
| `OTEL_SERVICE_NAME` | `vantadb-server` | Logical service name for trace identification |
| `RUST_LOG` | `info` | Tracing filter (`trace`, `debug`, `warn`, `error`, or module-level like `vantadb=debug`) |

Span coverage includes:
- All public SDK methods (`VantaMemory::put`, `get`, `search`, etc.) — `src/sdk.rs`
- All CLI command handlers (`cmd_put`, `cmd_get`, etc.) — `src/cli_handlers/`
- HTTP route handlers (`/health`, `/metrics`, `/api/v2/query`) — `src/cli_server.rs`

## 9. SIMD and Build Behavior

VantaDB still uses the runtime hardware profile to choose fast paths where available, but public claims should stay tied to validated behavior rather than to a specific SIMD tier alone.
