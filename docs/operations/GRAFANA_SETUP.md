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
| Memory & RSS | `process_resident_memory_bytes`, `vantadb_rss_bytes` | OS vs VantaDB-tracked RSS |
| Memory Pressure | `vantadb_memory_pressure_ratio` | Ratio vs 80% backpressure threshold |
| Vector Ops/sec | `rate(vantadb_vector_ops_total[1m])` | Quantized vs full-precision ops |
| Query Latency | `vantadb_query_duration_seconds_bucket` | P50/P95/P99 from histograms |
| Disk Usage | `vantadb_storage_bytes` | Per-backend storage size |
| Index Memory | `vantadb_index_memory_bytes` | Per-layer (L1/L2/L2.5/L3) memory breakdown |

## 4. Enabling on existing instances

Add `--metrics-addr 0.0.0.0:8080` (or the standard port 9090) to the server startup:

```bash
vantadb-server --metrics-addr 0.0.0.0:9090
```

For embedded use, ensure `VantaConfig` has:
```rust
config.with_metrics(Some("0.0.0.0:9090".parse().unwrap()))
```

> **Note:** The `L2.5` layer corresponds to SQ8 quantization. It will appear only after upgrading to v0.1.5+ (SQ8 is not available in earlier versions).
