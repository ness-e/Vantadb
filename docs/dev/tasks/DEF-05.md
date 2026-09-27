# DEF-05: North-star + success criteria (SPEC/VISION)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 13)
- **Fuente:** Backlog `DEF-05` (L940) · master Task 13 · decisión owner 2026-09-24 (SPEC Adenda #4)
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴
- **Tipo:** Docs (decisión + docs)
- **Turns estimados:** 7
- **Creado:** 2026-09-26T19:55
- **last-synced:** 2026-09-26T19:55
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 1 abierta — comando/consulta exacta de medición en el proxy actual (Step 1)
- **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ICP-01 (métrica instrumentada en demo CI) · F5/F6 (métricas de ICP y anuncio cuelgan de esto) · DEF-07 (criterio de clasificación = North Star) · `VISION.md` §Success Metrics |
| Callees | `vanta-proxy` (`/snapshot` turn reports/sessions, `[report]` OTLP spans; WIRE-01 ✅ loop de memoria, commit `679c75a9`) · `SPEC.md` §Success Criteria (campaña) · Backlog `DEF-05` |
| Implicaciones | Doc-only; VISION.md:265-269 ya declara la North Star (2026-09-24) → DEF-05 formaliza en SPEC + fija comando/consulta · guardrails tocan gates existentes (p99 >15% → FIND-154/HARD-06; artefactos sincronizados → HARD-01) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `SPEC.md` (122L) · `docs/dev/vision/VISION.md` (278L) · `docs/api/PROXY.md` (157L) · `docs/dev/avance/activo/operaciones.md` (sección WIRE-01)
- **Archivos referenciados hacia dentro (imports/dependencias):** VISION frontmatter `related: GO_TO_MARKET.md`; SPEC Adenda #4 apunta a DEF-05; PROXY.md documenta `/snapshot` y `[report]` (fuentes de medición)
- **Archivos que referencian a los editados (referencias entrantes):** `ICP-01` (Backlog L924, dep "WIRE-01 (métrica)") · master Task 13/14 (DEF-07 usa North Star como criterio)
- **Veredicto impacto:** **bajo** — se agrega/ajusta la sección de producto; riesgo = inconsistencia SPEC↔VISION si se editan sin alinear (Regla 3, mismo PR).

## Contrato
"SPEC/VISION declaran North Star + guardrails (0 hallazgo crítico seguridad, latencia) Y la métrica es medible con el proxy actual (comando/consulta documentada)"

## Spec (SDD — decisión de producto)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | North Star | A) "agentes activos que recuperan una memoria con éxito en ventana de 7 días" (sesiones MCP con put+search la misma semana) / B) otra métrica | A | ✅ decidido owner 2026-09-24 (`SPEC.md:115` Adenda #4; `VISION.md:267`) |
| 2 | Guardrails | A) set de VISION: 0 hallazgos high sin parche ≤7d · 0 regresión p99 >15% · 100% artefactos con versión sincronizada · 0 violaciones Regla 11 / B) subset | A | ✅ decidido-por-evidencia (`VISION.md:269`; confirmar con owner en Step 1 si hay duda) |
| 3 | Ubicación | A) `SPEC.md`: criterios de producto (North Star + guardrails), etiquetando los de campaña del MVP como tales · `VISION.md`: alinear + comando de medición · `ROADMAP.md` histórico = solo referencia (no editar) / B) doc nuevo | A | ✅ decidido-por-evidencia (Backlog `DEF-05` archivos; master Task 13; `SPEC.md:118`) |
| 4 | Medición | A) comando/consulta documentada contra el proxy actual (auth `/snapshot` + report spans); si no alcanza → declarar + FIND + DEFER parcial / B) declarar sin medición | A | ⏳ verificar en Step 1 (Incógnita 1) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) North Star = **frase canónica única**, idéntica en SPEC y VISION (sin variantes); (2) guardrails no decorativos — cada uno con fuente/comando de medición; (3) baseline inicial documentado (hoy 0, con fecha); (4) **no instrumentar código** en esta task (si falta medición → FIND + DEFER parcial, no implementar acá).
- **Comandos de verificación:** `rg -n "North Star" SPEC.md docs/dev/vision/VISION.md` (frase idéntica) · `pwsh scripts/validate-docs-coverage.ps1` → exit 0.
- **Deuda pendiente:** si la medición no es derivable del proxy actual → FIND propuesto (el orquestador lo ingresa al Backlog) + DEFER parcial declarado.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# DEF-05: North-star + success criteria (SPEC/VISION)` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Step 1 ⬜ PENDING (verificar medición en proxy) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia (comandos ejecutados) |
| `nextTask` | DEF-07 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (task doc-only; no introduce deuda nueva).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable ✅ (North Star + guardrails en SPEC/VISION + medición documentada o DEFER parcial + validate-docs-coverage verde) |
| **Commit** | 1 commit atómico `docs:` (SPEC + VISION + task file), `git diff` limpio, verificación mecánica registrada (nunca auto-reporte) |
| **Release** | N/A justificado (criterios de producto en docs; sin artefacto de release) — ver Notas |

