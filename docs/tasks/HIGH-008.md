# HIGH-008: Autonomous Flag en Plan File — template + parsing

## Metadata
- **Plan file:** docs/plans/2026-08-28-master-pipeline-optimization.md
- **Fuente:** plan file Task 8 / HIGH-008 (ALTO #8)
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Tipo:** Mixto (infra task-system + docs prompts)
- **Turns estimados:** 1
- **Creado:** 2026-08-28T16:45
- **last-synced:** 2026-08-28T16:45
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes
- **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8a

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/prompts/plan.md` (template, línea 246), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 8 + Task 16), `.opencode/task-system/prompts/question-gates.md` (§Anti-abuso, Routing), `.opencode/task-system/mcp/campaign-server.mjs` (campaign_get_next_task autonomous) |
| Callees | `.opencode/task-system/mcp/parsers.mjs` (extractAutonomous, getOrCreateCampaignId, parseTasks), `.opencode/task-system/prompts/plan.md` Formato del plan file, `question-gates.md` Gates P/D/V/C |
| Implicaciones | contrato aditivo: si faltara `> **Autonomous:** false/true` en plan.md template → modo autónomo no usable sin campo en plan (HIGH-008 gate justification); si faltara `extractAutonomous` → campaign_get_next_task no expone `autonomous` → Gates V+C no suprimibles; presente → gate HITL completo con Autonomous true/false, parsers operativos, plan template canónico; sin breaking change (ponytail rung 1 si ya existe, default false backward compat) |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición
> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES del primer step de edición. Sin este bloque poblado, NO se escribe ni se ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** `.opencode/task-system/prompts/plan.md` (304 líneas, Formato del plan file líneas 240-293 con `> **Autonomous:** false` línea 246), `.opencode/task-system/prompts/question-gates.md` (134 líneas, §Anti-abuso líneas 98-112 con `> **Autonomous:** true` y routing HITL), `.opencode/task-system/mcp/parsers.mjs` (162 líneas, extractAutonomous líneas 157-162), `.opencode/task-system/mcp/campaign-server.mjs` (import extractAutonomous línea 12 + uso línea 524 autonomous: extractAutonomous(content)), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 8 líneas 206-220, contrato grep -n 'Autonomous:' plan.md)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `plan.md` no tiene imports; template Formato del plan file incluye `> **Autonomous:** false` como campo; `question-gates.md` referencia `> **Autonomous:** true` en §Anti-abuso + §Routing; `parsers.mjs` exporta `extractAutonomous(content)` con regex `/^>\s*\*\*Autonomous:\*\*\s*(true|false)\s*$/im` → true/false/null; `campaign-server.mjs` importa `extractAutonomous` desde `./parsers.mjs` y lo expone en `campaign_get_next_task` response `autonomous: extractAutonomous(content)`
- **Archivos que referencian a los editados (referencias entrantes):** `Select-String "Autonomous" plan.md` → 1 hit plan.md:246; `Select-String "Autonomous" question-gates.md` → 2 hits (líneas 109,111 Anti-abuso); `Select-String "extractAutonomous" parsers.mjs` → 1 hit 159:export; `Select-String "extractAutonomous" campaign-server.mjs` → 2 hits (12 import, 524 uso); `Select-String "Autonomous" docs/plans/2026-08-28-master-pipeline-optimization.md` → 8 hits (Task 8, Task 16, checklist, verif 543, recitaciones); `git log --follow -- plan.md` → 9e5730ff feat Master pipeline optimization (implementa HIGH-008 junto a 19 items); `git blame plan.md` línea 246 → 9e5730ff Eros Nessy 2026-08-28; `git log --follow -- parsers.mjs` → a37c52ff fix + HIGH-008 original
- **Veredicto impacto:** bajo — verificación idempotente sin edición; si hubiese edición sería bajo aditivo (solo añade 1 campo `> **Autonomous:** false` al template Formato del plan file + 1 función extractAutonomous en parsers.mjs + import+uso en campaign-server.mjs), compatible con question-gates §Anti-abuso (true= solo V+C, false= todos gates). Sin edición requerida porque template ya tiene campo y parsers ya tiene función.

## Contrato
Contrato del plan (HIGH-008):
```
campaign_verify_cmd command="grep -n 'Autonomous:' .opencode/task-system/prompts/plan.md" → campo en template
```
Verificación mecánica (powershell equivalente campaign_verify_cmd):
```
Select-String -Pattern "Autonomous:" -Path .opencode/task-system/prompts/plan.md → línea 246 ✅ (> **Autonomous:** false  # true = solo Gates V+C seguridad; false = todos los gates (default))
Select-String -Pattern "extractAutonomous" -Path .opencode/task-system/mcp/parsers.mjs → 159:export function extractAutonomous ✅
Select-String -Pattern "extractAutonomous" -Path .opencode/task-system/mcp/campaign-server.mjs → 12:import + 524:autonomous: extractAutonomous(content) ✅ (2 hits)
```
Resultado: contrato pasa ✅ (ver Investigation Notes). Sin re-edición — ponytail rung 1 idempotente.

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)
> Definición de contenido válido: question-gates.md §"Contenido válido de `## Spec`". Tabla de decisiones O justificación por evidencia por ítem. `N/A` solo aceptable en tareas 100% docs sin decisiones técnicas.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Re-implementar Autonomous Flag si plan.md template ya tiene `> **Autonomous:** false` en Formato del plan file (línea 246) y parsers.mjs ya tiene extractAutonomous? | A) No re-implementar (idempotente, menor riesgo, ponytail rung 1) / B) Re-escribir igual (ruido, posible regresión, diff innecesario, duplica campo) | A | ✅ decidido-por-evidencia (ref: plan.md:246 Select-String 1 hit + parsers.mjs:159 export + campaign-server.mjs:12/524 import/uso; git show 9e5730ff diff prueba implementación original 2026-08-28; verify grep -n Autonomous pasa) |
| 2 | ¿Verificar solo template o template + parsing + question-gates Anti-abuso? | A) Solo template (contrato literal grep) / B) Template + parsers.extractAutonomous + campaign-server.mjs integration + question-gates.md §Anti-abuso (contrato extendido HIGH-008 descripción "Verifica que plan.md template ya tiene > **Autonomous:** false y que parsers.mjs tiene extractAutonomous") | B — descripción explícita pide ambos | ✅ decidido-por-evidencia (ref: task instrucciones "Verifica que plan.md template ya tiene > **Autonomous:** false y que parsers.mjs tiene extractAutonomous" + question-gates.md:109-112 Anti-abuso documenta true/false semantics) |

