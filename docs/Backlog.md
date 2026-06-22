---
type: backlog-tracking
status: active
tags: [vantadb, backlog, ingenieria, fases, prioridades]
links: "[Master Index](Master Index.md)"
last_refined: 2026-06-21 (CLI-EPIC + TUI + Py3.13 + ARM64 + Rust examples + Docs reorg + Homebrew + color-eyre ✅)
---

# Backlog Activo — VantaDB

> **Propósito:** Única fuente de verdad para todas las tareas del proyecto, activas y postergadas.
> **Historial de auditoría:** `REPORT_AUDITORIA_RECTIFICACION_2026-06-14.md`
> **Features ya implementadas:** `docs/progreso/README.md`
> **Verticales GTM:** [Estrategia de Ecosistema y GTM#Segmentación de Mercado 3 Verticales GTM](Estrategia de Ecosistema y GTM.mdsegmentaciãn-de-mercado-3-verticales-gtm)
> **Especificación CLI/TUI:** `Investigaciones/VantaDB_CLI_TUI_Design_Spec.md`

---

## FASE 3 — Pre-Lanzamiento (Julio-Agosto 2026)

### 3.C Core Engine

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `TSK-09` | OpenTelemetry traces (prematuro sin Prometheus básico) | 🟢 | ⏸️ Postpuesto |

## FASE 4 — Launch (Jul-Sep 2026)

### 4.0 Fundacional (bloqueante — hacer primero)

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `MKT-06` | Logo y branding (SVG, paleta, favicon) | 🔴 | ❌ |

### 4.B Framework Integrations

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `INT-01` | LangChain adapter (PyPI + PR langchain-community) | 🔴 | ❌ |
| `INT-02` | LlamaIndex adapter (PyPI + PR llama-index) | 🔴 | ❌ |
| `TSK-90` | Mem0: VantaDB como VectorStoreBackend | 🟠 | ❌ |
| `TSK-89` | CrewAI: VantaDBMemory para crews multi-agente | 🟡 | ❌ |
| `TSK-91` | DSPy: VantaDBRM (Retrieval Module) | 🟡 | ❌ |
| `TSK-92` | Haystack: VantaDBDocumentStore | 🟡 | ❌ |
| `TSK-116` | vantadb-openai (paquete embedding opcional) | 🟡 | ❌ |
| `TSK-117` | vantadb-ollama (embedding local offline) | 🟡 | ❌ |
| `TSK-95` | vantadb-litellm (gateway universal embeddings) | 🟡 | ❌ |

### 4.D Launch Campaign

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `MKT-01` | Landing page (vantadb.dev): hero, benchmarks, comparaciones | 🔴 | ❌ |
| `MKT-02` | Blog post "Introducing VantaDB" (técnico + benchmarks) | 🔴 | ❌ |
| `MKT-03` | Show HN post (timing, título, respuestas preparadas) | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟠 | ❌ |
| `MKT-05` | Blog posts técnicos (5+ pre-lanzamiento) | 🟠 | ❌ |
| `COM-01` | Discord: announcements, general, help, showcase, dev | 🔴 | ❌ |
| `TSK-106` | GitHub Discussions (Q&A, Ideas, Show & Tell) | 🟡 | ❌ |
| `TSK-107` | Community showcase (proyectos en docs/showcase.md) | 🟡 | ❌ |
| `TSK-108` | Newsletter (Substack/Beehiiv, monthly) | 🟢 | ❌ |
| — | Good first issues (20+ issues etiquetados) | 🟠 | ❌ |

### 4.F Distribución

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `TSK-121` | Hash SHA256 verification del wheel en tests | 🟢 | ❌ |

### 4.G Developer Experience

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB (showcase) | 🟠 | ❌ |
| `TSK-103` | Public benchmark site (compare.py vs chroma/lancedb/qdrant) | 🟠 | ❌ |

### 4.H Structural Debt Mitigation (2026-06-22)

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `TSK-123` | Promover `advanced-tokenizer` como feature default + integration test E2E | 🔴 | ❌ |
| `TSK-124` | Documentar `generate_snippet` y highlighting en PYTHON_SDK.md | 🔴 | ❌ |
| `TSK-125` | Alinear docs SLSA con workflows reales (@v2→@v4, SLSA2 vs SLSA3) | 🔴 | ❌ |
| `TSK-126` | Agregar `impl Drop for StorageEngine` para liberación explícita del lock | 🟡 | ❌ |
| `TSK-127` | Formalizar estado de IQL (API estable vs experimental documentado) | 🟡 | ❌ |
| `TSK-128` | Hacer configurable el timeout de `insert_lock` | 🟡 | ❌ |
| `TSK-129` | Hacer configurable el timeout de `.vanta.lock` | 🟡 | ❌ |
| `TSK-130` | Agregar instrumentación de heap memory drift (jemalloc stats) | 🟡 | ❌ |

### 4.I CI/CD Workflow Fixes (2026-06-22)

Hallazgos de revisión exhaustiva de 6 workflows + `rust-setup` action. Ver reporte completo en sesión de agente 2026-06-22.

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `TSK-134` | Fix `release.yml:73` — `sudo chmod 600` incompleto (falta `/swapfile`). Causa OOM en release build Ubuntu. | 🔴 | ❌ |
| `TSK-135` | Fix `python_wheels.yml:60` — `dtolnay/rust-toolchain@master` → `@stable` (único workflow que usa master) | 🟡 | ❌ |
| `TSK-136` | Fix `nightly_bench.yml:117` — `GITHUB_SHA` no propagado al `github-script` step (issues muestran 'unknown') | 🟡 | ❌ |
| `TSK-137` | Agregar swap en macOS/Windows para release builds (solo Ubuntu tiene swap) | 🟡 | ❌ |
| `TSK-138` | Eliminar double checkout en `heavy_certification.yml` (`rust-setup` + caller hacen checkout duplicado) | 🟢 | ❌ |
| `TSK-139` | Eliminar stale path trigger `packages/**` en `rust_ci.yml` (migrado a `integrations/`) | 🟢 | ❌ |
| `TSK-140` | Arreglar o eliminar job arm64 con `if: false` en `python_wheels.yml` (~120 líneas dead code) | 🟢 | ❌ |
| `TSK-141` | Remover `librocksdb-dev` innecesario de `rust-setup/action.yml` (crate bundlea su propia versión) | 🟢 | ❌ |

## FASE 5 — Post-Lanzamiento / Pre-Seed (Noviembre-Diciembre 2026)

### 5.A Enterprise Readiness

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `TSK-72` | Encriptación at-rest AES-256-GCM (clave externa) | 🟡 | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 | ❌ |
| `TSK-110` | SBOM (SPDX/CycloneDX) en cada release | 🟡 | ❌ |
| `BIZ-02` | WAL Shipping asíncrono (replicación sin Raft) | 🟡 | ❌ |
| `TSK-122` | Sharded-slab para HNSW lock-free (mitiga `insert_lock` bottleneck) | 🟡 | ❌ |
| `TSK-131` | Implementar PITR vía WAL archival (archivado + replay point-in-time) | 🟡 | ❌ |
| `TSK-132` | Investigar checkpoint API en Fjall upstream o contribuirla | 🟢 | ❌ |
| `TSK-133` | Backup incremental (full snapshot + WAL deltas) | 🟢 | ❌ |
| `TSK-48` | Cuantización dinámica (f32→SQ8 para nodos fríos) | 🟢 | ❌ |
| `LOW-01` | TLS 1.3 en vantadb-server | 🟢 | ⏸️ Postpuesto |

### 5.B VantaDB Cloud y Negocio

| ID | Tarea | Prioridad | Estado |
|----|-------|-----------|--------|
| `CLD-01` | VantaDB Cloud Beta (Fly.io, NVMe, Bearer auth) | 🟡 | ❌ |
| `CLD-02` | Pitch Deck + one-pager (10 slides pre-seed) | 🟡 | ❌ |
| `CLD-03` | Programa pilotos enterprise (3-5 early adopters) | 🟡 | ❌ |
| `CLD-04` | Case Studies (mínimo 2) | 🟡 | ❌ |
| `BIZ-01` | Bifurcación workspace open-source vs enterprise | 🟡 | ❌ |
| `BIZ-03` | Pricing page (Free/Pro/Enterprise) | 🟡 | ❌ |

**Criterio de salida FASE 5:** 10 enterprise pilots ✓ | $10K MRR ✓ | 3 case studies ✓ | Pitch deck ✓

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

---

## 📊 Veredicto: Estado Real del Proyecto

| Aspecto | Estado | Confianza |
|---------|--------|-----------|
| **Core Engine (Rust)** | 🟢 Sólido | 95% |
| **Persistencia (WAL, mmap)** | 🟢 Implementado | 90% |
| **Índices (HNSW, BM25)** | 🟢 Funcional | 85% |
| **Python SDK** | 🟢 Completo | 90% |
| **Documentación MPTS** | 🟢 Rectificada | 85% |
| **Testing** | 🟡 Parcial | 70% |
| **CLI + Server** | 🟢 Completo (repl, json/quiet, typos) | 95% |
| **API Methods** | 🟢 Completo (filter ops, delete_by_filter, similar_to_key, count, multi-ns) | 95% |

---

| `DISC-11` | Unificar binarios CLI+MCP+Server | ⏸️ Postpuesto (dependencia circular) |
## Véase También

- [Master Index](Master Index.md) — Documento padre
- [Roadmap e Hitos de Ingeniería](Roadmap e Hitos de Ingeniería.md) — Timeline, decisiones, criterios por fase
- [Estrategia de Ecosistema y GTM](Estrategia de Ecosistema y GTM.md) — Verticales GTM, integraciones, marketing
- [Operaciones, Calidad y Riesgos](Operaciones, Calidad y Riesgos.md) — Testing, CI/CD, riesgos estructurales
- `REPORT_AUDITORIA_RECTIFICACION_2026-06-14.md` — Hallazgos de auditoría (ERR, FEAT, HAZ, correcciones)
- `docs/progreso/README.md` — Historial completo de progreso técnico