## Herramientas necesarias
- `curl`/inspección de `vanta-proxy` (medición — Step 1) · `rg` sobre `vanta-proxy/src` (campos de telemetría)
- `pwsh scripts/validate-docs-coverage.ps1` · `npx markdownlint-cli2 SPEC.md docs/dev/vision/VISION.md`
- `pwsh dev-tools/ocr-review.ps1` (input del review P2-01)

**Skills cargadas (SDP):** `SDP: campaign_discover_skills_v2 phase=BUILD → campaign-executor, source-driven-development, spec-driven-development, interview-me, documentation-and-adrs, writing-guidelines`
- `campaign-executor` — flujo pipeline/task system (base).
- `source-driven-development` — verificar la medición contra el código real del proxy (no supuestos).
- `spec-driven-development` — tabla Spec.
- `interview-me` — confirmar guardrails/owner si hiciera falta (keyword mapping "spec").
- `documentation-and-adrs` — estándar de docs.
- `writing-guidelines` — prosa técnica (EN).

## Investigation Notes
- **VISION.md:265-269** ya declara la North Star (2026-09-24): "Agentes activos que recuperan una memoria con éxito en ventana de 7 días (medible en el proxy/MCP: sesiones con put+search la misma semana)" + guardrails (0 high sin parche ≤7d · 0 regresión p99 >15% · 100% artefactos con versión sincronizada · 0 violaciones Regla 11). DEF-05 = formalizar en SPEC + fijar medición.
- **SPEC.md:99-104** §Success Criteria actuales = de campaña MVP; **SPEC.md:118** (Adenda): "los success criteria de campaña siguen vigentes para el MVP; los de producto son la North Star de DEF-05" → etiquetar, no borrar.
- **Medición — estado real:** WIRE-01 ✅ (`679c75a9`, 2026-09-25) cerró el loop de memoria del proxy (dual-write L1 + cost + presupuesto de inyección). `PROXY.md`: `GET /snapshot` (auth `x-vanta-user-key`, sin bypass loopback) expone "recent turn reports, active sessions, write-back queue, rate-limit telemetry, cost snapshot"; `[report]` exporta spans por turno (OTLP-JSON, default off). Memoria en el wire: `memory_tools.rs` TOOL_CAPTURE/TOOL_SEARCH (put+search del proxy).
- **Frontera honesta:** si los campos actuales de `/snapshot`/report no derivan "sesiones con put+search misma semana", NO inventar: documentar la definición + registrar FIND para la instrumentación mínima (DEFER parcial, stop condition del master Task 13).
- **Downstream:** ICP-01 (demo CI + métrica instrumentada), DEF-07 (criterio de clasificación), F6 (anuncio).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — comando/consulta exacta de medición en el proxy actual (Step 1) |
| Pendientes de ejecución (downhill) | 4 — Steps 1–4 |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica a la edición (doc-only); el guardrail de hallazgos ≤7d es política, no instrumentación. Sin dependencias nuevas.
- [ ] **PERFORMANCE** — no aplica a la edición (sin código); el guardrail p99 >15% referencia el gate existente (FIND-154/HARD-06), no lo modifica.