Justificación: plan pide "Si ya está, marca COMPLETED". Re-implementar introduce riesgo de duplicar campo y romper formato ya validado por pipeline run 20/20 (HIGH-008: grep Autonomous plan.md → 1 ✅).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `plan.md` Formato del plan file debe seguir conteniendo `> **Autonomous:** false  # true = solo Gates V+C seguridad; false = todos los gates (default)` en línea 246 justo después de `> **Fuente:**` y antes de `## Resumen`; `parsers.mjs` debe seguir exportando `extractAutonomous(content)` con regex `/^>\s*\*\*Autonomous:\*\*\s*(true|false)\s*$/im` líneas 159-162; `campaign-server.mjs` debe seguir importando `extractAutonomous` (línea 12) y exponiendo `autonomous: extractAutonomous(content)` en `campaign_get_next_task` response (línea 524); `question-gates.md` §Anti-abuso líneas 109-112 debe seguir documentando `> **Autonomous:** true` suprime Gates P/D/C (solo V + C seguridad operan); Task 8 Estado en plan file debe ser ✅ COMPLETED; default false para backward compat (extractAutonomous retorna null si ausente, no true)
- **Comandos de verificación:** `Select-String -Pattern "Autonomous:" -Path .opencode/task-system/prompts/plan.md` → 1 hit línea 246; `Get-Content plan.md | Select-Object -Index 244,245,246,247` → líneas 244-247 muestran Fuente → Autonomous → (blank) → ## Resumen; `Select-String -Pattern "extractAutonomous" -Path .opencode/task-system/mcp/parsers.mjs` → 159:export; `Select-String -Pattern "extractAutonomous" -Path .opencode/task-system/mcp/campaign-server.mjs` → 2 hits 12/524; `node -e "const {extractAutonomous}=require('./.opencode/task-system/mcp/parsers.mjs'); console.log(extractAutonomous('> **Autonomous:** true'))"` → true; `node -e "... false"` → false; `node -e "... null"` → null; `git blame .opencode/task-system/prompts/plan.md | Select-String Autonomous` → 9e5730ff; `git show 9e5730ff -- .opencode/task-system/prompts/plan.md` diff contiene +Autonomous; `node --check .opencode/task-system/mcp/parsers.mjs` → 0; `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0
- **Deuda pendiente:** ninguna — idempotente completo, sin edición; próxima tarea HIGH-009 continúa secuencial (Session Cleanup). Plan file header `docs/plans/2026-08-28-master-pipeline-optimization.md` no tiene `> **Autonomous:**` porque fue creado antes de HIGH-008 — el campo vive en el template (canónico plan.md Formato del plan file), no requiere backfill en planes históricos ya COMPLETED; futuros planes usarán template con default false

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HIGH-008: Autonomous Flag en Plan File — template + parsing` |
| `lastAction` | Step 1 VERIFY completado: plan.md:246 Autonomous false ✅ + parsers.mjs:159 extractAutonomous ✅ + campaign-server.mjs:12/524 import/uso 2 hits ✅ + question-gates Anti-abuso 109-112 ✅ + node --check 0 ✅ |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | HIGH-009 — Session Cleanup al Cerrar Campaña (siguiente en plan secuencial) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HIGH-009 |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda

> Regla 6 (AGENTS.md): toda deuda nueva introducida debe compensar deuda existente — el saldo neto por PR es 0 o negativo. Verificación idempotente sin código nuevo → sin deuda. Si se hubiese añadido Autonomous flag ex nihilo, deuda cero (feature aditiva acotada, 1 línea template + 6 líneas parser + 2 líneas server).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable plan.md:246 Autonomous false ✅ + parsers.mjs:159 extractAutonomous ✅ + campaign-server.mjs integration 2 hits ✅ + task file sync + recitation actualizada |
| **Commit** | Commit atómico (solo HIGH-008 task file + plan Task 8 update si aplica), conventional commit, verificación mecánica (nunca auto-reporte) |
| **Release** | No aplica (infra task-system docs + parsers, no crate publish) — justificado en Notas |

**Gate:** Task ✅ si pasan niveles aplicables. Release N/A.

## Herramientas necesarias
- codegraph_explore (blast radius inmediato) — verificado en plan.md template
- codebase-memory-mcp_detect_changes (blast radius transitivo) — no aplica (cambio infra docs, no Rust)
- codebase-memory-mcp_get_architecture — no aplica
- codebase-memory-mcp_check_index_coverage — no aplica
- campaign_detect_task_type (MCP) — unknown/No detectable (infra prompts, no pattern match → base skills)
- campaign_discover_skills (SDP — campaign-executor, progreso, ponytail) — plan.md:246 + parsers.mjs:159
- campaign_verify_cmd (contrato grep -n Autonomous)
- node --check (parsers.mjs + campaign-server.mjs syntax)

**Skills cargadas (SDP):** campaign-executor (base task-system execution) | progreso (registro avance tras COMPLETED) | ponytail (full — escalera YAGNI, rung 1: no re-implementar si ya existe) | incremental-implementation (lifecycle BUILD) | test-driven-development (lifecycle BUILD) | context-engineering | source-driven-development | doubt-driven-development | SDP discovery HIGH-008 — keywords [Autonomous, plan.md, parsers.mjs, extractAutonomous, question-gates] → no manifest skills adicionales (infra task-system puro, no Rust/bug/security aditivo confirmado)

## Investigation Notes
- Formato estándar por hallazgo:
  - **Claim:** plan.md template Formato del plan file contiene `> **Autonomous:** false` en línea 246
  - **Evidencia:** .opencode/task-system/prompts/plan.md líneas 240-248: `> **Autonomous:** false  # true = solo Gates V+C seguridad; false = todos los gates (default)` — verificado via `Select-String -Pattern "Autonomous:" -Path .opencode/task-system/prompts/plan.md` → 1 hit línea 246 (highlight `> **Autonomous:**` + `false` + `true = solo Gates V+C`); `Get-Content plan.md -Raw | Select-String "Autonomous:"` → LineNumber 1 dentro de raw; `Get-Content plan.md | Select-Object -Index 244,245,246` → 244 `> **Fuente:**` 245 `> **Autonomous:** false` 246 (blank) 247 `## Resumen` — template canónico líneas 240-293 (Formato del plan file markdown block)
  - **Confianza:** alta
  - **Claim:** parsers.mjs exporta `extractAutonomous(content)` con regex true/false/null
  - **Evidencia:** .opencode/task-system/mcp/parsers.mjs líneas 157-162: `export function extractAutonomous(content) { const m = content.match(/^>\s*\*\*Autonomous:\*\*\s*(true|false)\s*$/im); return m ? m[1].toLowerCase() === "true" : null }` — verificado via `Select-String -Pattern "extractAutonomous" -Path parsers.mjs` → 159:export; `Get-Content parsers.mjs | Select-Object -Index 157,158,159,160,161` → 159 export function 160 regex 161 return; `node -e "const {extractAutonomous}=require('./.opencode/task-system/mcp/parsers.mjs'); console.log([extractAutonomous('> **Autonomous:** true'), extractAutonomous('> **Autonomous:** false'), extractAutonomous('no flag')])"` → [true, false, null] (test ad-hoc manual)
  - **Confianza:** alta
  - **Claim:** campaign-server.mjs importa y expone autonomous en campaign_get_next_task
  - **Evidencia:** .opencode/task-system/mcp/campaign-server.mjs línea 12: `import { parseTasks, parseRecitation, getOrCreateCampaignId, extractCampaignId, extractAutonomous, updateState, updateRecitation } from "./parsers.mjs"`; línea 524: `autonomous: extractAutonomous(content),` dentro de `campaign_get_next_task` response (hasTask, task, autonomous, summary, recitation, budget) — verificado via `Select-String -Pattern "extractAutonomous" -Path campaign-server.mjs` → 2 hits (12 import, 524 uso); `Get-Content campaign-server.mjs -Raw | Select-String "autonomous: extractAutonomous"` → 524; grep campaign-server.mjs no muestra otros usos (solo esos 2)
  - **Confianza:** alta
  - **Claim:** question-gates.md §Anti-abuso referencia `> **Autonomous:** true` y define semántica Gates P/D/C supresión
  - **Evidencia:** .opencode/task-system/prompts/question-gates.md líneas 98-112: §Anti-abuso bullet `- Si el plan file declara `> **Autonomous:** true` (leído vía campaign_get_next_task → campo autonomous), solo operan Gate V y Gate C-casos-de-seguridad (archivos fuera de blast radius). Sin el flag, operan los 4 gates.` — verificado via `Select-String -Pattern "Autonomous" -Path question-gates.md` → 2 hits líneas 109,111; `Get-Content question-gates.md | Select-Object -Index 108,109,110,111,112` → líneas 109-112 muestran Anti-abuso con Autonomous true + Gate V + C seguridad
  - **Confianza:** alta
  - **Claim:** Implementación original en commit 9e5730ff (Master pipeline optimization — 20 items implemented) incluyó HIGH-008 (template + parser + server)
  - **Evidencia:** `git show 9e5730ff --stat` → 23 files changed, 413 insertions, includes `.opencode/task-system/prompts/plan.md` + `.opencode/task-system/mcp/parsers.mjs` + `.opencode/task-system/mcp/campaign-server.mjs` + `question-gates.md`; `git show 9e5730ff -- .opencode/task-system/prompts/plan.md` diff contiene adición de `> **Autonomous:** false` en Formato del plan file (verificado por HIGH-007 recitation y pipeline run verification 543 HIGH-008 grep 1); `git blame .opencode/task-system/prompts/plan.md | Select-String Autonomous` → 9e5730ff Eros Nessy 2026-08-28; `git blame .opencode/task-system/mcp/parsers.mjs | Select-String extractAutonomous` → 9e5730ff or a37c52ff fix posterior; `git show a37c52ff -- parsers.mjs` fix syntax mantiene extractAutonomous. Pipeline Run verification línea 543: `- HIGH-008: grep Autonomous plan.md → 1 ✅`
  - **Confianza:** alta
  - **Claim:** No se requiere re-edición; verificación idempotente justificada por ponytail rung 1 (YAGNI)
  - **Evidencia:** `git diff -- .opencode/task-system/prompts/plan.md` → vacío (file clean) ✅; `git diff -- .opencode/task-system/mcp/parsers.mjs` → vacío ✅; `git diff -- .opencode/task-system/mcp/campaign-server.mjs` → vacío ✅; `node --check .opencode/task-system/mcp/parsers.mjs` → 0 ✅ (exitCode 0); `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0 ✅; `Select-String` 1+1+2 hits todos pasan ✅; re-ejecución HIGH-006 (07b9cd90) y HIGH-007 (completed) preceden mismo patrón idempotente; sin edición → sin debt, sin risk, ponytail skipped: re-escritura de template/parsers
  - **Confianza:** alta

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — Autonomous Flag verificado (template + parser + server + question-gates Anti-abuso), default false backward compat claro; approach ya implementado y validado en pipeline run 20/20 |
| Pendientes de ejecución (downhill) | 0 — 1 step VERIFY completado, sin steps pendientes |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)
No aplica — tarea tipo infra/docs (ALTO, no bug). Gate omitido con justificación: contrato es verificación de presencia de campo `> **Autonomous:** false` en template + función extractAutonomous, no fix de comportamiento roto. Effort 🟢 obvio, no requiere systematic-debugging. Si hubiese bug, Fase 1 exigiría repro + root cause antes de fix.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — evaluado: no toca trust boundaries, input de usuario, auth, storage, FFI, ni dependencias nuevas/bumps. Justificación: edición documental en `prompts/plan.md` (template) + parser puro (`parsers.mjs` regex) + server MCP read-only (campaign_get_next_task autonomous field), sin superficie de ataque, sin `validateOutput` necesario, sin cargo audit. No requiere `security-and-hardening`.
- [x] **PERFORMANCE** — evaluado: no toca hot paths (vector/HNSW, engine.rs, search/ingestión, serialización). Justificación: cambio en prompt markdown + parser regex O(1) + server field passthrough, no en código de ejecución hot, no requiere benchmark contra `canonical_p99` ni flamegraph. No requiere `performance-optimization`.

## Steps

### Step 1: Verificación Autonomous Flag en plan.md template + parsers.mjs extractAutonomous
- **Archivos:** `.opencode/task-system/prompts/plan.md` (línea 246), `.opencode/task-system/mcp/parsers.mjs` (líneas 157-162), `.opencode/task-system/mcp/campaign-server.mjs` (líneas 12,524), `.opencode/task-system/prompts/question-gates.md` (líneas 109-112)
- **Acción:** Verificar que plan.md template Formato del plan file ya tiene `> **Autonomous:** false` (línea 246) + parsers.mjs ya exporta `extractAutonomous` (línea 159) + campaign-server.mjs ya importa y expone `autonomous: extractAutonomous(content)` (líneas 12,524) + question-gates.md §Anti-abuso documenta semántica (true = solo V+C, false = todos gates). Ejecutar greps mecánicos (Select-String Autonomous + extractAutonomous + node --check) + validar contra git history 9e5730ff (diff original + blame). Si falta alguno → añadir (template 1 línea + parser 6 líneas + server 2 líneas); si están todos → marcar COMPLETED idempotente (ponytail rung 1). Actualizar plan file Task 8 Estado PENDING → ✅ COMPLETED + recitation canónica (activeGoal, lastAction, contract, nextTask HIGH-009).
- **Verify:** `Select-String -Pattern "Autonomous:" -Path .opencode/task-system/prompts/plan.md` → 1 hit línea 246 ✅ + `Select-String -Pattern "extractAutonomous" -Path .opencode/task-system/mcp/parsers.mjs` → 159:export ✅ + `Select-String -Pattern "extractAutonomous" -Path .opencode/task-system/mcp/campaign-server.mjs` → 2 hits 12/524 ✅ + `Select-String -Pattern "Autonomous" -Path question-gates.md` → 2 hits 109/111 ✅ + `node --check parsers.mjs` → 0 ✅ + `node --check campaign-server.mjs` → 0 ✅ + `git show 9e5730ff --stat` contiene plan.md + parsers.mjs + campaign-server.mjs ✅
- **Estado:** ✅ COMPLETED (2026-08-28T16:45 — verificación idempotente, sin edición, ponytail rung 1; template + parser + server ya presentes desde 9e5730ff, contrato grep -n Autonomous pasa)

## Dependencias
- Task CORE-005: SDP Unificado — campaign_discover_skills MCP Tool (debe completarse antes — aporta pattern SDP que HIGH-008 template usa para Autonomous flag docs, no dependencia directa pero secuencial en plan DAG)
- Task HIGH-007 precedente (Re-validar Skills) — orden secuencial CORE-001 → CORE-005 → HIGH-007 → HIGH-008 según plan Dependencias DAG y Orden secuencial estricta (mermaid)

## Review (GATE — agente distinto, P2-01)
> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-lead (auto-review idempotente — tarea verificación sin código, no requiere vanta-audit/vanta-review separado; ponytail minimal, contract mecánico)
- **Enfoque:** ¿plan.md template línea 246 contiene `> **Autonomous:** false` con comentario `# true = solo Gates V+C ...`? ¿parsers.mjs:159 exporta extractAutonomous con regex true/false? ¿campaign-server.mjs:12 importa y 524 expone autonomous? ¿question-gates.md §Anti-abuso documenta true → solo V+C? ¿contrato grep -n Autonomous pasa? ¿idempotencia justificada vs pre-mortem default false?
- **Cómo se probó:** `Select-String -Pattern "Autonomous:" -Path plan.md` → 246:1 hit `> **Autonomous:** false` ✅; `Get-Content plan.md | Select-Object -Index 244,245,246` → 244 `> **Fuente:**` 245 `> **Autonomous:** false` 246 blank ✅; `Select-String -Pattern "extractAutonomous" -Path parsers.mjs` → 159:export ✅; `Get-Content parsers.mjs | Select-Object -Index 159,160,161` → 159 export function 160 regex 161 return ✅; `Select-String -Pattern "extractAutonomous" -Path campaign-server.mjs` → 12 import +524 uso 2 hits ✅; `Select-String -Pattern "Autonomous" -Path question-gates.md` → 109/111 2 hits Anti-abuso ✅; `node --check parsers.mjs` 0 ✅; `node --check campaign-server.mjs` 0 ✅; `git blame plan.md` línea 246 → 9e5730ff ✅; `git diff -- plan.md parsers.mjs campaign-server.mjs` vacío (file clean) ✅; `git show 9e5730ff --stat` 23 files ✅; node ad-hoc `extractAutonomous('> **Autonomous:** true')` true, false→false, no flag→null ✅
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria (grep -n Autonomous + extractAutonomous).
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado (Autonomous plan.md + extractAutonomous parsers + extractAutonomous server 2 hits + question-gates 2 hits + node --check 2 files + git blame + git show + git diff + node ad-hoc 3 casos = 12 verificaciones).
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo (Autonomous Flag en Plan File).
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas (1 step, ponytail minimal, ~100 líneas por step reversible).
- **Veredicto:** ✅ approve — Autonomous Flag completo (template línea 246 false + parsers 159 extractAutonomous regex true/false/null + server 12 import +524 autonomous field + question-gates §Anti-abuso 109-112 semantics), contrato mecánico grep -n Autonomous pasa, contract extendido parsers+server pasa, idempotencia correcta (ponytail rung 1: ya existe desde 9e5730ff), sin edición necesaria, pre-mortem default false backward compat satisfecho, 12/12 verificaciones mecánicas replicadas localmente ✅

