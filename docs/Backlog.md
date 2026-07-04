---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
links: "[[master-index]]"
last_reviewed: 2026-07-04
aliases: []
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks, active and postponed.
> **Completed features:** `docs/CHANGELOG.md`

---

## TIER 0 — 🔴 Bloqueantes de Release (Semana 1, Jul 4-11)

> Items que bloquean cualquier release seguro o publicación pública.

### 🛡️ Seguridad & Corrección de Datos

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `SEC-08` | Migrar `rustls-pemfile` → `rustls-pki-types` (RUSTSEC activa) | 🟢 2-4h | 🔴 | ✅ |
| `SEC-09` | Eliminar `bincode` de `archive/experimental-quarantine/` (ya migrado a postcard) + actualizar docs | 🟢 2h | 🔴 | ✅ |
| `SEC-10` | Security test suite: IQL injection, auth bypass, input validation fuzzing | 🟡 1-2d | 🔴 | ✅ |

### ⚡ Corrección de Datos — Migration Runner

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DB-01` | **Migration runner operativo (`vanta-cli migrate`):** Sincronizar `migration.rs` con `vfile.rs` (aceptar rango v1-v2), usar `VECTOR_INDEX_VERSION` (5, no 4), añadir `WAL_POSTCARD_VERSION` tracking | 🔴 2-3d | 🔴 | ⏳ |
| `DB-02` | Documentar estrategia de versionado de formatos físicos (VantaFile, WAL, índice vectorial) en `docs/architecture/STORAGE_VERSIONING.md` | 🟡 1d | 🔴 | ✅ |
| `—` | Snapshot tests: WAL integrity, VantaFile format, HNSW index format, export/import versioning (reemplazar `TEST-05`) | 🟡 1-2d | 🔴 | ❌ |

### 📦 Publicación de Integraciones (BLOQUEA ADOPCIÓN)

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `INT-01` | **LangChain adapter → PyPI + PR upstream** (código existe en `integrations/langchain/`) | 🟡 1-2d | 🔴 | ❌ |
| `INT-02` | **LlamaIndex adapter → PyPI + PR upstream** (código existe en `integrations/llamaindex/`) | 🟡 1-2d | 🔴 | ❌ |
| `INT-03` | **Mem0 adapter → PyPI** (`vantadb-mem0` crate listo) | 🟡 1d | 🔴 | ❌ |
| `INT-04` | **CrewAI adapter → PyPI** (`vantadb-crewai` crate listo) | 🟡 1d | 🟠 | ❌ |
| `INT-05` | **DSPy adapter → PyPI** (`vantadb-dspy` crate listo) | 🟡 1d | 🟠 | ❌ |
| `INT-06` | **Haystack adapter → PyPI** (`vantadb-haystack` crate listo) | 🟡 1d | 🟠 | ❌ |
| `INT-07` | **Letta adapter → PyPI** (`vantadb-letta` crate listo) | 🟡 1d | 🟠 | ❌ |
| `INT-08` | **OpenAI adapter → PyPI** (`vantadb-openai` crate listo) | 🟡 1d | 🟠 | ❌ |
| `INT-09` | **Ollama adapter → PyPI** (`vantadb-ollama` crate listo) | 🟡 1d | 🟠 | ❌ |
| `INT-10` | **LiteLLM adapter → PyPI** (`vantadb-litellm` crate listo) | 🟡 1d | 🟢 | ❌ |
| `DEVOPS-05` | Pipeline CI unificado para publicar los 10 adapters a PyPI | 🟡 1-2d | 🔴 | ❌ |
| `REL-02` | **Publicar `vantadb-ts` en npm** (WASM build, 26/26 tests) | 🟡 1-2d | 🔴 | ❌ |

### 🧪 Testing Crítico

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `TEST-09` | Implementar tests WASM reales (39 tests en 11 categorías, 744 líneas) | 🔴 2-3d | 🔴 | ✅ |
| `TEST-10` | Configurar Vitest + React Testing Library para frontend (40 tests en 6 suites) | 🔴 2-3d | 🔴 | ✅ |
| `TEST-06` | Load/stress tests para Python (9) y TypeScript (6) SDKs | 🟡 2-3d | 🟡 | ✅ |

---

## TIER 1 — 🟠 Pre-Lanzamiento (Semanas 1-3, Jul 4-25)

> Items necesarios ANTES del Show HN para que el producto sea creíble.

### 🎯 Corrección de Marketing vs Realidad (La Brecha más Peligrosa)

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-11` | **Corregir `web/public/llms.txt`:** Eliminar referencias a SQL (deferido), IVF (no implementado). Corregir latencia real: HNSW 1.2ms ✅, hybrid fusion 2.1ms, p50 real 62ms. **NUEVO** | 🟢 1h | 🔴 | ❌ |
| `MKT-12` | **Auditar claims de performance en web/blog contra benchmarks reales:** Asegurar que ninguna cifra de marketing sea >2x de la real. Publicar metodología de cada benchmark. **NUEVO** | 🟡 1-2d | 🔴 | ❌ |
| `DX-02` | **Reducir p50 hybrid search de 62ms a <20ms:** Investigar cuello de botella exacto (HNSW ef_search? PyO3 overhead? RRF fusion?). Sin esto, "sub-millisecond" es engañoso. | 🟡 2-3d | 🔴 | ❌ |
| `—` | Eliminar `OldSerializationError` deprecated del enum `VantaError` (ya no se usa) | 🟢 1h | 🟡 | ❌ |
| `—` | Python SDK: mapear `VantaError` variants → excepciones Python específicas (hoy todo es `PyRuntimeError`) | 🟡 2-3d | 🔴 | ❌ |

