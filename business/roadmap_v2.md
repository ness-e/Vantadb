# IADBMS — Post-MVP Roadmap v2.0

> **Estado actual:** MVP completo (15 fases). Este documento define la ruta
> de features para las versiones v1.1 → v3.0 del motor.

---

## Visión de Versiones

```
v1.0 (ACTUAL)  → MVP: Single-node, local-first, Ollama-native
v1.1 (Q3 2026) → Polish: DX, docs site, playground, CLI improvements
v1.5 (Q4 2026) → Scale: WASM build, backup/restore, monitoring dashboard
v2.0 (Q1 2027) → Distributed: Sharding, replication, cloud-ready
v2.5 (Q2 2027) → Intelligence: ML query optimizer (CBO), auto-indexing
v3.0 (Q4 2027) → Platform: Marketplace, multi-tenant, edge federation
```

---

## v1.1 — Developer Experience (Target: Q3 2026)

### Prioridad: CRÍTICA (retención de early adopters)

| # | Feature | Esfuerzo | Impacto |
|---|---|---|---|
| 1 | **mdBook docs site** con GitHub Pages deploy | 1 semana | ⭐⭐⭐⭐⭐ |
| 2 | **README rewrite** (demo GIF, benchmarks, quickstart 60s) | 3 días | ⭐⭐⭐⭐⭐ |
| 3 | **CLI syntax highlighting** (colored + regex IQL) | 2 días | ⭐⭐⭐⭐ |
| 4 | **CLI `.explain`** (muestra LogicalPlan sin ejecutar) | 1 día | ⭐⭐⭐⭐ |
| 5 | **CLI table output** (tabled crate) | 1 día | ⭐⭐⭐⭐ |
| 6 | **Docker Compose** con Ollama preconfigurado | 2 días | ⭐⭐⭐⭐⭐ |
| 7 | **OpenAPI spec** generada desde Axum handlers | 3 días | ⭐⭐⭐ |
| 8 | **Error messages mejorados** con sugerencias IQL | 2 días | ⭐⭐⭐⭐ |
| 9 | **GitHub Release binarios** (Linux, macOS, Windows) | 1 día | ⭐⭐⭐⭐ |
| 10 | **Contributing.md + Issue templates** | 1 día | ⭐⭐⭐ |

---

## v1.5 — Scale & Robustness (Target: Q4 2026)

### Prioridad: ALTA (enterprise-readiness)

| # | Feature | Detalle | Esfuerzo |
|---|---|---|---|
| 1 | **WASM Build** | Compilar core a `wasm32-wasi` para browser playground. Sin RocksDB (in-memory backend). Dataset demo precargado. | 2 semanas |
| 2 | **Backup/Restore** | Export completo a archivo `.iadbms` (bincode snapshot). Import desde snapshot. Compatible con S3 upload vía CLI flag. | 1 semana |
| 3 | **Web UI Visualizador** | Panel web servido por Axum: graph explorer (vis.js), vector scatter (plotly), query editor (CodeMirror). | 3 semanas |
| 4 | **Bulk Import** | `.import file.csv` y `.import file.json` en CLI. Batch inserts con progress bar. Target: 100k nodes/sec. | 1 semana |
| 5 | **Multi-model Hooks** | Soporte para múltiples LLM backends: Ollama, vLLM, OpenAI API. Configurable por env var `IADBMS_LLM_PROVIDER`. | 1 semana |
| 6 | **Monitoring Dashboard** | Grafana dashboard preconfigurado. Docker Compose con Prometheus + Grafana + IADBMS. | 3 días |
| 7 | **Connection Pooling** | Tokio-based connection pool para el REST API. Max concurrent queries configurable. Backpressure via circuit breaker. | 1 semana |
| 8 | **TLS/HTTPS** | Soporte nativo de TLS en Axum server. Self-signed cert generator para dev. Let's Encrypt integration para prod. | 3 días |
| 9 | **Schema Validation** | Optional strict mode: definir schema por TYPE. Rechazar INSERTs que no cumplan. `CREATE SCHEMA Persona { nombre: String, edad: Int }`. | 1 semana |
| 10 | **Query Caching** | LRU cache para queries frecuentes. Cache invalidation on write. Configurable TTL. | 3 días |

---

## v2.0 — Distributed (Target: Q1 2027)

### Prioridad: ESTRATÉGICA (Cloud / Enterprise unlock)

