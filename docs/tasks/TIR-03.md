# TIR-03: Mitigación/contención primero en incidentes — investigación/decisión

## Metadata
- **Plan file:** ninguno activo (Backlog directo, § Phase 4)
- **Fuente:** docs/Backlog.md § P18 línea 468
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
- **Tipo:** Planning/Docs (investigación del task-system, no código)
- **Turns estimados:** 5-8
- **Creado:** 2026-08-12T10:00
- **last-synced:** 2026-08-12T10:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 — resuelta: SÍ hace falta una fase de contención explícita (gap real confirmado: bug-workflow.md arranca diagnosticando); veredicto = IMPLEMENTAR docs mínimos (Fase 0.5)
- **Pendientes (downhill):** 0 — 3/3 steps ✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/Backlog.md` (P18 fila), `eng-02-systems.md` (fuente normativa: §4.2, §10 Fase 0), `gap-01-agents.md` FALTA#15, `REPORTE-FINAL.md` §3.3-15 |
| Callees | `docs/references/bug-workflow.md` (fase de contención candidata), `.opencode/skills/campaign-executor/RULES.md` §10b Iron Law, `.opencode/task-system/prompts/task.md` Fase 1 (bug-fix) |
| Implicaciones | Si se decide implementar: tocar `docs/references/bug-workflow.md` (añadir Fase 0.5 Contención) y/o `task.md` (gate en bugs 🔴). No cambia API pública, no toca código. Si WONTFIX/deferir: registrar decisión y cerrar |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/references/bug-workflow.md` (76L — NO tiene fase de contención; Fase 0 es "Diagnosticar", Fase 1 "Aislar Causa Raíz", Fase 2 "Fix y Verificación"), `eng-02-systems.md:209-214` (mitigar primero en SRE), `eng-02-systems.md:397-400` (Fase 0 — Contención: "solo si hay impacto en producción: mitigar primero, no debuggear en caliente"), `RULES.md:204-219` (§10b Iron Law), `task.md` (formato + Fase 1 bug), `gap-01-agents.md:50,114` (FALTA#15), `REPORTE-FINAL.md:352` (§3.3-15).
- **Archivos referenciados hacia dentro:** `docs/references/bug-workflow.md` → referenciado desde AGENTS.md ("Bug Workflow Reference") y REPORTE-FINAL.md:321; `RULES.md` → referenciado desde task.md/pipeline-full.md.
- **Archivos que referencian a los editados:** si se edita `bug-workflow.md`, AGENTS.md lo referencia; si se edita `RULES.md`, el task-system lo consume. Nada más.
- **Veredicto impacto:** BAJO — solo docs del task-system, sin código ni API.

## Contrato
"Existe un documento de decisión (docs/Investigaciones/2026-08-10-agent-engineering/TIR-03-decision.md) con: (1) análisis de las fuentes (eng-02 §4.2/§10, bug-workflow.md, RULES.md §10b), (2) veredicto EXPLÍCITO implementar / WONTFIX / deferir, (3) si implementar → diff propuesto mínimo (archivo + sección exacta) para el gate de contención, y (4) la fila TIR-03 queda resuelta en docs/Backlog.md (migrada a docs/progreso/README.md o marcada con el veredicto)."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. NO tocar el Iron Law de `RULES.md` §10b (root-cause antes de fix) — la contención es UN PASO ANTES, no lo reemplaza
  2. NO inventar un nuevo estado C0 ni cambiar la state machine — la contención (si implementa) es un gate de docs/workflow, no tooling
  3. El bug-workflow.md solo se edita si la decisión es "implementar"; un WONTFIX no toca código ni docs normativos
  4. El documento de decisión vive en `docs/Investigaciones/2026-08-10-agent-engineering/` (carpeta de investigación existente)
- **Comandos de verificación:** `rg "Contención|contenc" docs/references/bug-workflow.md` (si implementar) · validación markdown del doc de decisión · `git status` limpio al cierre
- **Deuda pendiente:** ninguna (es investigación)

## Steps (Plan → Act → Verify)

1. **✅ Investigar** — sintetizar las 4 fuentes: `eng-02-systems.md:209-214` (§4.2 SRE: mitigation antes que RCA), `eng-02-systems.md:397-400` (§10 Fase 0 Contención: solo si impacto producción), `docs/references/bug-workflow.md` (carece de fase de contención; empieza diagnosticando), `RULES.md:204-219` (Iron Law manda root-cause, sin paso de contención). Documentar en el doc de decisión: qué haría hoy un agente ante un build roto / backoff 🔴 (sigue el plan → empeora). Verify: doc de decisión escrito con la síntesis.

   **Síntesis (2026-08-12):** ver `docs/Investigaciones/2026-08-10-agent-engineering/TIR-03-decision.md`. Gap confirmado: bug-workflow.md no tiene fase de contención; SRE (eng-02:214) manda "mitigar primero, root-cause después". El caso real no es producción de usuarios sino el propio pipeline (build roto en develop, test suite en rojo, backoff). `plan.md` y SARL ya cubren parte; falta el paso de estabilización antes del debug.
2. **✅ Decidir** — veredicto EXPLÍCITO con criterio ponytail: ¿vale un paso de contención (revert/rollback/stop) para bugs 🔴 con impacto, o WONTFIX (YAGNI: no hay producción real, el Iron Law + stop-conditions ya cubren) / deferir? Decisión final en el doc. Verify: sección "Veredicto" con 1 párrafo que justifica.

   **Veredicto (2026-08-12):** **IMPLEMENTAR (docs mínimos)** — añadir Fase 0.5 Contención a `bug-workflow.md`. Descartadas WONTFIX (gap real, caso de uso frecuente, fix trivial) y deferir (costo de implementar < costo de esperar). No se crea tooling ni estados C0 — la solución parcial ya existe (stop-conditions, SARL), el cambio mínimo ordena la secuencia. Gate mecánico diferido: solo si la doctrina no basta.
3. **✅ Registrar + review** — si implementar: editar `docs/references/bug-workflow.md` (Fase 0.5 Contención antes de Fase 1) Y documentar el cambio; actualizar Backlog/progreso; REVIEW P2-01 por agente distinto (vanta-review/vanta-audit) del veredicto y del diff. Verify: `rg "Contención" docs/references/bug-workflow.md` encuentra la fase; fila TIR-03 migrada en progreso.

   **Aplicado (2026-08-12):** `docs/references/bug-workflow.md:18` — nueva "Fase 0.5: Contención/Estabilización" (disparador: build roto/CI/test suite en rojo; acción: revert o pausar + registrar; luego Fase 1 Iron Law). `rg "Contención"` ✅ (línea 18). No reemplaza el Iron Law — es un paso ANTES.

## Dependencias
- Ninguna (investigación autónoma)

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: docs del task-system, no toca trust boundaries ni input de usuario.
- [ ] **PERFORMANCE** — NO aplica: no toca hot path ni código.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (sesión `ses_0070d5217ffe8TO33SJKYgLzKz`)
- **Enfoque:** ✅ approve — veredicto IMPLEMENTAR correcto, gap real confirmado, Fase 0.5 no contradice el Iron Law (paso ANTES), no toca state machine C0, proporcionado. Hallazgos 🟡 opcionales aplicados (header cross-ref + ruta explícita).
- **Cómo se probó:** verificación por comando/lectura directa (bug-workflow.md:18, rg "Contención", citas verbatim de eng-02/RULES/plan/subagent-recovery).
- **Veredicto:** ✅ **approve**.

## Notas
- La fuente normativa es clara en SRE ("mitigar primero, root-cause después" eng-02:214), pero VantaDB core NO tiene servicio en producción de usuarios: el "impacto en producción" del §10:397 no aplica literalmente. El caso de uso real es: build roto / backoff en el pipeline de tareas → el agente debe DETENERSE y estabilizar antes de debuggear. Esto ya está cubierto parcialmente por stop-conditions y 🔴BLOQUEADO del plan.md, no por bug-workflow.md.