---
type: action-plan
status: active
tags: [vantadb, strategy, planning, execution]
version: 1.0
created: 2026-07-03
---

# Plan de Acción — VantaDB v0.2.0 Launch

> **Objetivo:** Ejecutar todas las tareas del backlog de forma estructurada, respetando dependencias, maximizando impacto y minimizando riesgo.
> **Ventana:** Julio - Septiembre 2026 (~12 semanas)
> **Target:** Show HN + Lanzamiento público v0.2.0

---

## 📊 Resumen de Carga de Trabajo

| Work Stream | Tareas ❌ | Tareas ⏳ | Esfuerzo Est. | Prioridad |
|-------------|----------|-----------|---------------|-----------|
| 🛡️ Seguridad & Dependencias | 3 | 0 | 1 semana | 🔴 Crítica |
| ⚡ Performance Backend | 5 | 1 | 2-3 semanas | 🔴 Alta |
| 🧪 Testing & QA | 6 | 3 | 2-3 semanas | 🔴 Alta |
| 📚 Documentación | 7 | 2 | 2 semanas | 🔴 Alta |
| 🌐 Frontend Web | 7 | 0 | 2-3 semanas | 🟡 Media |
| 📦 Distribución & Release | 5 | 0 | 1-2 semanas | 🟡 Media |
| 🚀 Launch Marketing | 7 | 0 | 1 semana | 🟡 Media |
| 🗄️ Database Evolution | 4 | 0 | 3-4 semanas | 🟢 Baja |
| 🛠️ DevOps & CI/CD | 5 | 0 | 1-2 semanas | 🟢 Baja |
| 🧹 Code Health | 2 | 1 | 1 semana | 🟢 Baja |

---

## 🔗 Mapa de Dependencias

```
Semana 1-2: 🛡️ SEGURIDAD
  │
  ├──► Semana 1-3: ⚡ BACKEND PERFORMANCE ──► Semana 4-6: 🧪 TESTING
  │                       │                           │
  │                       ▼                           ▼
  ├──► Semana 2-4: 📚 DOCUMENTACIÓN ────► Semana 6-8: 📦 DISTRIBUCIÓN
  │                                                       │
  │                                                       ▼
  ├──► Semana 2-5: 🌐 FRONTEND ─────────► Semana 8-9: 🚀 LAUNCH (Show HN)
  │
  └──► Semana 4-8: 🗄️ DATABASE ────────► Semana 9-12: POST-LAUNCH (Phase 5)
                    🛠️ DEVOPS
```

---

## 🗓️ FASE 1 — Fundación (Semanas 1-2)

### 🥇 Día 1-3: Parches de Seguridad Críticos

> **Razón:** Vulnerabilidades activas (RUSTSEC) bloquean cualquier release seguro.

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 1 | `SEC-08` | Migrar `rustls-pemfile` → `rustls-pki-types` | 🟢 2-4h | — |
| 2 | `SEC-09` | Evaluar y migrar `bincode` → `postcard` o `rkyv` | 🟡 1-2d | — |
| 3 | `SEC-10` | Crear security test suite base (IQL injection, auth bypass) | 🟡 1-2d | — |
| 4 | `TSK-146` | Eliminar magic numbers (1024, 64, 0x8, 0.80) | 🟢 1-2h | — |
| 5 | `TSK-145` | Normalizar comentarios español/inglés a inglés | 🟢 2-4h | — |

### 🥈 Día 3-5: Quick Wins Backend

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 6 | `PERF-13` | Refactor `read_only` check → helper method | 🟢 1h | — |
| 7 | `PERF-12` | Refactor patrón WAL repetitivo → helper method | 🟢 2-4h | — |
| 8 | `PERF-15` | Agregar `#![warn(missing_docs)]` a todos los crates | 🟢 1h | — |
| 9 | `PERF-03` | Completar dynamic auto-scale de spawn_blocking (⏳) | 🟡 4-6h | — |
| 10 | `DB-02` | Diseñar estrategia de versionado de formatos en disco (DOC) | 🟡 1d | `SEC-09` |

