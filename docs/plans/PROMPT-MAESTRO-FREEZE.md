# Prompt Maestro: Feature Freeze — Estabilización y Zero-Bug Policy

```
╔══════════════════════════════════════════════════════════════╗
║  VantaDB — Feature Freeze                                   ║
║  Rama: main → tag → develop                                 ║
║  Zero-Bug Policy. No features. Only stabilization.          ║
╚══════════════════════════════════════════════════════════════╝
```

Ejecutá el siguiente plan de forma secuencial, fase por fase. No avances a la siguiente fase sin confirmar que la anterior está completa y certificada. Usá **todas** las herramientas disponibles: MCP tools, skills, sub-agents, commands, y pipeline system.

---

## FASE 0 — Pre-Freeze Assessment (read-only, ~30-45 min)

### 0.1 `/audit full` — Auditoría completa de 9 fases

Ejecutá `/audit full` desde la raíz del proyecto. Esto corre en waves paralelas:

- **Wave 1** (directa): Phase 0 (pre-check) → Phase 1 (CLI Mechanical: `just verify`, `just audit-cargo`, `just machete`, `just size`)
- **Wave 2** (paralela): Phase 2 (seguridad — sub-agent `security-and-hardening`) + Phase 3 (performance — sub-agent `performance-optimization`) + Phase 4 (code review — sub-agents `code-review-and-quality` + `ponytail-review`)
- **Wave 3** (condicional): Phase 5 (root cause analysis — solo si hay bugs abiertos en el reporte)
- **Wave 4** (paralela): Phase 6 (deep module — sub-agent `review-deep` con skill `vantadb-full-review`) + Phase 7 (Full ISO — skill `vantadb-full-review` con FASE 9 taxonomy inline, 12 categorías)
- **Wave 5** (bloqueante): Phase 8 (certify — sub-agent `vantadb-certify` + `just certify`)

**Output:** `docs/audit-reports/audit-full-<timestamp>.md`
**Checkpoint:** `docs/last-audit-state.json`

**No corrijas nada aún.** Solo registrá hallazgos. Clasificá cada hallazgo como `BUG`, `REFACTOR`, `OPTIMIZATION`, o `CLEANUP`.

### 0.2 Análisis de deuda técnica desde Backlog.md

Leé `docs/Backlog.md` completo. Identificá:

1. **Bugs abiertos** (Estado ❌ o 🔴 BLOQUEADO) — prioridad máxima, van al plan
2. **TIER-0 tasks** con Estado ✅ DO — evaluá si son refactor/optimización (pasan) o feature nueva (se bloquean)
3. **Deuda técnica explícita**: tasks marcadas como `tech-debt`, `refactor`, `fix`, `polish`
4. **Warnings de compilación**: `cargo check -p vantadb 2>&1 | Select-String "warning"` (cada warning es un bug menor)

Categorizá cada ítem en: **P0 (debe ir)** | **P1 (debería ir)** | **P2 (nice-to-have)**

### 0.3 Crear stabilization plan

Usá el pipeline command para crear el plan de estabilización:

```
/pipeline plan -Input docs/Backlog.md -Output docs/plans/<YYYY-MM-DD>-stabilization.md -Tier "bug,tech-debt,refactor,optimization"
```

El plan debe incluir SOLO estas categorías:
- **P0–Bugs:** Correcciones de bugs existentes (Zero-Bug Policy: 0 bugs al final)
- **P0–Refactor:** Refactorización estructural necesaria (no estética — YAGNI)
- **P0/P1–Optimizaciones:** Optimizaciones de performance, seguridad, tamaño de binario
- **P1–Cleanup:** Código muerto, warnings de compilación, dependencias no usadas
- **P2–Documentación:** Actualización de docstrings, comentarios obsoletos, README

**Bloqueá explícitamente:**
- Nuevas features (cualquier task que agregue funcionalidad nueva)
- Cambios de API pública (a menos que sea bug fix)
- Nuevas dependencias (a menos que sea para reemplazar deuda técnica existente)
- Refactors cosméticos (solo estructura, no estilo)

---

## FASE 1 — Stabilization Sprint (~2-4 horas, múltiples iteraciones)

