---
title: Grafana Dashboard Setup
type: operations
status: active
tags: [vanta, operations]
last_reviewed: 2026-07-21
aliases: []
---

# Grafana Dashboard Setup

## 1. Import the dashboard

1. Open Grafana (default: http://localhost:3000)
2. Go to **Dashboards → Import**
3. Upload `grafana-dashboard.json` or paste its contents
4. Select your Prometheus datasource
5. Click **Import**

## 2. Verify metrics are flowing

VantaDB exposes Prometheus metrics at `/metrics` on the HTTP port (default `8080`).

```yaml
# prometheus.yml scrape config
scrape_configs:
  - job_name: 'vantadb'
    static_configs:
      - targets: ['localhost:8080']
```

## 3. Available panels

| Panel | Metrics | Description |
|---|---|---|
| Process Memory | `vanta_process_rss_bytes`, `vanta_process_virtual_bytes` | RSS and virtual memory for the VantaDB process |
| Jemalloc Allocation | `vanta_jemalloc_allocated_bytes`, `vanta_jemalloc_active_bytes`, `vanta_jemalloc_resident_bytes`, `vanta_jemalloc_mapped_bytes`, `vanta_jemalloc_retained_bytes`, `vanta_jemalloc_metadata_bytes` | Per-component jemalloc heap breakdown |
| HNSW Index | `vanta_hnsw_nodes_count`, `vanta_hnsw_logical_bytes` | HNSW graph node count and estimated logical memory |
| Page Cache | `vanta_volatile_cache_entries`, `vanta_volatile_cache_cap_bytes` | Hot-node cache utilisation vs capacity |
| MMAP Resident | `vanta_mmap_resident_bytes` | OS-resident bytes for memory-mapped files |
| Query Latency | `vanta_query_latency_ms_bucket` | P50/P95/P99 from histograms |
| HTTP Request Rate | `rate(vanta_http_requests_total[1m])` | Requests/sec by method, route, and status |
| Query Planner | `vanta_planner_hybrid_queries_total`, `vanta_planner_text_only_queries_total`, `vanta_planner_vector_only_queries_total` | Query routing distribution (hybrid vs text-only vs vector-only) |

## 4. Enabling on existing instances

VantaDB serves metrics on the same HTTP port as the API (`/metrics`). Use `--port` (or `VANTADB_PORT`) to control the bind address:

```bash
vantadb-server --port 9090 --host 0.0.0.0
```

For embedded use, set `port` and `host` on `VantaConfig`:
```rust
config.port = 9090;
config.host = "0.0.0.0".into();
```

Override via environment:
```bash
export VANTADB_PORT=9090
export VANTADB_HOST=0.0.0.0
vantadb-server
```