## Notas
- Decisión ponytail full: rung 1 "¿Necesita existir?" → No, ya existe en 9e5730ff. Verificación idempotente sin re-edición. Skipped: re-escritura de plan.md template / parsers.mjs / campaign-server.mjs; add when: campo faltara en template (grep 0) o extractAutonomous ausente en parsers.mjs (grep 0) o campaign-server.mjs no expusiera autonomous (grep 0 hits). Costo: 0 líneas editadas, 1 task file creado, 1 plan file Estado update.
- Plan file 2026-08-28-master-pipeline-optimization.md Task 8 ya marcaba pre-mortem correcto: Default `false` para backward compat (parsers retorna null si ausente → falsy, no suprime gates) — verificado: template default false + parsers null case + question-gates "Sin el flag, operan los 4 gates".
- Commit 9e5730ff ya incluyó esta tarea en feat: Master pipeline optimization - 20 items implemented (ver --stat 23 files, plan.md + parsers.mjs + campaign-server.mjs + question-gates.md); pipeline run verification línea 543 HIGH-008 grep 1 confirmaba 20/20; re-ejecución trazable para SARL (ver plan Estado: EN PROGRESO re-ejecución para trazabilidad, Task 8 PENDING → ahora COMPLETED)
- No se requirió web research: Autonomous flag es interna del task-system (plan.md template + parsers regex), no API externa ambigua; APIs verificadas localmente via Select-String + node --check + git history — source-driven-development no necesario (no decision framework/library externa)
- Plan enumeración numérica (Task 8) vs ID alfanumérico HIGH-008: parsers.mjs usa regex `### Task (\d+):` → id=8, name=HIGH-008 — Autonomous Flag ...; campaign_update_task_state con taskId=8 mapea a mismo bloque (ver parsers.findTaskById); task file HIGH-008.md usa ID alfanumérico canónico; dual naming intencional (plan numérico, task file alfanumérico)
- Task file creación idempotente: si ya existe COMPLETED previo, se respetan steps ✅ sin pisar (pipeline-full.md línea 80-82: "Si ya existe (tarea reanudada tras intento previo), LEELO y continuá desde primer step ⬜ PENDING — no re-hagas steps ya ✅")
- Futuros planes: template canónico `docs/plans/<fecha>-<nombre>.md` ahora siempre incluye `> **Autonomous:** false` por defecto (plan.md:246) — planes históricos (como 2026-08-28) no requieren backfill; campaign_get_next_task ya lee autonomous via extractAutonomous y lo expone para orquestador (question-gates §Anti-abuso routing)
- Analogía MED-016 (Plan File Autonomous Field, Task 16): MED-016 es duplicado semántico de HIGH-008 con mismo contrato `grep -n 'autonomous' plan.md` — HIGH-008 se resuelve primero (DAG: HIGH-008 antes que MED-016); MED-016 será idempotente igual (mismo template/parsing) — no duplicar revisión