### 🥉 Día 5-7: CI/CD Bootstrap

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 11 | `DEVOPS-11` | Agregar CodeQL analysis a CI | 🟢 2h | — |
| 12 | `DEVOPS-07` | Revisar/mejorar Dockerfile multi-stage existente | 🟢 2h | — |
| 13 | `DEVOPS-10` | Investigar signed releases (Windows SmartScreen) | 🟡 1d | — |
| 14 | `TEST-07` | Completar Windows-specific test-threads override (⏳) | 🟢 2h | — |

---

## 🗓️ FASE 2 — Performance & Testing (Semanas 2-4)

### ⚡ Semana 2-3: Performance Backend

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 15 | `PERF-11` | **Batch KV loader (`get_many`/`multi_get`)** — eliminar N+1 en graph, scan, search | 🔴 3-5d | — |
| 16 | `PERF-14` | Refactor `init_telemetry` masivo (cli_server.rs:280-438) | 🟡 1d | — |
| 17 | `DOC-01` | Agregar `#[cfg(test)]` unit tests a módulos faltantes (⏳) | 🟡 2-3d | — |

### 🧪 Semana 3-4: Testing Gaps

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 18 | `TEST-09` | **Implementar tests WASM reales** (archivo vacío) | 🔴 2-3d | — |
| 19 | `TEST-10` | Configurar Vitest + React Testing Library para frontend | 🔴 2-3d | — |
| 20 | `TEST-05` | Snapshot testing (HNSW recall, export/import, WAL format) | 🟡 1-2d | — |
| 21 | `TEST-04` | Regression test suite para bugs corregidos (⏳) | 🟡 1-2d | — |
| 22 | `TEST-06` | Load/stress tests para Python y TS SDKs | 🟡 2-3d | — |

---

## 🗓️ FASE 3 — Preparación de Lanzamiento (Semanas 3-6)

### 📚 Semana 3-4: Documentación

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 23 | `DOC-13` | Crear 5-7 ADRs faltantes (Fjall vs RocksDB, HNSW params, RRF, PyO3, WASM) | 🟡 2-3d | — |
| 24 | `DOC-14` | Performance Tuning Guide | 🟡 2-3d | — |
| 25 | `DOC-15` | OpenAPI/Swagger spec para HTTP API | 🟡 1-2d | — |
| 26 | `DOC-17` | Diagramas de arquitectura formales (Mermaid/Diagrams) | 🟡 1-2d | — |
| 27 | `DOC-18` | Expandir HTTP_API.md (149L → ~400L) | 🟡 1d | — |
| 28 | `DOC-19` | Agregar términos faltantes al glosario | 🟢 1h | — |
| 29 | `DOC-06` | Unified frontmatter schema (⏳) | 🟡 1d | — |

### 🌐 Semana 4-5: Frontend Web

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 30 | `WEB-06` | Migrar inline styles → Tailwind classes (engine.tsx, architecture.tsx) | 🟡 3-5d | — |
| 31 | `WEB-07` | Unificar animation libraries: mantener solo GSAP, migrar route transitions + text-scramble | 🟡 1-2d | — |
| 32 | `WEB-18` | Crear componente `<VsTable>` reusable (eliminar duplicación Legacy vs VantaDB) | 🟢 4-6h | — |
| 33 | `WEB-19` | Implementar `React.lazy()` / code splitting por ruta | 🟢 2-4h | — |
| 34 | `WEB-20` | Agregar `React.memo`/`useMemo`/`useCallback` en componentes clave | 🟢 2-4h | — |
| 35 | `WEB-21` | Eliminar mutación directa del DOM (onMouseEnter/Leave) | 🟢 2-4h | — |
| 36 | `WEB-17` | Evaluar viabilidad de migrar TanStack Router → React Router | 🟡 2-3d | — |

---

## 🗓️ FASE 4 — Release Engineering (Semanas 5-8)