## Steps

### Step 1: Verificar medición de la North Star en el proxy actual
- **Archivos:** `vanta-proxy/src/**` (campos de telemetría), `docs/api/PROXY.md`
- **Acción:** determinar el comando/consulta exacta que produce "sesiones con put+search la misma semana" con el proxy actual (inspeccionar `/snapshot` + report/telemetry; `rg` de campos). Si no es derivable → documentar definición + FIND propuesto + DEFER parcial.
- **Verify:** comando ejecutado contra proxy local (o inspección con refs file:line) y resultado documentado: query exacta **o** gap explícito con FIND.
- **Estado:** ⬜ PENDING

### Step 2: Draft North Star + guardrails + baseline
- **Archivos:** borrador en este task file
- **Acción:** redactar la frase canónica (idéntica para SPEC/VISION), tabla guardrail→fuente→comando de medición y baseline inicial (0, con fecha).
- **Verify:** ≥4 guardrails con fuente citada (`VISION.md:269`, Backlog L940); baseline registrado con fecha 2026-09-26.
- **Estado:** ⬜ PENDING

### Step 3: Aplicar a SPEC + VISION
- **Archivos:** `SPEC.md` (criterios de producto; etiquetar los de campaña), `docs/dev/vision/VISION.md` (alinear + comando de medición); `ROADMAP.md` NO se edita (histórico)
- **Acción:** integrar North Star + guardrails; referenciar la medición documentada; mantener coherencia con la jerarquía de DEF-01.
- **Verify:** `rg -n "North Star" SPEC.md docs/dev/vision/VISION.md` → frase idéntica en ambos; `git diff --check` limpio.
- **Estado:** ⬜ PENDING

### Step 4: Gates + cierre
- **Archivos:** `docs/dev/tasks/DEF-05.md` (estado/recitation)
- **Acción:** correr gates, verificar links, registrar evidencia y commit local `docs:`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 · markdownlint sin errores · OCR sin Critical/High.
- **Estado:** ⬜ PENDING

## Dependencias
- **DEF-01** (jerarquía de producto — coherencia "agentes"/núcleo; DEF-05 cierra después o en coordinación; el master declara que DEF-01 bloquea la coherencia de DEF-05).
- **WIRE-01** ✅ (base de la métrica — loop de memoria del proxy, `679c75a9`).
- Downstream: ICP-01 (métrica instrumentada), DEF-07 (criterio), F6 (anuncio).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` (leaf, no implementa) — fallback: `doubt-driven-development` en contexto fresco.
- **Enfoque:** ¿la métrica es **medible de verdad** con el proxy actual (comando verificado, no supuesto)? ¿Guardrails con fuente (no decorativos)? ¿North Star idéntica en SPEC y VISION? ¿Consistente con DEF-01?
- **Cómo se probó:** comando de medición ejecutado/inspeccionado y registrado (no auto-reporte) + diff solo-docs + OCR delegation sin Critical/High.
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⬜ pendiente (✅ approve | ❌ cambios requeridos)

## Notas
- **No aplica Fase 1 — Evidencia de Debugging** (tipo Docs, no Bug).
- **Scope discipline:** no instrumentar código (si falta medición → FIND + DEFER parcial); no editar `docs/dev/strategy/ROADMAP.md` (histórico); no duplicar métricas de marketing.
- **Release N/A:** criterios de producto en docs; el cambio no participa del contrato de release.
- Creado por `vanta-docs` (subagente) desde el master roadmap — estado inicial ⬜ PENDING.
