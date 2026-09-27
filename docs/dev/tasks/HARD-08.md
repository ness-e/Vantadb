# HARD-08: SDP v3 — mejoras de búsqueda/lectura/revisión/selección automática de skills (PRE-RUN)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (F0 — pre-run)
- **Fuente:** decisión owner 2026-09-27 (question: "todas las mejoras") tras auditoría del SDP v2 (spec `skills-engineering.md` vs `campaign-server.mjs` vs 14 SDP reales)
- **Esfuerzo:** 🟡 1d · **Prioridad:** 🔴 · **Tipo:** Mixto (harness `.opencode/`)
- **Creado:** 2026-09-27 · **last-synced:** 2026-09-27
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 · **Pendientes (downhill):** 0

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | tool `campaign_discover_skills_v2` ← prompts (`pipeline-full.md` Paso 0b, `task.md`), plan maestro, task files HARD-*/DEF-*, sub-agentes del run |
| Callees | `.opencode/skills/*` (196) + `SKILLS-MANIFEST.md` (ratings/deprecated) vía `skills-index.json`; `memory/skill-outcomes.json` |
| Implicaciones | Contrato v2 preservado (campos + nombre del tool); output aditivo (`sdpVersion`/`pinned`/…); v1 (`campaign_discover_skills`) sigue operativo (fix H-01) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `campaign-server.mjs` (bloques L1146-1596), `skills-engineering.md`, 14 task files (líneas SDP), `SKILLS-MANIFEST.md` (tablas)
- **Referencias entrantes:** prompts del task-system, plan maestro, task files (nombre del tool intacto → 0 roturas)
- **Veredicto impacto:** medio (harness core) — mitigado con selftest + smoke E2E + Gate H 3 rondas

## Contrato
"`bun .opencode/task-system/scripts/sdp-selftest.mjs` → 15/15 Y `bun .opencode/task-system/scripts/smoke-mcp-sdp.mjs` → OK (sdpVersion=v3 + v1 operativo) Y índice 195 skills Y Gate H ✅ (vanta-harness)"

## Spec (decisiones — resueltas por evidencia + decisión owner)
| # | Decisión | Opciones | Resuelto |
|---|----------|----------|----------|
| 1 | Alias ES↔EN + morfología (B1) | tabla ~70 aliases + `singularize` | ✅ owner "todas las mejoras" |
| 2 | Drift spec↔impl (B2) | familias exact-token / deprecated excluidas / minScore 0.6 / taskType 0.2 / base fija | ✅ spec canónica |
| 3 | Índice enriquecido (B3) | JSON generado de front-matter + manifest | ✅ |
| 4 | Re-rank por descripción (L1) | overlap tokens (+0.05, cap 0.15) | ✅ |
| 5 | Policy pins (R1) | 7 reglas por path, no negociables | ✅ |
| 6 | Cuota de calidad (S1) | reserva pre-fill + fallback índice (R-01) | ✅ Gate H |
| 7 | Feedback loop (S2) | `skill-outcomes.json` + boosts (+0.1/+0.05/−0.1) | ✅ |
| 8 | v1 tool (H-01) | fix firma `(PROJECT_ROOT, kw)` (no retirar aún) | ✅ Gate H |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** nombre del tool `campaign_discover_skills_v2` y campos v2 del output (consumidores); `sdp-v3.mjs` solo lee (única escritura runtime = `record-skill-outcome.mjs` → `memory/skill-outcomes.json`); pins+base obligatorios.
- **Comandos de verificación:** `node --check` server/sdp-v3 → 0 · `bun scripts/sdp-selftest.mjs` → 15/15 · `bun scripts/smoke-mcp-sdp.mjs` → OK · rebuild índice → 195.
- **Deuda pendiente:** ninguna nueva (dup `design-taste-frontend` documentado con warn; ratings `null` para 4 skills sin fila = correcto).

## Deuda técnica (Regla 6)
Sin deuda nueva — saldo ≤0 (se cierran drift spec↔impl y gaps de selección).

## Definition of Done (multi-nivel)
| Nivel | Gate | Estado |
|-------|------|--------|
| Task | contrato ✅ + task file + recitation | ✅ |
| Commit | conventional + verify mecánico (selftest/smoke/check) | ✅ `903a029` (.opencode) |
| Release | N/A (harness interno) | — |

## Herramientas necesarias
- `bun` · `node --check` · vanta-harness (Gate H) · smoke propio (spawn del MCP)
- **Skills (SDP):** `campaign-executor` · `ci-cd-and-automation` · `test-driven-development` · `doubt-driven-development` · `documentation-and-adrs`

## Investigation Notes
- Gaps auditados: idioma ES↔EN, morfología, drift spec↔impl (familias/deprecated/minScore/taskType), selección sin lectura, sin guardrails, sin feedback, sin revisión pre-run.
- Gate H: 3 rondas (❌ H-01..H-06 → ❌ R-01/R-02 → ✅) — todas cerradas con evidencia mecánica re-ejecutada por el revisor.

## Incógnitas vs Pendientes
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases SECURITY | PERFORMANCE
- [x] SECURITY — no aplica: sin trust boundaries nuevos; solo lecturas locales + 1 escritura acotada a `memory/skill-outcomes.json`.
- [x] PERFORMANCE — no aplica: índice precomputado; scoring O(candidatas×keywords).

## Steps (todos ✅)
### Step 1: Auditoría SDP v2 vs spec — ✅ DONE (7 gaps documentados)
### Step 2: Módulo `sdp-v3.mjs` (B1/B2/B3/L1/R1/S1/S2) — ✅ DONE (selftest)
### Step 3: Migración `campaign-server.mjs` (−285/+9) — ✅ DONE (node --check + smoke)
### Step 4: Índice (`build-skills-index.mjs`) — ✅ DONE (195 skills / 279 ratings)
### Step 5: Selftest + smoke E2E — ✅ DONE (15/15 + OK v2+v1)
### Step 6: Spec + prompts (R3/S2) — ✅ DONE
### Step 7: Gate H (3 rondas) — ✅ APPROVE (H-01..H-06 + R-01/R-02)
### Step 8: Commits `.opencode` — ✅ DONE (`903a029` + `e97c093`)

## Dependencias
- Pre-run de F0 — sin dependencias (habilita skills v3 para los sub-agentes del run).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-harness` (Gate H, sesión `ses_f1f7f13ddffeyGojMVuXKuUUJ1`) — 3 rondas
- **Enfoque:** integridad de migración, contrato v2, spec↔impl, robustez, riesgos
- **Cómo se probó:** evidencia mecánica re-ejecutada por el revisor (selftest, smoke, node --check, probes propios con contraejemplos)
- **Veredicto:** ✅ APPROVE (ronda 3 — H-01..H-06 y R-01/R-02 cerrados)

## Notas
- ⚠️ **Owner action:** restart de OpenCode para que el MCP live cargue v3 (los sub-agentes del run usan el server en memoria).
- Evidencia completa en commits `.opencode`: `903a029` (feat sdp) + `e97c093` (memory).