### 📦 Semana 5-6: Distribución

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 37 | `INT-01` | Publicar LangChain adapter a PyPI + PR upstream | 🟡 1-2d | `TEST-06` |
| 38 | `INT-02` | Publicar LlamaIndex adapter a PyPI + PR upstream | 🟡 1-2d | `TEST-06` |
| 39 | `DEVOPS-05` | Publicar adapters PyPI (pipeline CI) | 🟡 1d | `INT-01`, `INT-02` |
| 40 | `REL-02` | Publicar `vantadb-ts` npm package | 🟡 1-2d | `TEST-09` |
| 41 | `TSK-121` | SHA256 hash verification del wheel en tests | 🟢 2-4h | — |
| 42 | `DEVOPS-02` | ARM64 wheels para Python SDK | 🟡 2-3d | — |
| 43 | `DEVOPS-06` | Homebrew formula para vanta-cli | 🟢 4-6h | — |
| 44 | `DEVOPS-08` | Docs build verification en CI | 🟢 2-4h | — |
| 45 | `DEVOPS-09` | Auto-deploy web a Vercel/Cloudflare Pages en CI | 🟡 1d | — |

### 🗄️ Semana 6-8: Database Evolution (FASE 4.N)

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 46 | `DB-01` | Implementar migration runner (`vanta-cli migrate`) | 🔴 3-5d | `DB-02` |
| 47 | `DB-03` | ACID transactions research + prototipo | 🟡 3-5d | — |
| 48 | `DB-04` | Expandir bitset 128→256 o dinámico | 🟢 1-2d | — |

### 🚀 Semana 7-8: Launch Campaign Setup

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 49 | `REL-01` | Bump workspace version v0.1.5 → v0.2.0 | 🟢 1h | `SEC-08`, `SEC-09` |
| 50 | `DOC-16` | Tutorial series (AI Agent Memory, Local RAG, Migration ChromaDB) | 🟡 2-3d | — |
| 51 | `MCP-03` | WASM benchmarks + feature comparison | 🟡 1-2d | — |
| 52 | `COM-01` | Crear Discord server (announcements, help, showcase, dev) | 🟢 2-4h | — |
| 53 | `TSK-106` | Habilitar GitHub Discussions | 🟢 1h | — |

---

## 🗓️ FASE 5 — Launch Execution (Semana 8-9)

### 🚀 Launch Week

| # | ID | Tarea | Esfuerzo | Dependencias |
|---|----|-------|----------|-------------|
| 54 | `LEG-01` | Registrar trademark "VantaDB" (USPTO + EUIPO) | 🟡 2-4h paper | — |
| 55 | `LEG-02` | Add CLA para contribuciones externas | 🟢 1-2h | — |
| 56 | `MKT-03` | Publicar Show HN post | 🟢 2h | Todo lo anterior |
| 57 | `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟢 2-4h | `REL-01` |
| 58 | `MKT-05` | Technical blog posts (5+ pre-launch) | 🟡 2-3d | — |
| 59 | `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | `DOC-16` |
| 60 | `TSK-103` | Public benchmark site | 🟡 2-3d | `PERF-11` |
| 61 | `TSK-104` | Demo agent: LangChain + Ollama + VantaDB | 🟡 1-2d | `INT-01` |
| 62 | `TSK-107` | Community showcase page | 🟢 4-6h | — |
| 63 | `TSK-108` | Newsletter setup (Substack/Beehiiv) | 🟢 2-4h | — |
| 64 | — | Good first issues (20+ tagged) | 🟢 2-4h | — |

---

## 📈 Matriz de Impacto vs Esfuerzo (Priorización)

```
                    Alta Impacto
                        │
          PERF-11  🔴   │   🔴  SEC-08
          TEST-09  🔴   │   🔴  SEC-10
          TEST-10  🔴   │
          DB-01    🔴   │
                        │
  Bajo ─────────────────┼────────────────── Alto
  Esfuerzo              │   Esfuerzo
                        │
          TSK-146  🟢   │   🟡  SEC-09
          PERF-13  🟢   │   🟡  WEB-06
          PERF-15  🟢   │   🟡  DOC-13
          DOC-19   🟢   │   🟡  DOC-14
          WEB-19   🟢   │   🟡  DOC-15
          WEB-20   🟢   │
          WEB-21   🟢   │
                        │
                    Bajo Impacto
```

### 🎯 Quick Wins (Alto Impacto, Bajo Esfuerzo) — HACER PRIMERO