### 1.1 Arrancar el harness de ejecución

Iniciá el loop PowerShell desde la terminal o vía MCP:

```
.opencode\task-system\harness\harness-executor.ps1 `
  -PlanFile docs\plans\<YYYY-MM-DD>-stabilization.md `
  -CampaignId "stabilization-v1" `
  -Interval 10 `
  -StallThreshold 3 `
  -Timeout 900 `
  -Model deepseek-v4-flash-free
```

Cada iteración del harness:
1. Lee el plan file, encuentra próxima tarea ⬜ PENDING
2. Inyecta `.opencode/task-system/prompts/iter.md` con `{{CAMPAIGN_ID}}` = "stabilization-v1"
3. Invoca `opencode run` con el prompt
4. El agente ejecuta **exactamente un paso** (no una tarea completa)
5. Usa MCP tools para todo: `campaign_get_next_task`, `campaign_update_task_state`, `campaign_verify_cmd`, `campaign_load_skills`, `campaign_detect_task_type`
6. Escribe recitation al final de cada iteración
7. El harness verifica progreso, detecta stalls, repite

**Reglas durante la ejecución:**
- Cada tarea termina con ✅ COMPLETADO o ❌ FAILED (no dejar ⏳ IN PROGRESS colgadas)
- Si una tarea falla 3 veces consecutivas → marcá ❌ FAILED, escribí por qué, pasá a la siguiente
- Usá `campaign_session_track` (MCP) para tracking de sesión
- Después de cada tarea: `git add -A && git commit -m "fix: <ID> — <descripción>"`
- **No cambies scope.** Si encontrás algo extra durante una tarea, anotalo en `campaign_memory_write` pero no lo implementes

### 1.2 Zero-Bug Policy enforcement

Para cada bug encontrado en la auditoría o backlog:

1. **Root cause first**: `codegraph_explore` para blast radius antes de tocar código
2. **Fix at the root**: no parches sintomáticos — el fix más lazy es el que resuelve la causa en el punto compartido más bajo (ponytail ladder: shared function guard > caller guard)
3. **Verificación**: Usá `campaign_verify_cmd` con el contrato de la task + corré `just verify` / `cargo test -p <crate>` para confirmar
4. **Commit**: `git commit -m "fix: <ID> — <one-line root cause>"` con el contrato en el body

**Política:** 0 bugs al final de la fase. No se cierra la fase si queda algún bug abierto.

### 1.3 Refactorización y optimización estructural

Para cada task de refactor/optimización:

1. **Medir antes**: Si es optimización, registrá la métrica antes del cambio (`cargo size`, `cargo bench`, line count, dependencias)
2. **Ponytail ladder obligatorio** para cada cambio:
   - Rung 1: ¿Ya existe en el codebase? (reuse > rewrite)
   - Rung 2: ¿Stdlib lo cubre? (stdlib > custom)
   - Rung 3: ¿Platform feature? (CSS > JS, DB constraint > app code)
   - Rung 4: ¿Dependencia ya instalada? (no nuevas)
   - Rung 5: ¿Puede ser una línea? (one-liner > function)
   - Rung 6: Mínimo código que funciona
3. **Medir después**: Verificá que la métrica mejoró
4. **No abstracciones prematuras**: No interfaces con una sola implementación, no factories para un producto, no configuración para valores que nunca cambian

### 1.4 Certificación integrada

Después de **cada 3-5 tareas** completadas, ejecutá:

```
/audit certify
```

Esto corre:
- Phase 0 (pre-check)
- Phase 1 (CLI Mechanical — warnings, clippy, machete, size)
- Phase 4 (code review — sub-agents, solo el diff desde el último certify)
- Phase 8 (certify gate — `vantadb-certify` + `just certify`)

**Si certify falla:** no avances, corregí lo que falló primero.
**Si certify pasa:** continuá con las siguientes tareas.

---

## FASE 2 — Seal & Ship (~15 min)

### 2.1 Feature freeze final check

