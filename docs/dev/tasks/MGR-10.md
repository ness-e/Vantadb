---
title: "MGR-10: research-doc — bitemporalidad (dim 5)"
kind: task
description: "research-doc cerrado con modelo valid-time vs transaction-time, tradeoffs (append-only vs invalidación + storage del historial) y plan de migración/backfill determinista, listo para SCH-01\""
---

# MGR-10: research-doc — bitemporalidad (dim 5)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 23, Fase F3 — bloque L599-623)
- **Fuente:** Backlog:839 (P49 MGR-10, dim 5/AM4) + Notion dim 5 (Propuesta Anexo A) + decisión owner 2026-09-24 (migración única MGR-10/12/13 → 0.8.0)
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** Docs (research/design, doc-only)
- **Turns estimados:** 15-30 (ejecutado en 1 sesión de discovery+redacción)
- **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ IN PROGRESS — steps ✅ completos; **pendiente review P2-01 + commit (LEAD)** por instrucción del orquestador (NO self-review)
- **Incógnitas (uphill):** 0 abiertas (las 6 decisiones de owner son output del Cierre MGR, no bloqueos de la tarea)
- **Pendientes (downhill):** 0 steps propios; handoff a SCH-01

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno (2 archivos NUEVOS: research-doc + task file; `src/` no se toca) |
| Callees | ninguno (docs-only; se apoya en lectura de `src/sdk/types/record.rs`, `src/sdk/version_history.rs`, `src/schema.rs`, `src/sdk/serialization/mod.rs`, `src/migration.rs`, ADR-0028) |
| Implicaciones | Cero cambio de contrato/API/serialización. El research-doc es insumo de SCH-01 (ADR-0046) → SCH-02/03/06. Los consumidores nuevos serán por path (SCH-01 cita `docs/dev/research/mgr-10-bitemporalidad.md`). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o regiones citadas):**
  - `src/sdk/types/record.rs` (completo, 553 L), `src/sdk/version_history.rs` (:1-160, :219-250, :252+), `src/schema.rs` (completo), `src/sdk/api/memory.rs` (:383-432, :520-609), `src/sdk/serialization/mod.rs` (:20-30, :340-357, :446-465, :480-539), `src/cli_handlers/migrate.rs` (:1-140, :165-279), `src/migration.rs` (:1-60, `plan_all` :119, `check_integrity` :359), `src/cli.rs` (:92-136, :395-448), `docs/dev/architecture/adr/ADR-0028-core-decay-supersession.md` (completo), `docs/dev/research/mgr-19-benchmarks-baseline-suites.md` (convención, completo), `docs/dev/Backlog.md` (filas :51,:55,:839-847,:932-946), plan master-roadmap L560-834.
  - Evidencia de gap: `rg "valid_at|valid_from|bitemporal|as_of" src/` → **0 hits (exit 1)**; `rg "point-in-time" src/` → solo MVCC/métricas/snapshots (txn.rs:227, rocksdb_backend.rs:350, metrics/core/mod.rs:622 …).
- **Archivos referenciados hacia dentro (imports/dependencias):** N/A — archivos nuevos de documentación; no importan código.
- **Archivos que referencian a los editados (referencias entrantes):** el plan master-roadmap (§Task 23 / §Task 26 SCH-01) nombra el path `docs/dev/research/mgr-10-bitemporalidad.md`; `docs/dev/tasks/MGR-19.md` fija la convención (research-doc + Cierre MGR). Ningún otro.
- **Veredicto impacto:** **bajo** — creación de 2 archivos de docs; nada existente se modifica salvo este task file. Sin riesgo de romper código, contratos ni datos.

## Contrato

"research-doc cerrado con modelo valid-time vs transaction-time, tradeoffs (append-only vs invalidación + storage del historial) y plan de migración/backfill determinista, listo para SCH-01"

**Verificación (doc-only):**
1. `rg -n "valid_at|valid_from|bitemporal|as_of" src/` → 0 hits (gap) · ✅
2. Doc contiene: §1 modelo 2 ejes · §2 tradeoffs · §4 plan migración/backfill determinista · §3 "0.8.0 vs v1.0" · §5 preguntas owner · §6 plan (Cierre MGR) · §7 fuentes con URL → ✅ (chequeo por headings, abajo)
3. `pwsh scripts/validate-docs-coverage.ps1` → ✅ (0 gaps)
4. GATE CITAS: 7 URLs resueltas/verificadas vía `webfetch`/Jina el 2026-09-28 (§7 del doc) → ✅

## Spec (SDD)

