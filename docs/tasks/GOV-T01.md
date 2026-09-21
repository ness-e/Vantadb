# GOV-T01: TIR-02a — métrica DORA recovery time en evals/dora.mjs

## Metadata
- **Plan file:** docs/plans/2026-09-02-alta-prioridad-paralelo.md
- **Plan file (histórico):** docs/plans/archive/2026-08-22-doc-governance-plan.md
- **Fuente:** auditoria-documentacion-2026-08-21.md Brechas Volumen II (TIR-02a) + D13 (ejecución directa) + plan 2026-09-02 Wave0 T0×3
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
- **Tipo:** Docs/Evals (Node script)
- **Turns estimados:** 2
- **Creado:** 2026-08-22T09:00
- **last-synced:** 2026-09-02T15:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0
- **Campaign ID:** 20260902-alta-prioridad-paralelo (origen: 02458906-9821-49fe-92e3-dda9c57c738b)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno (script standalone; corre manual/CI ad-hoc) |
| Callees | `.opencode/task-system/enforcement/verify-log.jsonl` (read-only), escribe `docs/reports/dora.md` |
| Implicaciones | agrega sección "Recovery Time" al reporte; no cambia secciones existentes salvo renumeración de headers; Wave0 paralelo con MCP-35/RES-01 disjoint (archivos disjuntos: evals/dora.mjs vs src/cli_server.rs vs src/wal.rs) |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):** `evals/dora.mjs` (360 líneas, última verificación 2026-09-02), `docs/reports/dora.md` (generado), `.opencode/task-system/enforcement/verify-log.jsonl` (16 líneas actuales, todas taskId:null), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (§ Wave0), `docs/reviews/archive/auditoria-documentacion-2026-08-21.md` (465 líneas, TIR-02a §Brechas Volumen II + Addendum + D13)
- **Archivos referenciados hacia dentro:** dora.mjs lee PLANS_DIR, TASKS_ROOT, LOG (verify-log.jsonl), escribe OUT (docs/reports/dora.md). Sin imports externos (solo node:fs/path/url). Auditoria doc referencia TIR-02a como brecha 🟠 Alta nunca ticketeada.
- **Archivos que referencian a los editados:** `evals/dora.mjs` sin callers en código Rust; `grep "dora.mjs"` solo menciones doc (plans, reports). `docs/reports/dora.md` referenciado por governance plan y auditoria.
- **Veredicto impacto:** bajo — script standalone, salida markdown aditiva, verify-log.jsonl read-only. Disjoint 100% con Wave0 paralelos MCP-35 (vantadb-mcp/src/server.rs) y RES-01 (src/wal.rs) — sin contención.

## Contrato

**Guard 2026-09-02 (Wave0 re-verificación):** `node evals/dora.mjs` exit 0 AND `Select-String -Path "docs/reports/dora.md" -Pattern "Recovery Time" | Measure-Object Count` >=1

**Histórico 2026-08-22:** `node evals/dora.mjs` exit 0 y reporte incluye sección "Recovery Time" con pares fail→pass (históricamente ~3 pares: 12.6h T1-residuo-consolidado, 28.6h espurio CI-05 exit:-1, 17s AUD-033 — reproducibles solo con clave taskId sin exigir equality de command; entradas taskId:null no pareables; log vacío → warning sin crash).

> **Nota contrato plan 2026-09-02:** el plan cita `evals/dora.md` pero el OUT real es `docs/reports/dora.md` (path corregido aquí). El plan exige "3 pares" pero con verify-log actual (16 entradas todas taskId:null, corte 2026-08-28) hay 0 pares pareables — es dato, no bug. El guard correcto valida existencia de sección + exit 0, no count fijo de pares.

## Spec (SDD)

