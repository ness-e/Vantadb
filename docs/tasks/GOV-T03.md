# GOV-T03: TIR-08c — criterios research-agent.md (saturación<20% + broadening/narrowing + WONTFIT-jitter)

## Metadata
- **Plan file:** docs/plans/2026-09-02-alta-prioridad-paralelo.md
- **Fuente:** docs/reviews/archive/auditoria-documentacion-2026-08-21.md Brechas Volumen II (TIR-08c ~6 líneas) + D13 (ejecución directa) + plan 2026-09-02 Wave0
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡 Media
- **Tipo:** Docs (prompts/research-agent.md)
- **Turns estimados:** 2
- **Creado:** 2026-09-02T19:30
- **last-synced:** 2026-09-02T19:35
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0
- **Campaign ID:** 20260902-alta-prioridad-paralelo

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno — prompt leído por humanos/agentes research, no hay callers de código |
| Callees | ninguno — markdown standalone, sin imports |
| Implicaciones | añade ~6 líneas en `.opencode/task-system/prompts/research-agent.md` sección Criterios (TIR-08c); no cambia código Rust/Python/TS; Wave0 paralelo disjoint con GOV-T01 (evals/dora.mjs), GOV-T02 (RULES.md/SKILL.md), MCP-35 (vantadb-mcp/src/server.rs), RES-01 (src/wal.rs) — 0 archivos solapados |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):** `.opencode/task-system/prompts/research-agent.md` (33 líneas, 2026-09-02, criterios TIR-08 existentes 3 bullets), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (§GOV-T03 contrato), `docs/reviews/archive/auditoria-documentacion-2026-08-21.md` (465L, TIR-08c §Brechas Volumen II:350 + D13:442), `.opencode/skills/campaign-executor/tasks/GOV-T01.md` y `GOV-T02.md` (templates referencia Wave0)
- **Archivos referenciados hacia dentro:** research-agent.md no tiene imports; es prompt standalone para research agents. Referenciado por `docs/references/research-modules.md` y prompts/research-module*.md indirectamente.
- **Archivos que referencian a los editados:** `grep -r "research-agent"` → solo `.opencode/task-system/prompts/research-agent.md` mismo + plan file mención GOV-T03 (docs/plans/2026-09-02-alta-prioridad-paralelo.md:120) + auditoria doc; sin callers en src/
- **Veredicto impacto:** bajo — docs-only, 1 archivo markdown, ~6 líneas aditivas, sin código, sin tests, sin build. Riesgo: contrato Select-String exige >=3 hits (saturación|broadening|WONTFIT) — ya pasa con 3 hits actuales; ampliación a 6 líneas mantiene/eleva hits a >=4. Disjoint 100% Wave0.

## Contrato

**Contrato plan 2026-09-02 (verificable):** `Select-String -Path ".opencode/task-system/prompts/research-agent.md" -Pattern "saturaci.*20%|broadening|WONTFIT" | Measure-Object Count` >=3

> Nota: contrato exige >=3 coincidencias entre saturación<20%, broadening, WONTFIT. Verificación secundaria: conteo de líneas criterios >=6 (TIR-08c ~6 líneas).

## Spec (SDD)

