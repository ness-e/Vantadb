---
title: Backlog v0.1 — Icebox + "No Hacer" (histórico 2026-06)
kind: research
status: archived
description: "Rescatado de OLD (2026-09-30) — listas Icebox (ROAD/DIST/LISP/LOW) y No Hacer del backlog pre-reorg. Descartes consolidados: DECISIONS-NOT-TAKEN.md."
tags: [vantadb, archive, historico, backlog]
---

> **[Histórico — rescatado 2026-09-30 de OLD]** Registro del backlog v0.1 (2026-06-21). Items rescatados: ROAD-02->BIZ-14 · ROAD-08->FUT-18 · ROAD-09->FUT-19 · DIST-07->FUT-22 · DIST-08->FUT-20 · DIST-11->FUT-21 · DIST-14->FUT-16. Descartes -> `DECISIONS-NOT-TAKEN.md`.

---
---

## ⚠️ Riesgos de No Hacer (pre-launch)

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| ~~License audit pendiente~~ | ✅ Mitigado — `cargo deny check licenses` pasa limpio, todas las dependencias compatibles con Apache 2.0 |
| Trademark "VantaDB" no registrado | Alguien más reclama el nombre | Registrar marca en USPTO + EUIPO antes de Show HN |
| CI/CD para forks externos | PRs de comunidad pueden romper CI o inyectar código malicioso | Workflow approval for first-time contributors + restricted secrets |

---

## ⏸️ Icebox — Postergado / Sin Prioridad Asignada

Tareas que no entran en el roadmap actual pero se mantienen como registro. Sin prioridad, sin fase asignada.

### Roadmap v2 (Visualización y Herramientas)

| ID | Tarea | Descripción |
|----|-------|-------------|
| `ROAD-02` | Backup/Restore a S3 | Exportar instantáneas .vantadb a almacenamiento de red |
| `ROAD-03` | Web UI Explorer | Visualizar topología HNSW + dispersión vectorial (Umap/TSNE) |
| `ROAD-04` | Bulk Import CLI | Importación optimizada de millones de nodos desde JSON/CSV con barra de progreso |
| `ROAD-05` | Multi-model Hooks | Integración con LLMs locales (Ollama) y remotos (OpenAI) para embeddings automáticos |
| `ROAD-07` | Connection Pooling | Cola de conexiones reutilizables con circuit breaker |
| `ROAD-08` | Schema Validation | Validaciones estrictas opcionales de tipos por namespace |
| `ROAD-09` | Query Caching | Caché LRU de búsqueda híbrida con TTL |
| `ROAD-11` | Docker Compose | Entorno preconfigurado VantaDB Server + Ollama + Web UI |

### Distribuido y Escalamiento Multi-nodo (v2.0+)

| ID | Tarea | Descripción |
|----|-------|-------------|
| `DIST-01` | Raft Consensus | Integración de `openraft` en vantadb-server |
| `DIST-02` | Hash Sharding | Distribución consistente de llaves por hash + consultas cross-shard |
| `DIST-03` | Zero-Downtime Upgrades | Rolling restarts sin pérdida de servicio |
| `DIST-04` | ML Cost-Based Optimizer | Optimizador heurístico basado en árboles de decisión |
| `DIST-05` | Auto-Indexing | Creación automática de índices sobre campos filtrados frecuentemente |
| `DIST-06` | Adaptive TEMPERATURE | Variación de hiperparámetros según frecuencia de lectura del agente |
| `DIST-07` | Query Recommendations | Sugerencias ortográficas y correcciones en consultas de texto |
| `DIST-08` | Anomaly Detection | Monitoreo de spikes de recursos en clusters |
| `DIST-09` | Multi-Tenant Isolation | Cuotas estrictas de RAM, IOPS e indexación por tenant |
| `DIST-10` | Plugin Marketplace | Ejecución en sandbox de módulos personalizados WASM |
| `DIST-11` | Edge Federation | Sincronización eventual P2P entre agentes desconectados |
| `DIST-12` | Time-Series Mode | Operadores y funciones de agregación por ventanas de tiempo |
| `DIST-13` | GraphQL API | Consultar namespaces, grafos y relaciones con GraphQL |
| `DIST-14` | CDC (Change Data Capture) | Eventos del WAL vía WebSocket a clientes externos |

### VantaLISP / VantaScript (Primitivas Cognitivas)

| ID | Tarea | Descripción |
|----|-------|-------------|
| `LISP-01` | Bytecode JIT | Traducción de consultas relacionales a bytecode de ejecución directa sobre mmap |
| `LISP-02` | Unificación multimodal | Operadores semántico-léxicos `~` y `SIGUE` en IQL |
| `LISP-03` | Fuel 2.0 | Límites de cómputo vinculados dinámicamente a telemetría de CPU/RAM |
| `LISP-04` | Metacognición | Algoritmos de rehidratación y reordenamiento de relaciones según flujo de conversación |
| `LISP-05` | Monotonic Logic | Lógica distribuida coordinada sin reloj global para agentes |
| `LISP-06` | Sandbox de ejecución | Restricciones FFI para que el motor no llame rutinas inseguras |
| `LISP-07` | CRDTs definibles en LISP | Tipos de datos para mezcla determinista |
| `LISP-08` | Multi-salto | Rutas de razonamiento semántico recursivas cruzando enlaces de grafos |
| `LISP-09` | Fuzzing del parser | Inyección aleatoria de tokens para robustez del compilador |
| `LISP-10` | VantaScript / Inference Logic | Renombrado a estándares más legibles para devs JS/Python |

### Bajo ROI / No Prioritario

| ID | Tarea | Razón |
|----|-------|-------|
| `LOW-02` | Background compaction en Fjall | Fjall maneja su propia compactación |
| `LOW-03` | OpenTelemetry traces | Prematuro sin Prometheus básico |

---

## ❌ No Hacer (hasta post-seed con equipo)

| Feature | Razón |
|---------|-------|
| SQL completo | 3-6 meses, ICP no lo necesita, pgvector ya lo tiene |
| Distributed / Raft | 6-12 meses, contradice filosofía embedded |
| IVF-PQ disk-based | LanceDB mejor, no es mercado VantaDB |
| GPU acceleration | Rompe zero-config, no resuelve bottleneck real |
| RBAC / SSO en core | Solo cloud managed, post-seed |
| Embedding models bundled | Destruye zero-config (500MB+ wheel) |
| GraphQL API | ICP prefiere API, ya tienes MCP |
| Versionado git-style | No es dolor del ICP, LanceDB ya lo tiene |
| Time-series mode | Producto diferente, fuera de scope |
| Cuantización 1.5/2-bit | Retorno marginal para datasets <1M |

