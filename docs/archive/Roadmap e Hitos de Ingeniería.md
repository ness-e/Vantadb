---
type: mpts-section
status: stable
tags: [vantadb, roadmap, hitos, plan, timeline, fases]
last_refined: 2026-06-21 (TSK-71 — WASM build ✅)
links: "[Master Index](Master Index.md)"
description: "Fases de desarrollo, criterios de salida, decisiones arquitectónicas y timeline. Tareas detalladas en [Backlog](../Backlog.md)."
aliases: [Roadmap, Hitos, Plan, Timeline, Fases]
---

# Roadmap e Hitos de Ingeniería

> **Dominio:** Producto & Técnico
> **Tareas detalladas:** → [Backlog](../Backlog.md)
> **Propósito:** Definir fases, criterios de aceptación y decisiones arquitectónicas. No duplica el backlog.

---

## Filosofía del Roadmap

1. **Metas de ingeniería, no fechas comerciales** — cada fase tiene criterios cuantitativos
2. **Estabilidad antes que features** — "done" significa "tested, documented, production-ready"
3. **Transparencia radical** — roadmap público, status semanal, problemas conocidos documentados

---

## Estado Actual (v0.1.4 — Junio 2026)

| Fase | Estado | Sub-fases | Descripción |
|------|--------|-----------|-------------|
| **FASE 0** | ✅ 100% | — | Estabilización post-cuarentena |
| **FASE 1** | ✅ 100% | — | HNSW Scalability & Performance |
| **FASE 2** | ✅ 100% | — | Hardening Arquitectónico |
| **FASE 3** | ✅ 100% | 3.A Bloqueantes → 3.G Code Quality | Pre-lanzamiento: auditoría completa (AUD-01→44 ✅) + crates.io publish ✅ |
| **FASE 4** | 🔄 ~50% | 4.A TypeScript SDK ✅ → 4.J Seguridad | Lanzamiento comunitario + ecosistema |
| **FASE 5** | ⬜ 0% | 5.A Enterprise → 5.B Cloud | Post-lanzamiento / Pre-seed |

---

## Hitos Alcanzados por Fase

### FASE 0: Estabilización Post-Cuarentena ✅

- Migración `connectome-server` → `vanta-server`, unificación perfiles, limpieza código muerto
- ADRs documentados, CI/CD funcional

### FASE 1: HNSW Scalability & Performance ✅

| Métrica | Objetivo | Alcanzado |
|---------|----------|-----------|
| Recall@10 (100K) | ≥0.95 | 0.998 |
| Latencia p50 (100K) | <20ms | 12.4ms |
| Memory efficiency | <1500 bytes/vector | 1172 bytes/vector |
| Factor de escalado | <6x (10K→50K) | 4.83x (O(N) lineal) |

### FASE 2: Hardening Arquitectónico ✅

| Riesgo | Severidad | Status |
|--------|-----------|--------|
| AUD-01: WAL durabilidad | 🔒 Bloqueante | ✅ Resuelto (fsync antes de ACK) |
| AUD-02: WAL sin checksums | 🔒 Bloqueante | ✅ Resuelto (CRC32C) |
| AUD-03: Concurrencia en rebuild | ⚠️ Alto | ✅ Resuelto (lock exclusivo) |
| AUD-04: Falta file locking | ⚠️ Alto | ✅ Resuelto (fs2) |
| AUD-05: GIL no liberado | ⚠️ Alto | ✅ Resuelto (py.allow_threads) |

---

## FASE 3: Pre-Lanzamiento 🔄

**Inicio:** 2026-05-01 → **Target:** 2026-08-31

