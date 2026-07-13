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
| `INT-01` | **LangChain adapter → PyPI** — Publicar `vantadb-langchain` en PyPI con CI pipeline | `vantadb-langchain/Cargo.toml`, `.github/workflows/` | 🟡 1-2d | `pip install vantadb-langchain` |
| `INT-02` | **LlamaIndex adapter → PyPI** — Publicar `vantadb-llamaindex` en PyPI | `vantadb-llamaindex/Cargo.toml`, `.github/workflows/` | 🟡 1-2d | `pip install vantadb-llamaindex` |
| `REL-02` | **vantadb-ts → npm** — Publicar `@vantadb/sdk` en npm | `vantadb-ts/package.json`, `.github/workflows/` | 🟡 1-2d | `npm install @vantadb/sdk` |
| `MKT-13` | **Enlazar demo WASM desde hero** — Botón "Try in browser" en NbTerminalHero | `web/src/components/` | 🟢 1-2h | Link visible en hero → `/demo` |
| `DEVOPS-05` | **Pipeline CI unificado adapters** — Unificar publish de 10 adapters a PyPI | `.github/workflows/` | 🟡 1-2d | CI publish all adapters |

### TIER 1 — 🟠 Código (12 tareas técnicas)

| ID | Tarea | Archivos | Esfuerzo |
|----|-------|----------|----------|
| `VFY-001` | **TS SDK catch {} silencioso** — 4+ bloques catch vacíos | `vantadb-ts/src/vantadb.ts:176,215,249` | 🟢 2h |
| `VFY-002` | **get_nns_by_id spawn por llamada** — Sin batching | `vantadb-ts/src/vantadb.ts:325` | 🟢 2h |
| `VFY-003` | **reindex_hnsw_from_text OOM** — Sin batch processing | `vantadb-python/src/lib.rs:1584` | 🟡 1d |
| `VFY-004` | **flat.rs O(n²) en filter** — Sin índice para filtros | `src/index/flat.rs:32` | 🟡 1-2d |
| `VFY-005` | **TS OperationalMetrics incompleto** — 3/10 métricas | `vantadb-ts/src/types.ts:148-168` | 🟢 4h |
| `VFY-006` | **add_node write lock toda la inserción** | `src/index/graph.rs:476-490` | 🟡 1-2d |
| `VFY-007` | **remove_node O(n²) neighbor fixup** | `src/index/core.rs` | 🟡 1-2d |
| `VFY-008` | **WAL fsync por escritura** — Write amplification | `src/storage/wal.rs` | 🟡 1-2d |
| `VFY-009` | **637 inline styles no migrados a Tailwind** | `web/src/` | 🟡 3-5d |
| `VFY-012` | **musllinux target gap** | CI config | 🟢 4h |
| `NUEVO-15` | **Code coverage report en CI** | `.github/workflows/` | 🟢 1d |
| `NUEVO-19` | **Mover SourceDesign/ fuera de web/src/** | `web/src/SourceDesign/` | 🟢 1h |

### TIER 1 — 🟠 Web & Contenido (9 tareas no-code)

| ID | Tarea | Esfuerzo |
|----|-------|----------|
| `MKT-14` | Case studies page `/case-studies/` | 🟡 1-2d |
| `TSK-106` | Habilitar GitHub Discussions | 🟢 1h |
| `NUEVO-01` | README hero con benchmarks + GIF demo | 🟡 2-3d |
| `NUEVO-07` | Migration tools Chroma→Vanta, LanceDB→Vanta | 🟡 3-5d |
| `NUEVO-08` | Learning path en tutorials/ | 🟡 2-3d |
| `NUEVO-10` | Benchmark suite pública reproducible | 🟡 3-5d |
| `TSK-107` | Community showcase page | 🟢 4-6h |

### TIER 1 — 🟠 WASM & Performance (6 tareas)

| ID | Tarea | Esfuerzo |
|----|-------|----------|
| `NUEVO-11` | WASM IndexedDB fallback | 🟡 2-3d |
| `NUEVO-12` | WASM multi-tab coordination | 🟡 2-3d |
| `NUEVO-13` | HNSW auto-tuning PID loop | 🟡 3-5d |
| `NUEVO-14` | WASM bundle size <500KB gzip | 🟡 1-2d |

### TIER 2-3 — 🔵 Features Avanzadas (8 tareas)

| ID | Tarea | Esfuerzo |
|----|-------|----------|
| `NUEVO-16` | Product Quantization (PQ) 96x | Alto |
| `NUEVO-17` | Segment LSM-style hot/warm/cold | Muy alto |
| `NUEVO-18` | Sparse vectors nativos | Alto |
| `NUEVO-20` | Server Docker image | 🟡 1-2d |
| `VFY-010` | ACID Phase 2: Buffered write transactions | 🟡 2-3d |
| `VFY-011` | ACID Phase 3: Snapshot isolation / MVCC | 🟠 3-5d |
| `ENT-04` | Connection pooling + circuit breaker | 🟡 2-3d |
| `BIZ-01` | Enterprise crate (encryption, audit, RBAC) | 🟡 3-5d |


---

## Orden de Ejecución Recomendado (Fases)

### Fase 1: Quick Wins (día 1-2)
```
VFY-001 → VFY-002 → VFY-005 → NUEVO-19 → VFY-012 → MKT-13
```

### Fase 2: Publicación SDKs (día 2-4)
```
INT-01 → INT-02 → REL-02 → DEVOPS-05
```

### Fase 3: Core fixes (día 3-7)
```
VFY-003 → VFY-004 → VFY-006 → VFY-007 → VFY-008 → NUEVO-15
```

### Fase 4: Web & WASM (día 5-10)
```
VFY-009 → NUEVO-11 → NUEVO-12 → NUEVO-14
```

### Fase 5: Contenido & Marketing (día 7-14)
```
NUEVO-01 → MKT-14 → NUEVO-07 → NUEVO-08 → NUEVO-10 → TSK-107
> TSK-106 (GitHub Discussions) requiere humano — skip en loop
```

### Fase 6: Features Avanzadas (semana 3-6)
```
VFY-010 → ENT-04 → NUEVO-13 → BIZ-01 → VFY-011
```

### Fase 7: Enterprise (mes 2+)
```
NUEVO-16 → NUEVO-17 → NUEVO-18 → NUEVO-20
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

| Fase | Total | ✅ | ❌ | ➖ |
|------|-------|----|----|-----|
| Fase 1: Quick Wins | 6 | 0 | 6 | 0 |
| Fase 2: Publicación | 4 | 0 | 4 | 0 |
| Fase 3: Core fixes | 6 | 0 | 6 | 0 |
| Fase 4: Web & WASM | 4 | 0 | 4 | 0 |
| Fase 5: Contenido | 6 | 0 | 6 | 0 |
| Fase 6: Features | 5 | 0 | 5 | 0 |
| Fase 7: Enterprise | 4 | 0 | 4 | 0 |
| **Total ejectable** | **35** | **0** | **35** | **0** |
| Requiere humano (en Backlog.md) | 13 | — | — | — |

---

## Recitation Block (última iteración)

> **Última tarea completada:** (ninguna)
> **Próxima tarea:** VFY-001 — TS SDK catch {} silence
> **Fase actual:** Fase 1: Quick Wins
> **Bloqueadores:** Ninguno
> **Check passes:** (no aplica)