### 🏗️ Base Técnica para Lanzamiento

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `PERF-11` | Batch KV loader (`get_many`/`multi_get`) para eliminar N+1 en graph traversal, scan, search | 🔴 3-5d | 🔴 | ✅ |
| `PERF-15` | Agregar `#![warn(missing_docs)]` en todos los crates del workspace (14 crates) | 🟢 1h | 🟢 | ✅ |
| `TSK-146` | Eliminar magic numbers (1024, 64, 0x8, 0.80) | 🟢 1-2h | 🟢 | ✅ |
| `TSK-145` | Normalizar comentarios español/inglés a inglés | 🟢 2-4h | 🟢 | ✅ |
| `DOC-15` | Crear OpenAPI/Swagger spec para HTTP API (3 endpoints) | 🟡 1-2d | 🟡 | ✅ |

### 🌐 Presencia Web y Landing Page

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-13` | **Integrar demo WASM interactiva en la landing page hero:** WASM-03 ya compila, falta botón "Try in browser" que abra el demo. **NUEVO** | 🟡 1-2d | 🔴 | ❌ |
| `MKT-14` | **Publicar 2 case studies en la web** (CodexAgent: 3.4x faster writes, 100% recall; EdgeSense: 100% crash recovery). Crear ruta `/case-studies/` y cards en hero. **NUEVO** | 🟡 1-2d | 🔴 | ❌ |
| `WEB-06` | Migrar 637 inline styles a Tailwind classes (engine.tsx, architecture.tsx) | 🟡 3-5d | 🟡 | ✅ |
| `WEB-07` | Unificar animation libraries: mantener solo GSAP, migrar route transitions | 🟡 1-2d | 🟡 | ✅ |
| `WEB-18` | Componente `<VsTable>` reusable (eliminar duplicación Legacy vs VantaDB) | 🟢 4-6h | 🟢 | ✅ |
| `WEB-19` | `React.lazy()` / code splitting por ruta | 🟢 2-4h | 🟢 | ✅ |

### 📚 Documentación Pre-Lanzamiento

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DOC-13` | ADRs faltantes: Fjall vs RocksDB, HNSW params (M=16, ef=200), RRF k=60, PyO3, WASM, community governance (6 creados de 11 planeados) | 🟡 2-3d | 🟡 | ✅ |
| `DOC-14` | Performance Tuning Guide (479 líneas) | 🟡 2-3d | 🟡 | ✅ |
| `DOC-16` | Tutorial series: AI Agent Memory, Local RAG Pipeline, Migrating from ChromaDB (3 tutorials creados) | 🟡 2-3d | 🟡 | ✅ |
| `DOC-17` | Diagramas de arquitectura formales Mermaid (5 diagramas) | 🟡 1-2d | 🟡 | ✅ |
| `DOC-18` | Expandir HTTP_API.md (149L → 504L) | 🟡 1d | 🟡 | ✅ |
| `—` | **Docs de setup MCP por IDE:** Cursor, Claude Code, Windsurf. **NUEVO** | 🟡 1-2d | 🔴 | ❌ |

