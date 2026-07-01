# VantaDB HTTP API

> REST interface for the VantaDB HTTP server (optional, enabled via `vanta-cli server --http` or the `server` Cargo feature).

## Base URL

```
http://<host>:<port>
```

Default: `http://127.0.0.1:8080`

## Authentication

Endpoints under `/api/` require a Bearer token if `api_key` is configured:

```
Authorization: Bearer <VANTADB_API_KEY>
```

## Endpoints

### `GET /health`

Liveness check. Returns `{"success": true, "data": "OK"}` when the server is running.

**Auth:** None

**Example:**
```bash
curl http://127.0.0.1:8080/health
```

**Response:**
```json
{"success": true, "data": "OK"}
```

### `GET /metrics`

Prometheus-formatted metrics text. Exposes operational metrics at `/metrics` for scraping by Prometheus or any OpenMetrics-compatible collector.

**Auth:** None

**Content-Type:** `text/plain; version=0.0.4`

**Example:**
```bash
curl http://127.0.0.1:8080/metrics
```

### `POST /api/v2/query`

Execute an IQL (Interactive Query Language) or hybrid query against the database.

**Auth:** Bearer token (if `api_key` configured)

**Request body:**
```json
{
  "query": "<query string>"
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <api-key>" \
  -d '{"query": "(memory:get \"agent/main\" \"memory-1\")"}'
```

**Response (read):**
```json
{
  "success": true,
  "data": "Read 1 nodes.",
  "node_id": null,
  "nodes": [
    {
      "id": 1,
      "semantic_cluster": 0,
      "relational": { "key": "memory-1", "namespace": "agent/main" },
      "hits": 3,
      "confidence_score": 0.95
    }
  ]
}
```

**Response (write):**
```json
{
  "success": true,
  "data": "Mutated 1 nodes: inserted",
  "node_id": 42,
  "nodes": null
}
```

## Rate Limiting

Configurable via `rate_limit_rpm` in `VantaConfig` (default: 100 requests per minute). When the limit is exceeded, the server returns HTTP 429.

## TLS

When `tls_cert_path` and `tls_key_path` are configured, the server binds with HTTPS. Requires the `tls` feature.

## Starting the Server

```bash
# HTTP server only
vanta-cli server --http --port 8080 --host 127.0.0.1 --db ./vanta_data

# Full MCP + HTTP
vanta-cli server --http --mcp --port 8080 --db ./vanta_data

# With TLS
vanta-cli server --http --port 443 --db ./vanta_data
# Requires VANTADB_TLS_CERT and VANTADB_TLS_KEY env vars
```

## Route Summary

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/health` | No | Liveness check |
| `GET` | `/metrics` | No | Prometheus metrics (OpenMetrics format) |
| `POST` | `/api/v2/query` | Bearer | Execute IQL query |

## Rate Limiting

Configurable via `rate_limit_rpm` in `VantaConfig` (default: 100 req/min). When exceeded, returns `HTTP 429 Too Many Requests` with a `Retry-After` header.

## TLS

When `VANTADB_TLS_CERT` and `VANTADB_TLS_KEY` are configured, the server binds with HTTPS on the specified port. Requires the `tls` Cargo feature. Self-signed certificates are not recommended for production.

## CORS

The server currently does not set CORS headers. For browser-based clients, a reverse proxy (nginx, Caddy) is recommended to add CORS support.
