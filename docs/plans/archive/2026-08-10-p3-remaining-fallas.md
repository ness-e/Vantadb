# Plan de Ejecución: P3 restante + fallas operativas §3.6

> **Inicio:** 2026-08-10
> **Estado:** ✅ COMPLETADO (2026-08-10)
> **Fuente:** docs/research/2026-08-10-agent-engineering/REPORTE-FINAL.md (§3.6-1 fallas operativas + gap-01/gap-02 §6)

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 6  | 3     | 0    | 0         |

Todos los contratos verificados por command (2026-08-10). Commits: 0e298f3d, e44b26a5, 44154c0f,
b1a117de, c14050eb, fe21a890.

Cierra las fallas operativas reales observadas en la sesión de investigación que P0-P2 no cubrieron,
más el P3 restante de trazabilidad. Cada tarea toca archivos **exclusivos** → sub-agentes paralelos sin
conflicto de edición. Ownership:
- `.opencode/AGENTS.md` → SOLO T6 (vanta-docs). Ningún otro agente lo edita.
- `campaign-server.mjs` → SOLO T2.
- `verify*.ps1` + `scripts/validate-docs-coverage.ps1` → SOLO T4.
- `docs/plans/*` (este plan) → SOLO el orquestador (yo).

DEFER: P3-2 (estimación calibrada — requiere histórico de effort real), P3-3 (mutation gate, 🔴 4-8h),
P3-9 (cargo-mutants CI Heavy, 8-16h). Se desbloquean cuando existan datos/tiempo.

---

### Task 1: P3-rem — SARL trazabilidad (peldaño + desenlace)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `.opencode/task-system/enforcement/session-tracking.ps1`, `.opencode/task-system/prompts/subagent-recovery.md`
- **Gate Justificación:** SARL define la escalera pero no se registra qué peldaño se usó ni el desenlace (§3.6-f4) → loop de aprendizaje incompleto.
- **Gate Result:** ✅ DO
- **Contrato:** `session-tracking.ps1` soporta registro del peldaño SARL (1..4) + desenlace (DONE/INCOMPLETE/FAILED) por taskId en el session file; `subagent-recovery.md` lo referencia como paso obligatorio §3 regla 7. Verificar con `rg -c "sarl" session-tracking.ps1` ≥1 y `rg -c "peldaño\|peldaño" subagent-recovery.md`.
- **Estado:** ✅ COMPLETED — `44154c0f` (SARL trace registry)

### Task 2: P3-rem — Telemetría skill/tool → primer intento
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-tuner
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`, `evals/eval-metrics.mjs`, `evals/northstar.mjs`
- **Gate Justificación:** No hay correlación skill/tool → tarea pasó verify al primer intento; el input que calibra P0-1 no se recolecta (§3.6-f8).
- **Gate Result:** ✅ DO
- **Contrato:** el verify-log.jsonl (append en campaign-server) agrega campos `skills` y `toolUsed` (derivados del task file / command) cuando están disponibles (sin romper el try/catch existente); eval-metrics reporta columna/estadística "skills → primer intento" en `docs/reports/pipeline-evals.md`; northstar.mjs lo suma a su scorecard. `node evals/eval-metrics.mjs` exit 0 con log vacío (degrade).
- **Estado:** ✅ COMPLETED — `b1a117de` (skill/tool telemetry)

### Task 3: P3-rem — Trazabilidad de impacto (Regla 0 auditable)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `.opencode/task-system/prompts/task.md`, `.opencode/task-system/prompts/pipeline-full.md`
- **Gate Justificación:** Regla 0 exige "leí + mapeé referencias" pero es prosa; sin registro auditable (§3.6-f9).
- **Gate Result:** ✅ DO
- **Contrato:** task.md template agrega campo "Impacto mapeado" (archivos leídos + referencias entrantes/salientes + veredicto) como paso obligatorio previo a editar; pipeline-full lo exige en la fase DISCOVERY/IMPLEMENT. `rg -c "Impacto mapeado\|blast radius\|Blast Radius" task.md` ≥1.
- **Estado:** ✅ COMPLETED — `e44b26a5` (auditable Regla 0 impact-mapping)

### Task 4: P3-rem — Gate de docs (Regla 3) + reconciliar rutas de script
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-lead
- **Archivos clave:** `dev-tools/verify.ps1`, `dev-tools/verify_changed.ps1`, `scripts/validate-docs-coverage.ps1`, `docs/operations/CI_POLICY.md`
- **Gate Justificación:** verify puede pasar rompiendo docs/api (Regla 3 no es gate mecánico, §3.6-f10); `verify_changed.ps1` (dev-tools/) vs `scripts/validate-docs-coverage` contradicen rutas (gap-02 §6.5).
- **Gate Result:** ✅ DO
- **Contrato:** `verify_changed.ps1` y/o `verify.ps1` corren `scripts/validate-docs-coverage.ps1` cuando el diff toca `src/`, bindings o `docs/api/` (guard si el script o ps1 no existe — no bloquear si es quick check); CI_POLICY.md documenta la ruta canónica (`dev-tools/verify_changed.ps1` quick; `scripts/validate-docs-coverage.ps1` gate docs). Verificar: `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` exit 0; `rg -c "validate-docs-coverage" verify_changed.ps1 verify.ps1` ≥1.
- **Estado:** ✅ COMPLETED — `c14050eb` (docs-coverage gate in verify)

### Task 5: P3-rem — Reconciliación de memorias
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `.opencode/skills/progreso/SKILL.md`
- **Gate Justificación:** `lessons.md`/`decisions.md` (agente) y `docs/progreso/*` + `docs/Backlog.md` (proyecto) pueden divergir sin reconciliación (§3.6-f11).
- **Gate Result:** ✅ DO
- **Contrato:** progreso SKILL.md agrega un Trigger/check de reconciliación: al cerrar milestone, comparar lessons/decisions con Backlog/progreso y registrar divergencias o vacíos a resolver. `rg -c "reconcil\|reconciliation\|Reconciliación" SKILL.md` ≥1.
- **Estado:** ✅ COMPLETED — `fe21a890` (memory reconciliation trigger)

### Task 6: P3-rem — Glob workaround documentado + AGENTS.md cifras reales
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-docs
- **Archivos clave:** `.opencode/VANTADB-OPERATING-MANUAL.md`, `.opencode/AGENTS.md`
- **Gate Justificación:** El glob tool devolvió vacío en Windows para patrones válidos y se resolvió con bash (gap-02 §6.2) sin documentar; AGENTS.md afirma "104 skills"/"32 skills" cuando el real es 82+29=111 (gap-02 §6.4, ya corregido en SKILLS-MANIFEST).
- **Gate Result:** ✅ DO
- **Contrato:** manual documenta el workaround (glob vacío → `Get-ChildItem` + `Test-Path` + Read de directorios); AGENTS.md actualiza conteos: "104 skills del proyecto" → 111, y "82 + 32" → "82 + 29". `rg -c "111\|29 skills" AGENTS.md` ≥1. NO tocar rutas de scripts en AGENTS.md (tabla CI/Hooks queda como está; T4 no edita AGENTS.md).
- **Estado:** ✅ COMPLETED — `0e298f3d` (stale skill counts + glob workaround)