### 🧪 WASM y MCP

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MCP-03` | **Benchmarks WASM vs EdgeVec/minimemory/altor-vec/lattice-db.** Establecer narrativa "most feature-complete WASM vector DB" | 🟡 2-3d | 🔴 | ❌ |
| `MCP-05` | Integration test suite MCP (9 actuales, target 25+) | 🟡 1-2d | 🟡 | ✅ |
| `WASM-03` | Demo AI Agent in browser (Transformers.js + VantaDB WASM + OPFS) | 🟡 2-3d | 🟡 | ✅ |
| `WASM-04` | WASM bundle size optimization (<500KB gzip) | 🟡 1-2d | 🟡 | ✅ |
| `WASM-05` | SIMD acceleration for WASM build (f32x8 cosine) | 🟡 1-2d | 🟡 | ✅ |

### 📦 Distribución

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DEVOPS-02` | ARM64 wheels para Python SDK (Apple Silicon, AWS Graviton, Raspberry Pi) | 🟡 2-3d | 🟠 | ❌ |
| `DEVOPS-06` | Homebrew formula para `vanta-cli` (macOS/Linux) | 🟢 4-6h | 🟢 | ❌ |
| `DEVOPS-10` | **Firma de binarios Windows** (SmartScreen). Research ✅, implementar signing | 🟡 2-3d | 🟡 | ❌ |
| `TSK-121` | SHA256 hash verification del wheel en tests | 🟢 2-4h | 🟢 | ❌ |
| `DEVOPS-07` | Dockerfile multi-stage mejorado (cache mounts, labels, HEALTHCHECK, non-root) | 🟡 2-4h | 🟡 | ✅ |
| `DEVOPS-11` | CodeQL analysis en CI para todos los PRs | 🟢 2h | 🟡 | ✅ |

### 🧹 Code Health

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `PERF-13` | Refactor `read_only` check repetido → helper method | 🟢 1h | 🟢 | ✅ |
| `PERF-14` | Refactor `init_telemetry` masivo (cli_server.rs:280-438) | 🟡 1d | 🟡 | ✅ |
| `DOC-01` | Unit tests (91 nuevos en config/engine/executor/gc/metrics/backends) | 🟡 2-3d | 🟡 | ✅ |
| `DOC-02` | Refactor `insert_hnsw()` (177L → 3 funciones) | 🟡 1d | 🟡 | ✅ |

---

## TIER 2 — 🟡 Launch Campaign (Semanas 3-6, Jul 18 - Ago 15)

> Items para el Show HN + Reddit + lanzamiento público.

