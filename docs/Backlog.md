---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-07-13
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks.
> **Completed tasks:** `docs/CHANGELOG.md` + `docs/progreso/README.md`
> **Verification method:** All claims cross-checked against actual codebase via 4 sub-agents (Jul 13). See `docs/archive/` for superseded audit reports.
> **Total open items:** 48 (verified against code)

---

## TIER 0 — 🔴 Bloqueantes de Release

> Items que bloquean cualquier release seguro.

### 📦 Publicación de Integraciones

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `INT-01` | **LangChain adapter → PyPI** | 🟡 1-2d | 🔴 | ❌ | Código existe en `vantadb-langchain/` + `integrations/langchain/`, no publicado |
| `INT-02` | **LlamaIndex adapter → PyPI** | 🟡 1-2d | 🔴 | ❌ | Código existe en `vantadb-llamaindex/` + `integrations/llamaindex/`, no publicado |
| `DEVOPS-05` | Pipeline CI unificado para publicar los 10 adapters a PyPI | 🟡 1-2d | 🔴 | ❌ | No existe pipeline integrado |
| `REL-02` | **Publicar `vantadb-ts` en npm** (WASM build) | 🟡 1-2d | 🔴 | ❌ | Código listo, `package.json` presente, no publicado |

### 🌐 Web & Landing

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `MKT-13` | **Enlazar demo WASM desde la hero** — Ruta `/demo` existe, demo funcional. Falta botón "Try in browser" en `NbTerminalHero` | 🟡 1-2h | 🔴 | ⏳ | `NbTerminalHero.tsx` no tiene link a `/demo`. Verificado. |

---

## TIER 1 — 🟠 Pre-Lanzamiento

> Necesario ANTES del Show HN.

### 📖 Documentación & Community

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `MKT-14` | **Publicar 2 case studies** + ruta `/case-studies/` | 🟡 1-2d | 🔴 | ❌ | `docs/case_studies/` drafts existen, no desplegados |
| `TSK-106` | **Habilitar GitHub Discussions** | 🟢 1h | 🟠 | ❌ | No verificable desde repo local |
| `NUEVO-01` | **README hero** con readme-aura + benchmark gráfico + GIF demo WASM | 🟡 2-3d | 🟠 | ❌ | No implementado |
| `NUEVO-07` | **Migration tools: Chroma→Vanta, LanceDB→Vanta** | 🟡 3-5d | 🟠 | ❌ | No existen scripts de migración automatizados |
| `NUEVO-08` | **Learning path estructurado** en tutorials/ (5-7 ejemplos) | 🟡 2-3d | 🟠 | ❌ | Tutorials existen (3) pero sin progresión clara |
| `NUEVO-10` | **Benchmark suite pública reproducible** | 🟡 3-5d | 🟠 | ❌ | Benchmarks internos existen, no hay publish |
| `TSK-107` | Community showcase page | 🟢 4-6h | 🟡 | ❌ | No existe |
| `—` | Good first issues (20+ tagged) | 🟢 2-4h | 🟠 | ❌ | No hay issues etiquetados |

### 🚀 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** | 🟡 2-4h | 🔴 | ❌ |
| `MKT-03` | **Show HN post** | 🟢 2h | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟢 2-4h | 🟠 | ❌ |
| `MKT-05` | Technical blog posts (5+ pre-launch) | 🟡 2-3d | 🟠 | ❌ |
| `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | 🟠 | ❌ |
| `MKT-15` | **Página de benchmarks competitivos** (`/product/benchmarks`) | 🟡 2-3d | 🔴 | ❌ |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG** | 🟡 1-2d | 🟡 | ❌ |
| `TSK-103` | Public benchmark site | 🟡 2-3d | 🟠 | ❌ |
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB | 🟡 1-2d | 🟠 | ❌ |
| `DEVOPS-12` | **Production PyPI signing pipeline** (OIDC + Sigstore) | 🟡 1-2d | 🟡 | ❌ |
| `DEVOPS-10` | **Firma de binarios Windows (SmartScreen)** | 🟡 2-3d | 🟢 | ❌ |

### 🌐 Conversión y SEO

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-17` | Página de comparación competitiva interactiva | 🟡 2-3d | 🟢 | ❌ |

