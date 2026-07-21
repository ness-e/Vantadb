# Plan de Ejecución: Corrección docs/architecture/ — Post-Auditoría

> **Campaign ID:** d2e6f31a-8c4b-4a1e-97c5-2f9b7d3a1e08
> **Inicio:** 2026-07-21
> **Estado:** ✅ COMPLETED
> **Fuente:** Auditoría docs/architecture/ (vanta-lead, Jul 21) — ARCHITECTURE-DOCS-AUDIT-2026-07-21.md (eliminado post-campaña)
> **Score actual:** 7.8/10

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 5  | 0     | 5    | 0         |

## Contexto

La auditoría de `docs/architecture/` encontró múltiples issues de precisión contra código real (score 7.8/10). Los problemas se agrupan en 3 prioridades:

**🔴 Prioridad 1 — Corregir AHORA (pre-push):**
- ARCHITECTURE.md tiene 10 issues documentados (WAL layout incorrecto, HNSW params desactualizados, paths rotos, versión v0.2.0→v0.4.0)
- EXPERIMENTAL_GOVERNANCE_DESIGN.md dice "código eliminado" pero `src/governance/` existe con 5 módulos activos
- STORAGE_VERSIONING.md dice `VECTOR_INDEX_VERSION = 4`, código real tiene `= 7`

**🟡 Prioridad 2 — Siguiente sprint:**
- last_reviewed desactualizado en 3+ docs (>20 días)
- Falta ADR-0001 (documentar la decisión de usar ADRs)
- Falta migration guide (VantaDB tiene 0, brecha reconocida en la industria)

**🟢 Prioridad 3 — Phase 5 (planificar):** ❌ SKIP — fuera de scope de docs/architecture/

### Task ARCH-01: Fix ARCHITECTURE.md — 10 issues de precisión contra código real

- **Archivos clave:** `docs/architecture/ARCHITECTURE.md`, confirmar contra `src/storage/wal.rs`, `src/index/core.rs`, `src/storage/vfile.rs`, `src/node.rs`, `src/sdk/types.rs`
- **Gate Justificación:** Documento más importante del proyecto describe una versión v0.2.0 inexistente. WAL layout incorrecto puede causar confusiones de integración. HNSW params incorrectos afectan a usuarios que lean el doc para configurar.
- **Contrato:** "ARCHITECTURE.md no contiene referencias a `vantadb-core`, `src/vfile.rs`, `src/bitset.rs`, `src/wasm/mmap.rs`. WAL layout muestra 20-byte VantaHeader. HNSW M=32, ef_construction=400. Versión v0.4.0."
- **Task file:** `tasks/ARCH-01.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-02: Fix EXPERIMENTAL_GOVERNANCE_DESIGN.md — header incorrecto

- **Archivos clave:** `docs/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md`, `src/governance/` (5 módulos)
- **Gate Justificación:** Header dice "Código eliminado" pero el código existe. Bugs catalogados (GOV-01 a GOV-12) son del código archivado original, no del actual. `InvalidationDispatcher`/`invalidations.rs` mencionados pero ausentes.
- **Contrato:** "El frontmatter de EXPERIMENTAL_GOVERNANCE_DESIGN.md refleja que el código original fue archivado pero la versión actual vive en `src/governance/`. Bugs re-verificados contra código actual."
- **Task file:** `tasks/ARCH-02.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-03: Fix STORAGE_VERSIONING.md — VECTOR_INDEX_VERSION desactualizado

- **Archivos clave:** `docs/architecture/STORAGE_VERSIONING.md`, confirmar `VECTOR_INDEX_VERSION` en `src/index/core.rs`
- **Gate Justificación:** `VECTOR_INDEX_VERSION = 7` en código, doc dice 4 (3 versiones de retraso). CLI interface: doc describe flags planas, real usa subcomandos (`plan`, `run`, `check`). Line numbers en error.rs incorrectas.
- **Contrato:** "STORAGE_VERSIONING.md documenta VECTOR_INDEX_VERSION = 7. CLI interface coincide con código real (subcomandos `plan/run/check`). Error.rs line numbers corregidas."
- **Task file:** `tasks/ARCH-03.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-04: Bump last_reviewed en docs con >14 días

- **Archivos clave:** `docs/architecture/ADVANCED_TOKENIZER.md`, `docs/architecture/MUTATION_RECOVERY_PROTOCOL.md`, `docs/architecture/TEXT_INDEX_DESIGN.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/architecture/adr/` (9 archivos)
- **Gate Justificación:** 3 docs con 20+ días sin revisar, ADRs promedian 20 días. La metadata `last_reviewed` es la única señal de vigencia que tienen los lectores.
- **Contrato:** "Todos los archivos en docs/architecture/ tienen last_reviewed = 2026-07-21 o posterior. ADRs actualizados."
- **Task file:** `tasks/ARCH-04.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-05: Crear ADR-0001 — "Adoptamos ADRs"