> Task 100% docs: el "spec" son las decisiones técnicas capturadas en el research-doc; se registran acá para el gate y las ratifica el ADR de SCH-01.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Eje faltante | valid-time nuevo vs transaction-time completo | `valid_at_ms`/`invalid_at_ms` (Backlog:939) + transaction por version_history | ✅ por-evidencia (Backlog:839/939; 0 hits en `src/`) |
| 2 | Semántica de período | cerrado-abierto `[start,end)` (SQL:2011) vs cerrado-cerrado | cerrado-abierto + `None`=abierto | ✅ por-evidencia (Kulkarni & Michels §2.1) |
| 3 | Mecanismo de historia | invalidación (ADR-028/Zep) vs append-only/system-versioned | invalidación + snapshots (evita rediseño de engine — stop condition) | ✅ por-evidencia (§2.1 del doc) |
| 4 | Storage historial | full snapshots (actual) / deltas / WAL-exact (P27) | mantener full snapshots; WAL-exact v1.0 | ✅ por-evidencia (§2.2) |
| 5 | Backfill | `valid_at:=created_at`, `invalid_at:=superseded_at` (pura, idempotente, sin reloj) | el del Backlog:939 | ✅ por-evidencia (§4.1) |
| 6 | Default de queries | filtrar validez por defecto (SQL:2011) vs opt-in | mantener comportamiento actual (opt-in) | ⬜ pregunta owner §5-1 (SCH-01 ratifica) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  - Nada de `src/` se toca en esta tarea; ningún campo/knowledge de serialización cambia.
  - El research-doc NO autoriza implementación: el ADR formal (campos finales/nombres) es SCH-01 — "no tocar `record.rs` antes" (Backlog:938).
  - Determinismo del backfill = función pura del estado v1 (`created_at`/`superseded_*`), sin reloj — cualquier desvío rompe el gate SCH-06.
- **Comandos de verificación:** `rg -n "valid_at|valid_from|bitemporal|as_of" src/` (0 hits) · `pwsh scripts/validate-docs-coverage.ps1` (0 gaps).
- **Deuda pendiente:** ninguna de esta tarea; las 6 preguntas owner (§5) y el diseño de edges (SCH-09) pasan a SCH-01.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado + Contrato |
| `lastAction` | Step 4 ✅ (task file + verificación) |
| `result` | `PARTIAL` — steps ✅, pendiente review/commit LEAD (NO self-review) |
| `nextAction` | LEAD: review P2-01 (tier fast, `docs/dev/**`) + commit local `docs: MGR-10 — research-doc bitemporalidad` |
| `contract` | `## Contrato` + `## Invariantes de dominio` + §7 fuentes del doc (evidencia por claim) |
| `nextTask` | SCH-01 (Task 26) — pre-requisito duro: espera también MGR-12/13 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (docs-only; saldo 0).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato verificable (arriba) + doc creado + GATE CITAS | ✅ |
| **Commit** | Commit atómico por LEAD (instrucción del orquestador: worker NO commitea) | ⏳ LEAD |
| **Release** | N/A — sin cambio de código; `validate-docs-coverage` aplicable ✅ | N/A/✅ |

## Herramientas necesarias

- Read/grep/codegraph (blast radius de solo lectura) · `pwsh scripts/validate-docs-coverage.ps1` · webfetch (GATE CITAS) · campaign MCP (SDP/estado).

**Skills cargadas (SDP — `campaign_discover_skills_v2`, Documentation/BUILD):** `deprecation-and-migration` (pinned storage/schema — expand→backfill→bump), `writing-plans` (§6), `writing-guidelines`, `source-driven-development` (citas §7), `context-engineering` (handoff SCH-01), `coordinated-web-search` (cascada router); base auto: `campaign-executor`/`progreso`. N/A justificadas: `incremental-implementation`/`test-driven-development` (doc-only, cero código).

## Investigation Notes