### 🚀 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** antes del Show HN | 🟡 2-4h paper | 🔴 | ❌ |
| `LEG-02` | CLA para contribuciones externas (Individual + Corporate) | 🟢 1-2h | 🟠 | ✅ |
| `MKT-03` | **Show HN post** (timing, título, respuestas preparadas) | 🟢 2h | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟢 2-4h | 🟠 | ❌ |
| `MKT-05` | Technical blog posts (5+ pre-launch: GraphRAG, WASM, MCP, hybrid search) | 🟡 2-3d | 🟠 | ❌ |
| `MKT-10` | "AI Agent Memory" narrative campaign: token reduction demos, benchmarks vs full-context | 🟡 2-3d | 🟠 | ❌ |
| `MKT-15` | **Página de benchmarks competitivos** (`/product/benchmarks`): gráficos interactivos VantaDB vs ChromaDB/LanceDB/Qdrant/FAISS. `competitive_bench.py` ya existe. **NUEVO** | 🟡 2-3d | 🔴 | ❌ |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG:** reproducible, con dataset, query set, y métricas. Claim actual de 40-60% sin respaldo. **NUEVO** | 🟡 1-2d | 🔴 | ❌ |
| `TSK-103` | Public benchmark site (comparativa con chroma/lancedb/qdrant) | 🟡 2-3d | 🟠 | ❌ |
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB | 🟡 1-2d | 🟠 | ❌ |

### 🌐 Conversión y SEO

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-17` | **Página de comparación competitiva interactiva** con matriz desplegable (VantaDB vs Pinecone/ChromaDB/Qdrant/LanceDB/FAISS). Datos ya existen en VISION.md. **NUEVO** | 🟡 2-3d | 🟡 | ❌ |
| `MKT-07` | Pricing page (Free/Pro/Enterprise). Señalar modelo aunque cloud no esté listo | 🟡 1-2d | 🔴 | ✅ |
| `WEB-08` | Anti-Slop Audit, Performance Budget, SEO Final Review | 🟢 1d | 🟢 | ✅ |
| `WEB-17` | Evaluar TanStack Router vs React Router (evaluation ✅, mantener) | 🟡 2-3d | 🟡 | ✅ |

### 🗄️ Database Evolution (FASE 4.N)

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DB-01` | Migration runner completo (ver TIER 0) | 🔴 3-5d | 🔴 | ⏳ |
| `DB-03` | ACID transactions research + prototipo | 🟡 3-5d | 🟡 | ✅ |
| `DB-04` | Expandir bitset 128→256 o dinámico (FilterBitset dinámico implementado) | 🟢 1-2d | 🟢 | ✅ |

### 👥 Comunidad

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `COM-01` | **Discord server**: announcements, general, help, showcase, dev | 🟢 2-4h | 🔴 | ❌ |
| `TSK-106` | **Habilitar GitHub Discussions** (Q&A, Ideas, Show & Tell) | 🟢 1h | 🟡 | ❌ |
| `TSK-107` | Community showcase page (projects in docs/showcase.md) | 🟢 4-6h | 🟡 | ❌ |
| `TSK-108` | Newsletter setup (Substack/Beehiiv, monthly) | 🟢 2-4h | 🟢 | ❌ |
| `—` | Good first issues (20+ tagged) | 🟢 2-4h | 🟠 | ❌ |

---

## TIER 3 — 🔵 Post-Lanzamiento (Semanas 6-12, Ago 15 - Sep 30)

> Items post-Show HN, previo a Phase 5.