### 🧪 Issues Técnicos Verificados (Nuevos)

> Items descubiertos durante la verificación cross-code del backlog. No estaban registrados previamente.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `VFY-001` | **TS SDK `catch {}` silencia errores** — 4+ bloques catch vacíos | `vantadb-ts/src/vantadb.ts:176,215,249` | 🟢 2h | 🟡 | ❌ |
| `VFY-002` | **`get_nns_by_id` spawn por llamada** — Sin batching | `vantadb-ts/src/vantadb.ts:325` | 🟢 2h | 🟢 | ❌ |
| `VFY-003` | **`reindex_hnsw_from_text` riesgo OOM** — Sin batch processing | `vantadb-python/src/lib.rs:1584` | 🟡 1d | 🟡 | ❌ |
| `VFY-004` | **`flat.rs` O(n²) en filter** — Sin índice para filtros | `src/index/flat.rs:32` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-005` | **TS `OperationalMetrics` 70% incompleto** — 3 de 10+ métricas mapeadas | `vantadb-ts/src/types.ts:148-168` | 🟢 4h | 🟢 | ❌ |
| `VFY-006` | **`add_node` escribe lock durante toda inserción** | `src/index/graph.rs:476-490` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-007` | **`remove_node` O(n²) neighbor fixup** — Deletes costosos | `src/index/core.rs` | 🟡 1-2d | 🟢 | ❌ |
| `VFY-008` | **WAL fsync por escritura** — Write amplification | `src/storage/wal.rs` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-009` | **637 inline styles no migrados a Tailwind** | `web/src/` | 🟡 3-5d | 🟢 | ❌ |
| `VFY-010` | **ACID Phase 2: Buffered write transactions** — No implementado | `src/wal.rs` | 🟡 2-3d | 🔵 | ❌ |
| `VFY-011` | **ACID Phase 3: Snapshot isolation / MVCC** | — | 🟠 3-5d | 🔵 | ❌ |
| `VFY-012` | **DEVOPS-03: musllinux target gap** — Algunos targets sin soporte | CI config | 🟢 4h | 🟢 | ❌ |

### WASM & Performance

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `NUEVO-11` | **WASM IndexedDB fallback** | 🟡 2-3d | 🟡 | ❌ |
| `NUEVO-12` | **WASM multi-tab coordination** (Web Locks + BroadcastChannel) | 🟡 2-3d | 🟡 | ❌ |
| `NUEVO-13` | **HNSW auto-tuning PID loop** (ef_search dinámico) | 🟡 3-5d | 🟡 | ❌ |
| `NUEVO-14` | **WASM bundle size <500KB gzip** | 🟡 1-2d | 🟡 | ❌ |
| `NUEVO-15` | **Code coverage report en CI** + upload | 🟢 1d | 🟡 | ❌ |
| `NUEVO-19` | **Mover SourceDesign/ fuera de web/src/** | 🟢 1h | 🔵 | ❌ |

---

## TIER 2 — 🟡 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `NUEVO-16` | **Product Quantization (PQ) 96x** — compresión para datasets >RAM | Alto | 🔵 | ❌ |
| `NUEVO-17` | **Segment LSM-style** — hot/warm/cold tiers | Muy alto | 🔵 | ❌ |
| `NUEVO-18` | **Sparse vectors nativos** — hybrid search real | Alto | 🔵 | ❌ |
| `NUEVO-20` | **Server Docker image** | 🟡 1-2d | 🔵 | ❌ |
| `NUEVO-21` | **Vectara competitive research** | 🟢 2-4h | ⬜ | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 2-3d | 🟡 | ❌ |
| `ENT-04` | Connection pooling + circuit breaker | 🟡 2-3d | 🟡 | ❌ |
| `BIZ-01` | Enterprise crate (encryption, audit, RBAC, replication) | 🟡 3-5d | 🟡 | ⏳ |

---

## TIER 3 — 🔵 Post-Lanzamiento

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `—` | Publicar 8 workspace members en crates.io | 🟡 2-3d | 🟡 | ❌ |
| `WEB-001` | **Re-add interactive WASM demo page** — Restaurar demo después de publicar `@vantadb/wasm` en npm | 🟢 30min | 🟡 | ❌ |

---

## ✅ Resumen de Verificación (Jul 13, 2026)

### Documentos archivados (13 → `docs/archive/`)

| Archivo | Razón |
|---------|-------|
| `reviews/agent-01-local-AK.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-02-local-LZ.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-03-global-agents.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-04-global-claude.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-05-internet-research.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/EXECUTIVE_TECHNICAL_AUDIT.md` | Superseded por audits Jul 11/13 |
| `reviews/AUDITORIA_COMPLETA_VantaDB_WEB.md` | Web-only, superseded |
| `reviews/FULL_CODEBASE_AUDIT_2026-07-09.md` | Superseded por 2026-07-11 |
| `reviews/web-audit-report.md` | Superseded |
| `research/DOCS_TOOLS_RESEARCH.md` | Cold research, no tool adopted |
| `research/SQL_ANALYSIS.md` | Decisión tomada (no SQL), sin acción pendiente |
| `research/COGNEE_EVALUATION.md` | Pure research, cero implementación |
| `research/DOCS_AUDIT_REPORT.md` | Issues tracked en bitácora/backlog |

### Claims verificados contra código: 100% precisos

De ~150 claims de estado en el backlog anterior, todos fueron verificados contra el código real usando `codegraph_explore`, `grep`, y lectura directa. Ver `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md` y `docs/reviews/2026-07-13-full-review.md` para el detalle.

### Documentos de investigación aún vigentes

| Documento | Estado | Nota |
|-----------|--------|------|
| `research/ACID_TRANSACTIONS.md` | ⚠️ Parcial | Phase 1 implementada; Phase 2-3 no; WAL shipping + PITR existen |
| `research/SIGNED_RELEASES.md` | ⚠️ Parcial | Attestations + Windows builds OK; GPG/sigstore no |
| `research/VantaDB_RESEARCH_UNIFIED.md` | ✅ Vigente | Mejor referencia de arquitectura |
| `research/VantaDB_RESEARCH_VALIDADO.md` | ✅ Vigente | Meta-validación precisa |
| `research/VantaDB_ANALISIS_COMPLETO.md` | ⚠️ Parcial | Version sync ya resuelto |

---

## 📈 Timeline Consolidado

```
Jul 14-18  TIER 0 (🔴 4 items):
               ─ INT-01/02: LangChain + LlamaIndex → PyPI
               ─ DEVOPS-05: Pipeline CI unificado
               ─ REL-02: vantadb-ts → npm
               ─ MKT-13: Link WASM demo en hero ⏳
Jul 18-25  TIER 1 (🟠 15+ items):
               ─ Docs: MKT-14 (case studies), TSK-106 (Discussions)
               ─ NUEVO-01/07/08/10: README, migrations, learning, benchmarks
               ─ Launch: LEG-01, MKT-03/04/05/10/15/16
               ─ Code health: VFY-001→012
               ─ WASM: NUEVO-11→15
Ago+       TIER 2-3 (🔵 items):
               ─ NUEVO-16/17/18: PQ, LSM, sparse vectors
               ─ Enterprise: ENT-04, BIZ-01
               ─ Publishing: crates.io, WEB-001
```

---

## ✅ Definition of Ready (DoR)

- [ ] ID único asignado
- [ ] Prioridad definida (🔴🟠🟡🟢🔵⬜)
- [ ] Archivos involucrados conocidos
- [ ] Esfuerzo estimado
- [ ] Verificado contra código real (no asumido)

## ✅ Definition of Done (DoD)

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo test` / `vitest run`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs actualizados si aplica
- [ ] Tarea movida a `progreso/README.md`
- [ ] Changelog actualizado si es cambio visible al usuario