- **Convención:** `docs/dev/research/mgr-19-benchmarks-baseline-suites.md` — título `MGR-XX — … (Cierre MGR)`, metadata (fecha/tipo/contrato/plan), § numerados, §Preguntas owner + §Plan de implementación = Cierre MGR (Backlog:934), notas con skills SDP.
- **Hallazgos de código clave:** (1) snapshot por versión best-effort post-commit con cap 32/key (`version_history.rs:10-14`, `config.rs:319`); (2) `SnapshotRecord` postcard oculto omite `superseded_*` y el `From` los resetea → historial no reconstruye invalidación (:144-145); (3) `vanta migrate` existe (plan/check/run, `cli.rs:406-431`) pero schema solo reescribe header (`migrate.rs:247-275`) — el backfill de datos es nuevo; (4) `record_from_export_line` rechaza `schema_version != 1` (:522-531) → compat v1 exigida por SCH-02; (5) postcard `from_bytes` ignora trailing bytes (docs.rs) → decode snapshots V2-first con fallback V1 determinista.
- **Fuentes externas (7, verificadas):** Snodgrass 1999 (`cs.arizona.edu`), Kulkarni & Michels SIGMOD 2012 (SQL:2011), Fowler "Bitemporal History", Zep arXiv 2501.13956, Graphiti README, XTDB docs, postcard docs. Detalle y URLs en §7 del research-doc.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (propios) |
| % completado | 100% (de la tarea; review/commit = gate externo LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A justificado: doc-only; no toca trust boundaries ni input de usuario; no agrega dependencias.
- [x] **PERFORMANCE** — N/A justificado: doc-only; cero hot path. El diseño propone filtros en assembly (patrón ADR-0028 "no index change") y difiere índices temporales a v1.0 explícitamente.

## Steps

### Step 1: Discovery + re-verificación de gap
- **Archivos:** `src/sdk/types/record.rs`, `src/sdk/version_history.rs`, `src/schema.rs`, `src/sdk/serialization/mod.rs`, `src/migration.rs`, `src/cli.rs`, ADR-0028, Backlog/plan
- **Acción:** confirmar gap (0 hits de dominio) + mapear los 4 formatos de persistencia y la maquinaria `vanta migrate`
- **Verify:** `rg -n "valid_at|valid_from|bitemporal|as_of" src/` → exit 1 (0 hits) ✅ ; `rg -n "point-in-time" src/` → solo storage/métricas ✅
- **Estado:** ✅ DONE

### Step 2: Web research + GATE CITAS (fuentes §7)
- **Archivos:** fuente externa (webs) — sin archivos locales
- **Acción:** verificar Snodgrass (página oficial + PDF), SQL:2011 (SIGMOD Record vía Jina), Fowler, Zep arXiv, Graphiti, XTDB, postcard docs
- **Verify:** cada URL resuelta vía `webfetch`/Jina el 2026-09-28 ✅ (7/7; el fetch directo de SIGMOD queda tras Cloudflare → fallback Jina OK; `tdbbook.pdf` responde `application/pdf`)
- **Estado:** ✅ DONE

### Step 3: Redactar research-doc (deliverable principal)
- **Archivos:** `docs/dev/research/mgr-10-bitemporalidad.md` (nuevo, ~230 L)
- **Acción:** §0 gap/estado del arte · §1 modelo 2 ejes + propuesta 0.8.0 · §2 tradeoffs · §3 "0.8.0 vs v1.0" · §4 plan de migración/backfill determinista con comandos · §5 preguntas owner · §6 plan de implementación · §7 fuentes
- **Verify:** headings presentes (`rg -n "^## §|^## " docs/dev/research/mgr-10-bitemporalidad.md`) ✅
- **Estado:** ✅ DONE

### Step 4: Task file + verificación de docs
- **Archivos:** `docs/dev/tasks/MGR-10.md` (nuevo)
- **Acción:** DISCOVERY completo (Impacto Regla 0 + Spec + invariantes + steps) y correr verificación doc-only
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` ✅ (ver RESULTADO del orquestador: 0 gaps) ; revisión de contrato por headings ✅
- **Estado:** ✅ DONE

## Dependencias

- Ninguna de entrada (Backlog:839 citaba MGR-01; fuera del plan — research lane post-1.0).
- Co-batch F3: MGR-12 / MGR-13 (otros research-docs; NO tocar sus archivos `mgr-12-*`/`mgr-13-*`).
- Pre-requisito duro *hacia adelante*: SCH-01 (Task 26) espera Cierre MGR de los 3 (Backlog:938) antes de tocar `record.rs`.

## Review (GATE — agente distinto, P2-01)

> Ejecutado por un agente DISTINTO al implementador. **NO auto-review** — lo ejecuta el LEAD/orquestador (instrucción explícita de la sesión). Tier de paths (`docs/dev/**` = **fast**): verify mecánico + veredicto registrado.

- **Revisor:** `vanta-review` fresco (sesión `ses_f16a02b60ffe8dD5zrGAhVSRsC`; ≠ autor `ses_f16afedccffeTnl5Knjwv7KH4t`) — **✅ APPROVE** (2026-09-28).
- **Enfoque:** ¿el modelo valid/transaction y el plan de migración son correctos y suficientes para SCH-01? ¿las citas son reales (GATE CITAS §7)? ¿coló scope v1.0 (sección §3 lo blinda)?
- **Cómo se probó:** comandos de §Contrato + §7 del doc (evidencia por URL), no auto-reporte.
- **Veredicto:** ✅ **APPROVE** — 0 Required; 2 Optional (default `valid_at_ms` en insert → SCH-01; boundaries de deserialización → SCH-02) + 3 Nits aplicados (MGR-19 path, `wal.rs:23`, `invalid_at` incondicional). ~20 refs re-verificadas línea a línea; 6/7 fuentes con citas textuales.

## Notas

- DoD de bloque: "the ADR formal se consolida en SCH-01" — este doc NO crea ADR (evita duplicar slot 046).
- Pre-mortem del bloque atendido: F1 → §3 "0.8.0 vs v1.0" + §5; F2 → §0 (mapa de formatos) + §4 (comandos exactos `vanta migrate plan|check|run`, export/import); F3 → metadata cita Backlog:839/Notion dim 5 + §7 refs externas (Snodgrass, Zep/Graphiti).
- Stop conditions respetadas: sin rediseño de engine (invalidación vs append-only, §2.1), sin MVCC/índices temporales (§2.3), edge-bitemporal deferido a SCH-09/SCH-01 (§4.5).
- Instrucción de sesión: **NO commit** (LEAD commitea) y **NO self-review**.
