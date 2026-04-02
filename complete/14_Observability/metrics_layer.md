# Observability & Metrics Layer
> **Status**: 🟡 In Progress — FASE 12

## 1. System Transparency
IADBMS operates under strict 16GB memory and <20ms latency constraints. To guarantee these Service Level Objectives (SLOs) during production usage by Autonomous Agents, we must continuously measure inner bottlenecks.

## 2. Telemetry Architecture
We use standard `Prometheus` Counter and Histogram abstractions to track:
- `iadbms_query_latency_ms`: Distribution of logical -> physical execution durations.
- `iadbms_oom_circuit_trips_total`: Occurrences of the ResourceGovernor panic interventions.
- `iadbms_cache_hits_total`: Measurements of CP-Index early filters bypassing RocksDB lookups.

## 3. Server Exporter
The existing `Axum` daemon is extended to expose `/metrics`, providing scraping capabilities for local Grafana or Datadog integrations.