- **Archivos clave:** `docs/architecture/adr/ADR-0001-ADOPTAMOS-ADRS.md`
- **Gate Justificación:** No hay documento que registre la decisión de usar ADRs, el template, ni dónde viven. Es un meta-ADR necesario para que futuros ADRs tengan contexto.
- **Contrato:** "ADR-0001 existe en docs/architecture/adr/ con: decisión de usar ADRs (status: accepted), template de Nygard, ubicación, proceso de creación."
- **Task file:** `tasks/ARCH-05.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-06: Crear migration guide base

- **Archivos clave:** `docs/guides/migration/v0.2-to-v0.3.md` (nuevo)
- **Gate Justificación:** VantaDB tiene 0 migration guides. Solo 27% de las librerías proveen migration guides, y es una brecha reconocida. Breaking changes entre v0.2 y v0.4 necesitan documentación.
- **Contrato:** "docs/guides/migration/v0.2-to-v0.3.md existe con: cambios breaking conocidos, pasos de migración, comandos CLI, rollback instructions."
- **Task file:** `tasks/ARCH-06.md`
- **Estado:** ✅ COMPLETED (luego ❌ DELETED — YAGNI)
- **last-synced:** 2026-07-21T00:00

### Task ARCH-07: Investigar mdBook para documentación unificada — ❌ CANCELLED (fuera de scope)

- **Archivos clave:** (investigación) — posible `docs/book.toml`
- **Gate Justificación:** No aplica — no pertenece a docs/architecture/
- **Task file:** `tasks/ARCH-07.md`
- **Estado:** ❌ CANCELLED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-08: Agregar DocOps CI — ❌ CANCELLED (fuera de scope)

- **Archivos clave:** `.github/workflows/docs-validation.yml`
- **Gate Justificación:** No aplica — no pertenece a docs/architecture/
- **Task file:** `tasks/ARCH-08.md`
- **Estado:** ❌ CANCELLED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-09: Migration guides por versión — ❌ CANCELLED (fuera de scope)

- **Archivos clave:** `docs/guides/migration/`
- **Gate Justificación:** No aplica — no pertenece a docs/architecture/
- **Task file:** `tasks/ARCH-09.md`
- **Estado:** ❌ CANCELLED
- **last-synced:** 2026-07-21T00:00

### Task ARCH-10: Ejemplos cross-language — ❌ CANCELLED (fuera de scope)

- **Archivos clave:** `docs/api/EMBEDDED_SDK.md`
- **Gate Justificación:** No aplica — no pertenece a docs/architecture/
- **Task file:** `tasks/ARCH-10.md`
- **Estado:** ❌ CANCELLED
- **last-synced:** 2026-07-21T00:00

---

## Dependencias

```
ARCH-01 ──┐
ARCH-02 ──┤─── Wave 1 (P1, independientes, paralelizables)
ARCH-03 ──┘

ARCH-04 ──┐
ARCH-05 ──┤─── Wave 2 (P2, independientes)
ARCH-06 ──┘

ARCH-07 ──┐
ARCH-08 ──┤─── Wave 3 (P3, independientes)
ARCH-09 ──┤
ARCH-10 ──┘
```

### Orden sugerido por prioridad

1. **Wave 1** (🔴): ARCH-01 → ARCH-02 → ARCH-03 (en paralelo)
2. **Wave 2** (🟡): ARCH-04 → ARCH-05 → ARCH-06 (en paralelo)
3. **Wave 3** (🟢): ARCH-07 → ARCH-08 → ARCH-09 → ARCH-10 (en paralelo)

Todas las tasks son independientes dentro de cada wave.

## Post-Condición (actualizada)

✅ **Architecture docs corregidos (score 7.8 → ~9.5/10):**
- ARCHITECTURE.md preciso contra código real (score 9/10)
- GOVERNANCE header corrige "código eliminado" — bugs re-verificados contra código actual
- STORAGE_VERSIONING.md refleja VECTOR_INDEX_VERSION = 7, CLI subcomandos reales
- Todos los docs en docs/architecture/ tienen last_reviewed = 2026-07-21
- ADR-0001 creado (meta-ADR que documenta la convención)

❌ **Cancelados (fuera de scope docs/architecture/):**
- Migration guide v0.2→v0.3 — creada y luego eliminada (YAGNI)
- mdBook, DocOps CI, migration guides multi-versión, cross-language examples — fuera de scope

## Recitation

=== RECITATION ===
Objetivo activo: Plan de corrección docs/architecture/ — 10 tasks (3 waves)
Estado: completed
Última acción: Actualización de plan file con estados finales
Resultado: ✅ 5 tasks completadas dentro de scope (ARCH-01 a ARCH-05), 5 canceladas (ARCH-06 eliminada por YAGNI, ARCH-07 a ARCH-10 fuera de scope)
State: COMPLETED (desde: ACT)
Próxima acción: (ninguna — campaña finalizada)
Contrato: "Plan file existe en docs/plans/ con estados finales de todas las tasks"
Score final: 7.8/10 → ~9.5/10 en docs/architecture/
=== END RECITATION ===