1. Corré `/audit full` **una vez más** para asegurar que no se introdujeron bugs nuevos
2. Verificá que el reporte final tenga 0 bugs en todas las categorías de la FASE 9 taxonomy
3. Verificá que no hay ⬜ PENDING en el plan file (deben estar todos ✅ COMPLETADO o ❌ FAILED — si hay FAILED, documentá como conocido)
4. Verificá que `git status --short` está limpio (solo archivos committed)
5. Verificá que `cargo check -p vantadb` no tiene warnings

### 2.2 Sellar versión estabilizada

```
# Crear tag con versión estabilizada
git tag -a v<MAJOR>.<MINOR>.<PATCH>-stable -m "Stabilization freeze — <YYYY-MM-DD>"

# Crear rama develop para futuro desarrollo
git branch develop

# Push todo
git push origin main --tags
git push origin develop

# Verificar
git branch -a
git log --oneline -5
```

**Version bump policy:** Si hay bug fixes → incrementar PATCH. Si hay refactors estructurales → incrementar MINOR. Si no hay cambios funcionales → mantener versión.

### 2.3 Reporte final

Creá `docs/stabilization-report.md` con:

```markdown
# Stabilization Report — <YYYY-MM-DD>

## Summary
- **Bugs fixed:** N
- **Refactors completed:** N
- **Optimizations applied:** N
- **Tech debt items closed:** N
- **Warnings eliminated:** N

## Zero-Bug Status
- **Bugs remaining:** 0 ✅
- **Warnings remaining:** 0 ✅

## Git
- **Tag:** `v<MAJOR>.<MINOR>.<PATCH>-stable`
- **main:** sealed
- **develop:** created for future feature development

## Known Issues (if any)
- <item>: <reason why not fixed>
```

Usá `campaign_memory_write` para persistir el reporte y la decisión de freeze en el historial de la sesión.

---

## Checklist de cumplimiento (ejecutar al final)

```
[ ] Audit full passed with 0 critical/high findings
[ ] All backlog bugs resolved (P0 bugs = 0)
[ ] `/audit certify` passes
[ ] `cargo check -p vantadb` — 0 warnings
[ ] `git status --short` — clean
[ ] `git log --oneline origin/main..HEAD` — solo fix/refactor commits (no features)
[ ] Tag v<X>.<Y>.<Z>-stable creado
[ ] Rama develop creada
[ ] `docs/stabilization-report.md` generado
[ ] `campaign_memory_write` con resumen del freeze
```

---

## Rutas clave descubiertas en el entorno

| Recurso | Ruta |
|---------|------|
| **Harness executor** | `.opencode/task-system/harness/harness-executor.ps1` |
| **Plan prompt** | `.opencode/task-system/prompts/plan.md` |
| **Iter prompt** | `.opencode/task-system/prompts/iter.md` |
| **Task prompt** | `.opencode/task-system/prompts/task.md` |
| **Pipeline command** | `.opencode/commands/pipeline.md` |
| **Audit command** | `.opencode/commands/audit.md` |
| **Audit full prompt** | `.opencode/task-system/prompts/audit-full.md` |
| **Pipeline run prompt** | `.opencode/task-system/prompts/pipeline-run.md` |
| **Certify skill** | `.opencode/skills/vantadb-certify/SKILL.md` |
| **Full review skill** | `.opencode/skills/vantadb-full-review/SKILL.md` |
| **Campaign executor skill** | `.opencode/skills/campaign-executor/SKILL.md` |
| **Campaign executor rules** | `.opencode/skills/campaign-executor/RULES.md` |
| **MCP server** | `.opencode/task-system/mcp/campaign-server.mjs` |
| **Agents (seguridad, perf, code review, deep review)** | `.opencode/agents/*.md` |
| **Operating manual** | `.opencode/VANTADB-OPERATING-MANUAL.md` |
| **Audit reports** | `docs/audit-reports/` |
| **Plans** | `docs/plans/` |
| **Backlog** | `docs/Backlog.md` |
| **Logs (harness)** | `.opencode/task-system/harness/logs/` |

---

**Ejecutá. No preguntes. Solo ejecutá.** Si algo no está claro, usá default (ponytail ladder) y anotá en `campaign_memory_write`. Si encontrás un bug fuera del plan, registralo pero no lo toques hasta que el plan lo incluya. Zero-Bug Policy significa arreglar bugs, no cazar fantasmas.