| # | Feature | Detalle |
|---|---|---|
| 1 | **Raft Consensus** | Integrar `openraft` crate. 3-node minimum cluster. Leader election + log replication. |
| 2 | **Hash Sharding** | Partition by `node_id % shard_count`. Automatic rebalancing on node join/leave. |
| 3 | **Cross-Shard Queries** | Scatter-gather para FROM queries. Merge sort para RANK BY across shards. |
| 4 | **Replication** | Configurable replication factor (1-5). Async replication by default, sync optional. |
| 5 | **Cluster CLI** | `iadbms cluster status`, `iadbms cluster add-node`, `iadbms cluster rebalance`. |
| 6 | **Zero-Downtime Upgrades** | Rolling restart. One node at a time. Automatic leader failover during upgrade. |

### Arquitectura Distribuida:
```
                    ┌─────────────────┐
                    │  Load Balancer  │
                    │  (HAProxy/K8s)  │
                    └────────┬────────┘
              ┌──────────────┼──────────────┐
              │              │              │
         ┌────▼───┐    ┌────▼───┐    ┌────▼───┐
         │ Node 1 │    │ Node 2 │    │ Node 3 │
         │ Leader │◄──►│Follower│◄──►│Follower│
         │Shard 0 │    │Shard 1 │    │Shard 2 │
         └────────┘    └────────┘    └────────┘
              │              │              │
         ┌────▼───┐    ┌────▼───┐    ┌────▼───┐
         │RocksDB │    │RocksDB │    │RocksDB │
         └────────┘    └────────┘    └────────┘
```

---

## v2.5 — Intelligence (Target: Q2 2027)

### Prioridad: DIFERENCIADOR (moat competitivo)

| # | Feature | Detalle |
|---|---|---|
| 1 | **ML Cost-Based Optimizer** | Micro ML model (decision tree) que predice el mejor plan de ejecución basado en estadísticas del dataset. Entrenado con historial de queries. |
| 2 | **Auto-Indexing** | Detectar queries frecuentes y crear índices HNSW automáticamente para campos vectoriales no indexados. |
| 3 | **Adaptive TEMPERATURE** | El motor ajusta automáticamente el parámetro TEMPERATURE basado en la cardinalidad del resultado. Muchos resultados → más estricto. |
| 4 | **Query Recommendations** | "Did you mean?" cuando una query devuelve 0 resultados. Sugiere campos similares o thresholds más relajados. |
| 5 | **Anomaly Detection** | Detectar patrones inusuales en writes (spike de inserts, vectores outliers) y alertar vía Prometheus. |

---

## v3.0 — Platform (Target: Q4 2027)

### Prioridad: VISIÓN (position for Series A)

| # | Feature | Detalle |
|---|---|---|
| 1 | **Multi-Tenant** | Aislamiento completo por tenant. Separate RocksDB instances. Shared HNSW with tenant masking. |
| 2 | **Plugin Marketplace** | Third-party connectors: Slack, Notion, Gmail, Jira. Rust WASM plugins. 70/30 revenue split. |
| 3 | **Edge Federation** | Multiple IADBMS nodes distribuidos geográficamente con sync eventual. Perfect for IoT + Edge AI. |
| 4 | **Time-Series Mode** | Window functions para datos temporales. Downsampling automático. Retention policies. |
| 5 | **GraphQL API** | Además de REST, ofrecer endpoint GraphQL auto-generado desde el schema. |
| 6 | **CDC (Change Data Capture)** | Stream de cambios en tiempo real vía WebSocket. Para sincronizar con sistemas externos. |

---

## Prioridades Técnicas Inmediatas (Próximas 4 semanas)

```
SEMANA 1:  README rewrite + Docker Compose + docs skeleton
SEMANA 2:  CLI improvements (syntax highlight, tables, .explain)
SEMANA 3:  mdBook docs deployment + OpenAPI spec
SEMANA 4:  GitHub Release binarios + Contributing guide + HN launch prep
```

---

## Decisiones Técnicas Pendientes

| Decisión | Opciones | Deadline |
|---|---|---|
| WASM backend | In-memory BTreeMap vs SQLite WASM | v1.5 planning |
| Distributed consensus | openraft vs custom Raft | v2.0 planning |
| Cloud provider | Fly.io vs Railway vs self-hosted K8s | v1.5 launch |
| Plugin format | WASM modules vs Rust dylib | v3.0 planning |
| Schema language | Custom DSL vs JSON Schema vs Protobuf | v1.5 planning |