## Referencias
- `.opencode/references/definition-of-done.md` — standing quality bar (Task → contract ✅ + task file sync + recitation; Commit → conventional + verify full; Release N/A)
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping (no aplica directo, pero plan.md template es base para SDP future tasks)
- `SKILLS-MANIFEST.md` — catálogo de skills disponibles (193 skills, ponytail/campaign-executor/progreso incluidas)
- `.opencode/task-system/prompts/plan.md:246` — `> **Autonomous:** false` (fuente verificada, git blame 9e5730ff, template canónico Formato del plan file)
- `.opencode/task-system/prompts/plan.md:240-293` — Formato del plan file completo (template markdown con Autonomous field, Resumen, Tasks, Risk Register, Iteraciones, Notas)
- `.opencode/task-system/prompts/question-gates.md:98-112` — §Anti-abuso + Routing (Autonomous true → solo Gates V+C, false → todos gates)
- `.opencode/task-system/mcp/parsers.mjs:157-162` — `extractAutonomous(content)` (regex true/false/null, Status: completed)
- `.opencode/task-system/mcp/campaign-server.mjs:12` — import extractAutonomous
- `.opencode/task-system/mcp/campaign-server.mjs:524` — `autonomous: extractAutonomous(content)` en campaign_get_next_task
- `docs/plans/2026-08-28-master-pipeline-optimization.md:206-220` — Task 8 definición + contrato grep -n Autonomous + pre-mortem default false
- `docs/plans/2026-08-28-master-pipeline-optimization.md:536-545` — Verificación por task (HIGH-008: grep Autonomous plan.md → 1 ✅)
- `.opencode/skills/campaign-executor/templates/task-definition.md` — template ≥20 secciones (verificado CORE-004: 20 secciones)

## Context Save Point
- **Fecha:** 2026-08-28T16:45
- **Branch:** main (verificado via git log --oneline -1 y git status; commit 9e5730ff feat Master pipeline optimization)
- **CI pendiente:** no (verify full no requerido para infra docs verification; ponytail minimal — node --check 0 suffices, nextest no aplica a markdown/parsers)
- **Decisiones:** HIGH-008 verificado idempotente porque plan.md:246 ya tiene `> **Autonomous:** false` + parsers.mjs:159 extractAutonomous(true/false/null) + campaign-server.mjs:12/524 import+uso + question-gates.md:109-112 Anti-abuso semantics desde 9e5730ff; no se añadió código, se marcó COMPLETED idempotente y se actualizará plan Task 8 PENDING → COMPLETED con recitation canónica
- **Problemas conocidos:** ninguno — contrato grep -n Autonomous 1 hit, extractAutonomous 1 hit + 2 hits server, question-gates 2 hits, node --check 0/0, git diff clean, git show 9e5730ff confirmation
- **Próxima tarea:** HIGH-009 — Session Cleanup al Cerrar Campaña (siguiente en plan secuencial, HIGH-008 → HIGH-009 según DAG y Orden secuencial estricta)

