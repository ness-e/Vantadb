# VantaDB Configuration Guide

> Verified against `src/config.rs`. VantaDB is configured exclusively through **environment variables** (`VANTADB_*` / `VANTA_*`) and CLI flags — there is **no `config.json`** and no config file for the engine or the MCP server. All variables have sensible defaults; set only what you need to override.

## Storage

### VANTADB_STORAGE_PATH

Storage directory for the database. Used by `vantadb-server` (e.g. `vantadb-server --mcp`) and embedded engines that load config from the environment.

```bash
export VANTADB_STORAGE_PATH=/custom/path
```

Default: `vantadb_data` (relative to the working directory).

> Note: the legacy name `VANTADB_PATH` does not exist. The CLI's `--db` flag (env `VANTA_DB`, default `./db`) is separate and only affects `vanta-cli`.

### VANTA_BACKEND

Key-value storage backend. Accepted values: `fjall` (default), `rocksdb`, `memory`.

```bash
export VANTA_BACKEND=fjall
```

### VANTA_PREFETCH

Mmap vector page prefetch during HNSW search. Accepted: `enabled`, `disabled` (default), `auto`.

```bash
export VANTA_PREFETCH=disabled
```

## Memory & Performance

### VANTADB_MEMORY_LIMIT

Optional memory limit in bytes. Accepts suffixes: `KB`, `MB`, `GB` (also `KiB`, `MiB`, `GiB`), e.g. `500MB` or `2GB`.

```bash
export VANTADB_MEMORY_LIMIT=1073741824
```

### VANTADB_MAX_BLOCKING_THREADS

Maximum blocking threads for the async runtime. Default: `available_parallelism() * 2`.

### VANTADB_MAX_CONNECTIONS

Maximum concurrent connections for the HTTP query pool. Default: `max_blocking_threads * 2`.

### VANTADB_INSERT_LOCK_TIMEOUT_MS / VANTADB_FILE_LOCK_TIMEOUT_MS

Insert spin-lock timeout (default 5000 ms) and process file-lock timeout (default 1000 ms).

### VANTADB_WAL_SHARDS

Number of WAL shards to reduce mutex contention. Default: `4`. `0` disables the WAL; `1` is single-file (legacy).

### VANTADB_WAL_BUFFER_SIZE

WAL buffer size in bytes. Default: 65536 (64 KB).

### VANTADB_FLUSH_THRESHOLD

Number of nodes before triggering an implicit WAL flush. Default: 10000.

### VANTADB_BATCH_SIZE / VANTADB_BULK_COMMIT_INTERVAL

Batch ingestion size (default 1000) and bulk-import commit interval in records (default 10000).

### VANTADB_FLAT_THRESHOLD

Use brute-force flat scan instead of HNSW when the number of index nodes is at or below this threshold. Default: 10000. `0` disables (always HNSW).

## Server (HTTP / MCP)

### VANTADB_HOST / VANTADB_PORT

Host (default `127.0.0.1`) and port (default `8080`) for the HTTP server.

### VANTADB_API_KEY

Optional Bearer token for HTTP API authentication. When set, the server requires `Authorization: Bearer <token>` on protected endpoints.

```bash
export VANTADB_API_KEY=secret-token
```

### VANTADB_REQUIRE_AUTH

When `true` (or `1`), the server refuses to start unless `VANTADB_API_KEY` is configured.

### VANTADB_RATE_LIMIT_RPM

Maximum HTTP requests per minute per remote IP. Default: 600. `0` disables rate limiting.

### VANTADB_TRUSTED_PROXIES

Comma-separated IPs of trusted reverse proxies whose `X-Forwarded-For` header is honored for client-IP resolution. Empty (default) means the header is ignored.

### VANTADB_ALLOWED_ORIGINS

Comma-separated origins allowed for CORS. Empty (default) means no CORS middleware is attached.

### VANTADB_TLS_CERT / VANTADB_TLS_KEY

Paths to PEM-encoded TLS certificate and private key. Requires the `tls` feature.

### VANTADB_POOL_ACQUIRE_TIMEOUT_MS

Pool-permit acquisition timeout in ms (default 5000).

### VANTADB_CIRCUIT_BREAKER_FAILURE_THRESHOLD / VANTADB_CIRCUIT_BREAKER_OPEN_TIMEOUT_SECS

HTTP query pool circuit breaker: consecutive failures before opening (default 5) and seconds it stays open (default 30).

## Logging

### VANTADB_LOG_FORMAT

Log output format: `compact` (default), `json`, `full`.

```bash
export VANTADB_LOG_FORMAT=json
```

### VANTADB_LOG_JSON

Legacy alias: `1`/`true` forces JSON log format.

## Security & Compliance

### VANTADB_ENCRYPTION_KEY

Optional AES-256-GCM at-rest encryption key (hex-encoded 32-byte / 64 hex chars). Requires the `encryption` feature.

### VANTADB_EXPORT_BASE_DIR

Base directory for export/import operations. When set, export/import paths are validated against it with canonical path resolution (including symlink protection).

### VANTADB_AUDIT_LOG_PATH

Optional append-only JSONL audit log of business operations (every put/delete/export/import with ISO 8601 timestamp, namespace, key, and outcome).

## LLM Integration

### VANTA_LLM_URL / VANTA_LLM_MODEL / VANTA_LLM_SUMMARIZE_MODEL

LLM inference endpoint (default `http://localhost:11434`), embedding model (default `all-minilm`), and summarisation model (default `llama3`). Only relevant when the `remote-inference` feature is enabled.

## HNSW Tuning

HNSW parameters (`m`, `m_max0`, `ef_construction`, `ef_search`, `ml`, `distance_metric`, `flat_threshold`, `index_type`, `auto_tune`) are engine-level (`HnswConfig` in `src/index/graph.rs`) and are **not** exposed as environment variables. They are set programmatically when constructing the engine — see `references/api-reference.md`.

## CLI Flags

The `vanta-cli` binary accepts the global `--db` flag (env `VANTA_DB`, default `./db`) and `--memory-limit`, plus per-subcommand flags. For the MCP server:

```bash
vanta-cli server --mcp --db ~/.vantadb
```

## Read-Only Mode

There is **no** `VANTADB_READ_ONLY` environment variable. Read-only mode is set programmatically via `VantaConfig::with_read_only(true)` (embedded SDK) or the engine's read-only construction path.

## Troubleshooting

### High Memory Usage

Reduce `VANTADB_MEMORY_LIMIT` or lower HNSW `ef_construction`/`m` (programmatic) and implement periodic cleanup (`purge_expired`).

### Slow Search

Increase `VANTADB_FLAT_THRESHOLD` behavior or raise HNSW `ef_search` (programmatic); rebuild the index with higher `ef_construction`; reduce dataset size; use metadata filters.

### Low Recall

Increase HNSW parameters: higher `m`, `ef_construction`, and `ef_search` (programmatic).

## Best Practices

1. **Start with defaults** - Use default configuration initially
2. **Benchmark before tuning** - Measure performance before changing parameters
3. **Tune incrementally** - Change one parameter at a time
4. **Monitor metrics** - Track operational metrics continuously
5. **Test with real data** - Use production-like data for configuration testing
6. **Document changes** - Keep track of configuration changes and their impact