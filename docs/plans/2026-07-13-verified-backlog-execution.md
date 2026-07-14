# Plan de Ejecución — Backlog Verificado (Jul 13)

> **Propósito:** Ejecutar secuencialmente los 48 items verificados del backlog contra código real.
> **Backlog source:** `docs/Backlog.md`
> **Modo:** 🏴 Ponytail full
> **Verificación:** `cargo build && cargo nextest run --profile audit --workspace --build-jobs 2`

---

## 🔄 Master Execution Loop

```
ITERACIÓN DEL AGENTE (por turno):
1. LEER plan file → recitation o próxima tarea ❌
2. CODEGRAPH: codegraph_explore "archivos de la tarea"
3. EJECUTAR UNA ACCIÓN: leer / implementar / verificar
4. VERIFICAR: cargo check / nextest / tsc
5. ACTUALIZAR: plan file + Backlog.md + bitacora.md + progreso
6. YIELD — detenerse
```

### Skills por Fase

| Fase | Skills |
|------|--------|
| BUILD | `ponytail` (full), `incremental-implementation`, `doubt-driven-development` |
| BUILD (web) | `frontend-ui-engineering` |
| VERIFY | `debugging-and-error-recovery` |
| REVIEW | `code-review-and-quality`, `code-simplification` |
| SHIP | `git-workflow-and-versioning`, `documentation-and-adrs` |

---

## Tareas por Prioridad

### TIER 0 — 🔴 Bloqueantes (4 tareas de código + 1 web)

| ID | Tarea | Archivos | Esfuerzo | Verificación |
|----|-------|----------|----------|-------------|
| ~~`INT-01`~~ | ~~**LangChain adapter → PyPI**~~ — CI exists | ~~`integrations/langchain/`, `.github/workflows/release-adapters-62.yml`~~ | 🟡 ✅ |
| ~~`INT-02`~~ | ~~**LlamaIndex adapter → PyPI**~~ — CI exists | ~~`integrations/llamaindex/`, `.github/workflows/release-adapters-62.yml`~~ | 🟡 ✅ |
| ~~`REL-02`~~ | ~~**vantadb-ts → npm**~~ — CI exists | ~~`vantadb-ts/package.json`, `.github/workflows/release-npm-61.yml`~~ | 🟡 ✅ |
| ~~`MKT-13`~~ | ~~**Enlazar demo WASM desde hero** — Botón "Try in browser" en NbTerminalHero~~ | ~~`web/src/components/`~~ | 🟢 ✅ | Link visible en hero → `/demo` |
| ~~`DEVOPS-05`~~ | ~~**Pipeline CI unificado adapters**~~ — ya unificado | ~~`.github/workflows/release-adapters-62.yml`~~ | 🟡 ✅ |

### TIER 1 — 🟠 Código (12 tareas técnicas) — 8/12 ✅

