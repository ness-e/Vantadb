# Plan: P4 Engineering Health — Wave 0

**Fecha:** 2026-07-25
**Fuente:** docs/Backlog.md (P4 tier)
**FAIL_MODE:** parallel (Wave 0 = 4 tasks independientes)

## Resumen

| Estado | Cantidad |
|--------|----------|
| ✅ DO | 4 |
| 🟡 DEFER | 4 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

## Gate Justificación

| ID | Decisión | Por qué |
|----|----------|---------|
| WEB-04 | ✅ DO | Sin migration path = corrupción silenciosa. Riesgo real pre-1.0 |
| DRV-121 | ✅ DO | CBO incompleto = plans sub-óptimos. Base para DRV-131 (more indexes) |
| DRV-123 | ✅ DO | Auto-embedding ya implementado parcialmente bajo `remote-inference`. Falta polish/feature flag |
| DOC-20 | ✅ DO | Docs fragmentados sin search = onboarding pobre. mdBook es effort bajo (~2-3d) |
| VFY-011 | 🟡 DEFER | MVCC es arquitectura pesada. Requiere diseño antes de implementación. Post-1.0 |
| DRV-122 | 🟡 DEFER | IQL JOINs dependen de DRV-121 (CBO) como prerequisito |
| DRV-130 | 🟡 DEFER | SIFT benchmark es validación externa. No bloquea nada ahora |
| DRV-131 | 🟡 DEFER | More index types depende de DRV-121 para integración con planner |

## Wave 0 (paralelo)

| ID | Descripción | Archivos clave | Agente | Esfuerzo |
|----|-------------|----------------|--------|----------|
| WEB-04 | Storage format versioning (draft→implement) | `docs/architecture/STORAGE_VERSIONING.md` | vanta-arch | 🟠 3-5d |
| DRV-121 | Planner CBO optimization | `src/query.rs`, `src/planner.rs` | vanta-engine | 🟠 3-5d |
| DRV-123 | Auto-embedding on INSERT (remote-inference) | `src/llm.rs`, `src/executor.rs` | vanta-worker | 🟡 2-3d |
| DOC-20 | mdBook adoption for docs site | root `book.toml`, `docs/book/` | vanta-docs | 🟡 2-3d |

## Tareas

### WEB-04: Storage format versioning (draft→implement)

- **Estado:** ⬜ PENDING
- **Backlog:** Line 128 — Sin migration path para VantaFile/HNSW/WAL. 4 formatos catalogados
- **Archivos clave:** `docs/architecture/STORAGE_VERSIONING.md`
- **Contrato:** `docs/architecture/STORAGE_VERSIONING.md` existe con draft implementado + codegraph_explore verifica que WAL/HNSW/VantaFile tienen version markers
- **Routing:** vanta-arch (arquitectura de almacenamiento, formatos)

### DRV-121: Planner CBO optimization

- **Estado:** ⬜ PENDING
- **Backlog:** Line 131 — `LogicalPlan` existe en `query.rs:210`, `PhysicalOperator` trait en `:283`, `optimize_and_compile` en `planner.rs:181`. Falta optimización CBO
- **Archivos clave:** `src/query.rs`, `src/planner.rs`
- **Contrato:** `cargo check -p vantadb` pasa + tests de planner existentes pasan + `optimize_and_compile` aplica al menos 1 rule CBO (predicate pushdown o projection pruning)
- **Routing:** vanta-engine (planner, query optimization)

### DRV-123: Auto-embedding on INSERT

- **Estado:** ⬜ PENDING
- **Backlog:** Line 133 — Implementado en `executor.rs:228-242` + `:353-360` bajo `#[cfg(feature = "remote-inference")]`. No es default
- **Archivos clave:** `src/llm.rs`, `src/executor.rs`
- **Contrato:** `cargo check -p vantadb --features remote-inference` pasa + tests de auto-embedding corren
- **Routing:** vanta-worker (Rust core, business logic)

### DOC-20: mdBook docs site ✅

- **Estado:** ✅ COMPLETED (`1f9f681d`)
- **Backlog:** Line 136 — Docs fragmentados, sin search unificado. No existe `book.toml` ni `docs/book/`
- **Archivos clave:** root (para `book.toml`), `docs/` (para contenido)
- **Contrato:** `mdbook build docs/book/` produce `docs/book/book/` con index.html funcional + search — ✅ build exitoso
- **Routing:** vanta-docs (documentación, API specs)
- **Resultado:** `docs/book/book.toml`, `docs/book/src/SUMMARY.md` (9 secciones), 73 `{{#include}}` stubs. Cero duplicación.