### 📦 Distribución Avanzada

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DEVOPS-06` | Homebrew formula para vanta-cli | 🟢 4-6h | 🟢 | ❌ |
| `DEVOPS-09` | Auto-deploy web a Vercel/Cloudflare Pages en push a main | 🟡 1d | 🟡 | ✅ |
| `DEVOPS-08` | Docs build verification en CI | 🟢 2-4h | 🟢 | ✅ |
| `—` | Publicar en crates.io los 8 workspace members restantes (`vantadb-mem0`, `vantadb-crewai`, etc.) | 🟡 2-3d | 🟡 | ❌ |

### 🧪 Testing Post-Launch

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `TEST-04` | Regression test suite: tests dedicados para cada bug corregido (12 tests) | 🟡 1-2d | 🟡 | ✅ |
| `TEST-05` | Snapshot testing: HNSW recall, export/import, WAL format (7 tests) | 🟡 1-2d | 🟡 | ✅ |
| `TEST-07` | Fix test-threads: Windows 2, Linux/macOS parallelism | 🟢 2h | 🟢 | ✅ |
| `TEST-08` | Fix `chaos_integrity` required-features | 🟠 1h | 🟠 | ✅ |

### 🎨 Mejoras SDK

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `—` | TypeScript SDK hardening: type safety (eliminar `any`), error wrapping, JSDoc completo, tests | 🟡 2-3d | 🔴 | ❌ |
| `—` | Python SDK: convertir `put_batch` de API posicional a keyword arguments | 🟢 1d | 🟡 | ❌ |
| `—` | Python SDK: eliminar LRU cache home-grown (64 entries, sin eviction real) | 🟢 1d | 🟢 | ❌ |
| `DX-01` | Refactor API: `VantaDB()` → `connect()` | 🟠 1-2d | 🟠 | ✅ |
| `DX-04` | TS SDK: mejorar de 18 tests a 50+ | 🟡 2-3d | 🟡 | ✅ |

### 🛡️ Seguridad Post-Launch

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `SEC-04` | Auth hardening: constant-time comparison, rate limiting, `/metrics` auth | 🟠 2-3d | 🟠 | ✅ |
| `SEC-05` | RBAC design: scoped API tokens | 🟡 1-2d | 🟡 | ✅ |
| `SEC-06` | SBOM generation in each release | 🟡 1-2d | 🟡 | ✅ |
| `SEC-07` | CodeQL + cargo-deny in CI | 🟡 1d | 🟡 | ✅ |

---

## PHASE 5 — ⬜ Enterprise / Pre-Seed (Q4 2026)

> Items post-lanzamiento público. No bloquean v0.2.0.

### 5.A Enterprise Readiness

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `TSK-72` | AES-256-GCM at-rest encryption | 🟡 3-5d | 🟡 | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 2-3d | 🟡 | ❌ |
| `TSK-110` | SBOM en cada release (vía SEC-06) | 🟡 1d | 🟡 | ✅ |
| `BIZ-02` | WAL shipping asíncrono (replication sin Raft) | 🟡 3-5d | 🟡 | ❌ |
| `TSK-122` | Sharded-slab para HNSW lock-free | 🟡 2-3d | 🟡 | ❌ |
| `TSK-131` | PITR via archival WAL | 🟡 3-5d | 🟡 | ❌ |
| `TSK-133` | Incremental backup (snapshot + WAL deltas) | 🟢 2-3d | 🟢 | ❌ |
| `TSK-142` | WASM persistence via OPFS + Web Workers | 🟡 2-3d | 🟡 | ❌ |
| `ENT-01` | SOC 2 prep (access controls, audit trails, retention) | 🟡 3-5d | 🟡 | ❌ |
| `ENT-02` | HIPAA assessment + BAA readiness | 🟡 2-3d | 🟡 | ❌ |
| `ENT-03` | Multi-tenant isolation (RAM, IOPS, storage quotas) | 🟡 3-5d | 🟡 | ❌ |
| `ENT-04` | Connection pooling + circuit breaker | 🟡 2-3d | 🟡 | ❌ |
| `LOW-01` | TLS 1.3 on vantadb-server | 🟢 1-2d | 🟢 | ✅ |

### 5.B VantaDB Cloud & Business

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `CLD-01` | VantaDB Cloud Beta (Fly.io, NVMe, Bearer auth) | 🟡 3-5d | 🟡 | ❌ |
| `CLD-02` | Pitch Deck + one-pager (10 pre-seed slides) | 🟡 2-3d | 🟡 | ❌ |
| `CLD-03` | Enterprise pilot program (3-5 early adopters) | 🟡 2-3d | 🟡 | ❌ |
| `CLD-04` | Case Studies (mínimo 2: AI agent memory, local RAG) | 🟡 2-3d | 🟡 | ❌ |
| `CLD-06` | Stripe billing integration | 🟡 2-3d | 🟡 | ❌ |
| `CLD-07` | Web dashboard (admin panel) | 🟡 3-5d | 🟡 | ❌ |
| `BIZ-01` | Enterprise crate (encryption, audit, RBAC, replication modules) | 🟡 3-5d | 🟡 | ⏳ |
| `BIZ-03` | Pricing page (Free/Pro/Enterprise) — ver MKT-07 | 🟡 1-2d | 🟡 | ✅ |
| `BIZ-04` | Cloud architecture design doc (WAL shipping to S3/R2) | 🟡 2-3d | 🟡 | ❌ |
| `BIZ-05` | Competitive pricing analysis | 🟡 1-2d | 🟡 | ❌ |
| `BIZ-06` | Pitch Deck (10 slides) | 🟡 2-3d | 🟡 | ❌ |

---

## 📊 Matriz de Impacto vs Esfuerzo (Priorización)

```
                    Alta Impacto
                        │
    🔴  DB-01           │   🔴  MKT-11 (llms.txt)
    🔴  INT-01/02       │   🔴  MKT-13 (demo WASM)
    🔴  REL-02 (npm)    │   🔴  MKT-14 (case studies)
    🔴  MKT-15 (bench)  │   🔴  MKT-16 (GraphRAG meth)
    🔴  TS SDK hardening│   🟡  DX-02 (62ms→20ms)
    🔴  Python errors    │
                        │