N/A — docs-only, sin símbolos públicos nuevos. Decisión ya tomada en TIR-08 (2026-08-17): saturación<20% + broadening/narrowing + WONTFIT-jitter como criterios runtime research; decision ya documentada en auditoria D13 excepción "ejecutar ya". No hay alternativas a evaluar; solo expandir 3 bullets existentes a 6 líneas explícitas para trazabilidad.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** research-agent.md mantiene formato Digest obligatorio (Hallazgos/Estructura/Riesgos/Referencias), reglas NO generar código + NO modificar archivos + preferir codegraph_explore; sección Criterios solo se expande, no se reescribe el digest; sin tocar código Rust/Python/TS; Wave0 disjoint preservado.
- **Comandos de verificación:** `Select-String -Path ".opencode/task-system/prompts/research-agent.md" -Pattern "saturaci.*20%|broadening|WONTFIT" | Measure-Object Count` >=3
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-T03 — TIR-08c criterios research-agent.md |
| `lastAction` | Step1+2 ejecutados: research-agent.md 3→6 líneas criterios (saturación<20%+broadening/narrowing+WONTFIT-jitter), verify 3≥3 ✅ |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | ninguno — tarea cerrada; Wave0 disjoint sigue con MCP-35 + RES-01 (si pendientes) |
| `contract` | `## Contrato` + evidencia: research-agent.md:30-36 + plan GOV-T03 + auditoria TIR-08c:350 |
| `nextTask` | MCP-35 (fallback HTTP) o RES-01 (WAL v2) — Wave0 paralelo disjoint (no bloquea) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda nueva — docs-only, 0 líneas de código, expansión de criterios. Saldo neto 0.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable >=3 hits + verify mecánico | ✅ |
| Commit | Lo ejecuta el LEAD (worker no commitea en vanta-docs) | delegado |
| Release | No aplica (docs-only, sin crate/npm) | justificado |

## Herramientas necesarias

- PowerShell Select-String / Measure-Object (contrato)
- Read/Edit (research-agent.md)
- Glob/Grep (verificación auditoria)

**Skills cargadas (SDP):** campaign-executor (orquestación) + documentation-and-adrs (research-agent.md/ADRs) + ponytail(full) — diff mínimo docs-only, 0 código, Blast Radius 0.

## Investigation Notes

- **research-agent.md actual (2026-09-02 DISCOVERY):** 33 líneas, formato Digest + 3 reglas No-generar-código + sección Criterios TIR-08 con 3 bullets: saturación<20% (línea 31), narrowing/broadening (línea 32), WONTFIT jitter (línea 33) → contrato Count=3 ✅ ya pasa, pero TIR-08c especifica ~6 líneas (auditoria:350) → requiere expandir a 6 bullets explícitos (saturación, broadening, narrowing separados + documentar decisión + referencia TIR-08).
- **Auditoria 2026-08-21:** TIR-08c listada en Brechas Volumen II 🟡 Media — "criterios en research-agent.md (~6 líneas) | ídem" (decisión tomada nunca ticketeada, idem TIR-02a/04b). D13 excepción a D2: micro-fixes TIR directo (~1h total Wave0 T0×3).
- **Plan 2026-09-02:** GOV-T03 Wave0 paralelo disjoint con GOV-T01/T02 + MCP-35 + RES-01; contrato exacto `saturaci.*20%|broadening|WONTFIT >=3`; task file ruta `.opencode/skills/campaign-executor/tasks/GOV-T03.md` (no existe, se crea now).
- **Disjoint Wave0:** GOV-T03 toca research-agent.md; GOV-T01 toca evals/dora.mjs; GOV-T02 toca RULES.md/SKILL.md; MCP-35 toca vantadb-mcp/src/server.rs; RES-01 toca src/wal.rs — 0 solapamiento → parallel 3 seguro.
- **Ponytail:** diff mínimo — solo 1 archivo markdown, 3→6 líneas netas (+3 líneas: split broadening/narrowing en 2 bullets + añadir 2 bullets documentación/ver TIR-08), 0 código.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: docs-only, sin trust boundaries, sin input usuario, sin auth, sin deps.
- [x] **PERFORMANCE** — no aplica: markdown estático, sin hot path, sin bench.

## Steps

### Step 1: Editar research-agent.md — expandir criterios TIR-08c a ~6 líneas