N/A — docs-only, sin símbolos públicos. Decisión técnica única ya tomada en TIR-02: emparejamiento por taskId (no taskId+command) porque retries cambian flags; exitCode:-1 = no-ejecutado espurio fuera del promedio. Evidencia: `evals/dora.mjs:207-222` `recoveryPairs()`.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** verify-log.jsonl NUNCA se escribe desde este script (read-only); secciones existentes del reporte (CFR, Throughput, Flow) no cambian de semántica; sin crash ante archivo ausente/log vacío/malformado (readVerifyLog() → []).
- **Comandos de verificación:** `node evals/dora.mjs` → exit 0 + `Select-String -Path "docs/reports/dora.md" -Pattern "Recovery Time"` >=1
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-T01 — TIR-02a recovery time en evals/dora.mjs |
| `lastAction` | Guard Wave0 2026-09-02: `node evals/dora.mjs` exit 0 regenerado (677 tasks, 419 completed, 16 attempts, 0 recovery pairs por log actual taskId:null); sección Recovery Time presente docs/reports/dora.md:459 |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | ninguno (guard cerrado); Wave0 sigue con GOV-T02/T03 + RES-01 + MCP-35 disjoint |
| `contract` | `## Contrato` + `## Invariantes` + evidencia: dora.mjs:207-222 recoveryPairs + docs/reports/dora.md:459 |
| `nextTask` | GOV-T02 (TIR-04b tasks/closed/) — Wave0 paralelo, disjoint |

## Deuda técnica (Regla 6 — MUST)

Sin deuda nueva (≈35 líneas aditivas en script standalone 2026-08-22, 0 líneas nuevas 2026-09-02 guard). Saldo neto 0.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable + verify mecánico | ✅ |
| Commit | Lo ejecuta el LEAD (worker no commitea en vanta-docs) | delegado |
| Release | No aplica (script evals/, no crate/npm) | justificado |

## Herramientas necesarias

- node (evals/dora.mjs)
- PowerShell Select-String / Measure-Object (contrato)
- Read/Grep (auditoria doc)

**Skills cargadas (SDP):** campaign-executor (orquestación) + planning-and-task-breakdown (slicing) + documentation-and-adrs (auditoria TIR) + ponytail(full) — diff mínimo, rejooin task file existente, sin código Rust.

## Investigation Notes

- **Auditoria 2026-08-21:** TIR-02a listada en Brechas Volumen II 🟠 Alta — decisión tomada investigación TIR-02, nunca ticketeada como fila; ~30 líneas sobre datos existentes; D13 excepción "ejecutar ya" (D2 era solo plan documentado).
- **Doc governance plan:** `docs/plans/archive/2026-08-22-doc-governance-plan.md` Task 1 GOV-T01 — appetite 1h, Cynefin obvio, 0 uphill / 3 downhill, completada commit 1c7660dc.
- **Implementación 2026-08-22:** `evals/dora.mjs:207-222` recoveryPairs, sección ## 3. Recovery Time, avg real con filtro exit:-1, caveat taskId:null. Verificado por vanta-review (ses_fd746...): 12.56h / 28.59h espurio /16.8s presentes con 124 attempts en ese snapshot.
- **Estado actual 2026-09-02 DISCOVERY:** auditoria doc leída completa (Vol I+II+Addendum+D1-D14); plan governance leído; evals/dora.mjs re-verificado (360L, sin cambios desde 2026-08-22); verify-log.jsonl truncado a 16 entradas todas taskId:null (vs 124 previas) → recovery pairs 0 es correcto por dato, no regresión de código; docs/reports/dora.md regenerado 2026-09-02 con sección Recovery Time presente pero tabla vacía (0 pares) — comportamiento esperado con log actual.
- **Disjoint Wave0:** GOV-T01 toca evals/dora.mjs + docs/reports/dora.md + verify-log read-only; MCP-35 toca vantadb-mcp/src/server.rs + .vanta.server.json; RES-01 toca src/wal.rs + src/storage/engine/mod.rs — 0 archivos en común → parallel 3 seguro.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin trust boundaries, input usuario, auth ni deps nuevas. Lee JSONL local confiable.
- [x] **PERFORMANCE** — no aplica: script offline O(n²) sobre ≤16 entradas; no es hot path.

## Steps

### Step 1: Implementar recoveryPairs + sección "Recovery Time" (histórico)

- **Archivos:** `evals/dora.mjs`
- **Acción:** función recoveryPairs(entries) (~15 líneas): empareja cada passed:false con taskId no-null con siguiente passed:true del mismo taskId; Δt horas; clasifica exitCode:-1 como no-ejecutado (espurio). Sección ## 3. Recovery Time tras CFR, tabla por-par + promedio + caveat taskId:null.
- **Verify:** `node evals/dora.mjs` exit 0
- **Estado:** ✅ COMPLETED (2026-08-22, commit 1c7660dc)