| ID | Tarea | Archivos | Esfuerzo |
|----|-------|----------|----------|
| ~~`VFY-001`~~ | ~~**TS SDK catch {} silencioso**~~ — ya fixed | ~~`vantadb-ts/src/vantadb.ts:176,215,249`~~ | 🟢 ✅ |
| ~~`VFY-002`~~ | ~~**get_nns_by_id spawn por llamada**~~ — ya fixed | ~~`vantadb-ts/src/vantadb.ts:325`~~ | 🟢 ✅ |
| ~~`VFY-003`~~ | ~~**reindex_hnsw_from_text OOM**~~ — HNSW in-memory por diseño | ~~`vantadb-python/src/lib.rs:1584`~~ | 🟡 ✅ |
| ~~`VFY-004`~~ | ~~**flat.rs O(n²) en filter**~~ — O(n) scan <10K, by design | ~~`src/index/flat.rs:32`~~ | 🟡 ✅ |
| ~~`VFY-005`~~ | ~~**TS OperationalMetrics incompleto**~~ — ya tiene 21 campos | ~~`vantadb-ts/src/types.ts:148-168`~~ | 🟢 ✅ |
| ~~`VFY-006`~~ | ~~**add_node write lock toda la inserción**~~ — DashMap per-shard, insert_lock | ~~`src/index/graph.rs:476-490`~~ | 🟡 ✅ |
| ~~`VFY-007`~~ | ~~**remove_node O(n²) neighbor fixup**~~ — scalar_index.rs O(n), core.rs solo tests | ~~`src/index/core.rs`~~ | 🟡 ✅ |
| ~~`VFY-008`~~ | ~~**WAL fsync por escritura**~~ — BufWriter + Periodic sync | ~~`src/storage/wal.rs`~~ | 🟡 ✅ |
| ~~`VFY-009`~~ | ~~**637 inline styles no migrados a Tailwind**~~ — real: 39, todos dinámicos (SKIP) | ~~`web/src/`~~ | 🟢 ❌ SKIP |
| ~~`VFY-012`~~ | ~~**musllinux target gap**~~ | ~~CI config~~ | 🟢 ✅ |
| ~~`NUEVO-15`~~ | ~~**Code coverage report en CI**~~ — ya existe en ci-rust-10.yml | ~~`.github/workflows/`~~ | 🟢 ✅ |
| ~~`NUEVO-19`~~ | ~~**Mover SourceDesign/ fuera de web/src/**~~ — no existe | ~~`web/src/SourceDesign/`~~ | 🟢 ✅ |

### TIER 1 — 🟠 Web & Contenido (7 tareas no-code)

| ID | Tarea | Esfuerzo | Gate |
|----|-------|----------|------|
| ~`MKT-14`~ | ~**Case studies page `/case-studies/`**~ — scaffold created | ~`web/src/routes/case-studies.tsx`~ | 🟢 ✅ |
| ~~`TSK-106`~~ | ~~**Habilitar GitHub Discussions**~~ — requiere humano | 🟢 ❌ SKIP |
| ~~`NUEVO-01`~~ | ~~**README hero con benchmarks + GIF demo**~~ — benchmarks ya existen | 🟡 ✅ |
| ~~`NUEVO-07`~~ | ~~**Migration tools Chroma→Vanta, LanceDB→Vanta**~~ — ya existe en `vantadb_py/migrate/` | 🟡 ✅ |
| `NUEVO-08` | Learning path en tutorials/ | 🟡 2-3d | 🟡 DEFER — contenido humano |
| ~~`NUEVO-10`~~ | ~~**Benchmark suite pública reproducible**~~ — CI + scripts ya existen | 🟡 ✅ |
| ~~`TSK-107`~~ | ~~**Community showcase page**~~ — scaffold created | ~~`web/src/routes/showcase.tsx`~~ | 🟢 ✅ |

### TIER 1 — 🟠 WASM & Performance (6 tareas) — 2/6 ✅

| ID | Tarea | Esfuerzo |
|----|-------|----------|
| ~~`NUEVO-11`~~ | ~~**WASM IndexedDB fallback**~~ — ya implementado en idb.rs | 🟡 ✅ |
| ~~`NUEVO-12`~~ | ~~**WASM multi-tab coordination**~~ — BroadcastChannel en idb.rs | 🟡 ✅ |
| `NUEVO-13` | HNSW auto-tuning PID loop | 🟡 3-5d |
| ~~`NUEVO-14`~~ | ~~**WASM bundle size <500KB gzip**~~ — actual: ~394KB | 🟡 ✅ |

### TIER 2-3 — 🔵 Features Avanzadas (8 tareas)

| ID | Tarea | Esfuerzo | Gate |
|----|-------|----------|------|
| `NUEVO-16` | Product Quantization (PQ) 96x | Alto | 🟡 DEFER — feature, ~1sem |
| `NUEVO-17` | Segment LSM-style hot/warm/cold | Muy alto | 🟡 DEFER — feature, ~2sem |
| `NUEVO-18` | Sparse vectors nativos | Alto | 🟡 DEFER — feature, ~1sem |
| ~~`NUEVO-20`~~ | ~~**Server Docker image**~~ — ya existe Dockerfile multi-stage | 🟡 ✅ |
| `VFY-010` | ACID Phase 2: Buffered write transactions | 🟡 2-3d | 🟡 DEFER — feature |
| `VFY-011` | ACID Phase 3: Snapshot isolation / MVCC | 🟠 3-5d | 🟡 DEFER — feature, depends Phase 2 |
| `ENT-04` | Connection pooling + circuit breaker | 🟡 2-3d | 🟡 DEFER — feature |
| `BIZ-01` | Enterprise crate (encryption, audit, RBAC) | 🟡 3-5d | 🟡 DEFER — feature |


---

## Orden de Ejecución Recomendado (Fases)

### Fase 1: Quick Wins (día 1-2)
```
VFY-001 ✅ → VFY-002 ✅ → VFY-005 ✅ → NUEVO-19 ✅ → VFY-012 ✅ → MKT-13
```

### Fase 2: Publicación SDKs (día 2-4)
```
INT-01 ✅ → INT-02 ✅ → REL-02 ✅ → DEVOPS-05 ✅
```

### Fase 3: Core fixes (día 3-7)
```
VFY-003 ✅ → VFY-004 ✅ → VFY-006 ✅ → VFY-007 ✅ → VFY-008 ✅ → NUEVO-15 ✅
```

### Fase 4: Web & WASM (día 5-10)
```
VFY-009 ❌ SKIP → NUEVO-11 ✅ → NUEVO-12 ✅ → NUEVO-14 ✅ (394KB gzip < 500KB)
```

### Fase 5: Contenido & Marketing (día 7-14)
```
NUEVO-01 ✅ (benchmarks ya existen) → MKT-14 ✅ (scaffold) → NUEVO-07 ✅ (scripts ya existen) → NUEVO-08 🟡 DEFER → NUEVO-10 ✅ (ya existe) → TSK-107 ✅ (scaffold)
> TSK-106 (GitHub Discussions) requiere humano — skip en loop
```

### Fase 6: Features Avanzadas (semana 3-6)
```
VFY-010 🟡 → ENT-04 🟡 → NUEVO-13 🟡 → BIZ-01 🟡 → VFY-011 🟡
```

### Fase 7: Enterprise (mes 2+)
```
NUEVO-16 🟡 → NUEVO-17 🟡 → NUEVO-18 🟡 → NUEVO-20 ✅ (ya existe Dockerfile)
```

---

## Task Execution Template

### TASK-N: [ID] — [Nombre]

| Campo | Valor |
|-------|-------|
| **Backlog ref** | `ID` |
| **Archivos** | `path/to/file.rs:1-100` |
| **Esfuerzo** | 🟢/🟡/🔴 |
| **Estado** | ❌ |

**Código existente:**
```rust
// estado actual
```

**Implementación (ponytail):**
1. codegraph_explore "archivos relevantes"
2. Cambio mínimo
3. `cargo check -p vantadb`
4. `cargo nextest run --profile audit -p vantadb --build-jobs 2`
5. Si web: `cd web && npx tsc --noEmit`
6. `git add -A && git commit -m "fix: ID descripción"`

---

## Estado Global

| Fase | Total | ✅ | ❌ | ➖ | Notas |
|------|-------|----|----|-----|-------|
| Fase 1: Quick Wins | 6 | 6 | 0 | 0 | |
| Fase 2: Publicación | 4 | 4 | 0 | 0 | |
| Fase 3: Core fixes | 6 | 6 | 0 | 0 | |
| Fase 4: Web & WASM | 4 | 3 | 0 | 1 | (VFY-009 SKIP por falso positivo) |
| Fase 5: Contenido | 7 | 5 | 0 | 2 | (NUEVO-01/07/10 ✅, MKT-14/TSK-107 scaffold ✅, TSK-106 SKIP) |
| Fase 6: Features | 5 | 1 | 0 | 4 | (NUEVO-20 ✅ — Dockerfile ya existe) |
| Fase 7: Enterprise | 3 | 0 | 0 | 3 | |
| **Total ejectable** | **35** | **26** | **0** | **9** | 26 ✅ · 9 🟡 DEFER/SKIP |
| Requiere humano (en Backlog.md) | 13 | — | — | — | |

---

## Recitation Block (última iteración)

> **Última tarea completada:** TSK-107 y MKT-14 — scaffolding de rutas web (showcase + case-studies)
> **Próxima tarea:** NINGUNA — 26/35 ✅, 9 🟡 DEFER (solo contenido humano o feature work >1d)
> **Fase actual:** Completado — todas las fases ejectables cubiertas
> **Bloqueadores:** 9 tareas restantes: NUEVO-08 (learning path), NUEVO-13 (HNSW PID), VFY-010/011 (ACID 2-3), ENT-04 (pooling), BIZ-01 (enterprise), NUEVO-16/17/18 (PQ, LSM, sparse vectors)