| ID | Tarea | Tiempo |
|----|-------|--------|
| `SEC-08` | Migrar rustls-pemfile | 2-4h |
| `TSK-146` | Magic numbers | 1-2h |
| `TSK-145` | Comentarios español/inglés | 2-4h |
| `PERF-13` | read_only check helper | 1h |
| `PERF-15` | missing_docs lint | 1h |
| `DEVOPS-11` | CodeQL en CI | 2h |
| `WEB-19` | React.lazy() code splitting | 2-4h |
| `DOC-19` | Glosario términos faltantes | 1h |

### 💎 High-Investment (Alto Impacto, Alto Esfuerzo) — PLANEAR BIEN

| ID | Tarea | Tiempo | Riesgo |
|----|-------|--------|--------|
| `PERF-11` | Batch KV loader (N+1) | 3-5d | ⚠️ Cambia API interna de storage |
| `SEC-09` | bincode → postcard/rkyv | 1-2d | ⚠️ Breaking change de formato en disco |
| `DB-01` | Migration runner | 3-5d | ⚠️ Feature crítico para v0.2.0 |
| `WEB-06` | Inline styles → Tailwind | 3-5d | ⚠️ Esfuerzo mecánico pero grande |

---

## ⚠️ Riesgos y Bloqueadores

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| `SEC-09` (bincode) rompe formato en disco | 🟡 Media | 🔴 Crítico | Release v0.2.0 con migración desde v0.1.x |
| WASM tests (TEST-09) requiere toolchain nightly | 🟡 Media | 🟡 Alto | Verificar que wasm-pack funcione en stable |
| Frontend tests (TEST-10) sin coverage previo | 🟢 Baja | 🟡 Alto | Vitest ya configurado (v4.1.9), solo agregar tests |
| TanStack Router migration (WEB-17) es cara | 🟡 Media | 🟡 Medio | Postergar si no hay tiempo — no bloquea launch |
| ACID transactions (DB-03) cambia arquitectura WAL | 🔴 Alta | 🟡 Medio | Mover a post-launch si es muy complejo |
| Trademark registration (LEG-01) es proceso legal lento | 🔴 Alta | 🔴 Alto | Iniciar ya, no bloquea release técnico |

---

## 📋 Resumen Semanal

| Semana | Fase | Entregables Clave |
|--------|------|-------------------|
| **1** | 🛡️ Seguridad | rustls-pemfile migrado, bincode evaluado, magic numbers eliminados, CodeQL activado |
| **2** | ⚡ Performance | read_only/WAL refactors, missing_docs, spawn_blocking dynamic, format versioning doc |
| **3** | 🧪 Testing | WASM tests implementados, Vitest+RTL configurado, snapshot tests |
| **4** | 📚 Docs + Frontend | ADRs creados, Performance Guide, OpenAPI spec, inline styles iniciados |
| **5** | 🌐 Frontend | Animation libs unificadas, VsTable component, lazy loading, memoization |
| **6** | 📦 Distribución | PyPI adapters publicados, npm package, ARM64 wheels, Homebrew formula |
| **7** | 🗄️ Database | Migration runner, bitset expansion, ACID research |
| **8** | 🚀 Launch | v0.2.0 released, Show HN, Reddit, Discord, benchmark site |

---

## ✅ Definition of Ready (DoR) para cada tarea

Antes de comenzar una tarea, debe tener:
- [ ] ID único asignado en Backlog.md
- [ ] Prioridad definida (🔴🟡🟢)
- [ ] Dependencias identificadas (si aplica)
- [ ] Archivos/directorios involucrados conocidos
- [ ] Esfuerzo estimado

## ✅ Definition of Done (DoD) general

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo test` / `vitest run`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs afectados fueron actualizados (ver skill progreso)
- [ ] Tarea movida de Backlog.md a progreso/README.md
- [ ] Changelog actualizado si es cambio visible al usuario
- [ ] `scripts/validate-docs-coverage.ps1` pasa

---

> **Próximo paso:** Elegir la primera tarea del plan y comenzar ejecución.
> Usar `progreso` skill para tracking de cada tarea completada.