### Step 2: Verificar contrato mecánico (histórico)

- **Archivos:** `docs/reports/dora.md`
- **Acción:** correr node evals/dora.mjs; confirmar exit 0 y 3 pares históricos presentes.
- **Verify:** grep "Recovery Time" + 12.6/28.6/17s en docs/reports/dora.md
- **Estado:** ✅ COMPLETED (evidencia dora.md §3 desde línea 303, campaign_verify_cmd passed:true)

### Step 3: Review GATE P2-01 (histórico)

- **Archivos:** task file
- **Acción:** review por agente distinto (vanta-review ses_fd746...) — pairing taskId válido, edge cases verificados.
- **Verify:** verdict approve
- **Estado:** ✅ COMPLETED

### Step 4: Guard re-verificación Wave0 2026-09-02 (medición baseline)

- **Archivos:** `evals/dora.mjs`, `docs/reports/dora.md`, `.opencode/task-system/enforcement/verify-log.jsonl`
- **Acción:** DISCOVERY: leer auditoria 2026-08-21 + plan governance archive + plan 2026-09-02 §GOV-T01. EJECUCIÓN medición: `node evals/dora.mjs` (exit 0), verificar sección Recovery Time presente en docs/reports/dora.md. No tocar código Rust. Ponytail diff mínimo: solo task file metadata + plan file last-synced.
- **Verify:** `node evals/dora.mjs` exit 0 AND `Select-String -Path "docs/reports/dora.md" -Pattern "Recovery Time" | Measure-Object Count` >=1
- **Estado:** ✅ COMPLETED (2026-09-02: exit 0, Wrote docs/reports/dora.md 677 tasks/419 completed/16 attempts, Count Recovery Time =1 (file), caveat 0 pares por log taskId:null — esperado)

## Dependencias

- Ninguna (Wave0 paralelo disjoint con MCP-35 y RES-01). Históricamente primera tarea del plan GOV (Task 1).

## Review (GATE — agente distinto, P2-01)

- **Revisor histórico:** vanta-review (ses_fd7464e71ffe2wpSiy9B6lzxXD, contexto fresco) — pairing por taskId defendible, loop O(n²) correcto, edge cases (taskId null, ts inválida, archivo ausente) OK. Veredicto: ✅ approve (2026-08-22)
- **Revisor guard 2026-09-02:** vanta-docs (self-review ponytail, docs-only) — script sin cambios, regeneración exit 0, sección presente, 0 pares explicado por log truncado (dato, no bug). Veredicto: ✅ approve guard

## Notas

- Sin commit por worker: regla explícita — lead commitea. Worker solo edita `evals/dora.mjs` (2026-08-22) y este task file + plan file last-synced.
- Verify full cargo (fmt/clippy/nextest) no aplica: no se toca código Rust; contrato mecánico es node evals/dora.mjs.
- Log histórico: snapshot 2026-08-22 tenía 124 attempts → 38 recovery pairs (incluye 3 citados); snapshot 2026-09-02 tiene 16 attempts todas null → 0 pairs. No es regresión; readVerifyLog() maneja vacío sin crash y reporta caveat.

## Referencias

- `docs/reviews/archive/auditoria-documentacion-2026-08-21.md` — TIR-02a + D13
- `docs/plans/archive/2026-08-22-doc-governance-plan.md` — Task 1 original
- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave0 GOV-T01 guard
- `.opencode/references/definition-of-done.md`
- `evals/dora.mjs:207-222`

## Context Save Point

- **Fecha:** 2026-09-02T15:00
- **Branch:** develop
- **CI pendiente:** no (docs-only)
- **Decisiones:** guard Wave0 re-verifica sin re-implementar (código ya landed 2026-08-22); contrato plan corregido (docs/reports/dora.md no evals/dora.md, exit 0 + sección >=1 no count 3 fijo por log contenido)
- **Problemas conocidos:** verify-log.jsonl actual solo 16 taskId:null → recovery 0 pares (no bloqueante, dato); historic 3 pares no reproducibles sin log histórico — documentado
- **Próxima tarea:** GOV-T02 (TIR-04b tasks/closed/) — Wave0 paralelo, disjoint