Bajo ───────────────────┼────────────────── Alto
Esfuerzo                │   Esfuerzo
                        │
    🟢  DEVOPS-06       │   🟡  DEVOPS-02 (ARM64)
    🟢  TSK-108         │   🟡  DEVOPS-10 (signing)
    🟢  COM-01          │   🟡  MCP-03 (WASM bench)
    🟢  TSK-106         │   🟡  MKT-17 (comparison)
    🟢  Good first      │
                        │
                    Bajo Impacto
```

### 🎯 Quick Wins (Alto Impacto, Bajo Esfuerzo) — HACER PRIMERO

| ID | Tarea | Tiempo | Dependencia |
|----|-------|--------|-------------|
| `MKT-11` | Corregir `llms.txt` (SQL, IVF, latency) | 🟢 1h | — |
| `COM-01` | Abrir Discord | 🟢 2-4h | — |
| `TSK-106` | Activar GitHub Discussions | 🟢 1h | — |
| `MKT-13` | Botón "Try in browser" WASM en hero | 🟡 1-2d | WASM-03 ✅ |
| `MKT-14` | Case studies en landing page | 🟡 1-2d | Docs exist |
| `—` | Eliminar `OldSerializationError` deprecated | 🟢 1h | — |
| `—` | Python LRU cache fix | 🟢 1d | — |

### 💎 High-Investment (Alto Impacto, Alto Esfuerzo) — PLANEAR BIEN

| ID | Tarea | Tiempo | Riesgo |
|----|-------|--------|--------|
| `DB-01` | Migration runner completo | 2-3d | ⚠️ Crítico para release |
| `INT-01/02` | LangChain + LlamaIndex → PyPI | 1-2d | ⚠️ Bloquea adopción |
| `MX-02` | Reducir latency 62ms→20ms | 2-3d | ⚠️ Puede requerir re-arquitectura |
| `—` | Python error mapping (VantaError → Python exceptions) | 2-3d | 🟢 Bajo riesgo |

---

## ⚠️ Riesgos y Bloqueadores Actualizados

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| Migration runner roto (vfile v1 vs v2) | 🟡 Media | 🔴 Data loss | ✅ RESUELTO en este PR |
| Vector index version out of sync (v4 vs v5) | 🟢 Baja | 🟡 Rebuild innecesario | ✅ RESUELTO en este PR |
| WAL postcard forward compat | 🟢 Baja | 🟡 Irrecuperable | ✅ RESUELTO (schema_version tracking) |
| LangChain/LlamaIndex no publicados | 🔴 Alta | 🔴 Sin adopción | INT-01/02 en TIER 0 |
| Latencia 62ms vs 20ms target | 🟡 Media | 🟡 Claims engañosos | DX-02 en TIER 1 |
| Trademark no registrado | 🟡 Media | 🔴 Name squatting | LEG-01 en TIER 2 |
| Sin ARM64 wheels | 🟡 Media | 🟡 Pierde edge/RPi | DEVOPS-02 en TIER 2 |
| `llms.txt` con datos falsos | 🔴 Alta | 🟡 AI crawlers mienten | MKT-11 en TIER 1 |

---

## 📋 Resumen de Carga de Trabajo por Categoría

| Categoría | TIER 0 ❌ | TIER 1 ❌ | TIER 2 ❌ | TIER 3 ❌ | PHASE 5 ❌ | Total |
|-----------|----------|----------|----------|----------|-----------|-------|
| 🛡️ Seguridad | 0 | 1 | 0 | 0 | 3 | 4 |
| ⚡ Performance Backend | 1 | 1 | 0 | 0 | 1 | 3 |
| 📦 Integraciones | 12 | 0 | 0 | 0 | 0 | 12 |
| 🧪 Testing | 0 | 0 | 0 | 0 | 0 | 0 |
| 📚 Documentación | 0 | 2 | 0 | 0 | 0 | 2 |
| 🌐 Web & Marketing | 0 | 5 | 3 | 0 | 0 | 8 |
| 🧹 Code Health | 0 | 2 | 0 | 0 | 0 | 2 |
| 🗄️ Database | 1 | 0 | 0 | 0 | 0 | 1 |
| 👥 Comunidad | 0 | 0 | 5 | 0 | 0 | 5 |
| 🎨 SDK Mejoras | 0 | 1 | 3 | 0 | 0 | 4 |
| 🏢 Enterprise | 0 | 0 | 0 | 0 | 10 | 10 |
| ☁️ Cloud | 0 | 0 | 0 | 0 | 7 | 7 |
| 📦 Distribución | 0 | 3 | 1 | 0 | 0 | 4 |
| **Total** | **14** | **15** | **12** | **0** | **21** | **62** |

---

## 📈 Timeline Consolidado

```
Jul 4-11   TIER 0: Seguridad + Migration runner + Publicar 10 adapters PyPI + npm
Jul 11-18  TIER 1: llms.txt fix, demo WASM, case studies web, MCP docs, 62ms→20ms
Jul 18-25  TIER 1: ARM64 wheels, Windows signing, Python error mapping
Jul 25-    TIER 1: TS SDK hardening, OpenAPI spec
Ago 1-15   TIER 2: Trademark, Show HN, Reddit, Discord, competitive benchmarks
Ago 15-    TIER 2: GraphRAG methodology, comparison page, blog posts, community setup
Sep 1-30   TIER 3: Homebrew, SDK polish, code health
Oct+       PHASE 5: Enterprise (encryption, RBAC, WAL shipping, cloud)
```

---

## ✅ Definition of Ready (DoR)

- [ ] ID único asignado
- [ ] Prioridad definida (🔴🟠🟡🟢)
- [ ] Dependencias identificadas
- [ ] Archivos/directorios involucrados conocidos
- [ ] Esfuerzo estimado

## ✅ Definition of Done (DoD)

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo test` / `vitest run`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs afectados actualizados
- [ ] Tarea movida de Backlog.md a progreso/README.md
- [ ] Changelog actualizado si es cambio visible al usuario
- [ ] `scripts/validate-docs-coverage.ps1` pasa

---

## See Also

- [[master-index]] — Central navigation
- [[docs/strategy/ACTION_PLAN.md]] — Detailed execution plan
- [[docs/strategy/ROADMAP.md]] — Phase definitions
- [[CHANGELOG.md]] — Release history