> **Tareas detalladas por sub-fase:** [Backlog#FASE 3 — Pre-Lanzamiento (Julio-Agosto 2026)](../Backlog.md#fase-3--pre-lanzamiento-julio-agosto-2026)

### Sub-fases
- **3.A** Bloqueantes Críticos (CI, telemetría, SIGTERM)
- **3.B** Performance Python SDK (zero-copy FFI, async, stubs, batch)
- **3.C** Core Engine (mmap HNSW ✅, SQ8 ✅, WAL vacuum ✅, TTL ✅, backpressure ✅, eviction ✅)
- **3.D** Testing y Calidad (datasets reales, proptest, regression gates)
- **3.E** Observabilidad (Prometheus, JSON logging, Grafana dashboard) ✅
- **3.F** Documentación Esencial (GraphRAG, durabilidad, migration guides, badges, CHANGELOG) ✅
- **3.F** Documentación Esencial (GraphRAG, durability, migration guides)

### FASE 4 Hitos (Junio 2026)
- [x] **TSK-45:** Core crate publicado en crates.io (v0.1.4, https://crates.io/crates/vantadb)
- [x] **TSK-112:** npm TypeScript SDK (26 tests, 64-byte alignment fix, u64 > 2^53 fix)
- [x] **TSK-118:** Ejemplos TS con LangChain.js, LlamaIndex.TS, Vercel AI SDK

### Hitos técnicos logrados (Mayo-Junio 2026)

- [x] Chaos testing: 30 iteraciones crash injection (entre writes + tight loop)
- [x] CI/CD audit: toolchain @stable, runners corregidos, FORCE_NODE24 eliminado
- [x] NaN/Inf validation en FFI Python (TSK-53)
- [x] Text index audit sin issues críticos (TSK-36)
- [x] Corpus BM25 extendido (8 validaciones, TSK-38)
- [x] Skills scripts funcionales en Windows (TSK-23)
- [x] Tests CLI (33), server (14 unit + 6 E2E), MCP (9)
- [x] Fusión servidor HTTP en `vanta-cli` (TSK-30)
- [x] WAL compaction / vacuum con trigger 256MB y CLI (TSK-75)
- [x] Windows file locking tests: FILE_SHARE_READ, DELETE, stale lock (DISC-02)
- [x] PrefetchMode config con env-var fallback (DISC-03)
- [x] SQ8 quantization: 4x RAM reduction (TSK-47)
- [x] rkyv zero-copy archives (TSK-49)
- [x] Grafana dashboard oficial (ROAD-06)
- [x] TTL en memory records con `expires_at_ms`, `ttl_ms` y `purge_expired()` (TSK-76)
- [x] WASM build para core: 5 deps opcionales, mmap shim Vec-backed, cfg-gated metrics, fs2 stub, rayon fallback (TSK-71)

### Criterios de Salida

| Criterio | Target |
|----------|--------|
| Python SDK p50 | <20ms |
| Windows CI | ✅ verde |
| RAM telemetría | Correcta (RSS vs mmap) |
| Chaos tests | En CI + 30/30 pass |
| 1M vectores | Sin OOM en 16GB RAM |
| Documentación | 90%+ coverage |

---

## FASE 4: Community Launch 🔄

**Inicio:** 2026-06-01 → **Target:** 2026-10-31

> **Tareas detalladas por sub-fase:** [Backlog#FASE 4 — Launch (Jul-Sep 2026)](../Backlog.md#fase-4--launch-jul-sep-2026)

### Sub-fases
- **4.0** Fundacional (crates.io ✅, SECURITY 🔴, logo 🔴, WASM ✅) — hacer primero, bloquea todo
- **4.A** TypeScript SDK (WASM, npm, ejemplos)
- **4.B** Framework Integrations (LangChain, LlamaIndex, Mem0, CrewAI, DSPy, etc.)
- **4.C** API Completeness (filtros expandidos, delete_by_filter, similar_to_key)
- **4.D** Launch Campaign (landing, blog, Show HN, Discord, CONTRIBUTING, comunidad)
- **4.E** CLI Polish (epic único: backup, restore, doctor, stats, inspect, repl, TUI)
- **4.F** Distribución (ARM64, Homebrew, Python 3.13)
- **4.G** Developer Experience (demo app, benchmark site, ejemplos Rust)

### Criterios de Salida

| Métrica | Target |
|---------|--------|
| GitHub stars | 1,000+ |
| PyPI downloads/mes | 10,000+ |
| Discord members | 500+ |
| Contributors | 20+ |
| TypeScript SDK | En npm |
| LangChain + LlamaIndex | En PyPI |

---

## FASE 5: Enterprise / Pre-seed ⬜

**Inicio Planeado:** 2026-Q4

> **Tareas detalladas:** [Backlog#FASE 5 — Post-Lanzamiento / Pre-Seed (Noviembre-Diciembre 2026)](../Backlog.md#fase-5--post-lanzamiento-pre-seed-noviembre-diciembre-2026)

### Sub-fases
- **5.A** Enterprise Readiness (encriptación, audit logs, WAL shipping)
- **5.B** VantaDB Cloud + Negocio (beta, pitch deck, pilotos, pricing)

### Criterios de Salida

| Métrica | Target |
|---------|--------|
| Enterprise pilots | 10+ |
| MRR | $10K+ |
| Case studies | 3+ publicados |
| Pitch deck | Completo |

---

## Decisiones Arquitectónicas Pendientes

### 1. Query Language
| Opción | Pros | Contras |
|--------|------|---------|
| **A:** Solo API programática | ✅ Simplicidad | ❌ Power users limitados |
| **B:** DSL simple (Mongo-like) | ✅ Más flexible | ❌ Complejidad media |
| **C:** SQL subset | ✅ Familiar | ❌ No encaja con vectores |
| **Decisión:** Postergada a FASE 5 (evaluar feedback de usuarios) |

### 2. Distributed Mode
| Opción | Pros | Contras |
|--------|------|---------|
| **A:** Permanecer single-node | ✅ Coherencia embedded | ❌ Límite enterprise |
| **B:** Replicación asíncrona master-slave | ✅ Read scalability | ❌ Consistencia eventual |
| **C:** Sharding + Raft | ✅ Full scale | ❌ Complejidad masiva |
| **Decisión:** Postergada a FASE 5+. WAL shipping (BIZ-02) como paso intermedio. |

### 3. Licensing
| Opción | Pros | Contras |
|--------|------|---------|
| **A:** Apache 2.0 (actual) | ✅ Adopción máxima | ❌ Monetización difícil |
| **B:** Open core (Apache + BSL enterprise) | ✅ Balance | ❌ Complejidad legal |
| **C:** AGPL | ✅ Protección cloud | ❌ Limita enterprise |
| **Decisión:** Apache 2.0 por ahora. Reevaluar FASE 5. |

---

## Timeline de Alto Nivel

```
Q3 2026 (Jul-Sep)
├── Jul ── 4.0 Fundacional (crates.io, SECURITY, logo, WASM), 4.C API ops
├── Ago ── 4.A TS SDK, 4.B Integraciones (LangChain, LlamaIndex), 4.E CLI
└── Sep ── 4.D Launch Campaign (landing, blog, Show HN, Discord), 🚀 LAUNCH

Q4 2026 (Oct-Dic)
├── Oct ── 4.F Distribución (ARM64, Homebrew), 4.G DevEx
├── Nov ── FASE 5: TSK-72 (encripción), BIZ-02 (WAL shipping), pilotos enterprise
└── Dic ── CLD-01 (Cloud alpha), CLD-02 (pitch deck), CLD-04 (case studies)
```

---

## Véase También

- [Master Index](Master Index.md) — Documento padre
- [Backlog](../Backlog.md) — Tareas detalladas por fase y prioridad
- [Visión y Posicionamiento Estratégico](Visión y Posicionamiento Estratégico.md) — Por qué estas fases
- [Operaciones, Calidad y Riesgos](Operaciones, Calidad y Riesgos.md) — Riesgos a mitigar
- [Estrategia de Ecosistema y GTM](Estrategia de Ecosistema y GTM.md) — Cómo se lanza