- **Archivos:** `.opencode/task-system/prompts/research-agent.md`
- **Acción:** Expandir sección `Criterios de investigación (TIR-08)` (líneas 30-33) de 3 bullets a 6 bullets: (1) Saturación <20% explícita, (2) Broadening, (3) Narrowing (split actual 1 bullet en 2), (4) WONTFIT-jitter, (5) Documentar decisión stop/broaden/narrow en Referencias, (6) Ver TIR-08 rationale. Mantener Digest + Reglas intactos; cambiar header a `TIR-08c`. Ponytail: diff mínimo 3 líneas netas.
- **Verify:** `Select-String -Path ".opencode/task-system/prompts/research-agent.md" -Pattern "saturaci.*20%|broadening|WONTFIT" | Measure-Object Count` >=3 → 3 ✅ (Saturación<20% + Broadening + WONTFIT)
- **Estado:** ✅ COMPLETED (2026-09-02T19:35, 33L→36L, +3 netas)

### Step 2: Verificar contrato + cierre plan

- **Archivos:** `.opencode/task-system/prompts/research-agent.md`, `docs/plans/2026-09-02-alta-prioridad-paralelo.md`, `.opencode/skills/campaign-executor/tasks/GOV-T03.md`
- **Acción:** Ejecutar contrato principal >=3 + contar líneas criterios >=6 (Select-String criterios). Actualizar plan file fila GOV-T03 PENDING/COMPLETED→COMPLETED con last-synced 2026-09-02T19:35, escribir recitation canónica, y actualizar este task file Steps → ✅ COMPLETED.
- **Verify:** contrato >=3 ✅ (3 hits) AND `Select-String -Path ".opencode/task-system/prompts/research-agent.md" -Pattern "Saturaci.*20%|Broadening|Narrowing|WONTFIT" | Measure-Object Count` >=4 → 4 ✅ + 6 bullets >=6 ✅
- **Estado:** ✅ COMPLETED (2026-09-02T19:35, plan last-synced actualizado)

## Dependencias

- Ninguna (Wave0 paralelo disjoint con GOV-T01, GOV-T02, MCP-35, RES-01). Predecesores Wave0 ya COMPLETED no bloquean (archivos disjuntos) → MAX 3 seguro. No tocar MCP-35 ni RES-01.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-docs (self-review ponytail docs-only, sin código) — Verificar que research-agent.md mantiene formato Digest + criterios 6 líneas con saturación<20%+broadening/narrowing+WONTFIT-jitter; contrato >=3; diff mínimo.
- **Enfoque:** ¿formalización preserva semántica TIR-08c sin inventar criterios extra? ¿diff mínimo ponytail?
- **Cómo se probó:** Select-String counts mecánicos (3≥3 + 4≥4) + inspección research-agent.md diff (+3 líneas, 0 código)
- **Veredicto:** ✅ approve (2026-09-02T19:35)

## Notas

- Ponytail diff mínimo: solo 1 archivo markdown, +3 líneas netas, 0 código. No tocar MCP-35 (proxy) ni RES-01 (wal.rs).
- Worker no commitea: LEAD hace commit. Este task file + research-agent.md + plan file son los únicos tocados.
- Contrato legacy plan path `.opencode/agents/research-agent.md` buscado pero canónico es `.opencode/task-system/prompts/research-agent.md` (ver glob) — ambos patrones cubiertos.

## Referencias

- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave0 GOV-T03 contrato y disjoint
- `docs/reviews/archive/auditoria-documentacion-2026-08-21.md:350,442` — TIR-08c + D13
- `.opencode/task-system/prompts/research-agent.md` — prompt actual (33L, TIR-08)
- `.opencode/task-system/prompts/task.md` — formato task file
- `.opencode/skills/campaign-executor/SKILL.md` — orquestación

## Context Save Point

- **Fecha:** 2026-09-02T19:35
- **Branch:** develop
- **CI pendiente:** no (docs-only)
- **Decisiones:** expandir de 3 a 6 líneas (split broadening/narrowing + añadir documentar decisión/ver TIR-08) para cumplir TIR-08c ~6 líneas sin romper contrato existente; header TIR-08→TIR-08c para trazabilidad auditoria.
- **Problemas conocidos:** ninguno — contrato 3≥3 ✅ + 6 bullets ✅ + disjoint Wave0 verificado
- **Próxima tarea:** MCP-35 o RES-01 — Wave0 paralelo disjoint (no bloquea)

